use super::{
    AlterOperation, ColumnInfo, Config, DatabaseDriver, QueryResult, SchemaObject, SchemaResult,
    TableColumn, TableDataResult,
};
use async_trait::async_trait;
use deadpool_oracle::{Pool, PoolBuilder};
use oracle_rs::{AuthMode as OracleAuthMode, Config as OracleConfig, Value as OracleValue};
use serde_json::Value as JsonValue;
use std::collections::{HashMap, HashSet};

pub struct OracleDriver {
    pool: Option<Pool>,
    current_schema: String,
    service_name: String,
}

impl OracleDriver {
    pub fn new() -> Self {
        Self {
            pool: None,
            current_schema: String::new(),
            service_name: String::new(),
        }
    }

    fn pool(&self) -> Result<&Pool, String> {
        self.pool.as_ref().ok_or("Not connected".to_string())
    }

    pub fn quote(ident: &str) -> String {
        format!("\"{}\"", ident.replace('"', "\"\""))
    }

    fn split_table_name<'a>(&'a self, table_name: &'a str) -> (&'a str, &'a str) {
        table_name
            .split_once('.')
            .unwrap_or((self.current_schema.as_str(), table_name))
    }

    pub fn qualify_table(owner: &str, table_name: &str) -> String {
        format!("{}.{}", Self::quote(owner), Self::quote(table_name))
    }

    pub fn pagination_clause(limit: i64, offset: i64) -> String {
        format!(" OFFSET {} ROWS FETCH NEXT {} ROWS ONLY", offset.max(0), limit.max(0))
    }

    fn json_to_oracle_value(value: &JsonValue) -> OracleValue {
        match value {
            JsonValue::Null => OracleValue::Null,
            JsonValue::Bool(v) => OracleValue::Boolean(*v),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    OracleValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    OracleValue::Float(f)
                } else {
                    OracleValue::String(n.to_string())
                }
            }
            JsonValue::String(v) => OracleValue::String(v.clone()),
            _ => OracleValue::Json(value.clone()),
        }
    }

    fn oracle_value_to_json(value: &OracleValue) -> JsonValue {
        match value {
            OracleValue::Null => JsonValue::Null,
            OracleValue::String(v) => JsonValue::String(v.clone()),
            OracleValue::Bytes(v) => JsonValue::String(hex::encode(v)),
            OracleValue::Integer(v) => JsonValue::Number((*v).into()),
            OracleValue::Float(v) => serde_json::Number::from_f64(*v)
                .map(JsonValue::Number)
                .unwrap_or(JsonValue::Null),
            OracleValue::Boolean(v) => JsonValue::Bool(*v),
            OracleValue::Json(v) => v.clone(),
            _ => JsonValue::String(value.to_string()),
        }
    }

    fn rows_to_maps(
        result: &oracle_rs::QueryResult,
    ) -> (Vec<HashMap<String, JsonValue>>, Vec<ColumnInfo>) {
        let fields: Vec<ColumnInfo> = result
            .columns
            .iter()
            .map(|col| ColumnInfo {
                name: col.name.clone(),
                data_type: format!("{:?}", col.oracle_type),
                is_primary_key: false,
                is_nullable: true,
            })
            .collect();

        let rows = result
            .rows
            .iter()
            .map(|row| {
                let mut map = HashMap::new();
                for (i, col) in result.columns.iter().enumerate() {
                    let value = row.get(i).unwrap_or(&OracleValue::Null);
                    map.insert(col.name.clone(), Self::oracle_value_to_json(value));
                }
                map
            })
            .collect();

        (rows, fields)
    }

    async fn execute_query(
        pool: &Pool,
        sql: &str,
        params: &[JsonValue],
    ) -> Result<(Vec<HashMap<String, JsonValue>>, Vec<ColumnInfo>, u64), String> {
        let conn = pool.get().await.map_err(|e| e.to_string())?;
        let bind_values: Vec<OracleValue> = params.iter().map(Self::json_to_oracle_value).collect();

        log::info!("oracle: executing query: {}", sql);
        let start = std::time::Instant::now();
        let result = conn
            .query(sql, &bind_values)
            .await
            .map_err(|e| e.to_string())?;
        let elapsed = start.elapsed().as_millis() as u64;
        log::info!("oracle: query finished in {}ms", elapsed);

        let (rows, fields) = Self::rows_to_maps(&result);
        Ok((rows, fields, elapsed))
    }

    async fn execute_dml(pool: &Pool, sql: &str, params: &[JsonValue]) -> Result<(), String> {
        let conn = pool.get().await.map_err(|e| e.to_string())?;
        let bind_values: Vec<OracleValue> = params.iter().map(Self::json_to_oracle_value).collect();
        conn.execute(sql, &bind_values)
            .await
            .map_err(|e| e.to_string())?;
        conn.commit().await.map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[async_trait]
impl DatabaseDriver for OracleDriver {
    async fn connect(&mut self, config: &Config) -> Result<(), String> {
        if config.database.trim().is_empty() {
            return Err("Oracle service name is required".to_string());
        }

        let connect_type = config.oracle_connect_type.trim();
        let oracle_role = config.oracle_role.trim().to_ascii_lowercase();
        let use_implicit_sysdba = oracle_role == "normal"
            && config.username.trim().eq_ignore_ascii_case("sys");
        let auth_mode = match oracle_role.as_str() {
            "sysdba" => OracleAuthMode::SysDba,
            "sysoper" => OracleAuthMode::SysOper,
            "normal" if use_implicit_sysdba => OracleAuthMode::SysDba,
            _ => OracleAuthMode::Normal,
        };

        let mut oracle_config = if connect_type.eq_ignore_ascii_case("sid") {
            OracleConfig::with_sid(
                config.host.clone(),
                config.port,
                config.database.clone(),
                config.username.clone(),
                config.password.clone(),
            )
        } else {
            OracleConfig::new(
                config.host.clone(),
                config.port,
                config.database.clone(),
                config.username.clone(),
                config.password.clone(),
            )
        }
        .auth_mode(auth_mode)
        .connect_timeout(std::time::Duration::from_secs(config.connection_timeout as u64));

        if config.ssl {
            oracle_config = oracle_config.with_tls().map_err(|e| e.to_string())?;
        }

        log::info!(
            "oracle: connecting to {}:{}/{} as {} with role {} using {}",
            config.host,
            config.port,
            config.database,
            config.username,
            if use_implicit_sysdba { "sysdba (implicit for SYS)" } else { config.oracle_role.as_str() },
            if connect_type.eq_ignore_ascii_case("sid") { "SID" } else { "service name" }
        );

        let pool = PoolBuilder::new(oracle_config)
            .max_size(5)
            .build()
            .map_err(|e| e.to_string())?;

        let conn = pool.get().await.map_err(|e| {
            let message = e.to_string();
            if message.contains("Server sent MARKER - authentication rejected") {
                format!(
                    "Oracle authentication rejected. If you connect as SYS, set Oracle Role to SYSDBA and make sure the service/SID matches DBeaver. Details: {}",
                    message
                )
            } else {
                message
            }
        })?;
        drop(conn);

        self.current_schema = config.username.to_uppercase();
        self.service_name = config.database.clone();
        self.pool = Some(pool);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), String> {
        self.pool.take();
        Ok(())
    }

    async fn get_schema(
        &self,
        all_databases: bool,
        schema_name: Option<&str>,
    ) -> Result<SchemaResult, String> {
        let pool = self.pool()?;
        let is_sys = self.current_schema == "SYS";
        let requested_schema = schema_name
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(|name| name.to_uppercase());

        // Always fetch all users (schemas)
        let schema_sql = if is_sys {
            "SELECT USERNAME FROM DBA_USERS ORDER BY USERNAME"
        } else {
            "SELECT USERNAME FROM ALL_USERS WHERE USERNAME NOT IN ('SYS', 'SYSTEM', 'OUTLN', 'XDB', 'CTXSYS', 'MDSYS') ORDER BY USERNAME"
        };
        let (schema_rows, _, _) = Self::execute_query(pool, schema_sql, &[]).await?;

        // Determine object filtering
        let (where_clause, params) = if all_databases {
            if is_sys {
                ("".to_string(), vec![])
            } else {
                ("WHERE OWNER NOT IN ('SYS', 'SYSTEM', 'OUTLN', 'XDB', 'CTXSYS', 'MDSYS')".to_string(), vec![])
            }
        } else {
            let owner = requested_schema.unwrap_or_else(|| self.current_schema.clone());
            ("WHERE OWNER = :1".to_string(), vec![JsonValue::String(owner)])
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

        let table_sql = format!("SELECT TABLE_NAME, OWNER FROM {} {} ORDER BY OWNER, TABLE_NAME", table_view, where_clause);
        let view_sql = format!("SELECT VIEW_NAME, OWNER FROM {} {} ORDER BY OWNER, VIEW_NAME", view_view, where_clause);
        let func_sql = format!(
            "SELECT OBJECT_NAME, OWNER, OBJECT_TYPE FROM {} {} {} OBJECT_TYPE IN ('PROCEDURE', 'FUNCTION') ORDER BY OWNER, OBJECT_NAME",
            obj_view,
            where_clause,
            if where_clause.is_empty() { "WHERE" } else { "AND" }
        );

        // Execute queries sequentially for Oracle to avoid "Connection refused" issues
        // which can happen when administrative accounts trigger rapid concurrent connection attempts.
        let (table_rows, _, _) = Self::execute_query(pool, &table_sql, &params).await?;
        let (view_rows, _, _) = Self::execute_query(pool, &view_sql, &params).await?;
        let (func_rows, _, _) = Self::execute_query(pool, &func_sql, &params).await?;

        let tables = table_rows
            .iter()
            .map(|r| SchemaObject {
                name: r.get("TABLE_NAME").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                schema: Some(r.get("OWNER").and_then(|v| v.as_str()).unwrap_or("").to_string()),
                obj_type: None,
            })
            .collect();

        let views = view_rows
            .iter()
            .map(|r| SchemaObject {
                name: r.get("VIEW_NAME").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                schema: Some(r.get("OWNER").and_then(|v| v.as_str()).unwrap_or("").to_string()),
                obj_type: None,
            })
            .collect();

        let functions = func_rows
            .iter()
            .map(|r| SchemaObject {
                name: r.get("OBJECT_NAME").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                schema: Some(r.get("OWNER").and_then(|v| v.as_str()).unwrap_or("").to_string()),
                obj_type: Some(
                    r.get("OBJECT_TYPE")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                ),
            })
            .collect();

        let schemas = schema_rows
            .iter()
            .map(|r| SchemaObject {
                name: r.get("USERNAME").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                schema: None,
                obj_type: None,
            })
            .collect();

        // For Oracle, we use the service name as the "database" if not listing all.
        // If listing all, we return None for databases so that schemas are listed directly under the connection,
        // which avoids trying to spawn sub-connections for schema names (which would fail as they aren't service names).
        let databases = if all_databases {
            None
        } else {
            Some(vec![SchemaObject {
                name: self.service_name.clone(),
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
        let (owner, table) = self.split_table_name(table_name);
        let qualified_table = Self::qualify_table(owner, table);

        let pk_sql = "SELECT cols.COLUMN_NAME FROM ALL_CONSTRAINTS cons \
            JOIN ALL_CONS_COLUMNS cols ON cons.OWNER = cols.OWNER \
                AND cons.CONSTRAINT_NAME = cols.CONSTRAINT_NAME \
            WHERE cons.CONSTRAINT_TYPE = 'P' AND cols.OWNER = :1 AND cols.TABLE_NAME = :2";
        let pk_params = [
            JsonValue::String(owner.to_string()),
            JsonValue::String(table.to_string()),
        ];

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
            "SELECT * FROM {}{}{}{}",
            qualified_table,
            where_clause,
            order_clause,
            Self::pagination_clause(limit, offset)
        );
        let count_sql = format!("SELECT COUNT(*) AS CNT FROM {}{}", qualified_table, where_clause);

        let (pk_res, data_res, count_res) = tokio::join!(
            Self::execute_query(pool, pk_sql, &pk_params),
            Self::execute_query(pool, &data_sql, &[]),
            Self::execute_query(pool, &count_sql, &[]),
        );

        let (pk_rows, _, _) = pk_res?;
        let (data, mut fields, elapsed) = data_res?;
        let (count_rows, _, _) = count_res?;

        let pk_set: HashSet<String> = pk_rows
            .iter()
            .filter_map(|r| r.get("COLUMN_NAME").and_then(|v| v.as_str()).map(str::to_string))
            .collect();

        for field in &mut fields {
            field.is_primary_key = pk_set.contains(&field.name);
        }

        let total = count_rows
            .first()
            .and_then(|r| r.get("CNT"))
            .and_then(|v| v.as_i64().or_else(|| v.as_str()?.parse::<i64>().ok()))
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
        let (rows, fields, elapsed) = Self::execute_query(pool, sql, &[]).await?;
        let row_count = rows.len();
        Ok(QueryResult {
            rows,
            fields,
            row_count,
            execution_time: elapsed,
        })
    }

    async fn update_cell(
        &self,
        table_name: &str,
        pk_column: &str,
        pk_value: &JsonValue,
        target_column: &str,
        new_value: &JsonValue,
    ) -> Result<(), String> {
        let pool = self.pool()?;
        let (owner, table) = self.split_table_name(table_name);
        let sql = format!(
            "UPDATE {} SET {} = :1 WHERE {} = :2",
            Self::qualify_table(owner, table),
            Self::quote(target_column),
            Self::quote(pk_column),
        );
        Self::execute_dml(pool, &sql, &[new_value.clone(), pk_value.clone()]).await
    }

    async fn insert_row(
        &self,
        table_name: &str,
        data: &HashMap<String, JsonValue>,
    ) -> Result<HashMap<String, JsonValue>, String> {
        if data.is_empty() {
            return Err("Oracle insert requires at least one column value".to_string());
        }

        let pool = self.pool()?;
        let (owner, table) = self.split_table_name(table_name);
        let cols: Vec<String> = data.keys().map(|k| Self::quote(k)).collect();
        let placeholders: Vec<String> = (1..=data.len()).map(|i| format!(":{}", i)).collect();
        let values: Vec<JsonValue> = data.values().cloned().collect();

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            Self::qualify_table(owner, table),
            cols.join(", "),
            placeholders.join(", "),
        );

        Self::execute_dml(pool, &sql, &values).await?;
        Ok(data.clone())
    }

    async fn delete_rows(
        &self,
        table_name: &str,
        pk_column: &str,
        pk_values: &[JsonValue],
    ) -> Result<(), String> {
        if pk_values.is_empty() {
            return Ok(());
        }

        let pool = self.pool()?;
        let (owner, table) = self.split_table_name(table_name);
        let placeholders: Vec<String> = (1..=pk_values.len()).map(|i| format!(":{}", i)).collect();
        let sql = format!(
            "DELETE FROM {} WHERE {} IN ({})",
            Self::qualify_table(owner, table),
            Self::quote(pk_column),
            placeholders.join(", "),
        );
        Self::execute_dml(pool, &sql, pk_values).await
    }

    async fn get_table_columns(&self, table_name: &str) -> Result<Vec<TableColumn>, String> {
        let pool = self.pool()?;
        let (owner, table) = self.split_table_name(table_name);
        let sql = "SELECT c.COLUMN_NAME, c.DATA_TYPE, c.NULLABLE, c.DATA_DEFAULT, \
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
        let params = [
            JsonValue::String(owner.to_string()),
            JsonValue::String(table.to_string()),
        ];
        let (rows, _, _) = Self::execute_query(pool, sql, &params).await?;

        Ok(rows
            .iter()
            .map(|r| TableColumn {
                name: r.get("COLUMN_NAME").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                data_type: r.get("DATA_TYPE").and_then(|v| v.as_str()).unwrap_or("").to_string(),
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
                default: r.get("DATA_DEFAULT").and_then(|v| v.as_str()).map(str::to_string),
            })
            .collect())
    }

    async fn create_table(&self, table_name: &str, columns: &[TableColumn]) -> Result<(), String> {
        let pool = self.pool()?;
        let (owner, table) = self.split_table_name(table_name);
        let safe_table = Self::qualify_table(owner, table);

        let sql = crate::drivers::utils::build_create_table_sql_generic(&safe_table, columns, |s| Self::quote(s))?;
        Self::execute_dml(pool, &sql, &[]).await?;
        Ok(())
    }

    async fn alter_table(
        &self,
        table_name: &str,
        operations: &[AlterOperation],
    ) -> Result<(), String> {
        let pool = self.pool()?;
        let (owner, table) = self.split_table_name(table_name);
        let safe_table = Self::qualify_table(owner, table);

        for op in operations {
            let sql = match op.op_type.as_str() {
                "ADD_COLUMN" => {
                    if !crate::drivers::utils::is_safe_data_type(&op.data_type) {
                        return Err(format!("Invalid or unsafe data type: {}", op.data_type));
                    }
                    let mut q = format!(
                        "ALTER TABLE {} ADD {} {}",
                        safe_table,
                        Self::quote(&op.name),
                        op.data_type
                    );
                    if let Some(ref d) = op.default {
                        let d_str = d.to_string();
                        if !crate::drivers::utils::is_safe_default(&d_str) {
                             return Err(format!("Invalid or unsafe default value: {}", d_str));
                        }
                        q.push_str(&format!(" DEFAULT {}", d_str));
                    }
                    if op.nullable == Some(false) {
                        q.push_str(" NOT NULL");
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

            Self::execute_dml(pool, &sql, &[]).await?;
        }
        Ok(())
    }

    async fn drop_table(&self, table_name: &str) -> Result<(), String> {
        let pool = self.pool()?;
        let (owner, table) = self.split_table_name(table_name);
        let sql = format!("DROP TABLE {}", Self::qualify_table(owner, table));
        Self::execute_dml(pool, &sql, &[]).await?;
        Ok(())
    }

    async fn create_schema(&self, schema_name: &str) -> Result<(), String> {
        let pool = self.pool()?;
        let sql = format!(
            "CREATE USER {} IDENTIFIED BY {}",
            Self::quote(schema_name),
            Self::quote(schema_name)
        );
        Self::execute_dml(pool, &sql, &[]).await?;
        Ok(())
    }

    async fn drop_schema(&self, schema_name: &str) -> Result<(), String> {
        let pool = self.pool()?;
        let sql = format!("DROP USER {} CASCADE", Self::quote(schema_name));
        Self::execute_dml(pool, &sql, &[]).await?;
        Ok(())
    }

    async fn drop_database(&self, _db_name: &str) -> Result<(), String> {
        Err("Dropping database via SQL is not supported in Oracle. Please use database management tools.".to_string())
    }

    async fn create_database(&self, _db_name: &str) -> Result<(), String> {
        Err("Creating database via SQL is not supported in Oracle. Please use database management tools.".to_string())
    }

    async fn export_to_csv(&self, table_name: &str, export_path: &str) -> Result<(), String> {
        let pool = self.pool()?;
        let (owner, table) = self.split_table_name(table_name);
        let sql = format!("SELECT * FROM {}", Self::qualify_table(owner, table));
        let (rows, _, _) = Self::execute_query(pool, &sql, &[]).await?;

        crate::drivers::utils::export_rows_to_csv(&rows, export_path)
    }
}

#[cfg(test)]
mod tests {
    use super::OracleDriver;

    #[test]
    fn quotes_oracle_identifiers() {
        assert_eq!(OracleDriver::quote("USER_TABLE"), "\"USER_TABLE\"");
        assert_eq!(OracleDriver::quote("odd\"name"), "\"odd\"\"name\"");
    }

    #[test]
    fn builds_owner_qualified_table_name() {
        assert_eq!(
            OracleDriver::qualify_table("HR", "EMPLOYEES"),
            "\"HR\".\"EMPLOYEES\""
        );
    }

    #[test]
    fn builds_12c_pagination_clause() {
        assert_eq!(
            OracleDriver::pagination_clause(50, 100),
            " OFFSET 100 ROWS FETCH NEXT 50 ROWS ONLY"
        );
    }
}
