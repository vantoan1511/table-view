use crate::drivers::ColumnInfo;
use oracledb::ToDbValue;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum OracleBind {
    Null,
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

pub type RowMap = HashMap<String, JsonValue>;
pub type CursorRawResult = (Vec<RowMap>, Vec<ColumnInfo>);
pub type QueryRawResult = (Vec<RowMap>, Vec<ColumnInfo>, u64);

/// Read LOBs in 16 KiB increments. Large enough for most Oracle TTC fetch
/// batches, small enough to live on the stack without risking stack overflow.
pub const LOB_READ_CHUNK_SIZE: usize = 16384;

pub fn json_to_bind(val: &JsonValue) -> OracleBind {
    match val {
        JsonValue::Null => OracleBind::Null,
        JsonValue::Bool(b) => OracleBind::Bool(*b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                OracleBind::Int(i)
            } else if let Some(f) = n.as_f64() {
                OracleBind::Float(f)
            } else {
                OracleBind::String(n.to_string())
            }
        }
        JsonValue::String(s) => OracleBind::String(s.clone()),
        _ => OracleBind::String(val.to_string()),
    }
}

pub fn oracle_json_to_serde(val: &oracledb::JsonValue) -> JsonValue {
    match val {
        oracledb::JsonValue::Null => JsonValue::Null,
        oracledb::JsonValue::Boolean(b) => JsonValue::Bool(*b),
        oracledb::JsonValue::Number(n) => {
            let s = n.to_string();
            if let Ok(i) = s.parse::<i64>() {
                JsonValue::Number(i.into())
            } else if let Ok(f) = s.parse::<f64>() {
                serde_json::Number::from_f64(f)
                    .map(JsonValue::Number)
                    .unwrap_or(JsonValue::Null)
            } else {
                JsonValue::String(s)
            }
        }
        oracledb::JsonValue::BinaryDouble(f) => serde_json::Number::from_f64(*f)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        oracledb::JsonValue::BinaryFloat(f) => serde_json::Number::from_f64(*f as f64)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        oracledb::JsonValue::String(s) => JsonValue::String(s.clone()),
        oracledb::JsonValue::Raw(b) => JsonValue::String(hex::encode(b)),
        oracledb::JsonValue::Timestamp(ts) => JsonValue::String(ts.to_string()),
        oracledb::JsonValue::IntervalDS(ds) => JsonValue::String(ds.to_string()),
        oracledb::JsonValue::IntervalYM(ym) => JsonValue::String(ym.to_string()),
        oracledb::JsonValue::JsonArray(arr) => {
            JsonValue::Array(arr.iter().map(oracle_json_to_serde).collect())
        }
        oracledb::JsonValue::JsonObject(map) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in map {
                obj.insert(k.clone(), oracle_json_to_serde(v));
            }
            JsonValue::Object(obj)
        }
        _ => JsonValue::Null,
    }
}

pub fn read_clob_to_json(lob: &mut oracledb::Lob) -> JsonValue {
    // Use a fixed stack buffer to avoid a 64KB heap allocation per LOB cell.
    let mut chunk = [0u8; LOB_READ_CHUNK_SIZE];
    let mut buf = Vec::new();
    let mut had_error = false;
    loop {
        match std::io::Read::read(lob, &mut chunk) {
            Ok(0) => break,
            Ok(n) => buf.extend_from_slice(&chunk[..n]),
            Err(e) => {
                log::warn!("oracle: error reading CLOB locator: {}", e);
                had_error = true;
                break;
            }
        }
    }
    // Return Null rather than an empty string when a read error occurred
    // before any data was received, so callers can distinguish failures
    // from genuinely empty CLOBs.
    if had_error && buf.is_empty() {
        JsonValue::Null
    } else {
        JsonValue::String(String::from_utf8_lossy(&buf).into_owned())
    }
}

pub fn read_blob_to_json(lob: &mut oracledb::Lob) -> JsonValue {
    let mut chunk = [0u8; LOB_READ_CHUNK_SIZE];
    let mut buf = Vec::new();
    let mut had_error = false;
    loop {
        match std::io::Read::read(lob, &mut chunk) {
            Ok(0) => break,
            Ok(n) => buf.extend_from_slice(&chunk[..n]),
            Err(e) => {
                log::warn!("oracle: error reading BLOB locator: {}", e);
                had_error = true;
                break;
            }
        }
    }
    if had_error && buf.is_empty() {
        JsonValue::Null
    } else {
        JsonValue::String(hex::encode(buf))
    }
}

