use super::{
    AlterOperation, Config, DatabaseDriver, QueryResult, SchemaObject, SchemaResult,
    TableColumn, TableDataResult, ColumnInfo, SchemaDetails, TableInfo, TableRelation
};
use crate::bind_json_value;
use async_trait::async_trait;
use serde_json::Value;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions, MySqlRow};
use sqlx::{Column, Row, TypeInfo, ValueRef};
use std::collections::HashMap;

pub struct MysqlDriver {
    pool: Option<MySqlPool>,
    database: String,
}

impl MysqlDriver {
    pub fn new() -> Self {
        Self { pool: None, database: String::new() }
    }

    fn pool(&self) -> Result<&MySqlPool, String> {
        self.pool.as_ref().ok_or("Not connected".to_string())
    }

    fn quote(ident: &str) -> String {
        format!("`{}`", ident.replace('`', "``"))
    }

    async fn execute_query(
        pool: &MySqlPool,
        sql: &str,
        params: &[Value],
    ) -> Result<(Vec<HashMap<String, Value>>, Vec<ColumnInfo>, u64), String> {
        let mut q = sqlx::query(sql);
        for p in params {
            q = bind_json_value!(q, p);
        }

        log::info!("mysql: executing query: {}", sql);
        let start = std::time::Instant::now();
        let rows = q.fetch_all(pool).await.map_err(|e| e.to_string())?;
        let elapsed = start.elapsed().as_millis() as u64;
        log::info!("mysql: query finished in {}ms", elapsed);

        if rows.is_empty() {
            return Ok((vec![], vec![], elapsed));
        }

        let mut fields = Vec::new();
        for col in rows[0].columns() {
            fields.push(ColumnInfo {
                name: col.name().to_string(),
                data_type: col.type_info().name().to_string(),
                is_primary_key: false,
                is_nullable: true,
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

    fn get_column_value(row: &MySqlRow, i: usize) -> Value {
        let raw = match row.try_get_raw(i) {
            Ok(v) => v,
            Err(_) => return Value::Null,
        };
        if raw.is_null() {
            return Value::Null;
        }

        let col = row.column(i);
        let type_info = col.type_info();
        let name = type_info.name();

        match name {
            "TINYINT" | "SMALLINT" | "INT" | "MEDIUMINT" | "BIGINT" | "YEAR" => {
                if let Ok(v) = row.try_get::<i64, _>(i) {
                    return Value::Number(v.into());
                }
            }
            "FLOAT" | "DOUBLE" | "DECIMAL" | "NEWDECIMAL" => {
                if let Ok(v) = row.try_get::<f64, _>(i) {
                    if let Some(num) = serde_json::Number::from_f64(v) {
                        return Value::Number(num);
                    }
                }
            }
            "TINYINT(1)" | "BIT" => {
                if let Ok(v) = row.try_get::<bool, _>(i) {
                    return Value::Bool(v);
                }
            }
            "VARCHAR" | "CHAR" | "TEXT" | "MEDIUMTEXT" | "LONGTEXT" | "TINYTEXT" | "ENUM" | "SET" => {
                if let Ok(v) = row.try_get::<String, _>(i) {
                    return Value::String(v);
                }
            }
            "DATETIME" | "TIMESTAMP" => {
                if let Ok(v) = row.try_get::<chrono::NaiveDateTime, _>(i) {
                    return Value::String(v.to_string());
                }
            }
            "DATE" => {
                if let Ok(v) = row.try_get::<chrono::NaiveDate, _>(i) {
                    return Value::String(v.to_string());
                }
            }
            "BLOB" | "MEDIUMBLOB" | "LONGBLOB" | "TINYBLOB" | "VARBINARY" | "BINARY" => {
                if let Ok(bytes) = row.try_get::<Vec<u8>, _>(i) {
                    return Value::String(hex::encode(bytes));
                }
            }
            "JSON" => {
                if let Ok(v) = row.try_get::<Value, _>(i) {
                    return v;
                }
            }
            _ => {}
        }

        // Fallback
        if let Ok(v) = row.try_get::<String, _>(i) {
            return Value::String(v);
        }

        Value::Null
    }


    fn build_drop_table_sql(table_name: &str) -> String {
        let safe_table = Self::quote(table_name);
        format!("DROP TABLE {}", safe_table)
    }
}

#[async_trait]
impl DatabaseDriver for MysqlDriver {
    async fn connect(&mut self, config: &Config) -> Result<(), String> {
        let host = if config.host.contains(':') && !config.host.starts_with('[') {
            format!("[{}]", config.host)
        } else {
            config.host.clone()
        };

        let dsn = format!(
            "mysql://{}:{}@{}:{}/{}",
            urlencoding::encode(&config.username),
            urlencoding::encode(&config.password),
            host,
            config.port,
            urlencoding::encode(&config.database)
        );

        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(config.connection_timeout as u64))
            .connect(&dsn)
            .await
            .map_err(|e| e.to_string())?;

        self.pool = Some(pool);
        self.database = config.database.clone();
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
        all_databases: bool,
        schema_name: Option<&str>,
    ) -> Result<SchemaResult, String> {
        let pool = self.pool()?;
        
        let (where_clause, params) = if all_databases {
            ("WHERE TABLE_SCHEMA NOT IN ('information_schema', 'performance_schema', 'mysql', 'sys')".to_string(), vec![])
        } else if let Some(s) = schema_name {
            ("WHERE TABLE_SCHEMA = ?".to_string(), vec![Value::String(s.to_string())])
        } else {
            ("WHERE TABLE_SCHEMA = DATABASE()".to_string(), vec![])
        };

        let table_sql = format!(
            "SELECT TABLE_NAME as table_name, TABLE_SCHEMA as table_schema FROM information_schema.TABLES {} AND TABLE_TYPE = 'BASE TABLE' ORDER BY TABLE_NAME",
            where_clause
        );
        let view_sql = format!(
            "SELECT TABLE_NAME as table_name, TABLE_SCHEMA as table_schema FROM information_schema.TABLES {} AND TABLE_TYPE = 'VIEW' ORDER BY TABLE_NAME",
            where_clause
        );
        let routine_where = where_clause.replace("TABLE_SCHEMA", "ROUTINE_SCHEMA");
        let func_sql = format!(
            "SELECT ROUTINE_NAME as routine_name, ROUTINE_SCHEMA as routine_schema, ROUTINE_TYPE as routine_type FROM information_schema.ROUTINES {} ORDER BY ROUTINE_NAME",
            routine_where
        );
        
        // In MySQL, schemas are databases. We always fetch all schemas.
        let all_schemas_sql = "SELECT SCHEMA_NAME as schema_name FROM information_schema.SCHEMATA WHERE SCHEMA_NAME NOT IN ('information_schema', 'performance_schema', 'mysql', 'sys') ORDER BY SCHEMA_NAME".to_string();
        
        // Databases listing depends on all_databases flag
        let db_sql = if all_databases {
            "SELECT SCHEMA_NAME as db_name FROM information_schema.SCHEMATA WHERE SCHEMA_NAME NOT IN ('information_schema', 'performance_schema', 'mysql', 'sys') ORDER BY SCHEMA_NAME".to_string()
        } else {
            format!("SELECT SCHEMA_NAME as db_name FROM information_schema.SCHEMATA WHERE SCHEMA_NAME = '{}'", self.database)
        };
        
        let (table_res, view_res, func_res, schema_res, db_res) = tokio::join!(
            Self::execute_query(pool, &table_sql, &params),
            Self::execute_query(pool, &view_sql, &params),
            Self::execute_query(pool, &func_sql, &params),
            Self::execute_query(pool, &all_schemas_sql, &[]),
            Self::execute_query(pool, &db_sql, &[])
        );

        let (table_rows, _, _) = table_res?;
        let (view_rows, _, _) = view_res?;
        let (func_rows, _, _) = func_res?;
        let (schema_rows, _, _) = schema_res?;
        let (db_rows, _, _) = db_res?;

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
                obj_type: Some(r.get("routine_type").and_then(|v| v.as_str()).unwrap_or("").to_string()),
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

        let databases: Vec<SchemaObject> = db_rows
            .iter()
            .map(|r| SchemaObject {
                name: r.get("db_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
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
        let safe_table = Self::quote(table_name);

        let pk_sql = "SELECT COLUMN_NAME FROM information_schema.COLUMNS \
            WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ? AND COLUMN_KEY = 'PRI'";
        
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

        let data_sql = format!(
            "SELECT * FROM {}{}{} LIMIT ? OFFSET ?",
            safe_table, where_clause, order_clause
        );
        let count_sql = format!("SELECT COUNT(*) as cnt FROM {}{}", safe_table, where_clause);

        let pk_params = [Value::String(table_name.to_string())];
        let data_params = [
            Value::Number(limit.into()),
            Value::Number(offset.into()),
        ];
        let count_params = [];

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
            .filter_map(|r| r.get("COLUMN_NAME").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .collect();

        for f in &mut fields {
            if pk_set.contains(&f.name) {
                f.is_primary_key = true;
            }
        }

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
            let sql = format!("INSERT INTO {} () VALUES ()", safe_table);
            let res = sqlx::query(&sql)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;
            let last_id = res.last_insert_id();
            let mut m = HashMap::new();
            m.insert("insertId".to_string(), Value::Number(last_id.into()));
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
        let last_id = res.last_insert_id();

        // Try to SELECT the inserted row back
        let select_sql = format!("SELECT * FROM {} WHERE id = ?", safe_table);
        let (rows, _, _) = Self::execute_query(pool, &select_sql, &[Value::Number(last_id.into())]).await
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
            q = bind_json_value!(q, v);
        }
        q.execute(pool).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn get_table_columns(&self, table_name: &str) -> Result<Vec<TableColumn>, String> {
        let pool = self.pool()?;
        let sql = "SELECT COLUMN_NAME, DATA_TYPE, IS_NULLABLE, COLUMN_DEFAULT \
             FROM INFORMATION_SCHEMA.COLUMNS \
             WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ? \
             ORDER BY ORDINAL_POSITION";
        
        let (rows, _, _) = Self::execute_query(pool, sql, &[Value::String(table_name.to_string())]).await?;
        
        let mut columns = Vec::new();
        for r in rows {
            columns.push(TableColumn {
                name: r.get("COLUMN_NAME").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                data_type: r.get("DATA_TYPE").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                nullable: r.get("IS_NULLABLE").and_then(|v| v.as_str()).map(|s| s == "YES").unwrap_or(true),
                is_primary_key: false, 
                default: r.get("COLUMN_DEFAULT").and_then(|v| v.as_str()).map(|s| s.to_string()),
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

    async fn drop_table(&self, table_name: &str) -> Result<(), String> {
        let pool = self.pool()?;
        let sql = Self::build_drop_table_sql(table_name);
        sqlx::query(&sql)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn create_schema(&self, schema_name: &str) -> Result<(), String> {
        let pool = self.pool()?;
        let sql = format!("CREATE DATABASE {}", Self::quote(schema_name));
        sqlx::query(&sql)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn drop_schema(&self, schema_name: &str) -> Result<(), String> {
        let pool = self.pool()?;
        let sql = format!("DROP DATABASE {}", Self::quote(schema_name));
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
        let safe_table = Self::quote(table_name);
        let sql = format!("SELECT * FROM {}", safe_table);
        
        let (rows, _, _) = Self::execute_query(pool, &sql, &[]).await?;
        crate::drivers::utils::export_rows_to_csv(&rows, export_path)
    }

    async fn get_schema_details(&self, schema_name: &str) -> Result<SchemaDetails, String> {
        let pool = self.pool()?;
        
        let column_sql = "SELECT \
                TABLE_NAME as table_name, \
                COLUMN_NAME as column_name, \
                DATA_TYPE as data_type, \
                (IS_NULLABLE = 'YES') as is_nullable, \
                COLUMN_DEFAULT as column_default, \
                (COLUMN_KEY = 'PRI') as is_primary_key \
            FROM INFORMATION_SCHEMA.COLUMNS \
            WHERE TABLE_SCHEMA = ? \
            ORDER BY TABLE_NAME, ORDINAL_POSITION";

        let relation_sql = "SELECT \
                CONSTRAINT_NAME as constraint_name, \
                TABLE_NAME as source_table, \
                COLUMN_NAME as source_column, \
                REFERENCED_TABLE_NAME as target_table, \
                REFERENCED_COLUMN_NAME as target_column \
            FROM INFORMATION_SCHEMA.KEY_COLUMN_USAGE \
            WHERE TABLE_SCHEMA = ? \
              AND REFERENCED_TABLE_NAME IS NOT NULL";

        let params = [Value::String(schema_name.to_string())];

        let (column_res, relation_res) = tokio::join!(
            Self::execute_query(pool, column_sql, &params),
            Self::execute_query(pool, relation_sql, &params)
        );

        let (column_rows, _, _) = column_res?;
        let (relation_rows, _, _) = relation_res?;

        let mut tables_map: HashMap<String, Vec<TableColumn>> = HashMap::new();
        let mut table_order: Vec<String> = Vec::new();

        for r in column_rows {
            let table_name = r.get("table_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let col = TableColumn {
                name: r.get("column_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                data_type: r.get("data_type").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                nullable: r.get("is_nullable").and_then(|v| {
                    if let Value::Bool(b) = v {
                        Some(*b)
                    } else if let Value::Number(n) = v {
                        Some(n.as_i64().unwrap_or(0) != 0)
                    } else {
                        None
                    }
                }).unwrap_or(true),
                is_primary_key: r.get("is_primary_key").and_then(|v| {
                    if let Value::Bool(b) = v {
                        Some(*b)
                    } else if let Value::Number(n) = v {
                        Some(n.as_i64().unwrap_or(0) != 0)
                    } else {
                        None
                    }
                }).unwrap_or(false),
                default: r.get("column_default").and_then(|v| v.as_str()).map(|s| s.to_string()),
            };

            if !tables_map.contains_key(&table_name) {
                table_order.push(table_name.clone());
            }
            tables_map.entry(table_name).or_insert_with(Vec::new).push(col);
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
                data_type: "INT".to_string(),
                nullable: false,
                is_primary_key: true,
                default: None,
            },
            TableColumn {
                name: "name".to_string(),
                data_type: "VARCHAR(255)".to_string(),
                nullable: true,
                is_primary_key: false,
                default: Some("'Guest'".to_string()),
            },
        ];

        let sql = crate::drivers::utils::build_create_table_sql_generic("`users`", &columns, |s| MysqlDriver::quote(s)).unwrap();
        assert_eq!(
            sql,
            "CREATE TABLE `users` (`id` INT NOT NULL PRIMARY KEY, `name` VARCHAR(255) DEFAULT 'Guest')"
        );
    }

    #[test]
    fn test_build_drop_table_sql() {
        let sql = MysqlDriver::build_drop_table_sql("users");
        assert_eq!(sql, "DROP TABLE `users`".to_string());
    }
}
