use super::{ColumnInfo, TableColumn};
use serde_json::Value;
use sqlx::{Column, Row, TypeInfo};
use std::collections::HashMap;

/// Scan all rows from a sqlx query result into a Vec of HashMaps + column info.
/// This is the Rust equivalent of the Go `ScanRows` helper.
pub async fn scan_rows(
    rows: Vec<sqlx::any::AnyRow>,
    columns: &[String],
    type_names: &[String],
) -> Result<(Vec<HashMap<String, Value>>, Vec<ColumnInfo>), String> {
    let fields: Vec<ColumnInfo> = columns
        .iter()
        .zip(type_names.iter())
        .map(|(name, dtype)| ColumnInfo {
            name: name.clone(),
            data_type_id: Value::String(dtype.clone()),
            is_primary_key: false,
        })
        .collect();

    let mut result = Vec::new();
    for row in &rows {
        let mut map = HashMap::new();
        for (i, col_name) in columns.iter().enumerate() {
            let val = get_column_value(row, i);
            map.insert(col_name.clone(), val);
        }
        result.push(map);
    }

    Ok((result, fields))
}

/// Extract a column value from a row as a serde_json::Value.
fn get_column_value(row: &sqlx::any::AnyRow, idx: usize) -> Value {
    // Try various types in order of likelihood
    if let Ok(v) = row.try_get::<Option<i64>, _>(idx) {
        return match v {
            Some(n) => Value::Number(n.into()),
            None => Value::Null,
        };
    }
    if let Ok(v) = row.try_get::<Option<f64>, _>(idx) {
        return match v {
            Some(n) => serde_json::Number::from_f64(n)
                .map(Value::Number)
                .unwrap_or(Value::Null),
            None => Value::Null,
        };
    }
    if let Ok(v) = row.try_get::<Option<bool>, _>(idx) {
        return match v {
            Some(b) => Value::Bool(b),
            None => Value::Null,
        };
    }
    if let Ok(v) = row.try_get::<Option<String>, _>(idx) {
        return match v {
            Some(s) => Value::String(s),
            None => Value::Null,
        };
    }
    // Fallback: try as bytes and convert to string
    if let Ok(v) = row.try_get::<Option<Vec<u8>>, _>(idx) {
        return match v {
            Some(bytes) => Value::String(String::from_utf8_lossy(&bytes).into_owned()),
            None => Value::Null,
        };
    }
    Value::Null
}

/// Execute a query and return rows + column metadata via sqlx's AnyPool.
pub async fn execute_query(
    pool: &sqlx::AnyPool,
    sql: &str,
    params: &[Value],
) -> Result<(Vec<HashMap<String, Value>>, Vec<ColumnInfo>), String> {
    let mut query = sqlx::query(sql);
    for p in params {
        query = bind_value(query, p);
    }

    let rows: Vec<sqlx::any::AnyRow> = query
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    if rows.is_empty() {
        return Ok((vec![], vec![]));
    }

    let columns: Vec<String> = rows[0].columns().iter().map(|c| c.name().to_string()).collect();
    let type_names: Vec<String> = rows[0]
        .columns()
        .iter()
        .map(|c| c.type_info().name().to_string())
        .collect();

    scan_rows(rows, &columns, &type_names).await
}

/// Bind a serde_json::Value to a sqlx query.
pub fn bind_value<'q>(
    query: sqlx::query::Query<'q, sqlx::Any, sqlx::any::AnyArguments<'q>>,
    val: &'q Value,
) -> sqlx::query::Query<'q, sqlx::Any, sqlx::any::AnyArguments<'q>> {
    match val {
        Value::Null => query.bind(None::<String>),
        Value::Bool(b) => query.bind(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                query.bind(i)
            } else if let Some(f) = n.as_f64() {
                query.bind(f)
            } else {
                query.bind(n.to_string())
            }
        }
        Value::String(s) => query.bind(s.as_str()),
        _ => query.bind(val.to_string()),
    }
}

/// Export a table to CSV file.
pub async fn export_to_csv(
    pool: &sqlx::AnyPool,
    table_expr: &str,
    export_path: &str,
) -> Result<(), String> {
    let sql = format!("SELECT * FROM {}", table_expr);
    let rows: Vec<sqlx::any::AnyRow> = sqlx::query(&sql)
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    let mut writer = csv::Writer::from_path(export_path).map_err(|e| e.to_string())?;

    if rows.is_empty() {
        return Ok(());
    }

    // Write header
    let columns: Vec<String> = rows[0].columns().iter().map(|c| c.name().to_string()).collect();
    writer.write_record(&columns).map_err(|e| e.to_string())?;

    // Write data
    for row in &rows {
        let record: Vec<String> = (0..columns.len())
            .map(|i| {
                let val = get_column_value(row, i);
                match val {
                    Value::Null => String::new(),
                    Value::String(s) => s,
                    other => other.to_string(),
                }
            })
            .collect();
        writer.write_record(&record).map_err(|e| e.to_string())?;
    }

    writer.flush().map_err(|e| e.to_string())?;
    Ok(())
}

/// Get table columns metadata (name, type, nullable, default).
pub async fn get_table_columns_generic(
    pool: &sqlx::AnyPool,
    query: &str,
    params: &[Value],
) -> Result<Vec<TableColumn>, String> {
    let mut q = sqlx::query(query);
    for p in params {
        q = bind_value(q, p);
    }

    let rows: Vec<sqlx::any::AnyRow> = q.fetch_all(pool).await.map_err(|e| e.to_string())?;

    let mut cols = Vec::new();
    for row in &rows {
        let name: String = row.try_get(0).unwrap_or_default();
        let data_type: String = row.try_get(1).unwrap_or_default();
        let nullable_str: String = row.try_get::<String, _>(2).unwrap_or_default();
        let default_val: Option<String> = row.try_get(3).ok();

        cols.push(TableColumn {
            name,
            data_type,
            nullable: nullable_str == "YES",
            default: default_val
                .map(|d| Value::String(d))
                .unwrap_or(Value::Null),
        });
    }

    Ok(cols)
}
