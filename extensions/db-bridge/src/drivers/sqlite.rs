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

pub struct SqliteDriver {
    pool: Option<AnyPool>,
}

impl SqliteDriver {
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
impl DatabaseDriver for SqliteDriver {
    async fn connect(&mut self, config: &Config) -> Result<(), String> {
        if config.database.is_empty() {
            return Err("SQLite requires a database file path".to_string());
        }

        let dsn = format!("sqlite:{}", config.database);

        let pool = AnyPoolOptions::new()
            .max_connections(1) // SQLite is single-writer
            .connect(&dsn)
            .await
            .map_err(|e| e.to_string())?;

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

    async fn get_schema(&self, _all_schemas: bool) -> Result<SchemaResult, String> {
        let pool = self.pool()?;

        // Tables
        let (table_rows, _) = execute_query(
            pool,
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
            &[],
        )
        .await?;
        let tables: Vec<SchemaObject> = table_rows
            .iter()
            .map(|r| SchemaObject {
                name: r.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                schema: Some("main".to_string()),
                obj_type: None,
            })
            .collect();

        // Views
        let (view_rows, _) = execute_query(
            pool,
            "SELECT name FROM sqlite_master WHERE type='view'",
            &[],
        )
        .await?;
        let views: Vec<SchemaObject> = view_rows
            .iter()
            .map(|r| SchemaObject {
                name: r.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                schema: Some("main".to_string()),
                obj_type: None,
            })
            .collect();

        Ok(SchemaResult {
            tables,
            views,
            functions: vec![],
            schemas: None,
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

        // Get PKs via PRAGMA
        let pragma_sql = format!("PRAGMA table_info({})", safe_table);
        let (pragma_rows, _) = execute_query(pool, &pragma_sql, &[]).await?;
        let pk_set: std::collections::HashSet<String> = pragma_rows
            .iter()
            .filter(|r| r.get("pk").and_then(|v| v.as_i64()).unwrap_or(0) > 0)
            .filter_map(|r| r.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .collect();

        let order_clause = if !sort_column.is_empty() {
            let dir = if sort_direction.eq_ignore_ascii_case("desc") { "DESC" } else { "ASC" };
            format!(" ORDER BY {} {}", Self::quote(sort_column), dir)
        } else {
            String::new()
        };

        let sql = format!(
            "SELECT * FROM {}{} LIMIT ? OFFSET ?",
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

        let count_sql = format!("SELECT COUNT(*) as cnt FROM {}", safe_table);
        let (count_rows, _) = execute_query(pool, &count_sql, &[]).await?;
        let total = count_rows
            .first()
            .and_then(|r| r.get("cnt"))
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
            "UPDATE {} SET {} = ? WHERE {} = ?",
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
            let sql = format!("INSERT INTO {} DEFAULT VALUES", safe_table);
            let res = sqlx::query(&sql)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;
            let last_id = res.last_insert_id().unwrap_or(0);

            // Fetch the row back
            let select_sql = format!("SELECT * FROM {} WHERE rowid = ?", safe_table);
            let (rows, _) = execute_query(pool, &select_sql, &[Value::Number(last_id.into())])
                .await
                .unwrap_or_default();
            if let Some(row) = rows.into_iter().next() {
                return Ok(row);
            }
            let mut m = HashMap::new();
            m.insert("lastID".to_string(), Value::Number(last_id.into()));
            return Ok(m);
        }

        let cols: Vec<String> = data.keys().map(|k| Self::quote(k)).collect();
        let placeholders: Vec<String> = (0..data.len()).map(|_| "?".to_string()).collect();
        let values: Vec<&Value> = data.values().collect();

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            safe_table,
            cols.join(", "),
            placeholders.join(", ")
        );

        let mut q = sqlx::query(&sql);
        for v in &values {
            q = bind_value(q, v);
        }
        let res = q.execute(pool).await.map_err(|e| e.to_string())?;
        let last_id = res.last_insert_id().unwrap_or(0);

        let select_sql = format!("SELECT * FROM {} WHERE rowid = ?", safe_table);
        let (rows, _) = execute_query(pool, &select_sql, &[Value::Number(last_id.into())])
            .await
            .unwrap_or_default();
        if let Some(row) = rows.into_iter().next() {
            return Ok(row);
        }

        let mut m = HashMap::new();
        m.insert("lastID".to_string(), Value::Number(last_id.into()));
        Ok(m)
    }

    async fn delete_rows(
        &self,
        table_name: &str,
        pk_column: &str,
        pk_values: &[Value],
    ) -> Result<(), String> {
        let pool = self.pool()?;
        let placeholders: Vec<String> = (0..pk_values.len()).map(|_| "?".to_string()).collect();
        let sql = format!(
            "DELETE FROM {} WHERE {} IN ({})",
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
        let pragma_sql = format!("PRAGMA table_info({})", Self::quote(table_name));
        let (rows, _) = execute_query(pool, &pragma_sql, &[]).await?;

        let cols: Vec<TableColumn> = rows
            .iter()
            .map(|r| TableColumn {
                name: r.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                data_type: r.get("type").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                nullable: r.get("notnull").and_then(|v| v.as_i64()).unwrap_or(0) == 0,
                default: r
                    .get("dflt_value")
                    .cloned()
                    .unwrap_or(Value::Null),
            })
            .collect();

        Ok(cols)
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
                        "ALTER TABLE {} ADD COLUMN {} {}",
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
        export_to_csv(pool, &Self::quote(table_name), export_path).await
    }
}
