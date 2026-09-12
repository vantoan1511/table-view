pub mod convert;
pub mod data;
pub mod ddl;
pub mod schema;

#[cfg(test)]
mod tests;

pub use convert::OracleBind;

use super::{
    AlterOperation, Config, DatabaseDriver, DbIndex, QueryResult, SchemaDetails, SchemaResult,
    TableColumn, TableConstraint, TableDataResult,
};
use async_trait::async_trait;
use convert::{cursor_to_maps, json_to_bind, to_db_value_refs, QueryRawResult};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;

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
        self.pool
            .as_ref()
            .ok_or_else(|| "Not connected".to_string())
    }

    #[allow(dead_code)]
    pub fn quote(ident: &str) -> String {
        ddl::quote(ident)
    }

    #[allow(dead_code)]
    pub fn qualify_table(owner: &str, table_name: &str) -> String {
        ddl::qualify_table(owner, table_name)
    }

    #[allow(dead_code)]
    pub fn pagination_clause(limit: i64, offset: i64) -> String {
        ddl::pagination_clause(limit, offset)
    }

    #[allow(dead_code)]
    pub fn json_to_bind(val: &JsonValue) -> OracleBind {
        convert::json_to_bind(val)
    }

    pub fn current_schema(&self) -> &str {
        &self.current_schema
    }

    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    pub fn split_table_name<'a>(&'a self, table_name: &'a str) -> (&'a str, &'a str) {
        let (owner, name) = table_name
            .split_once('.')
            .unwrap_or((self.current_schema.as_str(), table_name));
        if owner.eq_ignore_ascii_case("default") || owner.is_empty() {
            (self.current_schema.as_str(), name)
        } else {
            (owner, name)
        }
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

    fn execute_query_on_conn(
        conn: &oracledb::Connection,
        sql: &str,
        params: &[JsonValue],
    ) -> Result<QueryRawResult, String> {
        let binds: Vec<OracleBind> = params.iter().map(json_to_bind).collect();
        let null_str: Option<String> = None;
        let bind_refs = to_db_value_refs(&binds, &null_str);
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
            let mut stmt = conn.statement(&cleaned_sql).map_err(|e| e.to_string())?;
            stmt.fetch_lobs();
            let cursor = stmt.query(&bind_refs).map_err(|e| e.to_string())?;
            cursor_to_maps(cursor)?
        } else {
            conn.execute(&cleaned_sql, &bind_refs)
                .map_err(|e| e.to_string())?;
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
        let binds: Vec<OracleBind> = params.iter().map(json_to_bind).collect();
        let null_str: Option<String> = None;
        let bind_refs = to_db_value_refs(&binds, &null_str);
        let cleaned_sql = Self::clean_sql(sql);
        conn.execute(&cleaned_sql, &bind_refs)
            .map_err(|e| e.to_string())?;

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

    pub async fn run_query(
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

    pub async fn run_dml(&self, sql: &str, params: &[JsonValue]) -> Result<(), String> {
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
        let use_implicit_sysdba =
            oracle_role == "normal" && config.username.trim().eq_ignore_ascii_case("sys");
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
        schema::get_schema(self, all_databases, schema_name).await
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
        data::fetch_table_data(
            self,
            table_name,
            limit,
            offset,
            sort_column,
            sort_direction,
            filter,
        )
        .await
    }

    async fn query(&self, sql: &str) -> Result<QueryResult, String> {
        data::query(self, sql).await
    }

    async fn update_cell(
        &self,
        table_name: &str,
        pk_column: &str,
        pk_value: &JsonValue,
        target_column: &str,
        new_value: &JsonValue,
    ) -> Result<(), String> {
        data::update_cell(
            self,
            table_name,
            pk_column,
            pk_value,
            target_column,
            new_value,
        )
        .await
    }

    async fn insert_row(
        &self,
        table_name: &str,
        data: &HashMap<String, JsonValue>,
    ) -> Result<HashMap<String, JsonValue>, String> {
        data::insert_row(self, table_name, data).await
    }

    async fn delete_rows(
        &self,
        table_name: &str,
        pk_column: &str,
        pk_values: &[JsonValue],
    ) -> Result<(), String> {
        data::delete_rows(self, table_name, pk_column, pk_values).await
    }

    async fn get_table_columns(&self, table_name: &str) -> Result<Vec<TableColumn>, String> {
        schema::get_table_columns(self, table_name).await
    }

    async fn create_table(&self, table_name: &str, columns: &[TableColumn]) -> Result<(), String> {
        ddl::create_table(self, table_name, columns).await
    }

    async fn alter_table(
        &self,
        table_name: &str,
        operations: &[AlterOperation],
    ) -> Result<(), String> {
        ddl::alter_table(self, table_name, operations).await
    }

    async fn drop_table(&self, table_name: &str, cascade: bool) -> Result<(), String> {
        ddl::drop_table(self, table_name, cascade).await
    }

    async fn create_schema(&self, schema_name: &str) -> Result<(), String> {
        ddl::create_schema(self, schema_name).await
    }

    async fn drop_schema(&self, schema_name: &str) -> Result<(), String> {
        ddl::drop_schema(self, schema_name).await
    }

    async fn drop_database(&self, db_name: &str) -> Result<(), String> {
        ddl::drop_database(self, db_name).await
    }

    async fn create_database(&self, db_name: &str, password: Option<&str>) -> Result<(), String> {
        ddl::create_database(self, db_name, password).await
    }

    async fn export_to_csv(&self, table_name: &str, export_path: &str) -> Result<(), String> {
        data::export_to_csv(self, table_name, export_path).await
    }

    async fn get_schema_details(&self, schema_name: &str) -> Result<SchemaDetails, String> {
        schema::get_schema_details(self, schema_name).await
    }

    async fn get_table_indexes(&self, table_name: &str) -> Result<Vec<DbIndex>, String> {
        schema::get_table_indexes(self, table_name).await
    }

    async fn get_table_constraints(
        &self,
        table_name: &str,
    ) -> Result<Vec<TableConstraint>, String> {
        schema::get_table_constraints(self, table_name).await
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
