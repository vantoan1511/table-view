use super::ddl::{qualify_table, quote};
use super::OracleDriver;
use crate::drivers::{
    DbIndex, ForeignKeyDef, SchemaDetails, SchemaObject, SchemaResult, TableColumn,
    TableConstraint, TableInfo, TableRelation,
};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

pub async fn get_schema(
    driver: &OracleDriver,
    all_databases: bool,
    schema_name: Option<&str>,
) -> Result<SchemaResult, String> {
    let current_schema = driver.current_schema();
    let is_sys = current_schema == "SYS";
    let requested_schema = schema_name
        .map(str::trim)
        .filter(|name| !name.is_empty() && !name.eq_ignore_ascii_case("default"))
        .map(|name| name.to_uppercase());

    // Always fetch all users (schemas)
    let schema_sql = if is_sys {
        "SELECT USERNAME FROM DBA_USERS ORDER BY USERNAME"
    } else {
        "SELECT USERNAME FROM ALL_USERS WHERE USERNAME NOT IN ('SYS', 'SYSTEM', 'OUTLN', 'XDB', 'CTXSYS', 'MDSYS') ORDER BY USERNAME"
    };
    let (schema_rows, _, _) = driver.run_query(schema_sql, &[]).await?;

    // Determine object filtering
    let (where_clause, params) = if all_databases {
        if is_sys {
            ("".to_string(), vec![])
        } else {
            (
                "WHERE OWNER NOT IN ('SYS', 'SYSTEM', 'OUTLN', 'XDB', 'CTXSYS', 'MDSYS')"
                    .to_string(),
                vec![],
            )
        }
    } else {
        let owner = requested_schema.unwrap_or_else(|| current_schema.to_string());
        (
            "WHERE OWNER = :1".to_string(),
            vec![JsonValue::String(owner)],
        )
    };

    log::info!(
        "oracle: loading schema objects (is_sys={}, all_databases={}, where={})",
        is_sys,
        all_databases,
        where_clause
    );

    let table_view = if is_sys { "DBA_TABLES" } else { "ALL_TABLES" };
    let view_view = if is_sys { "DBA_VIEWS" } else { "ALL_VIEWS" };
    let obj_view = if is_sys { "DBA_OBJECTS" } else { "ALL_OBJECTS" };

    let table_sql = format!(
        "SELECT TABLE_NAME, OWNER FROM {} {} ORDER BY OWNER, TABLE_NAME",
        table_view, where_clause
    );
    let view_sql = format!(
        "SELECT VIEW_NAME, OWNER FROM {} {} ORDER BY OWNER, VIEW_NAME",
        view_view, where_clause
    );
    let func_sql = format!(
        "SELECT OBJECT_NAME, OWNER, OBJECT_TYPE FROM {} {} {} OBJECT_TYPE IN ('PROCEDURE', 'FUNCTION') ORDER BY OWNER, OBJECT_NAME",
        obj_view,
        where_clause,
        if where_clause.is_empty() { "WHERE" } else { "AND" }
    );

    // Execute queries sequentially for Oracle to avoid connection flooding on admin accounts
    let (table_rows, _, _) = driver.run_query(&table_sql, &params).await?;
    let (view_rows, _, _) = driver.run_query(&view_sql, &params).await?;
    let (func_rows, _, _) = driver.run_query(&func_sql, &params).await?;

    let tables = table_rows
        .iter()
        .map(|r| SchemaObject {
            name: r
                .get("TABLE_NAME")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            schema: Some(
                r.get("OWNER")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            ),
            obj_type: None,
        })
        .collect();

    let views = view_rows
        .iter()
        .map(|r| SchemaObject {
            name: r
                .get("VIEW_NAME")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            schema: Some(
                r.get("OWNER")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            ),
            obj_type: None,
        })
        .collect();

    let functions = func_rows
        .iter()
        .map(|r| SchemaObject {
            name: r
                .get("OBJECT_NAME")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            schema: Some(
                r.get("OWNER")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            ),
            obj_type: Some(
                r.get("OBJECT_TYPE")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            ),
        })
        .collect();

    let mut schemas: Vec<SchemaObject> = schema_rows
        .iter()
        .map(|r| SchemaObject {
            name: r
                .get("USERNAME")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            schema: None,
            obj_type: None,
        })
        .collect();

    if !current_schema.is_empty()
        && !schemas
            .iter()
            .any(|s| s.name.eq_ignore_ascii_case(current_schema))
    {
        schemas.insert(
            0,
            SchemaObject {
                name: current_schema.to_string(),
                schema: None,
                obj_type: None,
            },
        );
    }

    let databases = if all_databases {
        None
    } else {
        Some(vec![SchemaObject {
            name: driver.service_name().to_string(),
            schema: None,
            obj_type: None,
        }])
    };

    Ok(SchemaResult {
        tables,
        views,
        functions,
        schemas: Some(schemas),
        databases,
    })
}

