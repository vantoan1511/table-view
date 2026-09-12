use crate::drivers::ColumnInfo;
use chrono;
use hex;
use serde_json::Value;
use sqlx::postgres::PgRow;
use sqlx::{Column, Row, TypeInfo, ValueRef};
use std::collections::HashMap;
use uuid;

pub fn get_column_value(row: &PgRow, i: usize) -> Value {
    let raw = match row.try_get_raw(i) {
        Ok(v) => v,
        Err(e) => {
            log::error!("Error getting raw value for column {}: {}", i, e);
            return Value::Null;
        }
    };

    if raw.is_null() {
        return Value::Null;
    }
    let col = row.column(i);
    let type_info = col.type_info();
    let name = type_info.name();

    match name {
        "INT2" => {
            if let Ok(v) = row.try_get::<i16, _>(i) {
                return Value::Number(i64::from(v).into());
            }
        }
        "INT4" => {
            if let Ok(v) = row.try_get::<i32, _>(i) {
                return Value::Number(i64::from(v).into());
            }
        }
        "INT8" => {
            if let Ok(v) = row.try_get::<i64, _>(i) {
                return Value::Number(v.into());
            }
        }
        "OID" => {
            if let Ok(v) = row.try_get::<i32, _>(i) {
                return Value::Number(i64::from(v).into());
            }
        }
        "FLOAT4" | "FLOAT8" | "NUMERIC" => {
            if let Ok(v) = row.try_get::<f64, _>(i) {
                if let Some(num) = serde_json::Number::from_f64(v) {
                    return Value::Number(num);
                }
            }
        }
        "BOOL" => {
            if let Ok(v) = row.try_get::<bool, _>(i) {
                return Value::Bool(v);
            }
        }
        "TEXT" | "VARCHAR" | "BPCHAR" | "NAME" => {
            if let Ok(v) = row.try_get::<String, _>(i) {
                return Value::String(v);
            }
        }
        "UUID" => {
            if let Ok(v) = row.try_get::<uuid::Uuid, _>(i) {
                return Value::String(v.to_string());
            }
        }
        "TIMESTAMP" | "TIMESTAMPTZ" => {
            if let Ok(v) = row.try_get::<chrono::NaiveDateTime, _>(i) {
                return Value::String(v.to_string());
            }
            if let Ok(v) = row.try_get::<chrono::DateTime<chrono::Utc>, _>(i) {
                return Value::String(v.to_string());
            }
        }
        "DATE" => {
            if let Ok(v) = row.try_get::<chrono::NaiveDate, _>(i) {
                return Value::String(v.to_string());
            }
        }
        "JSON" | "JSONB" => {
            if let Ok(v) = row.try_get::<Value, _>(i) {
                return v;
            }
        }
        "BYTEA" => {
            if let Ok(bytes) = row.try_get::<Vec<u8>, _>(i) {
                return Value::String(hex::encode(bytes));
            }
        }
        _ => {}
    }

    // Final fallback: try as string
    if let Ok(v) = row.try_get::<String, _>(i) {
        return Value::String(v);
    }

    Value::Null
}

pub fn pg_rows_to_maps(rows: &[PgRow]) -> (Vec<HashMap<String, Value>>, Vec<ColumnInfo>) {
    if rows.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let orig_names: Vec<String> = rows[0]
        .columns()
        .iter()
        .map(|c| c.name().to_string())
        .collect();
    let unique_names = crate::drivers::utils::make_unique_column_names(&orig_names);

    let mut fields = Vec::new();
    for (col, unique_name) in rows[0].columns().iter().zip(unique_names.iter()) {
        let display_name = if unique_name != col.name() {
            Some(col.name().to_string())
        } else {
            None
        };
        fields.push(ColumnInfo {
            name: unique_name.clone(),
            data_type: col.type_info().name().to_string(),
            is_primary_key: false,
            is_nullable: true,
            display_name,
        });
    }

    let mut data = Vec::new();
    for row in rows {
        let mut map = HashMap::new();
        for (i, unique_name) in unique_names.iter().enumerate() {
            let val = get_column_value(row, i);
            map.insert(unique_name.clone(), val);
        }
        data.push(map);
    }

    (data, fields)
}
