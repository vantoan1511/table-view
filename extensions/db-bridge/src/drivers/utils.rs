#[macro_export]
macro_rules! bind_json_value {
    ($query:expr, $val:expr) => {
        match $val {
            serde_json::Value::Null => $query.bind(None::<String>),
            serde_json::Value::Bool(b) => $query.bind(*b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    $query.bind(i)
                } else if let Some(f) = n.as_f64() {
                    $query.bind(f)
                } else {
                    $query.bind(n.to_string())
                }
            }
            serde_json::Value::String(s) => $query.bind(s.as_str()),
            _ => $query.bind($val.to_string()),
        }
    };
}

pub fn is_safe_identifier(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

pub fn is_safe_data_type(s: &str) -> bool {
    // Basic allowlist approach for data types
    let s = s.to_uppercase();
    let parts: Vec<&str> = s.split(|c| c == '(' || c == ')' || c == ' ' || c == ',').filter(|p| !p.is_empty()).collect();
    
    let allowed = [
        "INT", "INTEGER", "SMALLINT", "BIGINT", "TINYINT",
        "VARCHAR", "CHAR", "TEXT", "LONGTEXT", "MEDIUMTEXT",
        "BOOLEAN", "BOOL",
        "DATE", "TIME", "TIMESTAMP", "DATETIME",
        "FLOAT", "DOUBLE", "DECIMAL", "NUMERIC",
        "UUID", "JSON", "JSONB", "BYTEA", "BLOB", "CLOB",
        "RAW", "NUMBER", "VARCHAR2", "NVARCHAR2", "NVARCHAR",
        "SERIAL", "BIGSERIAL", "SMALLSERIAL"
    ];

    parts.iter().all(|p| {
        p.chars().all(|c| c.is_alphanumeric() || c == '_') && 
        (p.parse::<u32>().is_ok() || allowed.contains(p))
    })
}

pub fn is_safe_default(s: &str) -> bool {
    // Defaults can be numbers, strings in quotes, or NULL
    let s = s.trim();
    if s.is_empty() || s.to_uppercase() == "NULL" {
        return true;
    }
    if s.parse::<f64>().is_ok() {
        return true;
    }
    // String literals like 'foo'
    if s.starts_with('\'') && s.ends_with('\'') {
        let inner = &s[1..s.len()-1];
        return !inner.contains('\'') || inner.contains("''"); // Simple escaping check
    }
    // Functions like now(), current_timestamp
    let allowed_funcs = ["NOW()", "CURRENT_TIMESTAMP", "CURRENT_DATE", "GETDATE()"];
    allowed_funcs.contains(&s.to_uppercase().as_str())
}
