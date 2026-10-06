#[cfg(test)]
mod tests {
    use crate::drivers::oracle::OracleDriver;

    #[test]
    fn quotes_oracle_identifiers() {
        assert_eq!(OracleDriver::quote("USER_TABLE"), "\"USER_TABLE\"");
        assert_eq!(OracleDriver::quote("odd\"name"), "\"odd\"\"name\"");
    }

    #[test]
    fn builds_owner_qualified_table_name() {
        assert_eq!(
            OracleDriver::qualify_table("HR", "EMPLOYEES"),
            "\"HR\".\"EMPLOYEES\""
        );
        assert_eq!(
            OracleDriver::qualify_table("", "EMPLOYEES"),
            "\"EMPLOYEES\""
        );
        assert_eq!(
            OracleDriver::qualify_table("default", "EMPLOYEES"),
            "\"EMPLOYEES\""
        );
    }

    #[test]
    fn test_oracle_build_create_table_sql() {
        use crate::drivers::TableColumn;

        let columns = vec![
            TableColumn {
                name: "id".to_string(),
                data_type: "NUMBER".to_string(),
                nullable: false,
                is_primary_key: true,
                default: None,
                foreign_key: None,
            },
            TableColumn {
                name: "status".to_string(),
                data_type: "VARCHAR2(20)".to_string(),
                nullable: false,
                is_primary_key: false,
                default: Some("'active'".to_string()),
                foreign_key: None,
            },
        ];

        let safe_table = OracleDriver::qualify_table("default", "new_table");
        let sql = crate::drivers::utils::build_create_table_sql_generic(
            &safe_table,
            &columns,
            OracleDriver::quote,
        )
        .unwrap();
        assert_eq!(
            sql,
            "CREATE TABLE \"new_table\" (\"id\" NUMBER PRIMARY KEY, \"status\" VARCHAR2(20) DEFAULT 'active' NOT NULL)"
        );
    }

    #[test]
    fn builds_12c_pagination_clause() {
        assert_eq!(
            OracleDriver::pagination_clause(50, 100),
            " OFFSET 100 ROWS FETCH NEXT 50 ROWS ONLY"
        );
    }

    #[test]
    fn converts_json_to_oracle_bind() {
        use super::super::OracleBind;
        use serde_json::json;

        let obj = json!({"key": "value"});
        let bind = OracleDriver::json_to_bind(&obj);
        if let OracleBind::String(s) = bind {
            assert_eq!(s, "{\"key\":\"value\"}");
        } else {
            panic!("Expected OracleBind::String");
        }

        let arr = json!([1, 2, 3]);
        let bind_arr = OracleDriver::json_to_bind(&arr);
        if let OracleBind::String(s) = bind_arr {
            assert_eq!(s, "[1,2,3]");
        } else {
            panic!("Expected OracleBind::String");
        }

        let num = json!(42);
        let bind_num = OracleDriver::json_to_bind(&num);
        if let OracleBind::Int(i) = bind_num {
            assert_eq!(i, 42);
        } else {
            panic!("Expected OracleBind::Int");
        }
    }

    #[tokio::test]
    #[ignore]
    async fn fetch_table_data_twice_on_same_driver() {
        use crate::drivers::{Config, DatabaseDriver};

        let host = std::env::var("ORACLE_TEST_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = std::env::var("ORACLE_TEST_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(1521);
        let service = std::env::var("ORACLE_TEST_SERVICE").unwrap_or_else(|_| "XEPDB1".to_string());
        let user = std::env::var("ORACLE_TEST_USER").unwrap_or_else(|_| "SYSTEM".to_string());
        let password = std::env::var("ORACLE_TEST_PASSWORD").unwrap_or_else(|_| "oracle".to_string());
        let table = std::env::var("ORACLE_TEST_TABLE").unwrap_or_else(|_| "HELP".to_string());

        let config = Config {
            db_type: "oracle".to_string(),
            host,
            port,
            database: service,
            username: user,
            password,
            ssl: false,
            ssl_mode: None,
            connection_timeout: 10,
            oracle_connect_type: "service_name".to_string(),
            oracle_role: "normal".to_string(),
            display_all_databases: false,
        };

        let mut driver = OracleDriver::new();
        driver.connect(&config).await.expect("Failed to connect to Oracle");

        // First fetch (e.g. user opens table in sidebar)
        let first = driver
            .fetch_table_data(&table, 50, 0, "", "", "")
            .await
            .expect("First fetch failed");

        // Second fetch (e.g. user clicks Refresh or triggers reload)
        let second = driver
            .fetch_table_data(&table, 50, 0, "", "", "")
            .await
            .expect("Second fetch (refresh) failed");

        assert_eq!(first.total_count, second.total_count);
        assert_eq!(first.rows.len(), second.rows.len());

        driver.disconnect().await.expect("Failed to disconnect");
    }
}

