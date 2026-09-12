use crate::drivers::ColumnInfo;
use chrono;
use hex;
use serde_json::Value;
use sqlx::postgres::PgRow;
use sqlx::{Column, Row, TypeInfo, ValueRef};
use std::collections::HashMap;
use uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PostgresType {
    Int2,
    Int4,
    Int8,
    Oid,
    Float4,
    Float8,
    Numeric,
    Bool,
    Text,
    Varchar,
    Bpchar,
    Name,
    Uuid,
    Timestamp,
    Timestamptz,
    Date,
    Json,
    Jsonb,
    Bytea,
    Other,
}

impl PostgresType {
    pub fn from_type_name(name: &str) -> Self {
        match name.to_ascii_uppercase().as_str() {
            "INT2" => Self::Int2,
            "INT4" => Self::Int4,
            "INT8" => Self::Int8,
            "OID" => Self::Oid,
            "FLOAT4" => Self::Float4,
            "FLOAT8" => Self::Float8,
            "NUMERIC" => Self::Numeric,
            "BOOL" => Self::Bool,
            "TEXT" => Self::Text,
            "VARCHAR" => Self::Varchar,
            "BPCHAR" => Self::Bpchar,
            "NAME" => Self::Name,
            "UUID" => Self::Uuid,
            "TIMESTAMP" => Self::Timestamp,
            "TIMESTAMPTZ" => Self::Timestamptz,
            "DATE" => Self::Date,
            "JSON" => Self::Json,
            "JSONB" => Self::Jsonb,
            "BYTEA" => Self::Bytea,
            _ => Self::Other,
        }
    }
}

impl From<&str> for PostgresType {
    fn from(name: &str) -> Self {
        Self::from_type_name(name)
    }
}

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

    match PostgresType::from_type_name(name) {
        PostgresType::Int2 => {
            if let Ok(v) = row.try_get::<i16, _>(i) {
                return Value::Number(i64::from(v).into());
            }
        }
        PostgresType::Int4 => {
            if let Ok(v) = row.try_get::<i32, _>(i) {
                return Value::Number(i64::from(v).into());
            }
        }
        PostgresType::Int8 => {
            if let Ok(v) = row.try_get::<i64, _>(i) {
                return Value::Number(v.into());
            }
        }
        PostgresType::Oid => {
            if let Ok(v) = row.try_get::<i32, _>(i) {
                return Value::Number(i64::from(v).into());
            }
        }
        PostgresType::Float4 | PostgresType::Float8 | PostgresType::Numeric => {
            if let Ok(v) = row.try_get::<f64, _>(i) {
                if let Some(num) = serde_json::Number::from_f64(v) {
                    return Value::Number(num);
                }
            }
        }
        PostgresType::Bool => {
            if let Ok(v) = row.try_get::<bool, _>(i) {
                return Value::Bool(v);
            }
        }
        PostgresType::Text
        | PostgresType::Varchar
        | PostgresType::Bpchar
        | PostgresType::Name => {
            if let Ok(v) = row.try_get::<String, _>(i) {
                return Value::String(v);
            }
        }
        PostgresType::Uuid => {
            if let Ok(v) = row.try_get::<uuid::Uuid, _>(i) {
                return Value::String(v.to_string());
            }
        }
        PostgresType::Timestamp | PostgresType::Timestamptz => {
            if let Ok(v) = row.try_get::<chrono::NaiveDateTime, _>(i) {
                return Value::String(v.to_string());
            }
            if let Ok(v) = row.try_get::<chrono::DateTime<chrono::Utc>, _>(i) {
                return Value::String(v.to_string());
            }
        }
        PostgresType::Date => {
            if let Ok(v) = row.try_get::<chrono::NaiveDate, _>(i) {
                return Value::String(v.to_string());
            }
        }
        PostgresType::Json | PostgresType::Jsonb => {
            if let Ok(v) = row.try_get::<Value, _>(i) {
                return v;
            }
        }
        PostgresType::Bytea => {
            if let Ok(bytes) = row.try_get::<Vec<u8>, _>(i) {
                return Value::String(hex::encode(bytes));
            }
        }
        PostgresType::Other => {}
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
