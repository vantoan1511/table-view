use crate::bridge::{Bridge, WsMessage};
use crate::drivers::{self, AlterOperation, Config};
use crate::pool::Pool;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Payload structure matching the Go version's payload parsing.
#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Payload {
    #[serde(default)]
    req_id: String,
    #[serde(default)]
    connection_id: String,
    #[serde(default)]
    config: Option<Config>,
    #[serde(default)]
    table_name: String,
    #[serde(default)]
    limit: i64,
    #[serde(default)]
    offset: i64,
    #[serde(default)]
    sort_column: String,
    #[serde(default)]
    sort_direction: String,
    #[serde(default)]
    filter: String,
    #[serde(default)]
    sql: String,
    #[serde(default)]
    pk_column: String,
    #[serde(default)]
    pk_value: Value,
    #[serde(default)]
    target_column: String,
    #[serde(default)]
    new_value: Value,
    #[serde(default)]
    export_path: String,
    #[serde(default)]
    data: HashMap<String, Value>,
    #[serde(default)]
    pk_values: Vec<Value>,
    #[serde(default)]
    operations: Vec<AlterOperation>,
    #[serde(default)]
    all_schemas: bool,
}

/// Handle an incoming WebSocket message — the main event dispatcher.
/// This is the Rust equivalent of the Go `b.Listen(func(msg) { ... })` block.
pub async fn handle_message(
    msg: WsMessage,
    bridge: Arc<Mutex<Bridge>>,
    pool: Arc<Mutex<Pool>>,
) {
    let event = match &msg.event {
        Some(e) if e.starts_with("dbBridge.") => e.clone(),
        _ => return,
    };

    let action = event.strip_prefix("dbBridge.").unwrap_or("");

    let payload: Payload = match &msg.data {
        Some(data) => serde_json::from_value(data.clone()).unwrap_or_default(),
        None => Payload::default(),
    };

    match action {
        // ── testConnection: ephemeral, doesn't touch the pool ────────────
        "testConnection" => {
            let config = match &payload.config {
                Some(c) => c,
                None => {
                    broadcast(
                        &bridge,
                        "dbBridge.testConnectionResult",
                        json!({"reqId": payload.req_id, "success": false, "error": "missing config"}),
                    )
                    .await;
                    return;
                }
            };

            let mut driver = match drivers::create_driver(&config.db_type) {
                Some(d) => d,
                None => {
                    broadcast(
                        &bridge,
                        "dbBridge.testConnectionResult",
                        json!({"reqId": payload.req_id, "success": false, "error": "unsupported driver type"}),
                    )
                    .await;
                    return;
                }
            };

            match driver.connect(config).await {
                Ok(()) => {
                    let _ = driver.disconnect().await;
                    broadcast(
                        &bridge,
                        "dbBridge.testConnectionResult",
                        json!({"reqId": payload.req_id, "success": true}),
                    )
                    .await;
                }
                Err(e) => {
                    broadcast(
                        &bridge,
                        "dbBridge.testConnectionResult",
                        json!({"reqId": payload.req_id, "success": false, "error": e}),
                    )
                    .await;
                }
            }
        }

        // ── connect: open a new connection and register in pool ──────────
        "connect" => {
            let config = match &payload.config {
                Some(c) => c,
                None => {
                    broadcast(
                        &bridge,
                        "dbBridge.connectResult",
                        json!({"reqId": payload.req_id, "success": false, "error": "missing config"}),
                    )
                    .await;
                    return;
                }
            };

            let mut driver = match drivers::create_driver(&config.db_type) {
                Some(d) => d,
                None => {
                    broadcast(
                        &bridge,
                        "dbBridge.connectResult",
                        json!({"reqId": payload.req_id, "success": false, "error": "unsupported driver type"}),
                    )
                    .await;
                    return;
                }
            };

            let conn_id = if payload.connection_id.is_empty() {
                format!("{}-default", config.db_type)
            } else {
                payload.connection_id.clone()
            };

            match driver.connect(config).await {
                Ok(()) => {
                    let mut p = pool.lock().await;
                    p.put(conn_id.clone(), driver).await;
                    log::info!("pool: registered connection {}", conn_id);
                    broadcast(
                        &bridge,
                        "dbBridge.connectResult",
                        json!({"reqId": payload.req_id, "success": true}),
                    )
                    .await;
                }
                Err(e) => {
                    broadcast(
                        &bridge,
                        "dbBridge.connectResult",
                        json!({"reqId": payload.req_id, "success": false, "error": e}),
                    )
                    .await;
                }
            }
        }

        // ── all other actions: route via pool ────────────────────────────
        _ => {
            let mut p = pool.lock().await;
            let driver = match p.get(&payload.connection_id) {
                Some(d) => d,
                None => {
                    log::warn!(
                        "pool: no driver for connectionId={:?} (action={})",
                        payload.connection_id,
                        action
                    );
                    let result_event = format!("dbBridge.{}Result", action);
                    broadcast(
                        &bridge,
                        &result_event,
                        json!({
                            "reqId": payload.req_id,
                            "success": false,
                            "error": format!("not connected (connectionId={:?})", payload.connection_id)
                        }),
                    )
                    .await;
                    return;
                }
            };

            match action {
                "getSchema" => {
                    let result = driver.get_schema(payload.all_schemas).await;
                    handle_result(&bridge, "dbBridge.getSchemaResult", &payload.req_id, "schema", result).await;
                }
                "fetchTableData" => {
                    let result = driver
                        .fetch_table_data(
                            &payload.table_name,
                            payload.limit,
                            payload.offset,
                            &payload.sort_column,
                            &payload.sort_direction,
                        )
                        .await;
                    match result {
                        Ok(data) => {
                            let mut resp = serde_json::to_value(&data).unwrap_or(json!({}));
                            if let Some(obj) = resp.as_object_mut() {
                                obj.insert("reqId".to_string(), json!(payload.req_id));
                                obj.insert("success".to_string(), json!(true));
                            }
                            broadcast(&bridge, "dbBridge.fetchTableDataResult", resp).await;
                        }
                        Err(e) => {
                            broadcast(
                                &bridge,
                                "dbBridge.fetchTableDataResult",
                                json!({"reqId": payload.req_id, "success": false, "error": e}),
                            )
                            .await;
                        }
                    }
                }
                "executeQuery" => {
                    let result = driver.query(&payload.sql).await;
                    match result {
                        Ok(data) => {
                            let mut resp = serde_json::to_value(&data).unwrap_or(json!({}));
                            if let Some(obj) = resp.as_object_mut() {
                                obj.insert("reqId".to_string(), json!(payload.req_id));
                                obj.insert("success".to_string(), json!(true));
                            }
                            broadcast(&bridge, "dbBridge.executeQueryResult", resp).await;
                        }
                        Err(e) => {
                            broadcast(
                                &bridge,
                                "dbBridge.executeQueryResult",
                                json!({"reqId": payload.req_id, "success": false, "error": e}),
                            )
                            .await;
                        }
                    }
                }
                "updateCell" => {
                    let result = driver
                        .update_cell(
                            &payload.table_name,
                            &payload.pk_column,
                            &payload.pk_value,
                            &payload.target_column,
                            &payload.new_value,
                        )
                        .await;
                    handle_result_void(&bridge, "dbBridge.updateCellResult", &payload.req_id, result).await;
                }
                "exportCSV" => {
                    let result = driver
                        .export_to_csv(&payload.table_name, &payload.export_path)
                        .await;
                    handle_result_void(&bridge, "dbBridge.exportCSVResult", &payload.req_id, result).await;
                }
                "insertRow" => {
                    let result = driver.insert_row(&payload.table_name, &payload.data).await;
                    match result {
                        Ok(row) => {
                            broadcast(
                                &bridge,
                                "dbBridge.insertRowResult",
                                json!({"reqId": payload.req_id, "success": true, "row": row}),
                            )
                            .await;
                        }
                        Err(e) => {
                            broadcast(
                                &bridge,
                                "dbBridge.insertRowResult",
                                json!({"reqId": payload.req_id, "success": false, "error": e}),
                            )
                            .await;
                        }
                    }
                }
                "deleteRows" => {
                    let result = driver
                        .delete_rows(&payload.table_name, &payload.pk_column, &payload.pk_values)
                        .await;
                    handle_result_void(&bridge, "dbBridge.deleteRowsResult", &payload.req_id, result).await;
                }
                "getTableColumns" => {
                    let result = driver.get_table_columns(&payload.table_name).await;
                    match result {
                        Ok(cols) => {
                            broadcast(
                                &bridge,
                                "dbBridge.getTableColumnsResult",
                                json!({"reqId": payload.req_id, "success": true, "columns": cols}),
                            )
                            .await;
                        }
                        Err(e) => {
                            broadcast(
                                &bridge,
                                "dbBridge.getTableColumnsResult",
                                json!({"reqId": payload.req_id, "success": false, "error": e}),
                            )
                            .await;
                        }
                    }
                }
                "alterTable" => {
                    let result = driver
                        .alter_table(&payload.table_name, &payload.operations)
                        .await;
                    handle_result_void(&bridge, "dbBridge.alterTableResult", &payload.req_id, result).await;
                }
                _ => {
                    log::warn!("Unknown action: {}", action);
                }
            }
        }
    }
}