pub async fn get_table_columns(
    driver: &OracleDriver,
    table_name: &str,
) -> Result<Vec<TableColumn>, String> {
    let (owner, table) = driver.split_table_name(table_name);

    let fk_sql = "SELECT \
            cols.COLUMN_NAME as column_name, \
            r.TABLE_NAME as target_table, \
            r_cols.COLUMN_NAME as target_column \
        FROM ALL_CONSTRAINTS a \
        JOIN ALL_CONS_COLUMNS cols ON a.OWNER = cols.OWNER AND a.CONSTRAINT_NAME = cols.CONSTRAINT_NAME \
        JOIN ALL_CONSTRAINTS r ON a.R_OWNER = r.OWNER AND a.R_CONSTRAINT_NAME = r.CONSTRAINT_NAME \
        JOIN ALL_CONS_COLUMNS r_cols ON r.OWNER = r_cols.OWNER AND r.CONSTRAINT_NAME = r_cols.CONSTRAINT_NAME AND cols.POSITION = r_cols.POSITION \
        WHERE a.CONSTRAINT_TYPE = 'R' \
          AND a.OWNER = :1 AND a.TABLE_NAME = :2";

    let params = [
        JsonValue::String(owner.to_string()),
        JsonValue::String(table.to_string()),
    ];
    let (fk_rows, _, _) = driver.run_query(fk_sql, &params).await?;

    let mut fk_map: HashMap<String, ForeignKeyDef> = HashMap::new();
    for r in fk_rows {
        if let (Some(col), Some(tgt_tbl), Some(tgt_col)) = (
            r.get("COLUMN_NAME").and_then(|v| v.as_str()),
            r.get("TARGET_TABLE").and_then(|v| v.as_str()),
            r.get("TARGET_COLUMN").and_then(|v| v.as_str()),
        ) {
            fk_map.insert(
                col.to_string(),
                ForeignKeyDef {
                    target_table: tgt_tbl.to_string(),
                    target_column: tgt_col.to_string(),
                },
            );
        }
    }

    let sql = "SELECT \
            c.COLUMN_NAME, \
            c.DATA_TYPE, \
            c.NULLABLE, \
            c.DATA_DEFAULT, \
            CASE WHEN pk.COLUMN_NAME IS NULL THEN 'NO' ELSE 'YES' END AS IS_PRIMARY_KEY \
        FROM ALL_TAB_COLUMNS c \
        LEFT JOIN ( \
            SELECT cols.OWNER, cols.TABLE_NAME, cols.COLUMN_NAME \
            FROM ALL_CONSTRAINTS cons \
            JOIN ALL_CONS_COLUMNS cols ON cons.OWNER = cols.OWNER \
                AND cons.CONSTRAINT_NAME = cols.CONSTRAINT_NAME \
            WHERE cons.CONSTRAINT_TYPE = 'P' \
        ) pk ON pk.OWNER = c.OWNER AND pk.TABLE_NAME = c.TABLE_NAME AND pk.COLUMN_NAME = c.COLUMN_NAME \
        WHERE c.OWNER = :1 AND c.TABLE_NAME = :2 \
        ORDER BY c.COLUMN_ID";

    let (rows, _, _) = driver.run_query(sql, &params).await?;

    Ok(rows
        .iter()
        .map(|r| {
            let col_name = r
                .get("COLUMN_NAME")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let foreign_key = fk_map.get(&col_name).cloned();
            TableColumn {
                name: col_name,
                data_type: r
                    .get("DATA_TYPE")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                nullable: r
                    .get("NULLABLE")
                    .and_then(|v| v.as_str())
                    .map(|v| v == "Y")
                    .unwrap_or(true),
                is_primary_key: r
                    .get("IS_PRIMARY_KEY")
                    .and_then(|v| v.as_str())
                    .map(|v| v == "YES")
                    .unwrap_or(false),
                default: r
                    .get("DATA_DEFAULT")
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
                foreign_key,
            }
        })
        .collect())
}

