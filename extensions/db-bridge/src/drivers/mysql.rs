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

pub struct MysqlDriver {
    pool: Option<AnyPool>,
}

impl MysqlDriver {
    pub fn new() -> Self {
        Self { pool: None }
    }

    fn pool(&self) -> Result<&AnyPool, String> {
        self.pool.as_ref().ok_or("Not connected".to_string())
    }

    fn quote(ident: &str) -> String {
        format!("`{}`", ident.replace('`', "``"))
    }
}

#[async_trait]
impl DatabaseDriver for MysqlDriver {
    async fn connect(&mut self, config: &Config) -> Result<(), String> {
        let dsn = format!(
            "mysql://{}:{}@{}:{}/{}",
            config.username, config.password, config.host, config.port, config.database
        );

        let pool = AnyPoolOptions::new()
            .max_connections(5)
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

    async fn get_schema(&self, all_schemas: bool) -> Result<SchemaResult, String> {
        let pool = self.pool()?;
        let where_clause = if all_schemas {
            "WHERE TABLE_SCHEMA NOT IN ('information_schema', 'performance_schema', 'mysql', 'sys')"
        } else {
            "WHERE TABLE_SCHEMA = DATABASE()"
        };

        // Tables
        let sql = format!(
            "SELECT TABLE_NAME as table_name, TABLE_SCHEMA as table_schema FROM information_schema.TABLES {} AND TABLE_TYPE = 'BASE TABLE'",
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
            "SELECT TABLE_NAME as table_name, TABLE_SCHEMA as table_schema FROM information_schema.TABLES {} AND TABLE_TYPE = 'VIEW'",
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

        // Routines
        let routine_where = where_clause.replace("TABLE_SCHEMA", "ROUTINE_SCHEMA");
        let sql = format!(
            "SELECT ROUTINE_NAME as routine_name, ROUTINE_SCHEMA as routine_schema, ROUTINE_TYPE as routine_type FROM information_schema.ROUTINES {}",
            routine_where
        );
        let (func_rows, _) = execute_query(pool, &sql, &[]).await?;
        let functions: Vec<SchemaObject> = func_rows
            .iter()
            .map(|r| SchemaObject {
                name: r.get("routine_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                schema: Some(r.get("routine_schema").and_then(|v| v.as_str()).unwrap_or("").to_string()),
                obj_type: Some(r.get("routine_type").and_then(|v| v.as_str()).unwrap_or("").to_string()),
            })
            .collect();

        // Schemas
        let schema_where = where_clause.replace("TABLE_SCHEMA", "SCHEMA_NAME");
        let sql = format!(
            "SELECT SCHEMA_NAME as schema_name FROM information_schema.SCHEMATA {}",
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
        let pk_sql = "SELECT COLUMN_NAME FROM information_schema.COLUMNS \
            WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ? AND COLUMN_KEY = 'PRI'";
        let (pk_rows, _) = execute_query(pool, pk_sql, &[Value::String(table_name.to_string())]).await?;
        let pk_set: std::collections::HashSet<String> = pk_rows
            .iter()
            .filter_map(|r| r.get("COLUMN_NAME").and_then(|v| v.as_str()).map(|s| s.to_string()))
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
            let sql = format!("INSERT INTO {} () VALUES ()", safe_table);
            let res = sqlx::query(&sql)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;
            let last_id = res.last_insert_id().unwrap_or(0);
            let mut m = HashMap::new();
            m.insert("insertId".to_string(), Value::Number(last_id.into()));
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

        // Try to SELECT the inserted row back
        let select_sql = format!("SELECT * FROM {} WHERE id = ?", safe_table);
        let (rows, _) = execute_query(pool, &select_sql, &[Value::Number(last_id.into())]).await
            .unwrap_or_default();
        if let Some(row) = rows.into_iter().next() {
            return Ok(row);
        }

        let mut m = HashMap::new();
        m.insert("insertId".to_string(), Value::Number(last_id.into()));
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
        get_table_columns_generic(
            pool,
            "SELECT COLUMN_NAME, DATA_TYPE, IS_NULLABLE, COLUMN_DEFAULT \
             FROM INFORMATION_SCHEMA.COLUMNS \
             WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ? \
             ORDER BY ORDINAL_POSITION",
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
