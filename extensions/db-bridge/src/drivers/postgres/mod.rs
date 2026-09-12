pub mod convert;
pub mod data;
pub mod ddl;
pub mod schema;

#[cfg(test)]
mod tests;

use super::{
    AlterOperation, Config, DatabaseDriver, DbIndex, QueryResult, SchemaDetails, SchemaResult,
    TableColumn, TableConstraint, TableDataResult,
};
use async_trait::async_trait;
use serde_json::Value;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::collections::HashMap;

pub struct PostgresDriver {
    pool: Option<PgPool>,
    tx_conn: tokio::sync::Mutex<Option<sqlx::pool::PoolConnection<sqlx::Postgres>>>,
}

impl PostgresDriver {
    pub fn new() -> Self {
        Self {
            pool: None,
            tx_conn: tokio::sync::Mutex::new(None),
        }
    }

    pub(crate) fn pool(&self) -> Result<&PgPool, String> {
        self.pool.as_ref().ok_or_else(|| "Not connected".to_string())
    }

    #[allow(dead_code)]
    pub fn quote(ident: &str) -> String {
        ddl::quote(ident)
    }

    #[allow(dead_code)]
    pub fn split_table_name(table_name: &str) -> (&str, &str) {
        ddl::split_table_name(table_name)
    }

    #[allow(dead_code)]
    pub fn qualified_table_name(table_name: &str) -> String {
        ddl::qualified_table_name(table_name)
    }

    #[allow(dead_code)]
    pub fn build_drop_table_sql(table_name: &str, cascade: bool) -> String {
        ddl::build_drop_table_sql(table_name, cascade)
    }
}

#[async_trait]
impl DatabaseDriver for PostgresDriver {
    async fn connect(&mut self, config: &Config) -> Result<(), String> {
        let ssl_mode = if config.ssl {
            config.ssl_mode.as_deref().unwrap_or("prefer")
        } else {
            "disable"
        };

        let host = if config.host.contains(':') && !config.host.starts_with('[') {
            format!("[{}]", config.host)
        } else {
            config.host.clone()
        };

        async fn connect_pool(config: &Config, host: &str, mode: &str) -> Result<PgPool, String> {
            let timeout_secs = 5u64;
            let dsn = format!(
                "postgres://{}:{}@{}:{}/{}?sslmode={}&application_name=db_manager&connect_timeout={}&tcp_user_timeout=5000",
                urlencoding::encode(&config.username),
                urlencoding::encode(&config.password),
                host,
                config.port,
                urlencoding::encode(&config.database),
                mode,
                timeout_secs
            );
            use std::str::FromStr;
            let mut connect_options = sqlx::postgres::PgConnectOptions::from_str(&dsn)
                .map_err(|e| e.to_string())?;
            connect_options = connect_options.statement_cache_capacity(0);

            let duration = std::time::Duration::from_secs(timeout_secs);

            match tokio::time::timeout(
                duration,
                PgPoolOptions::new()
                    .max_connections(5)
                    .acquire_timeout(duration)
                    .connect_with(connect_options),
            )
            .await
            {
                Ok(Ok(pool)) => Ok(pool),
                Ok(Err(e)) => Err(e.to_string()),
                Err(_) => Err(format!(
                    "Connection attempt timed out after {} seconds. Please check host, port, firewall, or SSL settings.",
                    timeout_secs
                )),
            }
        }

        let pool = match connect_pool(config, &host, ssl_mode).await {
            Ok(pool) => pool,
            Err(err_str) => {
                let is_eof = err_str.contains("EOF") || err_str.contains("0 bytes");

                if is_eof && ssl_mode == "disable" {
                    log::warn!(
                        "postgres connect failed with sslmode=disable ({}), retrying with sslmode=prefer",
                        err_str
                    );
                    match connect_pool(config, &host, "prefer").await {
                        Ok(pool) => pool,
                        Err(fallback_err) => {
                            return Err(format!(
                                "Database connection closed unexpectedly by server (EOF): {}. Please check if SSL/TLS is required by your provider (e.g. AWS RDS, Supabase, Neon) or verify host/port.",
                                fallback_err
                            ));
                        }
                    }
                } else if is_eof && ssl_mode == "prefer" {
                    log::warn!(
                        "postgres connect failed with sslmode=prefer ({}), retrying with sslmode=disable",
                        err_str
                    );
                    match connect_pool(config, &host, "disable").await {
                        Ok(pool) => pool,
                        Err(fallback_err) => {
                            return Err(format!(
                                "Database connection closed unexpectedly by server (EOF): {}. If connecting to local Docker/PostgreSQL, try using '127.0.0.1' instead of 'localhost', or check port mapping.",
                                fallback_err
                            ));
                        }
                    }
                } else if is_eof {
                    return Err(format!(
                        "Database connection closed unexpectedly by server (EOF): {}. If connecting to local Docker/PostgreSQL, try using '127.0.0.1' instead of 'localhost' or verify host/port.",
                        err_str
                    ));
                } else {
                    return Err(err_str);
                }
            }
        };

        self.pool = Some(pool);
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
        schema::get_schema(self.pool()?, all_databases, schema_name).await
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
            self.pool()?,
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
        data::query(self.pool()?, &self.tx_conn, sql).await
    }

