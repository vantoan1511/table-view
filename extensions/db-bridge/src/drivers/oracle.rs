use super::{
    AlterOperation, ColumnInfo, Config, DatabaseDriver, QueryResult, SchemaObject, SchemaResult,
    TableColumn, TableDataResult, SchemaDetails, TableInfo, TableRelation, ForeignKeyDef, DbIndex, TableConstraint
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
    tx_conn: tokio::sync::Mutex<Option<deadpool_oracle::Object>>,
}

impl OracleDriver {
    pub fn new() -> Self {
        Self {
            pool: None,
            current_schema: String::new(),
            service_name: String::new(),
            tx_conn: tokio::sync::Mutex::new(None),
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

    fn clean_sql(sql: &str) -> String {
        let mut cleaned = sql.trim().to_string();
        let lower = cleaned.to_ascii_lowercase();
        if !lower.starts_with("declare") && !lower.starts_with("begin") {
            while cleaned.ends_with(';') {
                cleaned.pop();
                cleaned = cleaned.trim().to_string();
            }
        }
        cleaned
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
            _ => OracleValue::String(value.to_string()),
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
        let orig_names: Vec<String> = result.columns.iter().map(|col| col.name.clone()).collect();
        let unique_names = crate::drivers::utils::make_unique_column_names(&orig_names);

        let fields: Vec<ColumnInfo> = result
            .columns
            .iter()
            .zip(unique_names.iter())
            .map(|(col, unique_name)| {
                let display_name = if unique_name != &col.name {
                    Some(col.name.clone())
                } else {
                    None
                };
                ColumnInfo {
                    name: unique_name.clone(),
                    data_type: format!("{:?}", col.oracle_type),
                    is_primary_key: false,
                    is_nullable: true,
                    display_name,
                }
            })
            .collect();

        let rows = result
            .rows
            .iter()
            .map(|row| {
                let mut map = HashMap::new();
                for (i, unique_name) in unique_names.iter().enumerate() {
                    let value = row.get(i).unwrap_or(&OracleValue::Null);
                    map.insert(unique_name.clone(), Self::oracle_value_to_json(value));
                }
                map
            })
            .collect();

        (rows, fields)
    }

    async fn execute_query_on_conn(
        conn: &oracle_rs::Connection,
        sql: &str,
        params: &[JsonValue],
    ) -> Result<(Vec<HashMap<String, JsonValue>>, Vec<ColumnInfo>, u64), String> {
        let bind_values: Vec<OracleValue> = params.iter().map(Self::json_to_oracle_value).collect();
        let cleaned_sql = Self::clean_sql(sql);

        log::info!("oracle: executing query: {}", cleaned_sql);
        let start = std::time::Instant::now();
        let lower = cleaned_sql.trim().to_ascii_lowercase();
        let is_query = lower.starts_with("select") || lower.starts_with("with");

        let result = if is_query {
            conn.query(&cleaned_sql, &bind_values)
                .await
                .map_err(|e| e.to_string())?
        } else {
            conn.execute(&cleaned_sql, &bind_values)
                .await
                .map_err(|e| e.to_string())?
        };
        let elapsed = start.elapsed().as_millis() as u64;
        log::info!("oracle: query finished in {}ms", elapsed);

        let (rows, fields) = Self::rows_to_maps(&result);
        Ok((rows, fields, elapsed))
    }

    async fn execute_query(
        pool: &Pool,
        sql: &str,
        params: &[JsonValue],
    ) -> Result<(Vec<HashMap<String, JsonValue>>, Vec<ColumnInfo>, u64), String> {
        let conn = pool.get().await.map_err(|e| e.to_string())?;
        Self::execute_query_on_conn(&conn, sql, params).await
    }

    async fn execute_dml(pool: &Pool, sql: &str, params: &[JsonValue]) -> Result<(), String> {
        let conn = pool.get().await.map_err(|e| e.to_string())?;
        let bind_values: Vec<OracleValue> = params.iter().map(Self::json_to_oracle_value).collect();
        let cleaned_sql = Self::clean_sql(sql);
        conn.execute(&cleaned_sql, &bind_values)
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
        {
            let mut tx = self.tx_conn.lock().await;
            *tx = None;
        }
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

        let mut schemas: Vec<SchemaObject> = schema_rows
            .iter()
            .map(|r| SchemaObject {
                name: r.get("USERNAME").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                schema: None,
                obj_type: None,
            })
            .collect();

        if !self.current_schema.is_empty()
            && !schemas.iter().any(|s| s.name.eq_ignore_ascii_case(&self.current_schema))
        {
            schemas.insert(
                0,
                SchemaObject {
                    name: self.current_schema.clone(),
                    schema: None,
                    obj_type: None,
                },
            );
        }

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
        let mut tx_guard = self.tx_conn.lock().await;
        let (rows, fields, elapsed) = if let Some(ref mut conn) = *tx_guard {
            Self::execute_query_on_conn(conn, sql, &[]).await?
        } else {
            let pool = self.pool()?;
            Self::execute_query(pool, sql, &[]).await?
        };
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
        
        let params = [
            JsonValue::String(owner.to_string()),
            JsonValue::String(table.to_string()),
        ];

        let fk_sql = "SELECT \
            a.COLUMN_NAME AS SOURCE_COLUMN, \
            CASE \
                WHEN c_pk.OWNER = c.OWNER THEN c_pk.TABLE_NAME \
                ELSE c_pk.OWNER || '.' || c_pk.TABLE_NAME \
            END AS TARGET_TABLE, \
            b.COLUMN_NAME AS TARGET_COLUMN \
        FROM ALL_CONS_COLUMNS a \
        JOIN ALL_CONSTRAINTS c ON a.CONSTRAINT_NAME = c.CONSTRAINT_NAME AND a.OWNER = c.OWNER \
        JOIN ALL_CONSTRAINTS c_pk ON c.R_CONSTRAINT_NAME = c_pk.CONSTRAINT_NAME AND c.R_OWNER = c_pk.OWNER \
        JOIN ALL_CONS_COLUMNS b ON c_pk.CONSTRAINT_NAME = b.CONSTRAINT_NAME AND c_pk.OWNER = b.OWNER AND a.POSITION = b.POSITION \
        WHERE c.CONSTRAINT_TYPE = 'R' \
            AND c.OWNER = :1 \
            AND c.TABLE_NAME = :2";

        let (fk_rows, _, _) = Self::execute_query(pool, fk_sql, &params)
            .await.unwrap_or((vec![], vec![], 0));

        let mut fk_map = HashMap::new();
        for r in fk_rows {
            if let (Some(col), Some(ref_tbl), Some(ref_col)) = (
                r.get("SOURCE_COLUMN").and_then(|v| v.as_str()),
                r.get("TARGET_TABLE").and_then(|v| v.as_str()),
                r.get("TARGET_COLUMN").and_then(|v| v.as_str()),
            ) {
                fk_map.insert(col.to_string(), ForeignKeyDef {
                    target_table: ref_tbl.to_string(),
                    target_column: ref_col.to_string(),
                });
            }
        }

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

        let (rows, _, _) = Self::execute_query(pool, sql, &params).await?;

        Ok(rows
            .iter()
            .map(|r| {
                let col_name = r.get("COLUMN_NAME").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let foreign_key = fk_map.get(&col_name).cloned();
                TableColumn {
                    name: col_name,
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
                    foreign_key,
                }
            })
            .collect())
    }

    async fn create_table(&self, table_name: &str, columns: &[TableColumn]) -> Result<(), String> {
        let pool = self.pool()?;
        let (owner, table) = self.split_table_name(table_name);
        let safe_table = Self::qualify_table(owner, table);

        let sql = crate::drivers::utils::build_create_table_sql_generic(&safe_table, columns, Self::quote)?;
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

            Self::execute_dml(pool, &sql, &[]).await?;
        }
        Ok(())
    }

    async fn drop_table(&self, table_name: &str, cascade: bool) -> Result<(), String> {
        let pool = self.pool()?;
        let (owner, table) = self.split_table_name(table_name);
        let sql = if cascade {
            format!("DROP TABLE {} CASCADE CONSTRAINTS", Self::qualify_table(owner, table))
        } else {
            format!("DROP TABLE {}", Self::qualify_table(owner, table))
        };
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

    async fn drop_database(&self, db_name: &str) -> Result<(), String> {
        let pool = self.pool()?;
        let sql = format!("DROP USER {} CASCADE", Self::quote(db_name));
        Self::execute_dml(pool, &sql, &[]).await?;
        Ok(())
    }

    async fn create_database(&self, db_name: &str, password: Option<&str>) -> Result<(), String> {
        let pool = self.pool()?;
        let pass = password.unwrap_or(db_name);
        let sql_create = format!("CREATE USER {} IDENTIFIED BY {}", Self::quote(db_name), Self::quote(pass));
        Self::execute_dml(pool, &sql_create, &[]).await?;

        let sql_grant = format!("GRANT CONNECT, RESOURCE TO {}", Self::quote(db_name));
        Self::execute_dml(pool, &sql_grant, &[]).await?;
        Ok(())
    }

    async fn export_to_csv(&self, table_name: &str, export_path: &str) -> Result<(), String> {
        let pool = self.pool()?;
        let (owner, table) = self.split_table_name(table_name);
        let sql = format!("SELECT * FROM {}", Self::qualify_table(owner, table));
        let (rows, _, _) = Self::execute_query(pool, &sql, &[]).await?;

        crate::drivers::utils::export_rows_to_csv(&rows, export_path)
    }

    async fn get_schema_details(&self, schema_name: &str) -> Result<SchemaDetails, String> {
        let pool = self.pool()?;
        let owner = if schema_name.trim().is_empty() {
            self.current_schema.clone()
        } else {
            schema_name.to_uppercase()
        };

        let column_sql = "SELECT \
                c.TABLE_NAME as table_name, \
                c.COLUMN_NAME as column_name, \
                c.DATA_TYPE as data_type, \
                CASE WHEN c.NULLABLE = 'Y' THEN 'YES' ELSE 'NO' END as is_nullable, \
                c.DATA_DEFAULT as data_default, \
                CASE WHEN pk.COLUMN_NAME IS NULL THEN 'NO' ELSE 'YES' END AS IS_PRIMARY_KEY \
            FROM ALL_TAB_COLUMNS c \
            LEFT JOIN ( \
                SELECT cols.OWNER, cols.TABLE_NAME, cols.COLUMN_NAME \
                FROM ALL_CONSTRAINTS cons \
                JOIN ALL_CONS_COLUMNS cols ON cons.OWNER = cols.OWNER \
                    AND cons.CONSTRAINT_NAME = cols.CONSTRAINT_NAME \
                WHERE cons.CONSTRAINT_TYPE = 'P' \
            ) pk ON pk.OWNER = c.OWNER AND pk.TABLE_NAME = c.TABLE_NAME AND pk.COLUMN_NAME = c.COLUMN_NAME \
            WHERE c.OWNER = :1 \
            ORDER BY c.TABLE_NAME, c.COLUMN_ID";

        let relation_sql = "SELECT \
                a.CONSTRAINT_NAME as constraint_name, \
                a.TABLE_NAME as source_table, \
                a_cols.COLUMN_NAME as source_column, \
                r.TABLE_NAME as target_table, \
                r_cols.COLUMN_NAME as target_column \
            FROM ALL_CONSTRAINTS a \
            JOIN ALL_CONS_COLUMNS a_cols ON a.OWNER = a_cols.OWNER AND a.CONSTRAINT_NAME = a_cols.CONSTRAINT_NAME \
            JOIN ALL_CONSTRAINTS r ON a.R_OWNER = r.OWNER AND a.R_CONSTRAINT_NAME = r.CONSTRAINT_NAME \
            JOIN ALL_CONS_COLUMNS r_cols ON r.OWNER = r_cols.OWNER AND r.CONSTRAINT_NAME = r_cols.CONSTRAINT_NAME AND a_cols.POSITION = r_cols.POSITION \
            WHERE a.CONSTRAINT_TYPE = 'R' \
              AND a.OWNER = :1";

        let params = [JsonValue::String(owner)];

        // Run queries sequentially for Oracle to avoid connection issues
        let (column_rows, _, _) = Self::execute_query(pool, column_sql, &params).await?;
        let (relation_rows, _, _) = Self::execute_query(pool, relation_sql, &params).await?;

        let mut tables_map: HashMap<String, Vec<TableColumn>> = HashMap::new();
        let mut table_order: Vec<String> = Vec::new();

        let mut fk_map: HashMap<(String, String), ForeignKeyDef> = HashMap::new();
        for r in &relation_rows {
            if let (Some(src_tbl), Some(src_col), Some(tgt_tbl), Some(tgt_col)) = (
                r.get("SOURCE_TABLE").and_then(|v| v.as_str()),
                r.get("SOURCE_COLUMN").and_then(|v| v.as_str()),
                r.get("TARGET_TABLE").and_then(|v| v.as_str()),
                r.get("TARGET_COLUMN").and_then(|v| v.as_str()),
            ) {
                fk_map.insert((src_tbl.to_string(), src_col.to_string()), ForeignKeyDef {
                    target_table: tgt_tbl.to_string(),
                    target_column: tgt_col.to_string(),
                });
            }
        }

        for r in column_rows {
            let table_name = r.get("TABLE_NAME").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let col_name = r.get("COLUMN_NAME").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let foreign_key = fk_map.get(&(table_name.clone(), col_name.clone())).cloned();
            let col = TableColumn {
                name: col_name,
                data_type: r.get("DATA_TYPE").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                nullable: r.get("IS_NULLABLE").and_then(|v| v.as_str()).map(|s| s == "YES").unwrap_or(true),
                is_primary_key: r.get("IS_PRIMARY_KEY").and_then(|v| v.as_str()).map(|s| s == "YES").unwrap_or(false),
                default: r.get("DATA_DEFAULT").and_then(|v| v.as_str()).map(|s| s.to_string()),
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
                constraint_name: r.get("CONSTRAINT_NAME").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                source_table: r.get("SOURCE_TABLE").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                source_column: r.get("SOURCE_COLUMN").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                target_table: r.get("TARGET_TABLE").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                target_column: r.get("TARGET_COLUMN").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            });
        }

        Ok(SchemaDetails { tables, relations })
    }

    async fn get_table_indexes(&self, table_name: &str) -> Result<Vec<DbIndex>, String> {
        let pool = self.pool()?;
        let (owner, table) = self.split_table_name(table_name);
        
        let sql = r#"
            SELECT 
                i.INDEX_NAME as index_name,
                i.UNIQUENESS as uniqueness,
                i.INDEX_TYPE as index_type,
                (SELECT COUNT(*) FROM ALL_CONSTRAINTS c WHERE c.OWNER = i.TABLE_OWNER AND c.TABLE_NAME = i.TABLE_NAME AND c.INDEX_NAME = i.INDEX_NAME AND c.CONSTRAINT_TYPE = 'P') as is_primary,
                (
                    SELECT JSON_ARRAYAGG(ic.COLUMN_NAME ORDER BY ic.COLUMN_POSITION)
                    FROM ALL_IND_COLUMNS ic
                    WHERE ic.INDEX_OWNER = i.OWNER AND ic.INDEX_NAME = i.INDEX_NAME
                ) as columns
            FROM ALL_INDEXES i
            WHERE i.TABLE_OWNER = :1 AND i.TABLE_NAME = :2
            ORDER BY i.INDEX_NAME
        "#;
        
        let (rows, _, _) = Self::execute_query(
            pool,
            sql,
            &[
                JsonValue::String(owner.to_string()),
                JsonValue::String(table.to_string()),
            ],
        ).await?;

        let mut indexes = Vec::new();
        for r in rows {
            let mut cols = vec![];
            if let Some(val) = r.get("COLUMNS") {
                if let JsonValue::Array(arr) = val {
                    cols = arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                } else if let JsonValue::String(s) = val {
                    if let Ok(JsonValue::Array(arr)) = serde_json::from_str(s) {
                        cols = arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect();
                    }
                }
            }

            let name = r.get("INDEX_NAME").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let uniqueness = r.get("UNIQUENESS").and_then(|v| v.as_str()).unwrap_or("");
            let is_unique = uniqueness == "UNIQUE";
            let is_primary_key = r.get("IS_PRIMARY").and_then(|v| {
                if let JsonValue::Number(n) = v {
                    Some(n.as_i64().unwrap_or(0) > 0)
                } else if let JsonValue::String(s) = v {
                    s.parse::<i64>().ok().map(|n| n > 0)
                } else {
                    None
                }
            }).unwrap_or(false);

            let ddl = if is_primary_key {
                format!("PRIMARY KEY ({})", cols.join(", "))
            } else if is_unique {
                format!("CREATE UNIQUE INDEX {} ON {} ({})", Self::quote(&name), Self::qualify_table(owner, table), cols.join(", "))
            } else {
                format!("CREATE INDEX {} ON {} ({})", Self::quote(&name), Self::qualify_table(owner, table), cols.join(", "))
            };

            indexes.push(DbIndex {
                name,
                is_unique,
                is_primary_key,
                index_type: r.get("INDEX_TYPE").and_then(|v| v.as_str()).map(|s| s.to_string()),
                ddl: Some(ddl),
                columns: cols,
            });
        }
        Ok(indexes)
    }

    async fn get_table_constraints(&self, table_name: &str) -> Result<Vec<TableConstraint>, String> {
        let pool = self.pool()?;
        let (owner, table) = self.split_table_name(table_name);
        
        let params = [
            JsonValue::String(owner.to_string()),
            JsonValue::String(table.to_string()),
        ];

        let sql = "SELECT \
            c.CONSTRAINT_NAME AS constraint_name, \
            c.CONSTRAINT_TYPE AS constraint_type, \
            cc.COLUMN_NAME AS column_name, \
            c_pk.TABLE_NAME AS r_table, \
            cc_pk.COLUMN_NAME AS r_column, \
            c.SEARCH_CONDITION AS search_condition \
        FROM ALL_CONSTRAINTS c \
        LEFT JOIN ALL_CONS_COLUMNS cc \
          ON c.CONSTRAINT_NAME = cc.CONSTRAINT_NAME \
          AND c.OWNER = cc.OWNER \
        LEFT JOIN ALL_CONSTRAINTS c_pk \
          ON c.R_CONSTRAINT_NAME = c_pk.CONSTRAINT_NAME \
          AND c.R_OWNER = c_pk.OWNER \
        LEFT JOIN ALL_CONS_COLUMNS cc_pk \
          ON c_pk.CONSTRAINT_NAME = cc_pk.CONSTRAINT_NAME \
          AND c_pk.OWNER = cc_pk.OWNER \
          AND cc.POSITION = cc_pk.POSITION \
        WHERE c.OWNER = :1 AND c.TABLE_NAME = :2 \
        ORDER BY c.CONSTRAINT_NAME, cc.POSITION";

        let (rows, _, _) = Self::execute_query(pool, sql, &params).await?;

        // Group rows by constraint name
        let mut constraint_map: HashMap<String, (String, Vec<String>, Vec<String>, String, String)> = HashMap::new();
        let mut constraint_order = Vec::new();

        for r in rows {
            let name = r.get("CONSTRAINT_NAME").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let c_type = r.get("CONSTRAINT_TYPE").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let col = r.get("COLUMN_NAME").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let ref_tbl = r.get("R_TABLE").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let ref_col = r.get("R_COLUMN").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let search_cond = r.get("SEARCH_CONDITION").and_then(|v| v.as_str()).unwrap_or("").to_string();

            if !constraint_map.contains_key(&name) {
                constraint_order.push(name.clone());
            }

            let entry = constraint_map.entry(name).or_insert_with(|| (c_type, Vec::new(), Vec::new(), ref_tbl, search_cond));
            if !col.is_empty() {
                entry.1.push(col);
            }
            if !ref_col.is_empty() {
                entry.2.push(ref_col);
            }
        }

        let mut constraints = Vec::new();
        for name in constraint_order {
            if let Some((c_type, cols, ref_cols, ref_tbl, search_cond)) = constraint_map.remove(&name) {
                let mapped_type = match c_type.as_str() {
                    "P" => "PRIMARY KEY".to_string(),
                    "R" => "FOREIGN KEY".to_string(),
                    "U" => "UNIQUE".to_string(),
                    "C" => "CHECK".to_string(),
                    _ => c_type,
                };

                let definition = match mapped_type.as_str() {
                    "PRIMARY KEY" => format!("PRIMARY KEY ({})", cols.join(", ")),
                    "UNIQUE" => format!("UNIQUE ({})", cols.join(", ")),
                    "FOREIGN KEY" => format!(
                        "FOREIGN KEY ({}) REFERENCES {} ({})",
                        cols.join(", "),
                        ref_tbl,
                        ref_cols.join(", ")
                    ),
                    "CHECK" => search_cond,
                    _ => format!("{} ({})", mapped_type, cols.join(", ")),
                };

                constraints.push(TableConstraint {
                    name,
                    constraint_type: mapped_type,
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
        
        let conn = pool.get().await.map_err(|e| e.to_string())?;
        *tx = Some(conn);
        Ok(())
    }

    async fn commit_transaction(&self) -> Result<(), String> {
        let mut tx = self.tx_conn.lock().await;
        if let Some(conn) = tx.take() {
            conn.commit().await.map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("No active transaction to commit".to_string())
        }
    }

    async fn rollback_transaction(&self) -> Result<(), String> {
        let mut tx = self.tx_conn.lock().await;
        if let Some(conn) = tx.take() {
            conn.rollback().await.map_err(|e| e.to_string())?;
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

    #[test]
    fn converts_json_to_oracle_value_string() {
        use oracle_rs::Value as OracleValue;
        use serde_json::json;

        let obj = json!({"key": "value"});
        let oracle_val = OracleDriver::json_to_oracle_value(&obj);
        if let OracleValue::String(s) = oracle_val {
            assert_eq!(s, "{\"key\":\"value\"}");
        } else {
            panic!("Expected OracleValue::String");
        }

        let arr = json!([1, 2, 3]);
        let oracle_val_arr = OracleDriver::json_to_oracle_value(&arr);
        if let OracleValue::String(s) = oracle_val_arr {
            assert_eq!(s, "[1,2,3]");
        } else {
            panic!("Expected OracleValue::String");
        }
    }
}