pub async fn get_schema_details(
    driver: &OracleDriver,
    schema_name: &str,
) -> Result<SchemaDetails, String> {
    let owner = if schema_name.trim().is_empty() {
        driver.current_schema().to_string()
    } else {
        schema_name.to_uppercase()
    };

    let column_sql = "SELECT \
            c.TABLE_NAME as table_name, \
            c.COLUMN_NAME as column_name, \
            c.DATA_TYPE as data_type, \
            CASE WHEN c.NULLABLE = 'Y' THEN 'YES' ELSE 'NO' END as is_nullable, \
            c.DATA_DEFAULT as data_default, \
            CASE WHEN pk.COLUMN_NAME IS NULL THEN 'NO' ELSE 'YES' END AS IS_PRIMARY_KEY \
        FROM ALL_TAB_COLUMNS c \
        LEFT JOIN ( \
            SELECT cols.OWNER, cols.TABLE_NAME, cols.COLUMN_NAME \
            FROM ALL_CONSTRAINTS cons \
            JOIN ALL_CONS_COLUMNS cols ON cons.OWNER = cols.OWNER \
                AND cons.CONSTRAINT_NAME = cols.CONSTRAINT_NAME \
            WHERE cons.CONSTRAINT_TYPE = 'P' \
        ) pk ON pk.OWNER = c.OWNER AND pk.TABLE_NAME = c.TABLE_NAME AND pk.COLUMN_NAME = c.COLUMN_NAME \
        WHERE c.OWNER = :1 \
        ORDER BY c.TABLE_NAME, c.COLUMN_ID";

    let relation_sql = "SELECT \
            a.CONSTRAINT_NAME as constraint_name, \
            a.TABLE_NAME as source_table, \
            a_cols.COLUMN_NAME as source_column, \
            r.TABLE_NAME as target_table, \
            r_cols.COLUMN_NAME as target_column \
        FROM ALL_CONSTRAINTS a \
        JOIN ALL_CONS_COLUMNS a_cols ON a.OWNER = a_cols.OWNER AND a.CONSTRAINT_NAME = a_cols.CONSTRAINT_NAME \
        JOIN ALL_CONSTRAINTS r ON a.R_OWNER = r.OWNER AND a.R_CONSTRAINT_NAME = r.CONSTRAINT_NAME \
        JOIN ALL_CONS_COLUMNS r_cols ON r.OWNER = r_cols.OWNER AND r.CONSTRAINT_NAME = r_cols.CONSTRAINT_NAME AND a_cols.POSITION = r_cols.POSITION \
        WHERE a.CONSTRAINT_TYPE = 'R' \
          AND a.OWNER = :1";

    let params = [JsonValue::String(owner)];

    // Run queries sequentially for Oracle to avoid connection issues
    let (column_rows, _, _) = driver.run_query(column_sql, &params).await?;
    let (relation_rows, _, _) = driver.run_query(relation_sql, &params).await?;

    let mut tables_map: HashMap<String, Vec<TableColumn>> = HashMap::new();
    let mut table_order: Vec<String> = Vec::new();

    let mut fk_map: HashMap<(String, String), ForeignKeyDef> = HashMap::new();
    for r in &relation_rows {
        if let (Some(src_tbl), Some(src_col), Some(tgt_tbl), Some(tgt_col)) = (
            r.get("SOURCE_TABLE").and_then(|v| v.as_str()),
            r.get("SOURCE_COLUMN").and_then(|v| v.as_str()),
            r.get("TARGET_TABLE").and_then(|v| v.as_str()),
            r.get("TARGET_COLUMN").and_then(|v| v.as_str()),
        ) {
            fk_map.insert(
                (src_tbl.to_string(), src_col.to_string()),
                ForeignKeyDef {
                    target_table: tgt_tbl.to_string(),
                    target_column: tgt_col.to_string(),
                },
            );
        }
    }

    for r in column_rows {
        let table_name = r
            .get("TABLE_NAME")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let col_name = r
            .get("COLUMN_NAME")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let foreign_key = fk_map.get(&(table_name.clone(), col_name.clone())).cloned();
        let col = TableColumn {
            name: col_name,
            data_type: r
                .get("DATA_TYPE")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            nullable: r
                .get("IS_NULLABLE")
                .and_then(|v| v.as_str())
                .map(|s| s == "YES")
                .unwrap_or(true),
            is_primary_key: r
                .get("IS_PRIMARY_KEY")
                .and_then(|v| v.as_str())
                .map(|s| s == "YES")
                .unwrap_or(false),
            default: r
                .get("DATA_DEFAULT")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            foreign_key,
        };

        if !tables_map.contains_key(&table_name) {
            table_order.push(table_name.clone());
        }
        tables_map.entry(table_name).or_default().push(col);
    }

    let mut tables = Vec::new();
    for name in table_order {
        if let Some(columns) = tables_map.remove(&name) {
            tables.push(TableInfo { name, columns });
        }
    }

    let mut relations = Vec::new();
    for r in relation_rows {
        relations.push(TableRelation {
            constraint_name: r
                .get("CONSTRAINT_NAME")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            source_table: r
                .get("SOURCE_TABLE")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            source_column: r
                .get("SOURCE_COLUMN")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            target_table: r
                .get("TARGET_TABLE")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            target_column: r
                .get("TARGET_COLUMN")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        });
    }

    Ok(SchemaDetails { tables, relations })
}