    async fn update_cell(
        &self,
        table_name: &str,
        pk_column: &str,
        pk_value: &Value,
        target_column: &str,
        new_value: &Value,
    ) -> Result<(), String> {
        data::update_cell(
            self.pool()?,
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
        data: &HashMap<String, Value>,
    ) -> Result<HashMap<String, Value>, String> {
        data::insert_row(self.pool()?, table_name, data).await
    }

    async fn delete_rows(
        &self,
        table_name: &str,
        pk_column: &str,
        pk_values: &[Value],
    ) -> Result<(), String> {
        data::delete_rows(self.pool()?, table_name, pk_column, pk_values).await
    }

    async fn get_table_columns(&self, table_name: &str) -> Result<Vec<TableColumn>, String> {
        schema::get_table_columns(self.pool()?, table_name).await
    }

    async fn alter_table(
        &self,
        table_name: &str,
        operations: &[AlterOperation],
    ) -> Result<(), String> {
        ddl::alter_table(self.pool()?, table_name, operations).await
    }

    async fn create_table(&self, table_name: &str, columns: &[TableColumn]) -> Result<(), String> {
        ddl::create_table(self.pool()?, table_name, columns).await
    }

    async fn drop_table(&self, table_name: &str, cascade: bool) -> Result<(), String> {
        ddl::drop_table(self.pool()?, table_name, cascade).await
    }

    async fn create_schema(&self, schema_name: &str) -> Result<(), String> {
        ddl::create_schema(self.pool()?, schema_name).await
    }

    async fn drop_schema(&self, schema_name: &str) -> Result<(), String> {
        ddl::drop_schema(self.pool()?, schema_name).await
    }

    async fn drop_database(&self, db_name: &str) -> Result<(), String> {
        ddl::drop_database(self.pool()?, db_name).await
    }

    async fn create_database(&self, db_name: &str, password: Option<&str>) -> Result<(), String> {
        ddl::create_database(self.pool()?, db_name, password).await
    }

    async fn export_to_csv(&self, table_name: &str, export_path: &str) -> Result<(), String> {
        data::export_to_csv(self.pool()?, table_name, export_path).await
    }

    async fn get_schema_details(&self, schema_name: &str) -> Result<SchemaDetails, String> {
        schema::get_schema_details(self.pool()?, schema_name).await
    }

    async fn get_table_indexes(&self, table_name: &str) -> Result<Vec<DbIndex>, String> {
        schema::get_table_indexes(self.pool()?, table_name).await
    }

    async fn get_table_constraints(&self, table_name: &str) -> Result<Vec<TableConstraint>, String> {
        schema::get_table_constraints(self.pool()?, table_name).await
    }

    async fn begin_transaction(&self) -> Result<(), String> {
        let pool = self.pool()?;
        let mut tx = self.tx_conn.lock().await;
        if tx.is_some() {
            return Err("Transaction already in progress".to_string());
        }

        let mut conn = pool.acquire().await.map_err(|e| e.to_string())?;
        sqlx::query("BEGIN")
            .execute(&mut *conn)
            .await
            .map_err(|e| e.to_string())?;
        *tx = Some(conn);
        Ok(())
    }

    async fn commit_transaction(&self) -> Result<(), String> {
        let mut tx = self.tx_conn.lock().await;
        if let Some(mut conn) = tx.take() {
            sqlx::query("COMMIT")
                .execute(&mut *conn)
                .await
                .map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("No active transaction to commit".to_string())
        }
    }

    async fn rollback_transaction(&self) -> Result<(), String> {
        let mut tx = self.tx_conn.lock().await;
        if let Some(mut conn) = tx.take() {
            sqlx::query("ROLLBACK")
                .execute(&mut *conn)
                .await
                .map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("No active transaction to rollback".to_string())
        }
    }

    async fn is_in_transaction(&self) -> bool {
        self.tx_conn.lock().await.is_some()
    }
}
