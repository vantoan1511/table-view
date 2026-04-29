use super::utils::{bind_value, execute_query, export_to_csv, get_table_columns_generic};
use super::{
    AlterOperation, Config, DatabaseDriver, QueryResult, SchemaObject, SchemaResult,
    TableColumn, TableDataResult,
};
use async_trait::async_trait;
use serde_json::Value;
use sqlx::any::AnyPoolOptions;
use sqlx::AnyPool;
use std::collections::HashMap;
use sqlx::{Row, Column, TypeInfo};

pub struct PostgresDriver {
    pool: Option<AnyPool>,
}

impl PostgresDriver {
    pub fn new() -> Self {
        Self { pool: None }
    }

    fn pool(&self) -> Result<&AnyPool, String> {
        self.pool.as_ref().ok_or("Not connected".to_string())
    }

    fn quote(ident: &str) -> String {
        format!("\"{}\"", ident.replace('"', "\"\""))
    }
}

#[async_trait]
impl DatabaseDriver for PostgresDriver {
    async fn connect(&mut self, config: &Config) -> Result<(), String> {
        let ssl_mode = if config.ssl { "require" } else { "disable" };
        let dsn = format!(
            "postgres://{}:{}@{}:{}/{}?sslmode={}&default_query_exec_mode=simple_protocol",
            config.username, config.password, config.host, config.port, config.database, ssl_mode
        );

        let pool = AnyPoolOptions::new()
            .max_connections(5)
            .connect(&dsn)
            .await
            .map_err(|e| e.to_string())?;

        // Ping to verify
        sqlx::query("SELECT 1")
            .fetch_one(&pool)
            .await
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

        // Tables
        let sql = format!(
            "SELECT table_name::text, table_schema::text FROM information_schema.tables {} AND table_type = 'BASE TABLE' ORDER BY table_name",
            where_clause
        );
        let (table_rows, _) = execute_query(pool, &sql, &[]).await?;
        let tables: Vec<SchemaObject> = table_rows
            .iter()
            .map(|r| SchemaObject {
                name: r.get("table_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                schema: Some(r.get("table_schema").and_then(|v| v.as_str()).unwrap_or("").to_string()),
                obj_type: None,
            })
            .collect();

        // Views
        let sql = format!(
            "SELECT table_name::text, table_schema::text FROM information_schema.views {} ORDER BY table_name",
            where_clause
        );
        let (view_rows, _) = execute_query(pool, &sql, &[]).await?;
        let views: Vec<SchemaObject> = view_rows
            .iter()
            .map(|r| SchemaObject {
                name: r.get("table_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                schema: Some(r.get("table_schema").and_then(|v| v.as_str()).unwrap_or("").to_string()),
                obj_type: None,
            })
            .collect();

        // Functions
        let func_where = where_clause.replace("table_schema", "routine_schema");
        let sql = format!(
            "SELECT routine_name::text, routine_schema::text FROM information_schema.routines {} AND routine_type = 'FUNCTION' ORDER BY routine_name",
            func_where
        );
        let (func_rows, _) = execute_query(pool, &sql, &[]).await?;
        let functions: Vec<SchemaObject> = func_rows
            .iter()
            .map(|r| SchemaObject {
                name: r.get("routine_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                schema: Some(r.get("routine_schema").and_then(|v| v.as_str()).unwrap_or("").to_string()),
                obj_type: None,
            })
            .collect();

        // Schemas
        let schema_where = where_clause.replace("table_schema", "schema_name");
        let sql = format!(
            "SELECT schema_name::text FROM information_schema.schemata {} ORDER BY schema_name",
            schema_where
        );
        let (schema_rows, _) = execute_query(pool, &sql, &[]).await?;
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

        // Get PKs
        let pk_sql = "SELECT kcu.column_name::text FROM information_schema.table_constraints tco \
            JOIN information_schema.key_column_usage kcu ON kcu.constraint_name = tco.constraint_name \
            AND kcu.constraint_schema = tco.constraint_schema \
            WHERE tco.constraint_type = 'PRIMARY KEY' AND kcu.table_name = $1 AND tco.table_schema = 'public'";
        let (pk_rows, _) = execute_query(pool, pk_sql, &[Value::String(table_name.to_string())]).await?;
        let pk_set: std::collections::HashSet<String> = pk_rows
            .iter()
            .filter_map(|r| r.get("column_name").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .collect();

        let order_clause = if !sort_column.is_empty() {
            let dir = if sort_direction.eq_ignore_ascii_case("desc") { "DESC" } else { "ASC" };
            format!(" ORDER BY {} {}", Self::quote(sort_column), dir)
        } else {
            String::new()
        };

        let sql = format!(
            "SELECT * FROM public.{}{} LIMIT $1 OFFSET $2",
            safe_table, order_clause
        );
        let (data, mut fields) = execute_query(pool, &sql, &[
            Value::Number(limit.into()),
            Value::Number(offset.into()),
        ]).await?;

        for f in &mut fields {
            if pk_set.contains(&f.name) {
                f.is_primary_key = true;
            }
        }

        // Count
        let count_sql = format!("SELECT COUNT(*) FROM public.{}", safe_table);
        let (count_rows, _) = execute_query(pool, &count_sql, &[]).await?;
        let total = count_rows
            .first()
            .and_then(|r| r.values().next())
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        Ok(TableDataResult {
            rows: data,
            fields,
            total_count: total,
        })
    }

    async fn query(&self, sql: &str) -> Result<QueryResult, String> {
        let pool = self.pool()?;
        let (data, fields) = execute_query(pool, sql, &[]).await?;
        let count = data.len();
        Ok(QueryResult {
            rows: data,
            fields,
            row_count: count,
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
        q = bind_value(q, new_value);
        q = bind_value(q, pk_value);
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
            let (rows, _) = execute_query(pool, &sql, &[]).await?;
            return rows.into_iter().next().ok_or("No row returned".to_string());
        }

        let cols: Vec<String> = data.keys().map(|k| Self::quote(k)).collect();
        let placeholders: Vec<String> = (1..=data.len()).map(|i| format!("${}", i)).collect();
        let values: Vec<&Value> = data.values().collect();

        let sql = format!(
            "INSERT INTO public.{} ({}) VALUES ({}) RETURNING *",
            safe_table,
            cols.join(", "),
            placeholders.join(", ")
        );

        let mut q = sqlx::query(&sql);
        for v in &values {
            q = bind_value(q, v);
        }

        let rows: Vec<sqlx::any::AnyRow> = q.fetch_all(pool).await.map_err(|e| e.to_string())?;
        if rows.is_empty() {
            return Err("No row returned".to_string());
        }

        // Use scan helper to parse the returned row
        let columns: Vec<String> = rows[0].columns().iter().map(|c| c.name().to_string()).collect();
        let type_names: Vec<String> = rows[0].columns().iter().map(|c| c.type_info().name().to_string()).collect();
        let (parsed, _) = super::utils::scan_rows(rows, &columns, &type_names).await?;
        parsed.into_iter().next().ok_or("No row returned".to_string())
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
            q = bind_value(q, v);
        }
        q.execute(pool).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn get_table_columns(&self, table_name: &str) -> Result<Vec<TableColumn>, String> {
        let pool = self.pool()?;
        get_table_columns_generic(
            pool,
            "SELECT column_name::text, data_type::text, is_nullable::text, column_default::text \
             FROM information_schema.columns \
             WHERE table_schema = 'public' AND table_name = $1 \
             ORDER BY ordinal_position",
            &[Value::String(table_name.to_string())],
        )
        .await
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
        let table_expr = format!("public.{}", Self::quote(table_name));
        export_to_csv(pool, &table_expr, export_path).await
    }
}
