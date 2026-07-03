use super::{
    AlterOperation, ColumnInfo, Config, DatabaseDriver, QueryResult, SchemaObject, SchemaResult,
    TableColumn, TableDataResult, SchemaDetails, TableInfo, TableRelation, ForeignKeyDef, DbIndex, TableConstraint
};
use crate::bind_json_value;
use async_trait::async_trait;
use chrono;
use serde_json::Value;
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::{Column, Row, TypeInfo, ValueRef};
use std::collections::HashMap;
use uuid;

pub struct PostgresDriver {
    pool: Option<PgPool>,
    tx_conn: tokio::sync::Mutex<Option<sqlx::pool::PoolConnection<sqlx::Postgres>>>,
}

impl PostgresDriver {
    pub fn new() -> Self {
        Self {
            pool: None,
            tx_conn: tokio::sync::Mutex::new(None),
        }
    }

    fn pool(&self) -> Result<&PgPool, String> {
        self.pool.as_ref().ok_or("Not connected".to_string())
    }

    fn quote(ident: &str) -> String {
        format!("\"{}\"", ident.replace('"', "\"\""))
    }

    fn split_table_name(table_name: &str) -> (&str, &str) {
        table_name
            .rsplit_once('.')
            .unwrap_or(("public", table_name))
    }

    fn qualified_table_name(table_name: &str) -> String {
        let (schema, table) = Self::split_table_name(table_name);
        format!("{}.{}", Self::quote(schema), Self::quote(table))
    }

    async fn execute_query(
        pool: &PgPool,
        sql: &str,
        params: &[Value],
    ) -> Result<(Vec<HashMap<String, Value>>, Vec<ColumnInfo>, u64), String> {
        let mut q = sqlx::query(sql);
        for p in params {
            q = bind_json_value!(q, p);
        }
        log::info!("postgres: executing query: {}", sql);
        let start = std::time::Instant::now();
        let rows = q.fetch_all(pool).await.map_err(|e| e.to_string())?;
        let elapsed = start.elapsed().as_millis() as u64;
        log::info!("postgres: query finished in {}ms", elapsed);

        if rows.is_empty() {
            return Ok((vec![], vec![], elapsed));
        }

        let orig_names: Vec<String> = rows[0].columns().iter().map(|c| c.name().to_string()).collect();
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
                let val = Self::get_column_value(&row, i);
                map.insert(unique_name.clone(), val);
            }
            data.push(map);
        }

        Ok((data, fields, elapsed))
    }

    fn get_column_value(row: &PgRow, i: usize) -> Value {
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

        match name {
            "INT2" => {
                if let Ok(v) = row.try_get::<i16, _>(i) {
                    return Value::Number(i64::from(v).into());
                }
            }
            "INT4" => {
                if let Ok(v) = row.try_get::<i32, _>(i) {
                    return Value::Number(i64::from(v).into());
                }
            }
            "INT8" => {
                if let Ok(v) = row.try_get::<i64, _>(i) {
                    return Value::Number(v.into());
                }
            }
            "OID" => {
                if let Ok(v) = row.try_get::<i32, _>(i) {
                    return Value::Number(i64::from(v).into());
                }
            }
            "FLOAT4" | "FLOAT8" | "NUMERIC" => {
                if let Ok(v) = row.try_get::<f64, _>(i) {
                    if let Some(num) = serde_json::Number::from_f64(v) {
                        return Value::Number(num);
                    }
                }
            }
            "BOOL" => {
                if let Ok(v) = row.try_get::<bool, _>(i) {
                    return Value::Bool(v);
                }
            }
            "TEXT" | "VARCHAR" | "BPCHAR" | "NAME" => {
                if let Ok(v) = row.try_get::<String, _>(i) {
                    return Value::String(v);
                }
            }
            "UUID" => {
                if let Ok(v) = row.try_get::<uuid::Uuid, _>(i) {
                    return Value::String(v.to_string());
                }
            }
            "TIMESTAMP" | "TIMESTAMPTZ" => {
                if let Ok(v) = row.try_get::<chrono::NaiveDateTime, _>(i) {
                    return Value::String(v.to_string());
                }
                if let Ok(v) = row.try_get::<chrono::DateTime<chrono::Utc>, _>(i) {
                    return Value::String(v.to_string());
                }
            }
            "DATE" => {
                if let Ok(v) = row.try_get::<chrono::NaiveDate, _>(i) {
                    return Value::String(v.to_string());
                }
            }
            "JSON" | "JSONB" => {
                if let Ok(v) = row.try_get::<Value, _>(i) {
                    return v;
                }
            }
            "BYTEA" => {
                if let Ok(bytes) = row.try_get::<Vec<u8>, _>(i) {
                    return Value::String(hex::encode(bytes));
                }
            }
            _ => {}
        }

        // Final fallback: try as string
        if let Ok(v) = row.try_get::<String, _>(i) {
            return Value::String(v);
        }

        Value::Null
    }


    fn build_drop_table_sql(table_name: &str, cascade: bool) -> String {
        let safe_table = Self::qualified_table_name(table_name);
        if cascade {
            format!("DROP TABLE {} CASCADE", safe_table)
        } else {
            format!("DROP TABLE {}", safe_table)
        }
    }
}