pub async fn get_table_indexes(
    driver: &OracleDriver,
    table_name: &str,
) -> Result<Vec<DbIndex>, String> {
    let (owner, table) = driver.split_table_name(table_name);

    let sql = r#"
        SELECT 
            i.INDEX_NAME as index_name,
            i.UNIQUENESS as uniqueness,
            i.INDEX_TYPE as index_type,
            (SELECT COUNT(*) FROM ALL_CONSTRAINTS c WHERE c.OWNER = i.TABLE_OWNER AND c.TABLE_NAME = i.TABLE_NAME AND c.INDEX_NAME = i.INDEX_NAME AND c.CONSTRAINT_TYPE = 'P') as is_primary,
            (
                SELECT JSON_ARRAYAGG(ic.COLUMN_NAME ORDER BY ic.COLUMN_POSITION)
                FROM ALL_IND_COLUMNS ic
                WHERE ic.INDEX_OWNER = i.OWNER AND ic.INDEX_NAME = i.INDEX_NAME
            ) as columns
        FROM ALL_INDEXES i
        WHERE i.TABLE_OWNER = :1 AND i.TABLE_NAME = :2
        ORDER BY i.INDEX_NAME
    "#;

    let (rows, _, _) = driver
        .run_query(
            sql,
            &[
                JsonValue::String(owner.to_string()),
                JsonValue::String(table.to_string()),
            ],
        )
        .await?;

    let mut indexes = Vec::new();
    for r in rows {
        let mut cols = vec![];
        if let Some(val) = r.get("COLUMNS") {
            if let JsonValue::Array(arr) = val {
                cols = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
            } else if let JsonValue::String(s) = val {
                if let Ok(JsonValue::Array(arr)) = serde_json::from_str(s) {
                    cols = arr
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                }
            }
        }

        let name = r
            .get("INDEX_NAME")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let uniqueness = r.get("UNIQUENESS").and_then(|v| v.as_str()).unwrap_or("");
        let is_unique = uniqueness == "UNIQUE";
        let is_primary_key = r
            .get("IS_PRIMARY")
            .and_then(|v| {
                if let JsonValue::Number(n) = v {
                    Some(n.as_i64().unwrap_or(0) > 0)
                } else if let JsonValue::String(s) = v {
                    s.parse::<i64>().ok().map(|n| n > 0)
                } else {
                    None
                }
            })
            .unwrap_or(false);

        let ddl = if is_primary_key {
            format!("PRIMARY KEY ({})", cols.join(", "))
        } else if is_unique {
            format!(
                "CREATE UNIQUE INDEX {} ON {} ({})",
                quote(&name),
                qualify_table(owner, table),
                cols.join(", ")
            )
        } else {
            format!(
                "CREATE INDEX {} ON {} ({})",
                quote(&name),
                qualify_table(owner, table),
                cols.join(", ")
            )
        };

        indexes.push(DbIndex {
            name,
            is_unique,
            is_primary_key,
            index_type: r
                .get("INDEX_TYPE")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            ddl: Some(ddl),
            columns: cols,
        });
    }
    Ok(indexes)
}

