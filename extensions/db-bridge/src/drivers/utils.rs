// use serde_json::Value; // Removed to fix warning

#[macro_export]
macro_rules! bind_json_value {
    ($query:expr, $val:expr) => {
        match $val {
            Value::Null => $query.bind(None::<String>),
            Value::Bool(b) => $query.bind(*b),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    $query.bind(i)
                } else if let Some(f) = n.as_f64() {
                    $query.bind(f)
                } else {
                    $query.bind(n.to_string())
                }
            }
            Value::String(s) => $query.bind(s.as_str()),
            _ => $query.bind($val.to_string()),
        }
    };
}