pub fn extract_column_value(
    row: &mut oracledb::Row,
    i: usize,
    meta: &oracledb::Metadata,
) -> JsonValue {
    let db_type = meta.db_type();

    if db_type.is_string_type() {
        if db_type == &oracledb::DB_TYPE_CLOB || db_type == &oracledb::DB_TYPE_NCLOB {
            match row.take::<Option<oracledb::Lob>>(i) {
                Ok(Some(mut lob)) => return read_clob_to_json(&mut lob),
                Ok(None) => return JsonValue::Null,
                Err(_) => {
                    if let Ok(Some(s)) = row.get::<Option<String>>(i) {
                        return JsonValue::String(s);
                    }
                    return JsonValue::Null;
                }
            }
        }
        if let Ok(Some(s)) = row.get::<Option<String>>(i) {
            return JsonValue::String(s);
        }
        return JsonValue::Null;
    }

    if db_type == &oracledb::DB_TYPE_NUMBER || db_type == &oracledb::DB_TYPE_BINARY_INTEGER {
        if let Ok(Some(num)) = row.get::<Option<oracledb::OracleNumber>>(i) {
            let s = num.to_string();
            if let Ok(i_val) = s.parse::<i64>() {
                return JsonValue::Number(i_val.into());
            } else if let Ok(f_val) = s.parse::<f64>() {
                return serde_json::Number::from_f64(f_val)
                    .map(JsonValue::Number)
                    .unwrap_or(JsonValue::Null);
            } else {
                return JsonValue::String(s);
            }
        }
        return JsonValue::Null;
    }

    if db_type == &oracledb::DB_TYPE_BINARY_DOUBLE {
        if let Ok(Some(f)) = row.get::<Option<f64>>(i) {
            return serde_json::Number::from_f64(f)
                .map(JsonValue::Number)
                .unwrap_or(JsonValue::Null);
        }
        return JsonValue::Null;
    }

    if db_type == &oracledb::DB_TYPE_BINARY_FLOAT {
        if let Ok(Some(f)) = row.get::<Option<f32>>(i) {
            return serde_json::Number::from_f64(f as f64)
                .map(JsonValue::Number)
                .unwrap_or(JsonValue::Null);
        }
        return JsonValue::Null;
    }

    if db_type == &oracledb::DB_TYPE_BOOLEAN {
        if let Ok(Some(b)) = row.get::<Option<bool>>(i) {
            return JsonValue::Bool(b);
        }
        return JsonValue::Null;
    }

    if db_type.is_date_type() {
        if let Ok(Some(ts)) = row.get::<Option<oracledb::OracleTimestamp>>(i) {
            return JsonValue::String(ts.to_string());
        }
        return JsonValue::Null;
    }

    // is_binary_type() covers RAW, LONG_RAW, and BLOB.
    // DB_TYPE_BFILE is not included in that set, so we extend the guard
    // explicitly so BFILE columns receive the same LOB-locator path.
    if db_type.is_binary_type() || db_type == &oracledb::DB_TYPE_BFILE {
        if db_type == &oracledb::DB_TYPE_BLOB || db_type == &oracledb::DB_TYPE_BFILE {
            match row.take::<Option<oracledb::Lob>>(i) {
                Ok(Some(mut lob)) => return read_blob_to_json(&mut lob),
                Ok(None) => return JsonValue::Null,
                Err(_) => {
                    if let Ok(Some(bytes)) = row.get::<Option<Vec<u8>>>(i) {
                        return JsonValue::String(hex::encode(bytes));
                    }
                    return JsonValue::Null;
                }
            }
        }
        if let Ok(Some(bytes)) = row.get::<Option<Vec<u8>>>(i) {
            return JsonValue::String(hex::encode(bytes));
        }
        return JsonValue::Null;
    }

    if db_type == &oracledb::DB_TYPE_JSON {
        if let Ok(Some(jv)) = row.get::<Option<oracledb::JsonValue>>(i) {
            return oracle_json_to_serde(&jv);
        }
        return JsonValue::Null;
    }

    if db_type == &oracledb::DB_TYPE_INTERVAL_DS {
        if let Ok(Some(ds)) = row.get::<Option<oracledb::OracleIntervalDS>>(i) {
            return JsonValue::String(ds.to_string());
        }
        return JsonValue::Null;
    }

    if db_type == &oracledb::DB_TYPE_INTERVAL_YM {
        if let Ok(Some(ym)) = row.get::<Option<oracledb::OracleIntervalYM>>(i) {
            return JsonValue::String(ym.to_string());
        }
        return JsonValue::Null;
    }

    if let Ok(Some(s)) = row.get::<Option<String>>(i) {
        JsonValue::String(s)
    } else {
        JsonValue::Null
    }
}

pub fn to_db_value_refs<'a>(
    binds: &'a [OracleBind],
    null_str: &'a Option<String>,
) -> Vec<&'a dyn ToDbValue> {
    binds
        .iter()
        .map(|b| match b {
            OracleBind::Null => null_str as &dyn ToDbValue,
            OracleBind::String(s) => s as &dyn ToDbValue,
            OracleBind::Int(i) => i as &dyn ToDbValue,
            OracleBind::Float(f) => f as &dyn ToDbValue,
            OracleBind::Bool(b) => b as &dyn ToDbValue,
        })
        .collect()
}

pub fn cursor_to_maps(cursor: oracledb::Cursor) -> Result<CursorRawResult, String> {
    let col_metas = cursor.columns().clone();
    let orig_names: Vec<String> = col_metas.iter().map(|col| col.name().to_string()).collect();
    let unique_names = crate::drivers::utils::make_unique_column_names(&orig_names);

    let fields: Vec<ColumnInfo> = col_metas
        .iter()
        .zip(unique_names.iter())
        .map(|(col, unique_name)| {
            let display_name = if unique_name != col.name() {
                Some(col.name().to_string())
            } else {
                None
            };
            ColumnInfo {
                name: unique_name.clone(),
                data_type: col.db_type().name().to_string(),
                is_primary_key: false,
                is_nullable: col.nullable(),
                display_name,
            }
        })
        .collect();

    let mut rows = Vec::new();
    for row_res in cursor {
        let mut row = row_res.map_err(|e| e.to_string())?;
        let mut map = HashMap::new();
        for (i, (unique_name, meta)) in unique_names.iter().zip(col_metas.iter()).enumerate() {
            let json_val = extract_column_value(&mut row, i, meta);
            map.insert(unique_name.clone(), json_val);
        }
        rows.push(map);
    }

    Ok((rows, fields))
}
