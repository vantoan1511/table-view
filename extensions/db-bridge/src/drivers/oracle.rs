use super::{
    AlterOperation, ColumnInfo, Config, DatabaseDriver, DbIndex, ForeignKeyDef, QueryResult,
    SchemaDetails, SchemaObject, SchemaResult, TableColumn, TableConstraint, TableDataResult,
    TableInfo, TableRelation,
};
use async_trait::async_trait;
use oracledb::ToDbValue;
use serde_json::Value as JsonValue;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub(crate) enum OracleBind {
    Null,
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

type RowMap = HashMap<String, JsonValue>;
type CursorRawResult = (Vec<RowMap>, Vec<ColumnInfo>);
type QueryRawResult = (Vec<RowMap>, Vec<ColumnInfo>, u64);

pub struct OracleDriver {
    pool: Option<Arc<oracledb::Pool>>,
    current_schema: String,
    service_name: String,
    tx_conn: tokio::sync::Mutex<Option<oracledb::Connection>>,
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

    fn pool(&self) -> Result<&Arc<oracledb::Pool>, String> {
        self.pool.as_ref().ok_or_else(|| "Not connected".to_string())
    }

    pub fn quote(ident: &str) -> String {
        format!("\"{}\"", ident.replace('"', "\"\""))
    }

    fn split_table_name<'a>(&'a self, table_name: &'a str) -> (&'a str, &'a str) {
        let (owner, name) = table_name
            .split_once('.')
            .unwrap_or((self.current_schema.as_str(), table_name));
        if owner.eq_ignore_ascii_case("default") || owner.is_empty() {
            (self.current_schema.as_str(), name)
        } else {
            (owner, name)
        }
    }

    pub fn qualify_table(owner: &str, table_name: &str) -> String {
        if owner.is_empty() || owner.eq_ignore_ascii_case("default") {
            Self::quote(table_name)
        } else {
            format!("{}.{}", Self::quote(owner), Self::quote(table_name))
        }
    }

    pub fn pagination_clause(limit: i64, offset: i64) -> String {
        format!(
            " OFFSET {} ROWS FETCH NEXT {} ROWS ONLY",
            offset.max(0),
            limit.max(0)
        )
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

    pub(crate) fn json_to_bind(val: &JsonValue) -> OracleBind {
        match val {
            JsonValue::Null => OracleBind::Null,
            JsonValue::Bool(b) => OracleBind::Bool(*b),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    OracleBind::Int(i)
                } else if let Some(f) = n.as_f64() {
                    OracleBind::Float(f)
                } else {
                    OracleBind::String(n.to_string())
                }
            }
            JsonValue::String(s) => OracleBind::String(s.clone()),
            _ => OracleBind::String(val.to_string()),
        }
    }

    fn oracle_json_to_serde(val: &oracledb::JsonValue) -> JsonValue {
        match val {
            oracledb::JsonValue::Null => JsonValue::Null,
            oracledb::JsonValue::Boolean(b) => JsonValue::Bool(*b),
            oracledb::JsonValue::Number(n) => {
                let s = n.to_string();
                if let Ok(i) = s.parse::<i64>() {
                    JsonValue::Number(i.into())
                } else if let Ok(f) = s.parse::<f64>() {
                    serde_json::Number::from_f64(f)
                        .map(JsonValue::Number)
                        .unwrap_or(JsonValue::Null)
                } else {
                    JsonValue::String(s)
                }
            }
            oracledb::JsonValue::BinaryDouble(f) => serde_json::Number::from_f64(*f)
                .map(JsonValue::Number)
                .unwrap_or(JsonValue::Null),
            oracledb::JsonValue::BinaryFloat(f) => serde_json::Number::from_f64(*f as f64)
                .map(JsonValue::Number)
                .unwrap_or(JsonValue::Null),
            oracledb::JsonValue::String(s) => JsonValue::String(s.clone()),
            oracledb::JsonValue::Raw(b) => JsonValue::String(hex::encode(b)),
            oracledb::JsonValue::Timestamp(ts) => JsonValue::String(ts.to_string()),
            oracledb::JsonValue::IntervalDS(ds) => JsonValue::String(ds.to_string()),
            oracledb::JsonValue::IntervalYM(ym) => JsonValue::String(ym.to_string()),
            oracledb::JsonValue::JsonArray(arr) => {
                JsonValue::Array(arr.iter().map(Self::oracle_json_to_serde).collect())
            }
            oracledb::JsonValue::JsonObject(map) => {
                let mut obj = serde_json::Map::new();
                for (k, v) in map {
                    obj.insert(k.clone(), Self::oracle_json_to_serde(v));
                }
                JsonValue::Object(obj)
            }
            _ => JsonValue::Null,
        }
    }

    fn extract_column_value(
        row: &mut oracledb::Row,
        i: usize,
        meta: &oracledb::Metadata,
    ) -> JsonValue {
        let db_type = meta.db_type();

        if db_type.is_string_type() {
            if db_type == &oracledb::DB_TYPE_CLOB || db_type == &oracledb::DB_TYPE_NCLOB {
                if let Ok(Some(mut lob)) = row.take::<Option<oracledb::Lob>>(i) {
                    let mut buf = Vec::new();
                    let _ = std::io::Read::read_to_end(&mut lob, &mut buf);
                    return JsonValue::String(String::from_utf8_lossy(&buf).into_owned());
                }
                return JsonValue::Null;
            }
            if let Ok(Some(s)) = row.get::<Option<String>>(i) {
                return JsonValue::String(s);
            }
            return JsonValue::Null;
        }

        if db_type == &oracledb::DB_TYPE_NUMBER || db_type == &oracledb::DB_TYPE_BINARY_INTEGER {
            if let Ok(Some(num)) = row.get::<Option<oracledb::OracleNumber>>(i) {
                let s = num.to_string();
                if let Ok(i_val) = s.parse::<i64>() {
                    return JsonValue::Number(i_val.into());
                } else if let Ok(f_val) = s.parse::<f64>() {
                    return serde_json::Number::from_f64(f_val)
                        .map(JsonValue::Number)
                        .unwrap_or(JsonValue::Null);
                } else {
                    return JsonValue::String(s);
                }
            }
            return JsonValue::Null;
        }

        if db_type == &oracledb::DB_TYPE_BINARY_DOUBLE {
            if let Ok(Some(f)) = row.get::<Option<f64>>(i) {
                return serde_json::Number::from_f64(f)
                    .map(JsonValue::Number)
                    .unwrap_or(JsonValue::Null);
            }
            return JsonValue::Null;
        }

        if db_type == &oracledb::DB_TYPE_BINARY_FLOAT {
            if let Ok(Some(f)) = row.get::<Option<f32>>(i) {
                return serde_json::Number::from_f64(f as f64)
                    .map(JsonValue::Number)
                    .unwrap_or(JsonValue::Null);
            }
            return JsonValue::Null;
        }

        if db_type == &oracledb::DB_TYPE_BOOLEAN {
            if let Ok(Some(b)) = row.get::<Option<bool>>(i) {
                return JsonValue::Bool(b);
            }
            return JsonValue::Null;
        }

        if db_type.is_date_type() {
            if let Ok(Some(ts)) = row.get::<Option<oracledb::OracleTimestamp>>(i) {
                return JsonValue::String(ts.to_string());
            }
            return JsonValue::Null;
        }

        if db_type.is_binary_type() {
            if db_type == &oracledb::DB_TYPE_BLOB || db_type == &oracledb::DB_TYPE_BFILE {
                if let Ok(Some(mut lob)) = row.take::<Option<oracledb::Lob>>(i) {
                    let mut buf = Vec::new();
                    let _ = std::io::Read::read_to_end(&mut lob, &mut buf);
                    return JsonValue::String(hex::encode(buf));
                }
                return JsonValue::Null;
            }
            if let Ok(Some(bytes)) = row.get::<Option<Vec<u8>>>(i) {
                return JsonValue::String(hex::encode(bytes));
            }
            return JsonValue::Null;
        }

        if db_type == &oracledb::DB_TYPE_JSON {
            if let Ok(Some(jv)) = row.get::<Option<oracledb::JsonValue>>(i) {
                return Self::oracle_json_to_serde(&jv);
            }
            return JsonValue::Null;
        }

        if db_type == &oracledb::DB_TYPE_INTERVAL_DS {
            if let Ok(Some(ds)) = row.get::<Option<oracledb::OracleIntervalDS>>(i) {
                return JsonValue::String(ds.to_string());
            }
            return JsonValue::Null;
        }

        if db_type == &oracledb::DB_TYPE_INTERVAL_YM {
            if let Ok(Some(ym)) = row.get::<Option<oracledb::OracleIntervalYM>>(i) {
                return JsonValue::String(ym.to_string());
            }
            return JsonValue::Null;
        }

        if let Ok(Some(s)) = row.get::<Option<String>>(i) {
            JsonValue::String(s)
        } else {
            JsonValue::Null
        }
    }

    fn to_db_value_refs<'a>(
        binds: &'a [OracleBind],
        null_str: &'a Option<String>,
    ) -> Vec<&'a dyn ToDbValue> {
        binds
            .iter()
            .map(|b| match b {
                OracleBind::Null => null_str as &dyn ToDbValue,
                OracleBind::String(s) => s as &dyn ToDbValue,
                OracleBind::Int(i) => i as &dyn ToDbValue,
                OracleBind::Float(f) => f as &dyn ToDbValue,
                OracleBind::Bool(b) => b as &dyn ToDbValue,
            })
            .collect()
    }

    fn cursor_to_maps(cursor: oracledb::Cursor) -> Result<CursorRawResult, String> {
        let col_metas = cursor.columns().clone();
        let orig_names: Vec<String> = col_metas.iter().map(|col| col.name().to_string()).collect();
        let unique_names = crate::drivers::utils::make_unique_column_names(&orig_names);

        let fields: Vec<ColumnInfo> = col_metas
            .iter()
            .zip(unique_names.iter())
            .map(|(col, unique_name)| {
                let display_name = if unique_name != col.name() {
                    Some(col.name().to_string())
                } else {
                    None
                };
                ColumnInfo {
                    name: unique_name.clone(),
                    data_type: col.db_type().name().to_string(),
                    is_primary_key: false,
                    is_nullable: col.nullable(),
                    display_name,
                }
            })
            .collect();

        let mut rows = Vec::new();
        for row_res in cursor {
            let mut row = row_res.map_err(|e| e.to_string())?;
            let mut map = HashMap::new();
            for (i, (unique_name, meta)) in unique_names.iter().zip(col_metas.iter()).enumerate() {
                let json_val = Self::extract_column_value(&mut row, i, meta);
                map.insert(unique_name.clone(), json_val);
            }
            rows.push(map);
        }

        Ok((rows, fields))
    }

    fn execute_query_on_conn(
        conn: &oracledb::Connection,
        sql: &str,
        params: &[JsonValue],
    ) -> Result<QueryRawResult, String> {
        let binds: Vec<OracleBind> = params.iter().map(Self::json_to_bind).collect();
        let null_str: Option<String> = None;
        let bind_refs = Self::to_db_value_refs(&binds, &null_str);
        let cleaned_sql = Self::clean_sql(sql);

        log::info!("oracle: executing query: {}", cleaned_sql);
        let start = std::time::Instant::now();
        let first_word = cleaned_sql
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        let is_query = matches!(first_word.as_str(), "select" | "with");

        let (rows, fields) = if is_query {
            let cursor = conn.query(&cleaned_sql, &bind_refs).map_err(|e| e.to_string())?;
            Self::cursor_to_maps(cursor)?
        } else {
            conn.execute(&cleaned_sql, &bind_refs).map_err(|e| e.to_string())?;
            (Vec::new(), Vec::new())
        };
        let elapsed = start.elapsed().as_millis() as u64;
        log::info!("oracle: query finished in {}ms", elapsed);

        Ok((rows, fields, elapsed))
    }

    fn execute_query(
        pool: &oracledb::Pool,
        sql: &str,
        params: &[JsonValue],
    ) -> Result<QueryRawResult, String> {
        let conn = pool.acquire().map_err(|e| e.to_string())?;
        Self::execute_query_on_conn(&conn, sql, params)
    }

    fn execute_dml_on_conn(
        conn: &oracledb::Connection,
        sql: &str,
        params: &[JsonValue],
        auto_commit: bool,
    ) -> Result<(), String> {
        let binds: Vec<OracleBind> = params.iter().map(Self::json_to_bind).collect();
        let null_str: Option<String> = None;
        let bind_refs = Self::to_db_value_refs(&binds, &null_str);
        let cleaned_sql = Self::clean_sql(sql);
        conn.execute(&cleaned_sql, &bind_refs).map_err(|e| e.to_string())?;

        let first_word = cleaned_sql
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_ascii_uppercase();
        let is_ddl = matches!(
            first_word.as_str(),
            "CREATE" | "ALTER" | "DROP" | "TRUNCATE" | "GRANT" | "REVOKE"
        );
        if auto_commit && !is_ddl {
            conn.commit().map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    async fn run_query(
        &self,
        sql: &str,
        params: &[JsonValue],
    ) -> Result<QueryRawResult, String> {
        let mut tx_guard = self.tx_conn.lock().await;
        if let Some(conn) = tx_guard.take() {
            let sql_owned = sql.to_string();
            let params_owned = params.to_vec();
            let (res, conn) = tokio::task::spawn_blocking(move || {
                let res = Self::execute_query_on_conn(&conn, &sql_owned, &params_owned);
                (res, conn)
            })
            .await
            .map_err(|e| e.to_string())?;
            *tx_guard = Some(conn);
            res
        } else {
            let pool = self.pool()?.clone();
            let sql_owned = sql.to_string();
            let params_owned = params.to_vec();
            tokio::task::spawn_blocking(move || {
                Self::execute_query(&pool, &sql_owned, &params_owned)
            })
            .await
            .map_err(|e| e.to_string())?
        }
    }

    async fn run_dml(&self, sql: &str, params: &[JsonValue]) -> Result<(), String> {
        let mut tx_guard = self.tx_conn.lock().await;
        if let Some(conn) = tx_guard.take() {
            let sql_owned = sql.to_string();
            let params_owned = params.to_vec();
            let (res, conn) = tokio::task::spawn_blocking(move || {
                let res = Self::execute_dml_on_conn(&conn, &sql_owned, &params_owned, false);
                (res, conn)
            })
            .await
            .map_err(|e| e.to_string())?;
            *tx_guard = Some(conn);
            res
        } else {
            let pool = self.pool()?.clone();
            let sql_owned = sql.to_string();
            let params_owned = params.to_vec();
            tokio::task::spawn_blocking(move || {
                let conn = pool.acquire().map_err(|e| e.to_string())?;
                Self::execute_dml_on_conn(&conn, &sql_owned, &params_owned, true)
            })
            .await
            .map_err(|e| e.to_string())?
        }
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
            "sysdba" => oracledb::AUTH_MODE_SYSDBA,
            "sysoper" => oracledb::AUTH_MODE_SYSOPER,
            "normal" if use_implicit_sysdba => oracledb::AUTH_MODE_SYSDBA,
            _ => oracledb::AUTH_MODE_DEFAULT,
        };

        let protocol = if config.ssl { "TCPS" } else { "TCP" };
        let is_sid = connect_type.eq_ignore_ascii_case("sid");
        let connect_data = if is_sid {
            format!("(CONNECT_DATA=(SID={}))", config.database)
        } else {
            format!("(CONNECT_DATA=(SERVICE_NAME={}))", config.database)
        };
        let connect_string = format!(
            "(DESCRIPTION=(ADDRESS=(PROTOCOL={})(HOST={})(PORT={})){})",
            protocol, config.host, config.port, connect_data
        );

        log::info!(
            "oracle: connecting to {}:{}/{} as {} with role {} using {}",
            config.host,
            config.port,
            config.database,
            config.username,
            if use_implicit_sysdba {
                "sysdba (implicit for SYS)"
            } else {
                config.oracle_role.as_str()
            },
            if is_sid { "SID" } else { "service name" }
        );

        let user = config.username.clone();
        let password = config.password.clone();
        let pool = tokio::task::spawn_blocking(move || {
            let pool_config = oracledb::PoolConfig::default()
                .set_credentials(&user, &password)
                .set_connect_string(&connect_string)?
                .set_auth_mode(auth_mode)
                .set_min_connections(1)
                .set_max_connections(5)
                .set_connection_increment(1);

            let pool = oracledb::create_pool(pool_config)?;
            let conn = pool.acquire()?;
            drop(conn);
            Ok::<oracledb::Pool, oracledb::Error>(pool)
        })
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| {
            let message = e.to_string();
            if message.contains("Server sent MARKER - authentication rejected")
                || message.contains("ORA-01017")
                || message.contains("ORA-28009")
            {
                format!(
                    "Oracle authentication rejected. If you connect as SYS, set Oracle Role to SYSDBA and make sure the service/SID matches DBeaver. Details: {}",
                    message
                )
            } else {
                message
            }
        })?;

        self.current_schema = config.username.to_uppercase();
        self.service_name = config.database.clone();
        self.pool = Some(Arc::new(pool));
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), String> {
        {
            let mut tx = self.tx_conn.lock().await;
            if let Some(mut conn) = tx.take() {
                let _ = tokio::task::spawn_blocking(move || {
                    if let Err(e) = conn.close() {
                        log::warn!("oracle: error closing transaction connection: {}", e);
                    }
                })
                .await;
            }
        }
        if let Some(pool) = self.pool.take() {
            let _ = tokio::task::spawn_blocking(move || {
                drop(pool);
            })
            .await;
        }
        Ok(())
    }

    async fn get_schema(
        &self,
        all_databases: bool,
        schema_name: Option<&str>,
    ) -> Result<SchemaResult, String> {
        let is_sys = self.current_schema == "SYS";
        let requested_schema = schema_name
            .map(str::trim)
            .filter(|name| !name.is_empty() && !name.eq_ignore_ascii_case("default"))
            .map(|name| name.to_uppercase());

        // Always fetch all users (schemas)
        let schema_sql = if is_sys {
            "SELECT USERNAME FROM DBA_USERS ORDER BY USERNAME"
        } else {
            "SELECT USERNAME FROM ALL_USERS WHERE USERNAME NOT IN ('SYS', 'SYSTEM', 'OUTLN', 'XDB', 'CTXSYS', 'MDSYS') ORDER BY USERNAME"
        };
        let (schema_rows, _, _) = self.run_query(schema_sql, &[]).await?;

        // Determine object filtering
        let (where_clause, params) = if all_databases {
            if is_sys {
                ("".to_string(), vec![])
            } else {
                (
                    "WHERE OWNER NOT IN ('SYS', 'SYSTEM', 'OUTLN', 'XDB', 'CTXSYS', 'MDSYS')"
                        .to_string(),
                    vec![],
                )
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

        let table_sql = format!(
            "SELECT TABLE_NAME, OWNER FROM {} {} ORDER BY OWNER, TABLE_NAME",
            table_view, where_clause
        );
        let view_sql = format!(
            "SELECT VIEW_NAME, OWNER FROM {} {} ORDER BY OWNER, VIEW_NAME",
            view_view, where_clause
        );
        let func_sql = format!(
            "SELECT OBJECT_NAME, OWNER, OBJECT_TYPE FROM {} {} {} OBJECT_TYPE IN ('PROCEDURE', 'FUNCTION') ORDER BY OWNER, OBJECT_NAME",
            obj_view,
            where_clause,
            if where_clause.is_empty() { "WHERE" } else { "AND" }
        );

        // Execute queries sequentially for Oracle to avoid connection flooding on admin accounts
        let (table_rows, _, _) = self.run_query(&table_sql, &params).await?;
        let (view_rows, _, _) = self.run_query(&view_sql, &params).await?;
        let (func_rows, _, _) = self.run_query(&func_sql, &params).await?;

        let tables = table_rows
            .iter()
            .map(|r| SchemaObject {
                name: r
                    .get("TABLE_NAME")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                schema: Some(
                    r.get("OWNER")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                ),
                obj_type: None,
            })
            .collect();

        let views = view_rows
            .iter()
            .map(|r| SchemaObject {
                name: r
                    .get("VIEW_NAME")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                schema: Some(
                    r.get("OWNER")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                ),
                obj_type: None,
            })
            .collect();

        let functions = func_rows
            .iter()
            .map(|r| SchemaObject {
                name: r
                    .get("OBJECT_NAME")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                schema: Some(
                    r.get("OWNER")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                ),
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
                name: r
                    .get("USERNAME")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                schema: None,
                obj_type: None,
            })
            .collect();

        if !self.current_schema.is_empty()
            && !schemas
                .iter()
                .any(|s| s.name.eq_ignore_ascii_case(&self.current_schema))
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
        let (owner, table) = self.split_table_name(table_name);
        let qualified_table = Self::qualify_table(owner, table);

        let pk_sql = "SELECT cols.COLUMN_NAME FROM ALL_CONSTRAINTS cons \
            JOIN ALL_CONS_COLUMNS cols ON cons.OWNER = cols.OWNER \
            AND cons.CONSTRAINT_NAME = cols.CONSTRAINT_NAME \
            WHERE cons.CONSTRAINT_TYPE = 'P' \
            AND cons.OWNER = :1 AND cons.TABLE_NAME = :2";
        let pk_params = [
            JsonValue::String(owner.to_string()),
            JsonValue::String(table.to_string()),
        ];

        let mut where_clause = String::new();
        if !filter.trim().is_empty() {
            where_clause = format!(" WHERE {}", filter);
        }

        let mut order_clause = String::new();
        if !sort_column.trim().is_empty() {
            let direction = if sort_direction.eq_ignore_ascii_case("desc") {
                "DESC"
            } else {
                "ASC"
            };
            order_clause = format!(" ORDER BY {} {}", Self::quote(sort_column), direction);
        }

        let data_sql = format!(
            "SELECT * FROM {}{}{}{}",
            qualified_table,
            where_clause,
            order_clause,
            Self::pagination_clause(limit, offset)
        );
        let count_sql = format!(
            "SELECT COUNT(*) AS CNT FROM {}{}",
            qualified_table, where_clause
        );

        let (pk_res, data_res, count_res) = tokio::join!(
            self.run_query(pk_sql, &pk_params),
            self.run_query(&data_sql, &[]),
            self.run_query(&count_sql, &[]),
        );

        let (pk_rows, _, _) = pk_res?;
        let (data, mut fields, elapsed) = data_res?;
        let (count_rows, _, _) = count_res?;

        let pk_set: HashSet<String> = pk_rows
            .iter()
            .filter_map(|r| {
                r.get("COLUMN_NAME")
                    .and_then(|v| v.as_str())
                    .map(str::to_string)
            })
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
        let (rows, fields, elapsed) = self.run_query(sql, &[]).await?;
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
        let (owner, table) = self.split_table_name(table_name);
        let sql = format!(
            "UPDATE {} SET {} = :1 WHERE {} = :2",
            Self::qualify_table(owner, table),
            Self::quote(target_column),
            Self::quote(pk_column),
        );
        self.run_dml(&sql, &[new_value.clone(), pk_value.clone()])
            .await
    }

    async fn insert_row(
        &self,
        table_name: &str,
        data: &HashMap<String, JsonValue>,
    ) -> Result<HashMap<String, JsonValue>, String> {
        let (owner, table) = self.split_table_name(table_name);
        let mut cols = Vec::new();
        let mut placeholders = Vec::new();
        let mut values = Vec::new();

        for (idx, (col, val)) in data.iter().enumerate() {
            cols.push(Self::quote(col));
            placeholders.push(format!(":{}", idx + 1));
            values.push(val.clone());
        }

        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            Self::qualify_table(owner, table),
            cols.join(", "),
            placeholders.join(", ")
        );

        self.run_dml(&sql, &values).await?;
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

        let (owner, table) = self.split_table_name(table_name);
        let placeholders: Vec<String> =
            (1..=pk_values.len()).map(|i| format!(":{}", i)).collect();
        let sql = format!(
            "DELETE FROM {} WHERE {} IN ({})",
            Self::qualify_table(owner, table),
            Self::quote(pk_column),
            placeholders.join(", ")
        );

        self.run_dml(&sql, pk_values).await
    }

    async fn get_table_columns(&self, table_name: &str) -> Result<Vec<TableColumn>, String> {
        let (owner, table) = self.split_table_name(table_name);

        let fk_sql = "SELECT \
                cols.COLUMN_NAME as column_name, \
                r.TABLE_NAME as target_table, \
                r_cols.COLUMN_NAME as target_column \
            FROM ALL_CONSTRAINTS a \
            JOIN ALL_CONS_COLUMNS cols ON a.OWNER = cols.OWNER AND a.CONSTRAINT_NAME = cols.CONSTRAINT_NAME \
            JOIN ALL_CONSTRAINTS r ON a.R_OWNER = r.OWNER AND a.R_CONSTRAINT_NAME = r.CONSTRAINT_NAME \
            JOIN ALL_CONS_COLUMNS r_cols ON r.OWNER = r_cols.OWNER AND r.CONSTRAINT_NAME = r_cols.CONSTRAINT_NAME AND cols.POSITION = r_cols.POSITION \
            WHERE a.CONSTRAINT_TYPE = 'R' \
              AND a.OWNER = :1 AND a.TABLE_NAME = :2";

        let params = [
            JsonValue::String(owner.to_string()),
            JsonValue::String(table.to_string()),
        ];
        let (fk_rows, _, _) = self.run_query(fk_sql, &params).await?;

        let mut fk_map: HashMap<String, ForeignKeyDef> = HashMap::new();
        for r in fk_rows {
            if let (Some(col), Some(tgt_tbl), Some(tgt_col)) = (
                r.get("COLUMN_NAME").and_then(|v| v.as_str()),
                r.get("TARGET_TABLE").and_then(|v| v.as_str()),
                r.get("TARGET_COLUMN").and_then(|v| v.as_str()),
            ) {
                fk_map.insert(
                    col.to_string(),
                    ForeignKeyDef {
                        target_table: tgt_tbl.to_string(),
                        target_column: tgt_col.to_string(),
                    },
                );
            }
        }

        let sql = "SELECT \
                c.COLUMN_NAME, \
                c.DATA_TYPE, \
                c.NULLABLE, \
                c.DATA_DEFAULT, \
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

        let (rows, _, _) = self.run_query(sql, &params).await?;

        Ok(rows
            .iter()
            .map(|r| {
                let col_name = r
                    .get("COLUMN_NAME")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let foreign_key = fk_map.get(&col_name).cloned();
                TableColumn {
                    name: col_name,
                    data_type: r
                        .get("DATA_TYPE")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
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
                    default: r
                        .get("DATA_DEFAULT")
                        .and_then(|v| v.as_str())
                        .map(str::to_string),
                    foreign_key,
                }
            })
            .collect())
    }

    async fn create_table(
        &self,
        table_name: &str,
        columns: &[TableColumn],
    ) -> Result<(), String> {
        let (owner, table) = self.split_table_name(table_name);
        let safe_table = if owner.is_empty()
            || owner.eq_ignore_ascii_case("default")
            || owner.eq_ignore_ascii_case(&self.current_schema)
        {
            Self::quote(table)
        } else {
            Self::qualify_table(owner, table)
        };

        let sql = crate::drivers::utils::build_create_table_sql_generic(
            &safe_table,
            columns,
            Self::quote,
        )?;
        self.run_dml(&sql, &[]).await
    }

    async fn alter_table(
        &self,
        table_name: &str,
        operations: &[AlterOperation],
    ) -> Result<(), String> {
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
                            fk.target_table
                                .split('.')
                                .map(Self::quote)
                                .collect::<Vec<String>>()
                                .join(".")
                        } else {
                            Self::quote(&fk.target_table)
                        };
                        q.push_str(&format!(
                            " REFERENCES {} ({})",
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
                    let constraint = op
                        .constraint_name
                        .as_ref()
                        .ok_or("constraint_name is required")?;
                    format!(
                        "ALTER TABLE {} DROP CONSTRAINT {}",
                        safe_table,
                        Self::quote(constraint)
                    )
                }
                "ADD_FOREIGN_KEY" => {
                    let fk = op.foreign_key.as_ref().ok_or("foreignKey is required")?;
                    let fk_name = format!("fk_{}_{}", table_name, op.name);
                    let quoted_target_table = if fk.target_table.contains('.') {
                        fk.target_table
                            .split('.')
                            .map(Self::quote)
                            .collect::<Vec<String>>()
                            .join(".")
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
                    let definition = op
                        .definition
                        .as_ref()
                        .ok_or("definition is required for ADD_CONSTRAINT")?;
                    format!(
                        "ALTER TABLE {} ADD CONSTRAINT {} {}",
                        safe_table,
                        Self::quote(&op.name),
                        definition
                    )
                }
                _ => continue,
            };

            self.run_dml(&sql, &[]).await?;
        }
        Ok(())
    }

    async fn drop_table(&self, table_name: &str, cascade: bool) -> Result<(), String> {
        let (owner, table) = self.split_table_name(table_name);
        let sql = if cascade {
            format!(
                "DROP TABLE {} CASCADE CONSTRAINTS",
                Self::qualify_table(owner, table)
            )
        } else {
            format!("DROP TABLE {}", Self::qualify_table(owner, table))
        };
        self.run_dml(&sql, &[]).await
    }

    async fn create_schema(&self, schema_name: &str) -> Result<(), String> {
        let sql = format!(
            "CREATE USER {} IDENTIFIED BY {}",
            Self::quote(schema_name),
            Self::quote(schema_name)
        );
        self.run_dml(&sql, &[]).await
    }

    async fn drop_schema(&self, schema_name: &str) -> Result<(), String> {
        let sql = format!("DROP USER {} CASCADE", Self::quote(schema_name));
        self.run_dml(&sql, &[]).await
    }

    async fn drop_database(&self, db_name: &str) -> Result<(), String> {
        let sql = format!("DROP USER {} CASCADE", Self::quote(db_name));
        self.run_dml(&sql, &[]).await
    }

    async fn create_database(
        &self,
        db_name: &str,
        password: Option<&str>,
    ) -> Result<(), String> {
        let pass = password.unwrap_or(db_name);
        let sql = format!(
            "CREATE USER {} IDENTIFIED BY \"{}\"",
            Self::quote(db_name),
            pass.replace('"', "\"\"")
        );
        self.run_dml(&sql, &[]).await?;
        let grant_sql = format!("GRANT CONNECT, RESOURCE TO {}", Self::quote(db_name));
        self.run_dml(&grant_sql, &[]).await
    }

    async fn export_to_csv(&self, table_name: &str, export_path: &str) -> Result<(), String> {
        let (owner, table) = self.split_table_name(table_name);
        let sql = format!("SELECT * FROM {}", Self::qualify_table(owner, table));
        let (rows, _, _) = self.run_query(&sql, &[]).await?;

        crate::drivers::utils::export_rows_to_csv(&rows, export_path)
    }

    async fn get_schema_details(&self, schema_name: &str) -> Result<SchemaDetails, String> {
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
        let (column_rows, _, _) = self.run_query(column_sql, &params).await?;
        let (relation_rows, _, _) = self.run_query(relation_sql, &params).await?;

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
                fk_map.insert(
                    (src_tbl.to_string(), src_col.to_string()),
                    ForeignKeyDef {
                        target_table: tgt_tbl.to_string(),
                        target_column: tgt_col.to_string(),
                    },
                );
            }
        }

        for r in column_rows {
            let table_name = r
                .get("TABLE_NAME")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let col_name = r
                .get("COLUMN_NAME")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let foreign_key = fk_map.get(&(table_name.clone(), col_name.clone())).cloned();
            let col = TableColumn {
                name: col_name,
                data_type: r
                    .get("DATA_TYPE")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                nullable: r
                    .get("IS_NULLABLE")
                    .and_then(|v| v.as_str())
                    .map(|s| s == "YES")
                    .unwrap_or(true),
                is_primary_key: r
                    .get("IS_PRIMARY_KEY")
                    .and_then(|v| v.as_str())
                    .map(|s| s == "YES")
                    .unwrap_or(false),
                default: r
                    .get("DATA_DEFAULT")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
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
                constraint_name: r
                    .get("CONSTRAINT_NAME")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                source_table: r
                    .get("SOURCE_TABLE")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                source_column: r
                    .get("SOURCE_COLUMN")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                target_table: r
                    .get("TARGET_TABLE")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                target_column: r
                    .get("TARGET_COLUMN")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            });
        }

        Ok(SchemaDetails { tables, relations })
    }

    async fn get_table_indexes(&self, table_name: &str) -> Result<Vec<DbIndex>, String> {
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

        let (rows, _, _) = self
            .run_query(
                sql,
                &[
                    JsonValue::String(owner.to_string()),
                    JsonValue::String(table.to_string()),
                ],
            )
            .await?;

        let mut indexes = Vec::new();
        for r in rows {
            let mut cols = vec![];
            if let Some(val) = r.get("COLUMNS") {
                if let JsonValue::Array(arr) = val {
                    cols = arr
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                } else if let JsonValue::String(s) = val {
                    if let Ok(JsonValue::Array(arr)) = serde_json::from_str(s) {
                        cols = arr
                            .iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect();
                    }
                }
            }

            let name = r
                .get("INDEX_NAME")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let uniqueness = r.get("UNIQUENESS").and_then(|v| v.as_str()).unwrap_or("");
            let is_unique = uniqueness == "UNIQUE";
            let is_primary_key = r
                .get("IS_PRIMARY")
                .and_then(|v| {
                    if let JsonValue::Number(n) = v {
                        Some(n.as_i64().unwrap_or(0) > 0)
                    } else if let JsonValue::String(s) = v {
                        s.parse::<i64>().ok().map(|n| n > 0)
                    } else {
                        None
                    }
                })
                .unwrap_or(false);

            let ddl = if is_primary_key {
                format!("PRIMARY KEY ({})", cols.join(", "))
            } else if is_unique {
                format!(
                    "CREATE UNIQUE INDEX {} ON {} ({})",
                    Self::quote(&name),
                    Self::qualify_table(owner, table),
                    cols.join(", ")
                )
            } else {
                format!(
                    "CREATE INDEX {} ON {} ({})",
                    Self::quote(&name),
                    Self::qualify_table(owner, table),
                    cols.join(", ")
                )
            };

            indexes.push(DbIndex {
                name,
                is_unique,
                is_primary_key,
                index_type: r
                    .get("INDEX_TYPE")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                ddl: Some(ddl),
                columns: cols,
            });
        }
        Ok(indexes)
    }

    async fn get_table_constraints(
        &self,
        table_name: &str,
    ) -> Result<Vec<TableConstraint>, String> {
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

        let (rows, _, _) = self.run_query(sql, &params).await?;

        let mut constraint_map: HashMap<
            String,
            (String, Vec<String>, Vec<String>, String, String),
        > = HashMap::new();
        let mut constraint_order = Vec::new();

        for r in rows {
            let name = r
                .get("CONSTRAINT_NAME")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let c_type = r
                .get("CONSTRAINT_TYPE")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let col = r
                .get("COLUMN_NAME")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let r_tbl = r
                .get("R_TABLE")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let r_col = r
                .get("R_COLUMN")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let condition = r
                .get("SEARCH_CONDITION")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            if !constraint_map.contains_key(&name) {
                constraint_order.push(name.clone());
            }

            let entry = constraint_map
                .entry(name)
                .or_insert_with(|| (c_type, Vec::new(), Vec::new(), r_tbl, condition));
            if !col.is_empty() && !entry.1.contains(&col) {
                entry.1.push(col);
            }
            if !r_col.is_empty() && !entry.2.contains(&r_col) {
                entry.2.push(r_col);
            }
        }

        let mut constraints = Vec::new();
        for name in constraint_order {
            if let Some((c_type, cols, r_cols, r_tbl, condition)) = constraint_map.remove(&name) {
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
                        r_tbl,
                        r_cols.join(", ")
                    ),
                    "CHECK" => condition,
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
        let mut tx = self.tx_conn.lock().await;
        if tx.is_some() {
            return Err("Transaction already in progress".to_string());
        }

        let pool = self.pool()?.clone();
        let conn = tokio::task::spawn_blocking(move || pool.acquire())
            .await
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;

        *tx = Some(conn);
        Ok(())
    }

    async fn commit_transaction(&self) -> Result<(), String> {
        let mut tx = self.tx_conn.lock().await;
        if let Some(conn) = tx.take() {
            tokio::task::spawn_blocking(move || conn.commit().map_err(|e| e.to_string()))
                .await
                .map_err(|e| e.to_string())?
        } else {
            Err("No active transaction to commit".to_string())
        }
    }

    async fn rollback_transaction(&self) -> Result<(), String> {
        let mut tx = self.tx_conn.lock().await;
        if let Some(conn) = tx.take() {
            tokio::task::spawn_blocking(move || conn.rollback().map_err(|e| e.to_string()))
                .await
                .map_err(|e| e.to_string())?
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
        assert_eq!(
            OracleDriver::qualify_table("", "EMPLOYEES"),
            "\"EMPLOYEES\""
        );
        assert_eq!(
            OracleDriver::qualify_table("default", "EMPLOYEES"),
            "\"EMPLOYEES\""
        );
    }

    #[test]
    fn test_oracle_build_create_table_sql() {
        use crate::drivers::TableColumn;

        let columns = vec![
            TableColumn {
                name: "id".to_string(),
                data_type: "NUMBER".to_string(),
                nullable: false,
                is_primary_key: true,
                default: None,
                foreign_key: None,
            },
            TableColumn {
                name: "status".to_string(),
                data_type: "VARCHAR2(20)".to_string(),
                nullable: false,
                is_primary_key: false,
                default: Some("'active'".to_string()),
                foreign_key: None,
            },
        ];

        let safe_table = OracleDriver::qualify_table("default", "new_table");
        let sql = crate::drivers::utils::build_create_table_sql_generic(
            &safe_table,
            &columns,
            OracleDriver::quote,
        )
        .unwrap();
        assert_eq!(
            sql,
            "CREATE TABLE \"new_table\" (\"id\" NUMBER PRIMARY KEY, \"status\" VARCHAR2(20) DEFAULT 'active' NOT NULL)"
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
    fn converts_json_to_oracle_bind() {
        use super::OracleBind;
        use serde_json::json;

        let obj = json!({"key": "value"});
        let bind = OracleDriver::json_to_bind(&obj);
        if let OracleBind::String(s) = bind {
            assert_eq!(s, "{\"key\":\"value\"}");
        } else {
            panic!("Expected OracleBind::String");
        }

        let arr = json!([1, 2, 3]);
        let bind_arr = OracleDriver::json_to_bind(&arr);
        if let OracleBind::String(s) = bind_arr {
            assert_eq!(s, "[1,2,3]");
        } else {
            panic!("Expected OracleBind::String");
        }

        let num = json!(42);
        let bind_num = OracleDriver::json_to_bind(&num);
        if let OracleBind::Int(i) = bind_num {
            assert_eq!(i, 42);
        } else {
            panic!("Expected OracleBind::Int");
        }
    }
}
