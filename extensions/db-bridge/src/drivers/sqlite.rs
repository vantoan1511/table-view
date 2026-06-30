use super::{
    AlterOperation, Config, DatabaseDriver, QueryResult, SchemaObject, SchemaResult,
    TableColumn, TableDataResult, ColumnInfo, SchemaDetails, TableInfo, TableRelation
};
use crate::bind_json_value;
use async_trait::async_trait;
use serde_json::Value;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions, SqliteRow};
use sqlx::{Column, Row, TypeInfo, ValueRef};
use std::collections::HashMap;

pub struct SqliteDriver {
    pool: Option<SqlitePool>,
}

impl SqliteDriver {
    pub fn new() -> Self {
        Self { pool: None }
    }

    fn pool(&self) -> Result<&SqlitePool, String> {
        self.pool.as_ref().ok_or("Not connected".to_string())
    }

    fn quote(ident: &str) -> String {
        format!("\"{}\"", ident.replace('"', "\"\""))
    }

    async fn execute_query(
        pool: &SqlitePool,
        sql: &str,
        params: &[Value],
    ) -> Result<(Vec<HashMap<String, Value>>, Vec<ColumnInfo>, u64), String> {
        let mut q = sqlx::query(sql);
        for p in params {
            q = bind_json_value!(q, p);
        }

        log::info!("sqlite: executing query: {}", sql);
        let start = std::time::Instant::now();
        let rows = q.fetch_all(pool).await.map_err(|e| e.to_string())?;
        let elapsed = start.elapsed().as_millis() as u64;
        log::info!("sqlite: query finished in {}ms", elapsed);

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

    fn get_column_value(row: &SqliteRow, i: usize) -> Value {
        let raw = match row.try_get_raw(i) {
            Ok(v) => v,
            Err(_) => return Value::Null,
        };
        if raw.is_null() {
            return Value::Null;
        }

        // SQLite is weakly typed, so we try common types
        if let Ok(v) = row.try_get::<i64, _>(i) {
            return Value::Number(v.into());
        }
        if let Ok(v) = row.try_get::<f64, _>(i) {
            if let Some(num) = serde_json::Number::from_f64(v) {
                return Value::Number(num);
            }
        }
        if let Ok(v) = row.try_get::<bool, _>(i) {
            return Value::Bool(v);
        }
        if let Ok(v) = row.try_get::<String, _>(i) {
            return Value::String(v);
        }
        if let Ok(v) = row.try_get::<Vec<u8>, _>(i) {
            return Value::String(hex::encode(v));
        }

        Value::Null
    }


    fn build_drop_table_sql(table_name: &str) -> String {
        let safe_table = Self::quote(table_name);
        format!("DROP TABLE {}", safe_table)
    }
}

#[async_trait]
impl DatabaseDriver for SqliteDriver {
    async fn connect(&mut self, config: &Config) -> Result<(), String> {
        let dsn = format!("sqlite://{}", config.database);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(config.connection_timeout as u64))
            .connect(&dsn)
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