struct RawConstraintInfo {
    constraint_type: String,
    columns: Vec<String>,
    referenced_columns: Vec<String>,
    referenced_table: String,
    search_condition: String,
}

pub async fn get_table_constraints(
    driver: &OracleDriver,
    table_name: &str,
) -> Result<Vec<TableConstraint>, String> {
    let (owner, table) = driver.split_table_name(table_name);

    let params = [
        JsonValue::String(owner.to_string()),
        JsonValue::String(table.to_string()),
    ];

    let sql = "SELECT \
        c.CONSTRAINT_NAME AS constraint_name, \
        c.CONSTRAINT_TYPE AS constraint_type, \
        cc.COLUMN_NAME AS column_name, \
        c_pk.TABLE_NAME AS r_table, \
        cc_pk.COLUMN_NAME AS r_column, \
        c.SEARCH_CONDITION AS search_condition \
    FROM ALL_CONSTRAINTS c \
    LEFT JOIN ALL_CONS_COLUMNS cc \
      ON c.CONSTRAINT_NAME = cc.CONSTRAINT_NAME \
      AND c.OWNER = cc.OWNER \
    LEFT JOIN ALL_CONSTRAINTS c_pk \
      ON c.R_CONSTRAINT_NAME = c_pk.CONSTRAINT_NAME \
      AND c.R_OWNER = c_pk.OWNER \
    LEFT JOIN ALL_CONS_COLUMNS cc_pk \
      ON c_pk.CONSTRAINT_NAME = cc_pk.CONSTRAINT_NAME \
      AND c_pk.OWNER = cc_pk.OWNER \
      AND cc.POSITION = cc_pk.POSITION \
    WHERE c.OWNER = :1 AND c.TABLE_NAME = :2 \
    ORDER BY c.CONSTRAINT_NAME, cc.POSITION";

    let (rows, _, _) = driver.run_query(sql, &params).await?;

    let mut constraint_map: HashMap<String, RawConstraintInfo> = HashMap::new();
    let mut constraint_order = Vec::new();

    for r in rows {
        let name = r
            .get("CONSTRAINT_NAME")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let c_type = r
            .get("CONSTRAINT_TYPE")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let col = r
            .get("COLUMN_NAME")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let r_tbl = r
            .get("R_TABLE")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let r_col = r
            .get("R_COLUMN")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let condition = r
            .get("SEARCH_CONDITION")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        if !constraint_map.contains_key(&name) {
            constraint_order.push(name.clone());
        }

        let entry = constraint_map
            .entry(name)
            .or_insert_with(|| RawConstraintInfo {
                constraint_type: c_type,
                columns: Vec::new(),
                referenced_columns: Vec::new(),
                referenced_table: r_tbl,
                search_condition: condition,
            });
        if !col.is_empty() && !entry.columns.contains(&col) {
            entry.columns.push(col);
        }
        if !r_col.is_empty() && !entry.referenced_columns.contains(&r_col) {
            entry.referenced_columns.push(r_col);
        }
    }

    let mut constraints = Vec::new();
    for name in constraint_order {
        if let Some(info) = constraint_map.remove(&name) {
            let mapped_type = match info.constraint_type.as_str() {
                "P" => "PRIMARY KEY".to_string(),
                "R" => "FOREIGN KEY".to_string(),
                "U" => "UNIQUE".to_string(),
                "C" => "CHECK".to_string(),
                _ => info.constraint_type,
            };

            let definition = match mapped_type.as_str() {
                "PRIMARY KEY" => format!("PRIMARY KEY ({})", info.columns.join(", ")),
                "UNIQUE" => format!("UNIQUE ({})", info.columns.join(", ")),
                "FOREIGN KEY" => format!(
                    "FOREIGN KEY ({}) REFERENCES {} ({})",
                    info.columns.join(", "),
                    info.referenced_table,
                    info.referenced_columns.join(", ")
                ),
                "CHECK" => info.search_condition,
                _ => format!("{} ({})", mapped_type, info.columns.join(", ")),
            };

            constraints.push(TableConstraint {
                name,
                constraint_type: mapped_type,
                definition,
            });
        }
    }

    Ok(constraints)
}
