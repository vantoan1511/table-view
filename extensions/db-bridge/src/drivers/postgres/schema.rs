use super::data::execute_query;
use super::ddl::split_table_name;
use crate::drivers::{
    DbIndex, ForeignKeyDef, SchemaDetails, SchemaObject, SchemaResult, TableColumn,
    TableConstraint, TableInfo, TableRelation,
};
use serde_json::Value;
use sqlx::postgres::PgPool;
use std::collections::HashMap;

pub async fn get_schema(
    pool: &PgPool,
    all_databases: bool,
    schema_name: Option<&str>,
) -> Result<SchemaResult, String> {
    let (where_clause, params) = if let Some(s) = schema_name {
        (
            "WHERE table_schema::text = $1::text".to_string(),
            vec![Value::String(s.to_string())],
        )
    } else {
        (
            "WHERE table_schema NOT IN ('information_schema') AND table_schema NOT LIKE 'pg_%'"
                .to_string(),
            vec![],
        )
    };

    // Run all schema queries in parallel
    let table_sql = format!(
        "SELECT table_name::text, table_schema::text FROM information_schema.tables {} AND table_type = 'BASE TABLE' ORDER BY table_name",
        where_clause
    );
    let view_sql = format!(
        "SELECT table_name::text, table_schema::text FROM information_schema.tables {} AND table_type = 'VIEW' ORDER BY table_name",
        where_clause
    );
    let routine_where = where_clause.replace("table_schema", "routine_schema");
    let func_sql = format!(
        "SELECT routine_name::text, routine_schema::text, data_type::text FROM information_schema.routines {} ORDER BY routine_name",
        routine_where
    );
    let schema_sql = "SELECT schema_name::text FROM information_schema.schemata WHERE schema_name NOT IN ('information_schema') AND schema_name NOT LIKE 'pg_%' ORDER BY schema_name";

    // Optional database list
    let db_sql = if all_databases {
        "SELECT datname::text as name FROM pg_database WHERE datistemplate = false ORDER BY datname"
    } else {
        "SELECT current_database()::text as name"
    };

    let (table_res, view_res, func_res, schema_res, db_res) = tokio::join!(
        execute_query(pool, &table_sql, &params),
        execute_query(pool, &view_sql, &params),
        execute_query(pool, &func_sql, &params),
        execute_query(pool, schema_sql, &[]),
        execute_query(pool, db_sql, &[])
    );

    let (table_rows, _, _) = table_res?;
    let (view_rows, _, _) = view_res?;
    let (func_rows, _, _) = func_res?;
    let (schema_rows, _, _) = schema_res?;
    let (db_rows, _, _) = db_res?;

    let tables: Vec<SchemaObject> = table_rows
        .iter()
        .map(|r| SchemaObject {
            name: r
                .get("table_name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            schema: Some(
                r.get("table_schema")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            ),
            obj_type: None,
        })
        .collect();

    let views: Vec<SchemaObject> = view_rows
        .iter()
        .map(|r| SchemaObject {
            name: r
                .get("table_name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            schema: Some(
                r.get("table_schema")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            ),
            obj_type: None,
        })
        .collect();

    let functions: Vec<SchemaObject> = func_rows
        .iter()
        .map(|r| SchemaObject {
            name: r
                .get("routine_name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            schema: Some(
                r.get("routine_schema")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            ),
            obj_type: Some(
                r.get("data_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            ),
        })
        .collect();

    let schemas: Vec<SchemaObject> = schema_rows
        .iter()
        .map(|r| SchemaObject {
            name: r
                .get("schema_name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            schema: None,
            obj_type: None,
        })
        .collect();

    let databases: Vec<SchemaObject> = db_rows
        .iter()
        .map(|r| SchemaObject {
            name: r
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            schema: None,
            obj_type: None,
        })
        .collect();

    Ok(SchemaResult {
        tables,
        views,
        functions,
        schemas: Some(schemas),
        databases: Some(databases),
    })
}

pub async fn get_table_columns(
    pool: &PgPool,
    table_name: &str,
) -> Result<Vec<TableColumn>, String> {
    let (schema_name, bare_table_name) = split_table_name(table_name);

    let fk_sql = "SELECT \
        kcu.column_name::text as source_column, \
        CASE \
            WHEN ccu.table_schema::text = tc.table_schema::text THEN ccu.table_name::text \
            ELSE (ccu.table_schema::text || '.' || ccu.table_name::text) \
        END as target_table, \
        ccu.column_name::text as target_column \
    FROM information_schema.table_constraints tc \
    JOIN information_schema.key_column_usage kcu \
        ON tc.constraint_name = kcu.constraint_name \
        AND tc.table_schema = kcu.table_schema \
    JOIN information_schema.constraint_column_usage ccu \
        ON ccu.constraint_name = tc.constraint_name \
        AND ccu.table_schema = tc.table_schema \
    WHERE tc.constraint_type = 'FOREIGN KEY' \
      AND tc.table_schema::text = $1::text \
      AND tc.table_name::text = $2::text";

    let (fk_rows, _, _) = execute_query(
        pool,
        fk_sql,
        &[
            Value::String(schema_name.to_string()),
            Value::String(bare_table_name.to_string()),
        ],
    )
    .await
    .unwrap_or((vec![], vec![], 0));

    let mut fk_map = HashMap::new();
    for r in fk_rows {
        if let (Some(src_col), Some(tgt_tbl), Some(tgt_col)) = (
            r.get("source_column").and_then(|v| v.as_str()),
            r.get("target_table").and_then(|v| v.as_str()),
            r.get("target_column").and_then(|v| v.as_str()),
        ) {
            fk_map.insert(
                src_col.to_string(),
                ForeignKeyDef {
                    target_table: tgt_tbl.to_string(),
                    target_column: tgt_col.to_string(),
                },
            );
        }
    }

    let sql = "SELECT column_name::text, data_type::text, is_nullable::text, column_default::text \
         FROM information_schema.columns \
         WHERE table_schema::text = $1::text AND table_name::text = $2::text \
         ORDER BY ordinal_position";

    let (rows, _, _) = execute_query(
        pool,
        sql,
        &[
            Value::String(schema_name.to_string()),
            Value::String(bare_table_name.to_string()),
        ],
    )
    .await?;

    let mut columns = Vec::new();
    for r in rows {
        let col_name = r
            .get("column_name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let foreign_key = fk_map.get(&col_name).cloned();
        columns.push(TableColumn {
            name: col_name,
            data_type: r
                .get("data_type")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            nullable: r
                .get("is_nullable")
                .and_then(|v| v.as_str())
                .map(|s| s == "YES")
                .unwrap_or(true),
            is_primary_key: false,
            default: r
                .get("column_default")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            foreign_key,
        });
    }
    Ok(columns)
}

pub async fn get_schema_details(pool: &PgPool, schema_name: &str) -> Result<SchemaDetails, String> {
    let column_sql = "SELECT \
            c.table_name::text, \
            c.column_name::text, \
            c.udt_name::text as data_type, \
            (c.is_nullable = 'YES') as is_nullable, \
            c.column_default::text as column_default, \
            (pk.column_name IS NOT NULL) as is_primary_key \
        FROM information_schema.columns c \
        LEFT JOIN ( \
            SELECT kcu.table_schema, kcu.table_name, kcu.column_name \
            FROM information_schema.table_constraints tc \
            JOIN information_schema.key_column_usage kcu \
                ON tc.constraint_name = kcu.constraint_name \
                AND tc.table_schema = kcu.table_schema \
                AND tc.table_name = kcu.table_name \
            WHERE tc.constraint_type = 'PRIMARY KEY' \
                AND tc.table_schema::text = $1::text \
        ) pk ON pk.table_schema = c.table_schema \
            AND pk.table_name = c.table_name \
            AND pk.column_name = c.column_name \
        WHERE c.table_schema::text = $1::text \
        ORDER BY c.table_name, c.ordinal_position";

    let relation_sql = "SELECT \
            tc.constraint_name::text, \
            tc.table_name::text as source_table, \
            kcu.column_name::text as source_column, \
            ccu.table_name::text as target_table, \
            ccu.column_name::text as target_column \
        FROM information_schema.table_constraints tc \
        JOIN information_schema.key_column_usage kcu \
            ON tc.constraint_name = kcu.constraint_name \
            AND tc.table_schema = kcu.table_schema \
        JOIN information_schema.constraint_column_usage ccu \
            ON ccu.constraint_name = tc.constraint_name \
            AND ccu.table_schema = tc.table_schema \
        WHERE tc.constraint_type = 'FOREIGN KEY' \
          AND tc.table_schema::text = $1::text";

    let params = [Value::String(schema_name.to_string())];

    let (column_res, relation_res) = tokio::join!(
        execute_query(pool, column_sql, &params),
        execute_query(pool, relation_sql, &params)
    );

    let (column_rows, _, _) = column_res?;
    let (relation_rows, _, _) = relation_res?;

    // Group columns by table_name to build TableInfo objects
    let mut tables_map: HashMap<String, Vec<TableColumn>> = HashMap::new();
    let mut table_order: Vec<String> = Vec::new();

    let mut fk_map: HashMap<(String, String), ForeignKeyDef> = HashMap::new();
    for r in &relation_rows {
        if let (Some(src_tbl), Some(src_col), Some(tgt_tbl), Some(tgt_col)) = (
            r.get("source_table").and_then(|v| v.as_str()),
            r.get("source_column").and_then(|v| v.as_str()),
            r.get("target_table").and_then(|v| v.as_str()),
            r.get("target_column").and_then(|v| v.as_str()),
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
            .get("table_name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let col_name = r
            .get("column_name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let foreign_key = fk_map
            .get(&(table_name.clone(), col_name.clone()))
            .cloned();
        let col = TableColumn {
            name: col_name,
            data_type: r
                .get("data_type")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            nullable: r
                .get("is_nullable")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
            is_primary_key: r
                .get("is_primary_key")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            default: r
                .get("column_default")
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
                .get("constraint_name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            source_table: r
                .get("source_table")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            source_column: r
                .get("source_column")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            target_table: r
                .get("target_table")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            target_column: r
                .get("target_column")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        });
    }

    Ok(SchemaDetails { tables, relations })
}

pub async fn get_table_indexes(pool: &PgPool, table_name: &str) -> Result<Vec<DbIndex>, String> {
    let (schema_name, bare_table_name) = split_table_name(table_name);
    let sql = r#"
        SELECT
            i.relname as index_name,
            ix.indisunique as is_unique,
            ix.indisprimary as is_primary,
            am.amname as index_type,
            pg_get_indexdef(i.oid) as ddl,
            array_to_json(array_agg(a.attname ORDER BY c.ord)) as columns
        FROM pg_class t
        JOIN pg_index ix ON t.oid = ix.indrelid
        JOIN pg_class i ON i.oid = ix.indexrelid
        JOIN pg_am am ON i.relam = am.oid
        JOIN pg_namespace n ON n.oid = t.relnamespace
        CROSS JOIN LATERAL unnest(ix.indkey) WITH ORDINALITY AS c(colnum, ord)
        LEFT JOIN pg_attribute a ON t.oid = a.attrelid AND a.attnum = c.colnum
        WHERE t.relname = $2 AND n.nspname = $1
        GROUP BY i.relname, ix.indisunique, ix.indisprimary, am.amname, i.oid, pg_get_indexdef(i.oid)
        ORDER BY i.relname
    "#;

    let (rows, _, _) = execute_query(
        pool,
        sql,
        &[
            Value::String(schema_name.to_string()),
            Value::String(bare_table_name.to_string()),
        ],
    )
    .await?;

    let mut indexes = Vec::new();
    for r in rows {
        let mut cols = vec![];
        if let Some(val) = r.get("columns") {
            if let Value::Array(arr) = val {
                cols = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
            } else if let Value::String(s) = val {
                if let Ok(Value::Array(arr)) = serde_json::from_str(s) {
                    cols = arr
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                }
            }
        }

        indexes.push(DbIndex {
            name: r
                .get("index_name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            is_unique: r
                .get("is_unique")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            is_primary_key: r
                .get("is_primary")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            index_type: r
                .get("index_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            ddl: r
                .get("ddl")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            columns: cols,
        });
    }
    Ok(indexes)
}

pub async fn get_table_constraints(
    pool: &PgPool,
    table_name: &str,
) -> Result<Vec<TableConstraint>, String> {
    let (schema_name, bare_table_name) = split_table_name(table_name);
    let sql = r#"
        SELECT
            con.conname AS constraint_name,
            CASE con.contype
                WHEN 'p' THEN 'PRIMARY KEY'
                WHEN 'f' THEN 'FOREIGN KEY'
                WHEN 'u' THEN 'UNIQUE'
                WHEN 'c' THEN 'CHECK'
                ELSE con.contype::text
            END AS constraint_type,
            pg_get_constraintdef(con.oid) AS definition
        FROM pg_constraint con
        JOIN pg_class rel ON rel.oid = con.conrelid
        JOIN pg_namespace nsp ON nsp.oid = rel.relnamespace
        WHERE rel.relname::text = $2::text AND nsp.nspname::text = $1::text
        ORDER BY con.conname
    "#;

    let (rows, _, _) = execute_query(
        pool,
        sql,
        &[
            Value::String(schema_name.to_string()),
            Value::String(bare_table_name.to_string()),
        ],
    )
    .await?;

    let mut constraints = Vec::new();
    for r in rows {
        constraints.push(TableConstraint {
            name: r
                .get("constraint_name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            constraint_type: r
                .get("constraint_type")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            definition: r
                .get("definition")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        });
    }
    Ok(constraints)
}