    async fn get_schema(
        &self,
        _all_databases: bool,
        _schema_name: Option<&str>,
    ) -> Result<SchemaResult, String> {
        let pool = self.pool()?;

        // Tables
        let (table_rows, _, _) = Self::execute_query(
            pool,
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
            &[],
        )
        .await?;
        let tables: Vec<SchemaObject> = table_rows
            .iter()
            .map(|r| SchemaObject {
                name: r.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                schema: None,
                obj_type: None,
            })
            .collect();

        // Views
        let (view_rows, _, _) = Self::execute_query(
            pool,
            "SELECT name FROM sqlite_master WHERE type='view'",
            &[],
        )
        .await?;
        let views: Vec<SchemaObject> = view_rows
            .iter()
            .map(|r| SchemaObject {
                name: r.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                schema: None,
                obj_type: None,
            })
            .collect();

        Ok(SchemaResult {
            tables,
            views,
            functions: vec![],
            schemas: None,
            databases: None,
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
        let safe_table = Self::quote(table_name);

        // Get PKs
        let pk_sql = format!("PRAGMA table_info({})", safe_table);
        let (pk_rows, _, _) = Self::execute_query(pool, &pk_sql, &[]).await?;
        let pk_set: std::collections::HashSet<String> = pk_rows
            .iter()
            .filter(|r| r.get("pk").and_then(|v| v.as_i64()).unwrap_or(0) > 0)
            .filter_map(|r| r.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .collect();

        let where_clause = if !filter.trim().is_empty() {
            format!(" WHERE {}", filter.trim())
        } else {
            String::new()
        };

        let order_clause = if !sort_column.is_empty() {
            let dir = if sort_direction.eq_ignore_ascii_case("desc") { "DESC" } else { "ASC" };
            format!(" ORDER BY {} {}", Self::quote(sort_column), dir)
        } else {
            String::new()
        };

        let sql = format!(
            "SELECT * FROM {}{}{} LIMIT ? OFFSET ?",
            safe_table, where_clause, order_clause
        );
        let (data, mut fields, elapsed) = Self::execute_query(
            pool,
            &sql,
            &[Value::Number(limit.into()), Value::Number(offset.into())],
        )
        .await?;

        for f in &mut fields {
            if pk_set.contains(&f.name) {
                f.is_primary_key = true;
            }
        }

        let count_sql = format!("SELECT COUNT(*) as cnt FROM {}{}", safe_table, where_clause);
        let (count_rows, _, _) = Self::execute_query(pool, &count_sql, &[]).await?;
        let total = count_rows
            .first()
            .and_then(|r| r.get("cnt"))
            .and_then(|v| v.as_i64())
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
            "UPDATE {} SET {} = ? WHERE {} = ?",
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
            let sql = format!("INSERT INTO {} DEFAULT VALUES", safe_table);
            let res = sqlx::query(&sql)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;
            let last_id = res.last_insert_rowid();
            let mut m = HashMap::new();
            m.insert("rowid".to_string(), Value::Number(last_id.into()));
            return Ok(m);
        }

        let cols: Vec<String> = data.keys().map(|k| Self::quote(k)).collect();
        let placeholders: Vec<String> = (0..data.len()).map(|_| "?".to_string()).collect();
        let values: Vec<Value> = data.values().cloned().collect();

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            safe_table,
            cols.join(", "),
            placeholders.join(", ")
        );

        let mut q = sqlx::query(&sql);
        for v in &values {
            q = bind_json_value!(q, v);
        }
        let res = q.execute(pool).await.map_err(|e| e.to_string())?;
        let last_id = res.last_insert_rowid();

        let select_sql = format!("SELECT * FROM {} WHERE rowid = ?", safe_table);
        let (rows, _, _) = Self::execute_query(pool, &select_sql, &[Value::Number(last_id.into())]).await
            .unwrap_or_default();
        if let Some(row) = rows.into_iter().next() {
            return Ok(row);
        }

        let mut m = HashMap::new();
        m.insert("rowid".to_string(), Value::Number(last_id.into()));
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
            q = bind_json_value!(q, v);
        }
        q.execute(pool).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn get_table_columns(&self, table_name: &str) -> Result<Vec<TableColumn>, String> {
        let pool = self.pool()?;
        let sql = format!("PRAGMA table_info({})", Self::quote(table_name));
        let (rows, _, _) = Self::execute_query(pool, &sql, &[]).await?;
        
        let mut columns = Vec::new();
        for r in rows {
            columns.push(TableColumn {
                name: r.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                data_type: r.get("type").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                nullable: r.get("notnull").and_then(|v| v.as_i64()).map(|i| i == 0).unwrap_or(true),
                is_primary_key: r.get("pk").and_then(|v| v.as_i64()).map(|i| i > 0).unwrap_or(false),
                default: r.get("dflt_value").and_then(|v| v.as_str()).map(|s| s.to_string()),
                foreign_key: None,
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
                            fk.target_table.split('.').map(|p| Self::quote(p)).collect::<Vec<String>>().join(".")
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
                "DROP_CONSTRAINT" | "ADD_FOREIGN_KEY" => {
                    return Err("SQLite does not support altering constraints on existing tables".into());
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
        let safe_table = Self::quote(table_name);
        let sql = crate::drivers::utils::build_create_table_sql_generic(&safe_table, columns, |s| Self::quote(s))?;
        sqlx::query(&sql)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn drop_table(&self, table_name: &str, _cascade: bool) -> Result<(), String> {
        let pool = self.pool()?;
        let sql = Self::build_drop_table_sql(table_name);
        sqlx::query(&sql)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn create_schema(&self, _schema_name: &str) -> Result<(), String> {
        Err("Schemas are not supported in SQLite".to_string())
    }

    async fn drop_schema(&self, _schema_name: &str) -> Result<(), String> {
        Err("Schemas are not supported in SQLite".to_string())
    }

    async fn drop_database(&self, _db_name: &str) -> Result<(), String> {
        Err("Dropping database via SQL is not supported in SQLite. Please delete the file manually.".to_string())
    }

    async fn create_database(&self, _db_name: &str) -> Result<(), String> {
        Err("Creating database via SQL is not supported in SQLite. Please create a new connection to a new file.".to_string())
    }


    async fn export_to_csv(&self, table_name: &str, export_path: &str) -> Result<(), String> {
        let pool = self.pool()?;
        let safe_table = Self::quote(table_name);
        let sql = format!("SELECT * FROM {}", safe_table);
        
        let (rows, _, _) = Self::execute_query(pool, &sql, &[]).await?;
        crate::drivers::utils::export_rows_to_csv(&rows, export_path)
    }

    async fn get_schema_details(&self, _schema_name: &str) -> Result<SchemaDetails, String> {
        let pool = self.pool()?;
        
        // 1. Get all tables
        let (table_rows, _, _) = Self::execute_query(
            pool,
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
            &[],
        )
        .await?;

        let mut tables = Vec::new();
        let mut relations = Vec::new();

        for r in table_rows {
            let table_name = r.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            
            // Fetch columns
            let columns = self.get_table_columns(&table_name).await?;
            tables.push(TableInfo {
                name: table_name.clone(),
                columns,
            });

            // Fetch foreign keys
            let fk_sql = format!("PRAGMA foreign_key_list({})", Self::quote(&table_name));
            let (fk_rows, _, _) = Self::execute_query(pool, &fk_sql, &[]).await?;
            for (idx, fk) in fk_rows.iter().enumerate() {
                let target_table = fk.get("table").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let source_col = fk.get("from").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let target_col = fk.get("to").and_then(|v| v.as_str()).unwrap_or("").to_string();
                
                // Construct relation
                relations.push(TableRelation {
                    constraint_name: format!("fk_{}_{}_{}", table_name, source_col, idx),
                    source_table: table_name.clone(),
                    source_column: source_col,
                    target_table,
                    target_column: target_col,
                });
            }
        }

        Ok(SchemaDetails { tables, relations })
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
                data_type: "INTEGER".to_string(),
                nullable: false,
                is_primary_key: true,
                default: None,
                foreign_key: None,
            },
            TableColumn {
                name: "name".to_string(),
                data_type: "TEXT".to_string(),
                nullable: true,
                is_primary_key: false,
                default: Some("'Guest'".to_string()),
                foreign_key: None,
            },
        ];

        let sql = crate::drivers::utils::build_create_table_sql_generic("\"users\"", &columns, |s| SqliteDriver::quote(s)).unwrap();
        assert_eq!(
            sql,
            "CREATE TABLE \"users\" (\"id\" INTEGER NOT NULL PRIMARY KEY, \"name\" TEXT DEFAULT 'Guest')"
        );
    }

    #[test]
    fn test_build_drop_table_sql() {
        let sql = SqliteDriver::build_drop_table_sql("users");
        assert_eq!(sql, "DROP TABLE \"users\"");
    }
}
