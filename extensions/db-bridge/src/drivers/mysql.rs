use super::{
    AlterOperation, Config, DatabaseDriver, QueryResult, SchemaObject, SchemaResult,
    TableColumn, TableDataResult, ColumnInfo, SchemaDetails, TableInfo, TableRelation, ForeignKeyDef, DbIndex, TableConstraint
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
    tx_conn: tokio::sync::Mutex<Option<sqlx::pool::PoolConnection<sqlx::MySql>>>,
}

impl MysqlDriver {
    pub fn new() -> Self {
        Self {
            pool: None,
            database: String::new(),
            tx_conn: tokio::sync::Mutex::new(None),
        }
    }

    fn pool(&self) -> Result<&MySqlPool, String> {
        self.pool.as_ref().ok_or("Not connected".to_string())
    }

    fn quote(ident: &str) -> String {
        format!("`{}`", ident.replace('`', "``"))
    }

    async fn execute_query_on_executor<'a, E>(
        executor: E,
        sql: &str,
        params: &[Value],
    ) -> Result<(Vec<HashMap<String, Value>>, Vec<ColumnInfo>, u64), String>
    where
        E: sqlx::Executor<'a, Database = sqlx::MySql>,
    {
        let mut q = sqlx::query(sql);
        for p in params {
            q = bind_json_value!(q, p);
        }

        log::info!("mysql: executing query: {}", sql);
        let start = std::time::Instant::now();
        let rows = q.fetch_all(executor).await.map_err(|e| e.to_string())?;
        let elapsed = start.elapsed().as_millis() as u64;
        log::info!("mysql: query finished in {}ms", elapsed);

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

    async fn execute_query(
        pool: &MySqlPool,
        sql: &str,
        params: &[Value],
    ) -> Result<(Vec<HashMap<String, Value>>, Vec<ColumnInfo>, u64), String> {
        Self::execute_query_on_executor(pool, sql, params).await
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
        use sqlx::Executor;
        use sqlx::Either;
        use futures_util::StreamExt;

        let start = std::time::Instant::now();
        log::info!("mysql: executing raw query: {}", sql);

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
        log::info!("mysql: raw query finished in {}ms", elapsed);

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
        for row in rows {
            let mut map = std::collections::HashMap::new();
            for (i, unique_name) in unique_names.iter().enumerate() {
                let val = Self::get_column_value(&row, i);
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
        
        let fk_sql = "SELECT \
            COLUMN_NAME, \
            CASE \
                WHEN REFERENCED_TABLE_SCHEMA = TABLE_SCHEMA THEN REFERENCED_TABLE_NAME \
                ELSE CONCAT(REFERENCED_TABLE_SCHEMA, '.', REFERENCED_TABLE_NAME) \
            END AS REFERENCED_TABLE, \
            REFERENCED_COLUMN_NAME \
        FROM \
            INFORMATION_SCHEMA.KEY_COLUMN_USAGE \
        WHERE \
            TABLE_SCHEMA = DATABASE() \
            AND TABLE_NAME = ? \
            AND REFERENCED_TABLE_NAME IS NOT NULL";

        let (fk_rows, _, _) = Self::execute_query(pool, fk_sql, &[Value::String(table_name.to_string())])
            .await.unwrap_or((vec![], vec![], 0));

        let mut fk_map = HashMap::new();
        for r in fk_rows {
            if let (Some(col), Some(ref_tbl), Some(ref_col)) = (
                r.get("COLUMN_NAME").and_then(|v| v.as_str()),
                r.get("REFERENCED_TABLE").and_then(|v| v.as_str()),
                r.get("REFERENCED_COLUMN_NAME").and_then(|v| v.as_str()),
            ) {
                fk_map.insert(col.to_string(), ForeignKeyDef {
                    target_table: ref_tbl.to_string(),
                    target_column: ref_col.to_string(),
                });
            }
        }

        let sql = "SELECT COLUMN_NAME, DATA_TYPE, IS_NULLABLE, COLUMN_DEFAULT \
             FROM INFORMATION_SCHEMA.COLUMNS \
             WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ? \
             ORDER BY ORDINAL_POSITION";
        
        let (rows, _, _) = Self::execute_query(pool, sql, &[Value::String(table_name.to_string())]).await?;
        
        let mut columns = Vec::new();
        for r in rows {
            let col_name = r.get("COLUMN_NAME").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let foreign_key = fk_map.get(&col_name).cloned();
            columns.push(TableColumn {
                name: col_name,
                data_type: r.get("DATA_TYPE").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                nullable: r.get("IS_NULLABLE").and_then(|v| v.as_str()).map(|s| s == "YES").unwrap_or(true),
                is_primary_key: false, 
                default: r.get("COLUMN_DEFAULT").and_then(|v| v.as_str()).map(|s| s.to_string()),
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
                            fk.target_table.split('.').map(Self::quote).collect::<Vec<String>>().join(".")
                        } else {
                            Self::quote(&fk.target_table)
                        };
                        q.push_str(&format!(
                            ", ADD FOREIGN KEY ({}) REFERENCES {} ({})",
                            Self::quote(&op.name),
                            quoted_target_table,
                            Self::quote(&fk.target_column)
                        ));
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
                    format!("ALTER TABLE {} DROP FOREIGN KEY {}", safe_table, Self::quote(constraint))
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
        let safe_table = Self::quote(table_name);
        let sql = crate::drivers::utils::build_create_table_sql_generic(&safe_table, columns, Self::quote)?;
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
                default: r.get("column_default").and_then(|v| v.as_str()).map(str::to_string),
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
        
        let sql = r#"
            SELECT 
                INDEX_NAME as index_name,
                NON_UNIQUE as non_unique,
                INDEX_TYPE as index_type,
                JSON_ARRAYAGG(COLUMN_NAME) as columns
            FROM INFORMATION_SCHEMA.STATISTICS
            WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ?
            GROUP BY INDEX_NAME, NON_UNIQUE, INDEX_TYPE
            ORDER BY INDEX_NAME
        "#;
        
        let (rows, _, _) = Self::execute_query(
            pool,
            sql,
            &[Value::String(table_name.to_string())],
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

            let name = r.get("index_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let is_unique = r.get("non_unique").and_then(|v| v.as_i64()).unwrap_or(1) == 0;
            let is_primary_key = name == "PRIMARY";

            let ddl = if is_primary_key {
                format!("PRIMARY KEY ({})", cols.join(", "))
            } else if is_unique {
                format!("UNIQUE KEY {} ({})", Self::quote(&name), cols.join(", "))
            } else {
                format!("KEY {} ({})", Self::quote(&name), cols.join(", "))
            };

            indexes.push(DbIndex {
                name,
                is_unique,
                is_primary_key,
                index_type: r.get("index_type").and_then(|v| v.as_str()).map(|s| s.to_string()),
                ddl: Some(ddl),
                columns: cols,
            });
        }
        Ok(indexes)
    }

    async fn get_table_constraints(&self, table_name: &str) -> Result<Vec<TableConstraint>, String> {
        let pool = self.pool()?;
        
        let sql = r#"
            SELECT
                tc.CONSTRAINT_NAME as constraint_name,
                tc.CONSTRAINT_TYPE as constraint_type,
                kcu.COLUMN_NAME as column_name,
                kcu.REFERENCED_TABLE_NAME as referenced_table,
                kcu.REFERENCED_COLUMN_NAME as referenced_column,
                cc.CHECK_CLAUSE as check_clause
            FROM INFORMATION_SCHEMA.TABLE_CONSTRAINTS tc
            LEFT JOIN INFORMATION_SCHEMA.KEY_COLUMN_USAGE kcu
              ON tc.CONSTRAINT_SCHEMA = kcu.CONSTRAINT_SCHEMA
              AND tc.CONSTRAINT_NAME = kcu.CONSTRAINT_NAME
              AND tc.TABLE_NAME = kcu.TABLE_NAME
            LEFT JOIN INFORMATION_SCHEMA.CHECK_CONSTRAINTS cc
              ON tc.CONSTRAINT_SCHEMA = cc.CONSTRAINT_SCHEMA
              AND tc.CONSTRAINT_NAME = cc.CONSTRAINT_NAME
            WHERE tc.TABLE_SCHEMA = DATABASE() AND tc.TABLE_NAME = ?
            ORDER BY tc.CONSTRAINT_NAME, kcu.ORDINAL_POSITION
        "#;

        let (rows, _, _) = Self::execute_query(
            pool,
            sql,
            &[Value::String(table_name.to_string())],
        ).await?;

        // Group rows by constraint name
        let mut constraint_map: HashMap<String, (String, Vec<String>, Vec<String>, String, String)> = HashMap::new();
        let mut constraint_order = Vec::new();

        for r in rows {
            let name = r.get("constraint_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let c_type = r.get("constraint_type").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let col = r.get("column_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let ref_tbl = r.get("referenced_table").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let ref_col = r.get("referenced_column").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let check_clause = r.get("check_clause").and_then(|v| v.as_str()).unwrap_or("").to_string();

            if !constraint_map.contains_key(&name) {
                constraint_order.push(name.clone());
            }

            let entry = constraint_map.entry(name).or_insert_with(|| (c_type, Vec::new(), Vec::new(), ref_tbl, check_clause));
            if !col.is_empty() {
                entry.1.push(col);
            }
            if !ref_col.is_empty() {
                entry.2.push(ref_col);
            }
        }

        let mut constraints = Vec::new();
        for name in constraint_order {
            if let Some((c_type, cols, ref_cols, ref_tbl, check_clause)) = constraint_map.remove(&name) {
                let definition = match c_type.as_str() {
                    "PRIMARY KEY" => format!("PRIMARY KEY ({})", cols.join(", ")),
                    "UNIQUE" => format!("UNIQUE ({})", cols.join(", ")),
                    "FOREIGN KEY" => format!(
                        "FOREIGN KEY ({}) REFERENCES {} ({})",
                        cols.join(", "),
                        ref_tbl,
                        ref_cols.join(", ")
                    ),
                    "CHECK" => format!("CHECK ({})", check_clause),
                    _ => format!("{} ({})", c_type, cols.join(", ")),
                };

                constraints.push(TableConstraint {
                    name,
                    constraint_type: c_type,
                    definition,
                });
            }
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
                data_type: "INT".to_string(),
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

        let sql = crate::drivers::utils::build_create_table_sql_generic("`users`", &columns, MysqlDriver::quote).unwrap();
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
