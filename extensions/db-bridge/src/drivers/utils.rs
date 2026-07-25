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

#[allow(dead_code)]
pub fn is_safe_identifier(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

pub fn is_safe_data_type(s: &str) -> bool {
    // Basic allowlist approach for data types
    let s = s.to_uppercase();
    let parts: Vec<&str> = s.split(['(', ')', ' ', ',']).filter(|p| !p.is_empty()).collect();
    
    let allowed = [
        "INT", "INTEGER", "SMALLINT", "BIGINT", "TINYINT",
        "VARCHAR", "CHAR", "TEXT", "LONGTEXT", "MEDIUMTEXT",
        "BOOLEAN", "BOOL",
        "DATE", "TIME", "TIMESTAMP", "DATETIME",
        "FLOAT", "DOUBLE", "DECIMAL", "NUMERIC",
        "UUID", "JSON", "JSONB", "BYTEA", "BLOB", "CLOB",
        "RAW", "NUMBER", "VARCHAR2", "NVARCHAR2", "NVARCHAR",
        "SERIAL", "BIGSERIAL", "SMALLSERIAL", "NCLOB", "BFILE"
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
    let allowed_funcs = [
        "NOW()", "CURRENT_TIMESTAMP", "CURRENT_DATE", "GETDATE()",
        "SYSDATE", "EMPTY_CLOB()", "EMPTY_BLOB()", "SYS_GUID()", "LOCALTIMESTAMP"
    ];
    allowed_funcs.contains(&s.to_uppercase().as_str())
}

pub fn make_unique_column_names<I, S>(names: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut unique_names: Vec<String> = Vec::new();
    let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for name in names {
        let base = name.as_ref().to_string();
        let count = seen.entry(base.clone()).or_insert(0);
        let unique = if *count == 0 {
            base.clone()
        } else {
            format!("{}_{}", base, count)
        };
        *count += 1;
        unique_names.push(unique);
    }
    unique_names
}

pub fn export_rows_to_csv(
    rows: &[std::collections::HashMap<String, serde_json::Value>],
    export_path: &str,
) -> Result<(), String> {
    if rows.is_empty() {
        return Ok(());
    }

    let mut wtr = csv::Writer::from_path(export_path).map_err(|e| e.to_string())?;

    // Header extraction
    let headers: Vec<String> = rows[0].keys().cloned().collect();
    wtr.write_record(&headers).map_err(|e| e.to_string())?;

    // Row rendering
    for row in rows {
        let record: Vec<String> = headers
            .iter()
            .map(|h| {
                let v = row.get(h).unwrap_or(&serde_json::Value::Null);
                match v {
                    serde_json::Value::Null => "".to_string(),
                    serde_json::Value::String(s) => s.clone(),
                    _ => v.to_string(),
                }
            })
            .collect();
        wtr.write_record(&record).map_err(|e| e.to_string())?;
    }

    wtr.flush().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn build_create_table_sql_generic<F>(
    qualified_table_name: &str,
    columns: &[super::TableColumn],
    quote_ident: F,
) -> Result<String, String>
where
    F: Fn(&str) -> String,
{
    let mut column_defs = Vec::new();
    for col in columns {
        if !is_safe_data_type(&col.data_type) {
            return Err(format!("Invalid or unsafe data type: {}", col.data_type));
        }
        let mut def = format!("{} {}", quote_ident(&col.name), col.data_type);
        if let Some(ref d) = col.default {
            if !is_safe_default(d) {
                return Err(format!("Invalid or unsafe default value: {}", d));
            }
            def.push_str(&format!(" DEFAULT {}", d));
        }
        if !col.nullable && !col.is_primary_key {
            def.push_str(" NOT NULL");
        }
        if col.is_primary_key {
            def.push_str(" PRIMARY KEY");
        }
        if let Some(ref fk) = col.foreign_key {
            let quoted_target_table = if fk.target_table.contains('.') {
                fk.target_table
                    .split('.')
                    .map(&quote_ident)
                    .collect::<Vec<String>>()
                    .join(".")
            } else {
                quote_ident(&fk.target_table)
            };
            let quoted_target_column = quote_ident(&fk.target_column);
            def.push_str(&format!(
                " REFERENCES {} ({})",
                quoted_target_table, quoted_target_column
            ));
        }
        column_defs.push(def);
    }

    Ok(format!(
        "CREATE TABLE {} ({})",
        qualified_table_name,
        column_defs.join(", ")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_unique_column_names() {
        let input = vec!["id", "name", "id", "created_at", "id", "name"];
        let expected = vec!["id", "name", "id_1", "created_at", "id_2", "name_1"];
        assert_eq!(make_unique_column_names(&input), expected);
    }

    #[test]
    fn test_oracle_safe_data_types() {
        assert!(is_safe_data_type("NCLOB"));
        assert!(is_safe_data_type("BFILE"));
        assert!(is_safe_data_type("VARCHAR2(255)"));
        assert!(is_safe_data_type("CLOB"));
    }

    #[test]
    fn test_oracle_safe_defaults() {
        assert!(is_safe_default("EMPTY_CLOB()"));
        assert!(is_safe_default("EMPTY_BLOB()"));
        assert!(is_safe_default("SYSDATE"));
        assert!(is_safe_default("SYS_GUID()"));
    }
}
