use super::*;
use crate::drivers::TableColumn;

#[test]
fn test_quote() {
    assert_eq!(PostgresDriver::quote("users"), "\"users\"");
    assert_eq!(PostgresDriver::quote("odd\"name"), "\"odd\"\"name\"");
}

#[test]
fn test_split_table_name() {
    assert_eq!(PostgresDriver::split_table_name("users"), ("public", "users"));
    assert_eq!(
        PostgresDriver::split_table_name("my_schema.users"),
        ("my_schema", "users")
    );
}

#[test]
fn test_qualified_table_name() {
    assert_eq!(
        PostgresDriver::qualified_table_name("users"),
        "\"public\".\"users\""
    );
    assert_eq!(
        PostgresDriver::qualified_table_name("my_schema.users"),
        "\"my_schema\".\"users\""
    );
}

#[test]
fn test_build_create_table_sql() {
    let columns = vec![
        TableColumn {
            name: "id".to_string(),
            data_type: "SERIAL".to_string(),
            nullable: false,
            is_primary_key: true,
            default: None,
            foreign_key: None,
        },
        TableColumn {
            name: "name".to_string(),
            data_type: "VARCHAR(255)".to_string(),
            nullable: true,
            is_primary_key: false,
            default: Some("'Guest'".to_string()),
            foreign_key: None,
        },
    ];

    let safe_table = PostgresDriver::qualified_table_name("public.users");
    let sql = crate::drivers::utils::build_create_table_sql_generic(
        &safe_table,
        &columns,
        PostgresDriver::quote,
    )
    .unwrap();
    assert_eq!(
        sql,
        "CREATE TABLE \"public\".\"users\" (\"id\" SERIAL PRIMARY KEY, \"name\" VARCHAR(255) DEFAULT 'Guest')"
    );
}

#[test]
fn test_build_drop_table_sql() {
    let sql = PostgresDriver::build_drop_table_sql("public.users", false);
    assert_eq!(sql, "DROP TABLE \"public\".\"users\"");
    let sql_cascade = PostgresDriver::build_drop_table_sql("public.users", true);
    assert_eq!(sql_cascade, "DROP TABLE \"public\".\"users\" CASCADE");
}

#[test]
fn test_unsafe_data_type() {
    let columns = vec![TableColumn {
        name: "id".to_string(),
        data_type: "INT; DROP TABLE users".to_string(),
        nullable: false,
        is_primary_key: true,
        default: None,
        foreign_key: None,
    }];

    let res = crate::drivers::utils::build_create_table_sql_generic(
        "\"users\"",
        &columns,
        PostgresDriver::quote,
    );
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("Invalid or unsafe data type"));
}