/// Broadcast a JSON value via the bridge.
async fn broadcast(bridge: &Arc<Mutex<Bridge>>, event: &str, data: Value) {
    let mut b = bridge.lock().await;
    if let Err(e) = b.broadcast(event, data).await {
        log::error!("Broadcast error for {}: {}", event, e);
    }
}

/// Handle a result that returns data under a named key (e.g. "schema").
async fn handle_result<T: serde::Serialize>(
    bridge: &Arc<Mutex<Bridge>>,
    event: &str,
    req_id: &str,
    key: &str,
    result: Result<T, String>,
) {
    match result {
        Ok(data) => {
            broadcast(
                bridge,
                event,
                json!({"reqId": req_id, "success": true, key: data}),
            )
            .await;
        }
        Err(e) => {
            broadcast(
                bridge,
                event,
                json!({"reqId": req_id, "success": false, "error": e}),
            )
            .await;
        }
    }
}

/// Handle a result that returns no data (success/error only).
async fn handle_result_void(
    bridge: &Arc<Mutex<Bridge>>,
    event: &str,
    req_id: &str,
    result: Result<(), String>,
) {
    match result {
        Ok(()) => {
            broadcast(
                bridge,
                event,
                json!({"reqId": req_id, "success": true}),
            )
            .await;
        }
        Err(e) => {
            broadcast(
                bridge,
                event,
                json!({"reqId": req_id, "success": false, "error": e}),
            )
            .await;
        }
    }
}