#[async_trait]
impl DatabaseDriver for PostgresDriver {
    async fn connect(&mut self, config: &Config) -> Result<(), String> {
        let ssl_mode = if config.ssl { "require" } else { "disable" };
        let host = if config.host.contains(':') && !config.host.starts_with('[') {
            format!("[{}]", config.host)
        } else {
            config.host.clone()
        };

        let dsn = format!(
            "postgres://{}:{}@{}:{}/{}?sslmode={}&options=-c%20search_path=public&application_name=db_manager&connect_timeout={}&tcp_user_timeout=5000",
            urlencoding::encode(&config.username),
            urlencoding::encode(&config.password),
            host,
            config.port,
            urlencoding::encode(&config.database),
            ssl_mode,
            config.connection_timeout
        );

        use std::str::FromStr;
        let mut connect_options = sqlx::postgres::PgConnectOptions::from_str(&dsn)
            .map_err(|e| e.to_string())?;
        connect_options = connect_options.statement_cache_capacity(0);

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(
                config.connection_timeout as u64,
            ))
            .connect_with(connect_options)
            .await
            .map_err(|e| e.to_string())?;

        self.pool = Some(pool);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), String> {
        {
            let mut tx = self.tx_conn.lock().await;
            *tx = None;
        }
        if let Some(pool) = self.pool.take() {
            pool.close().await;
        }
        Ok(())
    }

    async fn get_schema(
        &self,
        all_databases: bool,
        schema_name: Option<&str>,
    ) -> Result<SchemaResult, String> {
        let pool = self.pool()?;

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
            Self::execute_query(pool, &table_sql, &params),
            Self::execute_query(pool, &view_sql, &params),
            Self::execute_query(pool, &func_sql, &params),
            Self::execute_query(pool, schema_sql, &[]),
            Self::execute_query(pool, db_sql, &[])
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

    async fn fetch_table_data(
        &self,
        table_name: &str,
        limit: i64,
        offset: i64,
        sort_column: &str,
        sort_direction: &str,
        filter: &str,
    ) -> Result<TableDataResult, String> {
        let pool = self.pool()?;
        let (schema_name, bare_table_name) = Self::split_table_name(table_name);
        let safe_table = Self::qualified_table_name(table_name);

        let column_sql = "SELECT \
                c.column_name::text, \
                c.udt_name::text as data_type, \
                (c.is_nullable = 'YES') as is_nullable, \
                EXISTS ( \
                    SELECT 1 \
                    FROM information_schema.table_constraints tc \
                    JOIN information_schema.key_column_usage kcu \
                        ON tc.constraint_name = kcu.constraint_name \
                        AND tc.table_schema = kcu.table_schema \
                        AND tc.table_name = kcu.table_name \
                    WHERE tc.constraint_type = 'PRIMARY KEY' \
                        AND tc.table_schema = c.table_schema \
                        AND tc.table_name = c.table_name \
                        AND kcu.column_name = c.column_name \
                ) as is_primary_key \
            FROM information_schema.columns c \
            WHERE c.table_schema::text = $1::text AND c.table_name::text = $2::text \
            ORDER BY c.ordinal_position";

        let where_clause = if !filter.trim().is_empty() {
            format!(" WHERE {}", filter.trim())
        } else {
            String::new()
        };

        let order_clause = if !sort_column.is_empty() {
            let dir = if sort_direction.eq_ignore_ascii_case("desc") {
                "DESC"
            } else {
                "ASC"
            };
            format!(" ORDER BY {} {}", Self::quote(sort_column), dir)
        } else {
            String::new()
        };

        let data_sql = format!(
            "SELECT * FROM {}{}{} LIMIT $1 OFFSET $2",
            safe_table, where_clause, order_clause
        );
        let count_sql = format!("SELECT COUNT(*) FROM {}{}", safe_table, where_clause);

        let column_params = [
            Value::String(schema_name.to_string()),
            Value::String(bare_table_name.to_string()),
        ];
        let data_params = [Value::Number(limit.into()), Value::Number(offset.into())];
        let count_params = [];

        // Run all three queries in parallel
        let (column_res, data_res, count_res) = tokio::join!(
            Self::execute_query(pool, column_sql, &column_params),
            Self::execute_query(pool, &data_sql, &data_params),
            Self::execute_query(pool, &count_sql, &count_params)
        );

        let (column_rows, _, _) = column_res?;
        let (data, mut fields, elapsed) = data_res?;
        let (count_rows, _, _) = count_res?;

        let column_meta: Vec<ColumnInfo> = column_rows
            .iter()
            .map(|r| ColumnInfo {
                name: r
                    .get("column_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                data_type: r
                    .get("data_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                is_primary_key: r
                    .get("is_primary_key")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                is_nullable: r
                    .get("is_nullable")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
                display_name: None,
            })
            .collect();

        if !column_meta.is_empty() {
            let metadata_by_name: std::collections::HashMap<String, ColumnInfo> = column_meta
                .iter()
                .map(|column| (column.name.clone(), column.clone()))
                .collect();

            if fields.is_empty() {
                fields = column_meta;
            } else {
                for field in &mut fields {
                    if let Some(metadata) = metadata_by_name.get(&field.name) {
                        field.is_primary_key = metadata.is_primary_key;
                        field.is_nullable = metadata.is_nullable;
                        field.data_type = metadata.data_type.clone();
                    }
                }
            }
        }

        let total = count_rows
            .first()
            .and_then(|r| r.values().next())
            .and_then(|v| {
                if let Value::Number(n) = v {
                    n.as_i64()
                } else if let Value::String(s) = v {
                    s.parse::<i64>().ok()
                } else {
                    None
                }
            })
            .unwrap_or(0);

        Ok(TableDataResult {
            rows: data,
            fields,
            total_count: total,
            execution_time: elapsed,
        })
    }

    async fn query(&self, sql: &str) -> Result<QueryResult, String> {
        use sqlx::Executor;
        use sqlx::Either;
        use futures_util::StreamExt;

        let start = std::time::Instant::now();
        log::info!("postgres: executing raw query: {}", sql);

        let mut rows = Vec::new();
        let mut rows_affected = 0;

        let mut tx_guard = self.tx_conn.lock().await;
        let mut stream = if let Some(ref mut conn) = *tx_guard {
            (&mut **conn).fetch_many(sqlx::raw_sql(sql))
        } else {
            let pool = self.pool()?;
            pool.fetch_many(sqlx::raw_sql(sql))
        };

        while let Some(res) = stream.next().await {
            match res.map_err(|e| e.to_string())? {
                Either::Left(result) => {
                    rows_affected += result.rows_affected();
                }
                Either::Right(row) => {
                    rows.push(row);
                }
            }
        }

        let elapsed = start.elapsed().as_millis() as u64;
        log::info!("postgres: raw query finished in {}ms", elapsed);

        if rows.is_empty() {
            return Ok(QueryResult {
                rows: vec![],
                fields: vec![],
                row_count: rows_affected as usize,
                execution_time: elapsed,
            });
        }

        let orig_names: Vec<String> = rows[0].columns().iter().map(|c| c.name().to_string()).collect();
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
        for row in &rows {
            let mut map = std::collections::HashMap::new();
            for (i, unique_name) in unique_names.iter().enumerate() {
                let val = Self::get_column_value(row, i);
                map.insert(unique_name.clone(), val);
            }
            data.push(map);
        }

        let count = data.len();
        Ok(QueryResult {
            rows: data,
            fields,
            row_count: count,
            execution_time: elapsed,
        })
    }

    async fn update_cell(
        &self,
        table_name: &str,
        pk_column: &str,
        pk_value: &Value,
        target_column: &str,
        new_value: &Value,
    ) -> Result<(), String> {
        let pool = self.pool()?;
        let safe_table = Self::qualified_table_name(table_name);
        let sql = format!(
            "UPDATE {} SET {} = $1 WHERE {}::text = $2::text",
            safe_table,
            Self::quote(target_column),
            Self::quote(pk_column)
        );
        let mut q = sqlx::query(&sql);
        q = bind_json_value!(q, new_value);
        q = bind_json_value!(q, pk_value);
        q.execute(pool).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn insert_row(
        &self,
        table_name: &str,
        data: &HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, String> {
        let pool = self.pool()?;
        let safe_table = Self::qualified_table_name(table_name);

        if data.is_empty() {
            let sql = format!("INSERT INTO {} DEFAULT VALUES RETURNING *", safe_table);
            let (rows, _, _) = Self::execute_query(pool, &sql, &[]).await?;
            return rows.into_iter().next().ok_or("No row returned".to_string());
        }

        let cols: Vec<String> = data.keys().map(|k| Self::quote(k)).collect();
        let placeholders: Vec<String> = (1..=data.len()).map(|i| format!("${}", i)).collect();
        let values: Vec<Value> = data.values().cloned().collect();

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING *",
            safe_table,
            cols.join(", "),
            placeholders.join(", ")
        );

        let (rows, _, _) = Self::execute_query(pool, &sql, &values).await?;
        rows.into_iter().next().ok_or("No row returned".to_string())
    }

    async fn delete_rows(
        &self,
        table_name: &str,
        pk_column: &str,
        pk_values: &[Value],
    ) -> Result<(), String> {
        let pool = self.pool()?;
        let safe_table = Self::qualified_table_name(table_name);
        let placeholders: Vec<String> = (1..=pk_values.len()).map(|i| format!("${}::text", i)).collect();
        let sql = format!(
            "DELETE FROM {} WHERE {}::text IN ({})",
            safe_table,
            Self::quote(pk_column),
            placeholders.join(", ")
        );
        let mut q = sqlx::query(&sql);
        for v in pk_values {
            q = bind_json_value!(q, v);
        }
        q.execute(pool).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn get_table_columns(&self, table_name: &str) -> Result<Vec<TableColumn>, String> {
        let pool = self.pool()?;
        let (schema_name, bare_table_name) = Self::split_table_name(table_name);
        
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

        let (fk_rows, _, _) = Self::execute_query(
            pool,
            fk_sql,
            &[
                Value::String(schema_name.to_string()),
                Value::String(bare_table_name.to_string()),
            ],
        )
        .await.unwrap_or((vec![], vec![], 0));

        let mut fk_map = HashMap::new();
        for r in fk_rows {
            if let (Some(src_col), Some(tgt_tbl), Some(tgt_col)) = (
                r.get("source_column").and_then(|v| v.as_str()),
                r.get("target_table").and_then(|v| v.as_str()),
                r.get("target_column").and_then(|v| v.as_str()),
            ) {
                fk_map.insert(src_col.to_string(), ForeignKeyDef {
                    target_table: tgt_tbl.to_string(),
                    target_column: tgt_col.to_string(),
                });
            }
        }

        let sql =
            "SELECT column_name::text, data_type::text, is_nullable::text, column_default::text \
             FROM information_schema.columns \
             WHERE table_schema::text = $1::text AND table_name::text = $2::text \
             ORDER BY ordinal_position";

        let (rows, _, _) = Self::execute_query(
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

    async fn alter_table(
        &self,
        table_name: &str,
        operations: &[AlterOperation],
    ) -> Result<(), String> {
        let pool = self.pool()?;
        let safe_table = Self::qualified_table_name(table_name);

        for op in operations {
            let sql = match op.op_type.as_str() {
                "ADD_COLUMN" => {
                    if !crate::drivers::utils::is_safe_data_type(&op.data_type) {
                        return Err(format!("Invalid or unsafe data type: {}", op.data_type));
                    }
                    let mut q = format!(
                        "ALTER TABLE {} ADD COLUMN {} {}",
                        safe_table,
                        Self::quote(&op.name),
                        op.data_type
                    );
                    if op.nullable == Some(false) {
                        q.push_str(" NOT NULL");
                    }
                    if let Some(ref d) = op.default {
                        let d_str = d.to_string();
                        if !crate::drivers::utils::is_safe_default(&d_str) {
                            return Err(format!("Invalid or unsafe default value: {}", d_str));
                        }
                        q.push_str(&format!(" DEFAULT {}", d_str));
                    }
                    if let Some(ref fk) = op.foreign_key {
                        let quoted_target_table = if fk.target_table.contains('.') {
                            fk.target_table.split('.').map(Self::quote).collect::<Vec<String>>().join(".")
                        } else {
                            Self::quote(&fk.target_table)
                        };
                        q.push_str(&format!(" REFERENCES {} ({})", quoted_target_table, Self::quote(&fk.target_column)));
                    }
                    q
                }
                "DROP_COLUMN" => format!(
                    "ALTER TABLE {} DROP COLUMN {}",
                    safe_table,
                    Self::quote(&op.name)
                ),
                "RENAME_COLUMN" => format!(
                    "ALTER TABLE {} RENAME COLUMN {} TO {}",
                    safe_table,
                    Self::quote(&op.old_name),
                    Self::quote(&op.new_name)
                ),
                "DROP_CONSTRAINT" => {
                    let constraint = op.constraint_name.as_ref().ok_or("constraint_name is required")?;
                    format!("ALTER TABLE {} DROP CONSTRAINT {}", safe_table, Self::quote(constraint))
                }
                "ADD_FOREIGN_KEY" => {
                    let fk = op.foreign_key.as_ref().ok_or("foreignKey is required")?;
                    let fk_name = format!("fk_{}_{}", table_name, op.name);
                    let quoted_target_table = if fk.target_table.contains('.') {
                        fk.target_table.split('.').map(Self::quote).collect::<Vec<String>>().join(".")
                    } else {
                        Self::quote(&fk.target_table)
                    };
                    format!(
                        "ALTER TABLE {} ADD CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {} ({})",
                        safe_table,
                        Self::quote(&fk_name),
                        Self::quote(&op.name),
                        quoted_target_table,
                        Self::quote(&fk.target_column)
                    )
                }
                "ADD_CONSTRAINT" => {
                    let definition = op.definition.as_ref().ok_or("definition is required for ADD_CONSTRAINT")?;
                    format!("ALTER TABLE {} ADD CONSTRAINT {} {}", safe_table, Self::quote(&op.name), definition)
                }
                _ => continue,
            };

            sqlx::query(&sql)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    async fn create_table(&self, table_name: &str, columns: &[TableColumn]) -> Result<(), String> {
        let pool = self.pool()?;
        let safe_table = Self::qualified_table_name(table_name);
        let sql = crate::drivers::utils::build_create_table_sql_generic(&safe_table, columns, Self::quote)?;
        sqlx::query(&sql)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn drop_table(&self, table_name: &str, cascade: bool) -> Result<(), String> {
        let pool = self.pool()?;
        let sql = Self::build_drop_table_sql(table_name, cascade);
        sqlx::query(&sql)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn create_schema(&self, schema_name: &str) -> Result<(), String> {
        let pool = self.pool()?;
        let sql = format!("CREATE SCHEMA {}", Self::quote(schema_name));
        sqlx::query(&sql)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn drop_schema(&self, schema_name: &str) -> Result<(), String> {
        let pool = self.pool()?;
        let sql = format!("DROP SCHEMA {} CASCADE", Self::quote(schema_name));
        sqlx::query(&sql)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn drop_database(&self, db_name: &str) -> Result<(), String> {
        let pool = self.pool()?;
        let sql = format!("DROP DATABASE {}", Self::quote(db_name));
        sqlx::query(&sql)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn create_database(&self, db_name: &str) -> Result<(), String> {
        let pool = self.pool()?;
        let sql = format!("CREATE DATABASE {}", Self::quote(db_name));
        sqlx::query(&sql)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }


    async fn export_to_csv(&self, table_name: &str, export_path: &str) -> Result<(), String> {
        let pool = self.pool()?;
        let safe_table = Self::qualified_table_name(table_name);
        let sql = format!("SELECT * FROM {}", safe_table);

        let (rows, _, _) = Self::execute_query(pool, &sql, &[]).await?;
        crate::drivers::utils::export_rows_to_csv(&rows, export_path)
    }

    async fn get_schema_details(&self, schema_name: &str) -> Result<SchemaDetails, String> {
        let pool = self.pool()?;
        
        let column_sql = "SELECT \
                c.table_name::text, \
                c.column_name::text, \
                c.udt_name::text as data_type, \
                (c.is_nullable = 'YES') as is_nullable, \
                c.column_default::text as column_default, \
                EXISTS ( \
                    SELECT 1 \
                    FROM information_schema.table_constraints tc \
                    JOIN information_schema.key_column_usage kcu \
                        ON tc.constraint_name = kcu.constraint_name \
                        AND tc.table_schema = kcu.table_schema \
                        AND tc.table_name = kcu.table_name \
                    WHERE tc.constraint_type = 'PRIMARY KEY' \
                        AND tc.table_schema = c.table_schema \
                        AND tc.table_name = c.table_name \
                        AND kcu.column_name = c.column_name \
                ) as is_primary_key \
            FROM information_schema.columns c \
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
            Self::execute_query(pool, column_sql, &params),
            Self::execute_query(pool, relation_sql, &params)
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
                fk_map.insert((src_tbl.to_string(), src_col.to_string()), ForeignKeyDef {
                    target_table: tgt_tbl.to_string(),
                    target_column: tgt_col.to_string(),
                });
            }
        }

        for r in column_rows {
            let table_name = r.get("table_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let col_name = r.get("column_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let foreign_key = fk_map.get(&(table_name.clone(), col_name.clone())).cloned();
            let col = TableColumn {
                name: col_name,
                data_type: r.get("data_type").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                nullable: r.get("is_nullable").and_then(|v| v.as_bool()).unwrap_or(true),
                is_primary_key: r.get("is_primary_key").and_then(|v| v.as_bool()).unwrap_or(false),
                default: r.get("column_default").and_then(|v| v.as_str()).map(|s| s.to_string()),
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
                constraint_name: r.get("constraint_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                source_table: r.get("source_table").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                source_column: r.get("source_column").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                target_table: r.get("target_table").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                target_column: r.get("target_column").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            });
        }

        Ok(SchemaDetails { tables, relations })
    }

    async fn get_table_indexes(&self, table_name: &str) -> Result<Vec<DbIndex>, String> {
        let pool = self.pool()?;
        let (schema_name, bare_table_name) = Self::split_table_name(table_name);
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
        
        let (rows, _, _) = Self::execute_query(
            pool,
            sql,
            &[
                Value::String(schema_name.to_string()),
                Value::String(bare_table_name.to_string()),
            ],
        ).await?;

        let mut indexes = Vec::new();
        for r in rows {
            let mut cols = vec![];
            if let Some(val) = r.get("columns") {
                if let Value::Array(arr) = val {
                    cols = arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                } else if let Value::String(s) = val {
                    if let Ok(Value::Array(arr)) = serde_json::from_str(s) {
                        cols = arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                    }
                }
            }

            indexes.push(DbIndex {
                name: r.get("index_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                is_unique: r.get("is_unique").and_then(|v| v.as_bool()).unwrap_or(false),
                is_primary_key: r.get("is_primary").and_then(|v| v.as_bool()).unwrap_or(false),
                index_type: r.get("index_type").and_then(|v| v.as_str()).map(|s| s.to_string()),
                ddl: r.get("ddl").and_then(|v| v.as_str()).map(|s| s.to_string()),
                columns: cols,
            });
        }
        Ok(indexes)
    }

    async fn get_table_constraints(&self, table_name: &str) -> Result<Vec<TableConstraint>, String> {
        let pool = self.pool()?;
        let (schema_name, bare_table_name) = Self::split_table_name(table_name);
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
        
        let (rows, _, _) = Self::execute_query(
            pool,
            sql,
            &[
                Value::String(schema_name.to_string()),
                Value::String(bare_table_name.to_string()),
            ],
        ).await?;

        let mut constraints = Vec::new();
        for r in rows {
            constraints.push(TableConstraint {
                name: r.get("constraint_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                constraint_type: r.get("constraint_type").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                definition: r.get("definition").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            });
        }
        Ok(constraints)
    }

    async fn begin_transaction(&self) -> Result<(), String> {
        let pool = self.pool()?;
        let mut tx = self.tx_conn.lock().await;
        if tx.is_some() {
            return Err("Transaction already in progress".to_string());
        }
        
        let mut conn = pool.acquire().await.map_err(|e| e.to_string())?;
        sqlx::query("BEGIN").execute(&mut *conn).await.map_err(|e| e.to_string())?;
        *tx = Some(conn);
        Ok(())
    }

    async fn commit_transaction(&self) -> Result<(), String> {
        let mut tx = self.tx_conn.lock().await;
        if let Some(mut conn) = tx.take() {
            sqlx::query("COMMIT").execute(&mut *conn).await.map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("No active transaction to commit".to_string())
        }
    }

    async fn rollback_transaction(&self) -> Result<(), String> {
        let mut tx = self.tx_conn.lock().await;
        if let Some(mut conn) = tx.take() {
            sqlx::query("ROLLBACK").execute(&mut *conn).await.map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("No active transaction to rollback".to_string())
        }
    }

    async fn is_in_transaction(&self) -> bool {
        self.tx_conn.lock().await.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drivers::TableColumn;

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
        let sql = crate::drivers::utils::build_create_table_sql_generic(&safe_table, &columns, |s| PostgresDriver::quote(s)).unwrap();
        assert_eq!(
            sql,
            "CREATE TABLE \"public\".\"users\" (\"id\" SERIAL NOT NULL PRIMARY KEY, \"name\" VARCHAR(255) DEFAULT 'Guest')"
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

        let res = crate::drivers::utils::build_create_table_sql_generic("\"users\"", &columns, |s| PostgresDriver::quote(s));
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Invalid or unsafe data type"));
    }
}
