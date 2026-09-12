use super::OracleDriver;

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
    use super::OracleBind;
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
