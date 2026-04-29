use super::{
    AlterOperation, Config, DatabaseDriver, QueryResult, SchemaObject, SchemaResult,
    TableColumn, TableDataResult, ColumnInfo
};
use crate::bind_json_value;
use async_trait::async_trait;
use serde_json::Value;
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::{Column, Row, TypeInfo, ValueRef};
use std::collections::HashMap;
use uuid;
use chrono;

pub struct PostgresDriver {
    pool: Option<PgPool>,
}

impl PostgresDriver {
    pub fn new() -> Self {
        Self { pool: None }
    }

    fn pool(&self) -> Result<&PgPool, String> {
        self.pool.as_ref().ok_or("Not connected".to_string())
    }

    fn quote(ident: &str) -> String {
        format!("\"{}\"", ident.replace('"', "\"\""))
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

        let mut fields = Vec::new();
        for col in rows[0].columns() {
            fields.push(ColumnInfo {
                name: col.name().to_string(),
                data_type: col.type_info().name().to_string(),
                is_primary_key: false,
            });
        }

        let mut data = Vec::new();
        for row in rows {
            let mut map = HashMap::new();
            for (i, col) in row.columns().iter().enumerate() {
                let val = Self::get_column_value(&row, i);
                map.insert(col.name().to_string(), val);
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
            "INT2" | "INT4" | "INT8" | "OID" => {
                if let Ok(v) = row.try_get::<i64, _>(i) {
                    return Value::Number(v.into());
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
}

#[async_trait]
impl DatabaseDriver for PostgresDriver {
    async fn connect(&mut self, config: &Config) -> Result<(), String> {
        let ssl_mode = if config.ssl { "require" } else { "disable" };
        let dsn = format!(
            "postgres://{}:{}@{}:{}/{}?sslmode={}&options=-c%20search_path=public&application_name=db_manager&connect_timeout={}&tcp_user_timeout=5000",
            config.username, config.password, config.host, config.port, config.database, ssl_mode, config.connection_timeout
        );

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_lazy(&dsn)
            .map_err(|e| e.to_string())?;

        self.pool = Some(pool);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), String> {
        if let Some(pool) = self.pool.take() {
            pool.close().await;
        }
        Ok(())
    }

    async fn get_schema(&self, all_schemas: bool) -> Result<SchemaResult, String> {
        let pool = self.pool()?;
        let where_clause = if all_schemas {
            "WHERE table_schema NOT IN ('information_schema', 'pg_catalog')"
        } else {
            "WHERE table_schema = 'public'"
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
        let func_sql = format!(
            "SELECT routine_name::text, routine_schema::text, data_type::text FROM information_schema.routines {} ORDER BY routine_name",
            where_clause.replace("table_schema", "routine_schema")
        );
        let schema_sql = "SELECT schema_name::text FROM information_schema.schemata WHERE schema_name NOT IN ('information_schema', 'pg_catalog') ORDER BY schema_name";

        let (table_res, view_res, func_res, schema_res) = tokio::join!(
            Self::execute_query(pool, &table_sql, &[]),
            Self::execute_query(pool, &view_sql, &[]),
            Self::execute_query(pool, &func_sql, &[]),
            Self::execute_query(pool, schema_sql, &[])
        );

        let (table_rows, _, _) = table_res?;
        let (view_rows, _, _) = view_res?;
        let (func_rows, _, _) = func_res?;
        let (schema_rows, _, _) = schema_res?;

        let tables: Vec<SchemaObject> = table_rows
            .iter()
            .map(|r| SchemaObject {
                name: r.get("table_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                schema: Some(r.get("table_schema").and_then(|v| v.as_str()).unwrap_or("").to_string()),
                obj_type: None,
            })
            .collect();

        let views: Vec<SchemaObject> = view_rows
            .iter()
            .map(|r| SchemaObject {
                name: r.get("table_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                schema: Some(r.get("table_schema").and_then(|v| v.as_str()).unwrap_or("").to_string()),
                obj_type: None,
            })
            .collect();

        let functions: Vec<SchemaObject> = func_rows
            .iter()
            .map(|r| SchemaObject {
                name: r.get("routine_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                schema: Some(r.get("routine_schema").and_then(|v| v.as_str()).unwrap_or("").to_string()),
                obj_type: Some(r.get("data_type").and_then(|v| v.as_str()).unwrap_or("").to_string()),
            })
            .collect();

        let schemas: Vec<SchemaObject> = schema_rows
            .iter()
            .map(|r| SchemaObject {
                name: r.get("schema_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                schema: None,
                obj_type: None,
            })
            .collect();

        Ok(SchemaResult {
            tables,
            views,
            functions,
            schemas: Some(schemas),
        })
    }

    async fn fetch_table_data(
        &self,
        table_name: &str,
        limit: i64,
        offset: i64,
        sort_column: &str,
        sort_direction: &str,
    ) -> Result<TableDataResult, String> {
        let pool = self.pool()?;
        let safe_table = Self::quote(table_name);

        // Faster PK detection query using pg_index
        let pk_sql = "SELECT a.attname::text as column_name \
            FROM pg_index i \
            JOIN pg_attribute a ON a.attrelid = i.indrelid AND a.attnum = ANY(i.indkey) \
            WHERE i.indrelid = $1::regclass AND i.indisprimary";

        let order_clause = if !sort_column.is_empty() {
            let dir = if sort_direction.eq_ignore_ascii_case("desc") { "DESC" } else { "ASC" };
            format!(" ORDER BY {} {}", Self::quote(sort_column), dir)
        } else {
            String::new()
        };

        let data_sql = format!(
            "SELECT * FROM public.{}{} LIMIT $1 OFFSET $2",
            safe_table, order_clause
        );
        let count_sql = format!("SELECT COUNT(*) FROM public.{}", safe_table);

        let pk_params = [Value::String(format!("public.{}", table_name))];
        let data_params = [
            Value::Number(limit.into()),
            Value::Number(offset.into()),
        ];
        let count_params = [];

        // Run all three queries in parallel
        let (pk_res, data_res, count_res) = tokio::join!(
            Self::execute_query(pool, pk_sql, &pk_params),
            Self::execute_query(pool, &data_sql, &data_params),
            Self::execute_query(pool, &count_sql, &count_params)
        );

        let (pk_rows, _, _) = pk_res?;
        let (data, mut fields, elapsed) = data_res?;
        let (count_rows, _, _) = count_res?;

        let pk_set: std::collections::HashSet<String> = pk_rows
            .iter()
            .filter_map(|r| r.get("column_name").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .collect();

        for f in &mut fields {
            if pk_set.contains(&f.name) {
                f.is_primary_key = true;
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
        let pool = self.pool()?;
        let (data, fields, elapsed) = Self::execute_query(pool, sql, &[]).await?;
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
        let sql = format!(
            "UPDATE public.{} SET {} = $1 WHERE {} = $2",
            Self::quote(table_name),
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
        let safe_table = Self::quote(table_name);

        if data.is_empty() {
            let sql = format!(
                "INSERT INTO public.{} DEFAULT VALUES RETURNING *",
                safe_table
            );
            let (rows, _, _) = Self::execute_query(pool, &sql, &[]).await?;
            return rows.into_iter().next().ok_or("No row returned".to_string());
        }

        let cols: Vec<String> = data.keys().map(|k| Self::quote(k)).collect();
        let placeholders: Vec<String> = (1..=data.len()).map(|i| format!("${}", i)).collect();
        let values: Vec<Value> = data.values().cloned().collect();

        let sql = format!(
            "INSERT INTO public.{} ({}) VALUES ({}) RETURNING *",
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
        let placeholders: Vec<String> = (1..=pk_values.len()).map(|i| format!("${}", i)).collect();
        let sql = format!(
            "DELETE FROM public.{} WHERE {} IN ({})",
            Self::quote(table_name),
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
        let sql = "SELECT column_name::text, data_type::text, is_nullable::text, column_default::text \
             FROM information_schema.columns \
             WHERE table_schema = 'public' AND table_name = $1 \
             ORDER BY ordinal_position";
        
        let (rows, _, _) = Self::execute_query(pool, sql, &[Value::String(table_name.to_string())]).await?;
        
        let mut columns = Vec::new();
        for r in rows {
            columns.push(TableColumn {
                name: r.get("column_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                data_type: r.get("data_type").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                nullable: r.get("is_nullable").and_then(|v| v.as_str()).map(|s| s == "YES").unwrap_or(true),
                is_primary_key: false, 
                default: r.get("column_default").and_then(|v| v.as_str()).map(|s| s.to_string()),
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
        let safe_table = Self::quote(table_name);

        for op in operations {
            let sql = match op.op_type.as_str() {
                "ADD_COLUMN" => {
                    let mut q = format!(
                        "ALTER TABLE public.{} ADD COLUMN {} {}",
                        safe_table,
                        Self::quote(&op.name),
                        op.data_type
                    );
                    if op.nullable == Some(false) {
                        q.push_str(" NOT NULL");
                    }
                    if let Some(ref d) = op.default {
                        q.push_str(&format!(" DEFAULT {}", d));
                    }
                    q
                }
                "DROP_COLUMN" => format!(
                    "ALTER TABLE public.{} DROP COLUMN {}",
                    safe_table,
                    Self::quote(&op.name)
                ),
                "RENAME_COLUMN" => format!(
                    "ALTER TABLE public.{} RENAME COLUMN {} TO {}",
                    safe_table,
                    Self::quote(&op.old_name),
                    Self::quote(&op.new_name)
                ),
                _ => continue,
            };

            sqlx::query(&sql)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    async fn export_to_csv(&self, table_name: &str, export_path: &str) -> Result<(), String> {
        let pool = self.pool()?;
        let safe_table = Self::quote(table_name);
        let sql = format!("SELECT * FROM public.{}", safe_table);
        
        let (rows, _, _) = Self::execute_query(pool, &sql, &[]).await?;
        
        if rows.is_empty() {
            return Ok(());
        }

        let mut wtr = csv::Writer::from_path(export_path).map_err(|e| e.to_string())?;
        
        // Header
        let headers: Vec<String> = rows[0].keys().cloned().collect();
        wtr.write_record(&headers).map_err(|e| e.to_string())?;

        // Data
        for row in rows {
            let record: Vec<String> = headers
                .iter()
                .map(|h| {
                    let v = row.get(h).unwrap_or(&Value::Null);
                    match v {
                        Value::Null => "".to_string(),
                        Value::String(s) => s.clone(),
                        _ => v.to_string(),
                    }
                })
                .collect();
            wtr.write_record(&record).map_err(|e| e.to_string())?;
        }

        wtr.flush().map_err(|e| e.to_string())?;
        Ok(())
    }
}
