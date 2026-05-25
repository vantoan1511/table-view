use crate::bridge::{WsMessage, WsWriter, Bridge};
use crate::drivers::{self, AlterOperation, Config};
use crate::pool::Pool;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

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
    #[serde(rename = "allDatabases", default)]
    all_databases: bool,
    #[serde(default)]
    columns: Vec<drivers::TableColumn>,
    #[serde(default)]
    schema_name: String,
    #[serde(default)]
    target_database: String,
    #[serde(default)]
    db_name: String,
    #[serde(default)]
    message: String,
    #[serde(default)]
    level: String,
}

pub async fn handle_message(
    mut msg: WsMessage,
    writer: Arc<Mutex<WsWriter>>,
    token: String,
    pool: Arc<Mutex<Pool>>,
) {
    let event = match &msg.event {
        Some(e) if e.starts_with("dbBridge.") => msg.event.take().unwrap(),
        _ => return,
    };

    let action = event.strip_prefix("dbBridge.").unwrap_or("");

    let payload: Payload = match msg.data.take() {
        Some(data) => {
            match serde_json::from_value::<Payload>(data.clone()) {
                Ok(p) => p,
                Err(e) => {
                    log::error!("Failed to parse payload: {}. Data: {:?}", e, data);
                    let mut p = Payload::default();
                    if let Some(req_id) = data.get("reqId").and_then(|v| v.as_str()) { p.req_id = req_id.to_string(); }
                    if let Some(conn_id) = data.get("connectionId").and_then(|v| v.as_str()) { p.connection_id = conn_id.to_string(); }
                    p
                }
            }
        }
        None => Payload::default(),
    };

    match action {
        "testConnection" => {
            let config = match &payload.config {
                Some(c) => c,
                None => {
                    broadcast(&writer, &token, "dbBridge.testConnectionResult", json!({"reqId": payload.req_id, "success": false, "error": "missing config"})).await;
                    return;
                }
            };
            let mut driver = match drivers::create_driver(&config.db_type) {
                Some(d) => d,
                None => {
                    broadcast(&writer, &token, "dbBridge.testConnectionResult", json!({"reqId": payload.req_id, "success": false, "error": "unsupported driver type"})).await;
                    return;
                }
            };
            match driver.connect(config).await {
                Ok(()) => {
                    let _ = driver.disconnect().await;
                    broadcast(&writer, &token, "dbBridge.testConnectionResult", json!({"reqId": payload.req_id, "success": true})).await;
                }
                Err(e) => {
                    broadcast(&writer, &token, "dbBridge.testConnectionResult", json!({"reqId": payload.req_id, "success": false, "error": e})).await;
                }
            }
        }

        "connect" => {
            let config = match &payload.config {
                Some(c) => c,
                None => {
                    broadcast(&writer, &token, "dbBridge.connectResult", json!({"reqId": payload.req_id, "success": false, "error": "missing config"})).await;
                    return;
                }
            };
            let mut driver = match drivers::create_driver(&config.db_type) {
                Some(d) => d,
                None => {
                    broadcast(&writer, &token, "dbBridge.connectResult", json!({"reqId": payload.req_id, "success": false, "error": "unsupported driver type"})).await;
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
                    p.remove_all_for_connection(&conn_id).await;
                    p.put(conn_id.clone(), driver, config.clone()).await;
                    broadcast(&writer, &token, "dbBridge.connectResult", json!({"reqId": payload.req_id, "success": true})).await;
                }
                Err(e) => {
                    broadcast(&writer, &token, "dbBridge.connectResult", json!({"reqId": payload.req_id, "success": false, "error": e})).await;
                }
            }
        }

        "shutdown" => {
            log::info!("Received shutdown request. Closing all connections and exiting.");
            let mut p = pool.lock().await;
            p.close_all().await;
            std::process::exit(0);
        }

        "log" => {
            match payload.level.as_str() {
                "error" => log::error!("[UI] {}", payload.message),
                "warn" => log::warn!("[UI] {}", payload.message),
                "debug" => log::debug!("[UI] {}", payload.message),
                _ => log::info!("[UI] {}", payload.message),
            }
        }

        "dropDatabase" => {
            // 1. Remove the connection to the database being dropped from the pool to close it
            {
                let mut p = pool.lock().await;
                p.remove(&payload.connection_id, &payload.db_name).await;
            }

            // 2. We need a driver connected to a database other than payload.db_name to execute the DROP command.
            // Let's get the original connection config to see what database/type it is.
            let config_opt = {
                let p = pool.lock().await;
                p.get_config(&payload.connection_id).cloned()
            };

            let config = match config_opt {
                Some(c) => c,
                None => {
                    broadcast(&writer, &token, "dbBridge.dropDatabaseResult", json!({"reqId": payload.req_id, "success": false, "error": format!("connection config not found for {}", payload.connection_id)})).await;
                    return;
                }
            };

            // Determine the fallback/temporary database to connect to
            let fallback_db = match config.db_type.as_str() {
                "postgres" | "postgresql" => {
                    if config.database == payload.db_name {
                        "postgres".to_string()
                    } else {
                        config.database.clone()
                    }
                }
                "mysql" => {
                    if config.database == payload.db_name {
                        "mysql".to_string()
                    } else {
                        config.database.clone()
                    }
                }
                _ => {
                    if config.database == payload.db_name {
                        "".to_string()
                    } else {
                        config.database.clone()
                    }
                }
            };

            // 3. Create a temporary driver connected to the fallback/temporary database
            let mut temp_driver = match drivers::create_driver(&config.db_type) {
                Some(d) => d,
                None => {
                    broadcast(&writer, &token, "dbBridge.dropDatabaseResult", json!({"reqId": payload.req_id, "success": false, "error": format!("unsupported driver type: {}", config.db_type)})).await;
                    return;
                }
            };

            let mut temp_config = config.clone();
            temp_config.database = fallback_db;

            if let Err(e) = temp_driver.connect(&temp_config).await {
                broadcast(&writer, &token, "dbBridge.dropDatabaseResult", json!({"reqId": payload.req_id, "success": false, "error": format!("failed to connect to temporary database: {}", e)})).await;
                return;
            }

            // 4. For PostgreSQL, we should also terminate all other active connections to payload.db_name.
            // This is a common and robust practice to make sure we don't get the "database is being accessed by other users" error.
            if config.db_type == "postgres" || config.db_type == "postgresql" {
                let escaped_db_name = payload.db_name.replace('\'', "''");
                let terminate_sql = format!(
                    "SELECT pg_terminate_backend(pid) \
                     FROM pg_stat_activity \
                     WHERE datname = '{}' \
                       AND pid <> pg_backend_pid()",
                    escaped_db_name
                );
                if let Err(e) = temp_driver.query(&terminate_sql).await {
                    log::warn!("Failed to terminate active connections for database '{}': {}", payload.db_name, e);
                }
            }

            // 5. Drop the database
            let result = temp_driver.drop_database(&payload.db_name).await;

            // 6. Clean up temporary driver
            let _ = temp_driver.disconnect().await;

            // 7. Handle and return result
            handle_result_void(&writer, &token, "dbBridge.dropDatabaseResult", &payload.req_id, result).await;
        }

        _ => {
            let driver_arc = {
                let mut p = pool.lock().await;
                let target_db = (!payload.target_database.trim().is_empty())
                    .then_some(payload.target_database.trim());
                p.get(&payload.connection_id, target_db)
            };

            let driver_arc = if let Some(d) = driver_arc {
                d
            } else {
                // Not in pool, need to create outside global lock
                let target_db = (!payload.target_database.trim().is_empty())
                    .then_some(payload.target_database.trim());
                
                log::info!("handler: driver not in pool, creating outside lock for connection {}", payload.connection_id);
                
                // Get config while locked, then release
                let config = {
                    let p = pool.lock().await;
                    p.get_config(&payload.connection_id).cloned()
                };

                let base_config = match config {
                    Some(c) => c,
                    None => {
                        let result_event = format!("dbBridge.{}Result", action);
                        broadcast(&writer, &token, &result_event, json!({
                            "reqId": payload.req_id,
                            "success": false,
                            "error": format!("connection config not found for {}", payload.connection_id)
                        })).await;
                        return;
                    }
                };

                let target_db_name = target_db
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| base_config.database.clone());

                let mut new_config = base_config.clone();
                new_config.database = target_db_name.clone();

                let mut driver = match crate::drivers::create_driver(&new_config.db_type) {
                    Some(d) => d,
                    None => {
                        let result_event = format!("dbBridge.{}Result", action);
                        broadcast(&writer, &token, &result_event, json!({
                            "reqId": payload.req_id,
                            "success": false,
                            "error": format!("unsupported driver type: {}", new_config.db_type)
                        })).await;
                        return;
                    }
                };

                // CONNECT OUTSIDE LOCK
                if let Err(e) = driver.connect(&new_config).await {
                    let result_event = format!("dbBridge.{}Result", action);
                    broadcast(&writer, &token, &result_event, json!({
                        "reqId": payload.req_id,
                        "success": false,
                        "error": e
                    })).await;
                    return;
                }

                // Put back in pool
                let mut p = pool.lock().await;
                p.put(payload.connection_id.clone(), driver, new_config).await;
                // Re-get from pool to ensure LRU is updated and we get the Arc
                p.get(&payload.connection_id, Some(&target_db_name)).unwrap()
            };

            let driver = driver_arc.read().await;
            match action {
                "getSchema" | "getDbSchema" => {
                    let schema_name = (!payload.schema_name.trim().is_empty())
                         .then_some(payload.schema_name.trim());
                    let result = driver.get_schema(payload.all_databases, schema_name).await;
                    let event = if action == "getSchema" { "dbBridge.getSchemaResult" } else { "dbBridge.getDbSchemaResult" };
                    match result {
                        Ok(schema) => {
                            let mut resp = serde_json::to_value(&schema).unwrap_or(json!({}));
                            if let Some(obj) = resp.as_object_mut() {
                                obj.insert("reqId".to_string(), json!(payload.req_id));
                                obj.insert("success".to_string(), json!(true));
                                if action == "getDbSchema" {
                                    obj.insert("database".to_string(), json!(payload.target_database));
                                }
                            }
                            broadcast(&writer, &token, event, resp).await;
                        }
                        Err(e) => broadcast(&writer, &token, event, json!({"reqId": payload.req_id, "success": false, "error": e})).await,
                    }
                }
                "fetchTableData" => {
                    let result = driver.fetch_table_data(&payload.table_name, payload.limit, payload.offset, &payload.sort_column, &payload.sort_direction, &payload.filter).await;
                    match result {
                        Ok(data) => {
                            let mut resp = serde_json::to_value(&data).unwrap_or(json!({}));
                            if let Some(obj) = resp.as_object_mut() {
                                obj.insert("reqId".to_string(), json!(payload.req_id));
                                obj.insert("success".to_string(), json!(true));
                            }
                            broadcast(&writer, &token, "dbBridge.fetchTableDataResult", resp).await;
                        }
                        Err(e) => broadcast(&writer, &token, "dbBridge.fetchTableDataResult", json!({"reqId": payload.req_id, "success": false, "error": e})).await,
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
                            broadcast(&writer, &token, "dbBridge.executeQueryResult", resp).await;
                        }
                        Err(e) => broadcast(&writer, &token, "dbBridge.executeQueryResult", json!({"reqId": payload.req_id, "success": false, "error": e})).await,
                    }
                }
                "updateCell" => {
                    let result = driver.update_cell(&payload.table_name, &payload.pk_column, &payload.pk_value, &payload.target_column, &payload.new_value).await;
                    handle_result_void(&writer, &token, "dbBridge.updateCellResult", &payload.req_id, result).await;
                }
                "insertRow" => {
                    let result = driver.insert_row(&payload.table_name, &payload.data).await;
                    match result {
                        Ok(row) => broadcast(&writer, &token, "dbBridge.insertRowResult", json!({"reqId": payload.req_id, "success": true, "row": row})).await,
                        Err(e) => broadcast(&writer, &token, "dbBridge.insertRowResult", json!({"reqId": payload.req_id, "success": false, "error": e})).await,
                    }
                }
                "deleteRows" => {
                    let result = driver.delete_rows(&payload.table_name, &payload.pk_column, &payload.pk_values).await;
                    handle_result_void(&writer, &token, "dbBridge.deleteRowsResult", &payload.req_id, result).await;
                }
                "getTableColumns" => {
                    let result = driver.get_table_columns(&payload.table_name).await;
                    match result {
                        Ok(cols) => broadcast(&writer, &token, "dbBridge.getTableColumnsResult", json!({"reqId": payload.req_id, "success": true, "columns": cols})).await,
                        Err(e) => broadcast(&writer, &token, "dbBridge.getTableColumnsResult", json!({"reqId": payload.req_id, "success": false, "error": e})).await,
                    }
                }
                "alterTable" => {
                    let result = driver.alter_table(&payload.table_name, &payload.operations).await;
                    handle_result_void(&writer, &token, "dbBridge.alterTableResult", &payload.req_id, result).await;
                }
                "dropTable" => {
                    let result = driver.drop_table(&payload.table_name).await;
                    handle_result_void(&writer, &token, "dbBridge.dropTableResult", &payload.req_id, result).await;
                }
                "createTable" => {
                    let result = driver.create_table(&payload.table_name, &payload.columns).await;
                    handle_result_void(&writer, &token, "dbBridge.createTableResult", &payload.req_id, result).await;
                }
                "createSchema" => {
                    let result = driver.create_schema(&payload.schema_name).await;
                    handle_result_void(&writer, &token, "dbBridge.createSchemaResult", &payload.req_id, result).await;
                }
                "dropSchema" => {
                    let result = driver.drop_schema(&payload.schema_name).await;
                    handle_result_void(&writer, &token, "dbBridge.dropSchemaResult", &payload.req_id, result).await;
                }
                "createDatabase" => {
                    let result = driver.create_database(&payload.db_name).await;
                    handle_result_void(&writer, &token, "dbBridge.createDatabaseResult", &payload.req_id, result).await;
                }
                "exportCSV" => {
                    let result = driver.export_to_csv(&payload.table_name, &payload.export_path).await;
                    handle_result_void(&writer, &token, "dbBridge.exportCSVResult", &payload.req_id, result).await;
                }
                "getSchemaDetails" => {
                    let result = driver.get_schema_details(&payload.schema_name).await;
                    match result {
                        Ok(details) => {
                            let mut resp = serde_json::to_value(&details).unwrap_or(json!({}));
                            if let Some(obj) = resp.as_object_mut() {
                                obj.insert("reqId".to_string(), json!(payload.req_id));
                                obj.insert("success".to_string(), json!(true));
                            }
                            broadcast(&writer, &token, "dbBridge.getSchemaDetailsResult", resp).await;
                        }
                        Err(e) => broadcast(&writer, &token, "dbBridge.getSchemaDetailsResult", json!({"reqId": payload.req_id, "success": false, "error": e})).await,
                    }
                }
                _ => log::warn!("Unknown action: {}", action),
            }
        }
    }
}

async fn broadcast(writer: &Arc<Mutex<WsWriter>>, token: &str, event: &str, data: Value) {
    if let Err(e) = Bridge::broadcast(writer, token, event, data).await {
        log::error!("Broadcast error for {}: {}", event, e);
    }
}

async fn handle_result<T: serde::Serialize>(writer: &Arc<Mutex<WsWriter>>, token: &str, event: &str, req_id: &str, key: &str, result: Result<T, String>) {
    match result {
        Ok(data) => broadcast(writer, token, event, json!({"reqId": req_id, "success": true, key: data})).await,
        Err(e) => broadcast(writer, token, event, json!({"reqId": req_id, "success": false, "error": e})).await,
    }
}

async fn handle_result_void(writer: &Arc<Mutex<WsWriter>>, token: &str, event: &str, req_id: &str, result: Result<(), String>) {
    match result {
        Ok(()) => broadcast(writer, token, event, json!({"reqId": req_id, "success": true})).await,
        Err(e) => broadcast(writer, token, event, json!({"reqId": req_id, "success": false, "error": e})).await,
    }
}
