use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use db_bridge::{
    create_driver, init_crypto, Config, DatabaseDriver, Pool, QueryResult, SchemaResult,
    TableDataResult,
};
use tokio::runtime::{Builder, Runtime};
use tokio::sync::{oneshot, Mutex, RwLock};

use crate::workspace::ConnectionColorTag;

/// Boxed `'static` future bridged from the Tokio runtime to any async executor (including GPUI).
pub type DbFuture<T> = Pin<Box<dyn Future<Output = Result<T, String>> + Send + 'static>>;

/// Metadata describing an active connection for workspace state updates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionMetadata {
    pub connection_id: String,
    pub connection_name: String,
    pub engine: String,
    pub host_port: String,
    pub schema: String,
    pub color: ConnectionColorTag,
}

impl ConnectionMetadata {
    /// Builds display-ready connection metadata from a driver [`Config`].
    pub fn from_config(
        connection_id: impl Into<String>,
        connection_name: impl Into<String>,
        color: ConnectionColorTag,
        config: &Config,
    ) -> Self {
        let engine = match config.db_type.trim().to_ascii_lowercase().as_str() {
            "postgres" | "postgresql" => "PostgreSQL".to_string(),
            "mysql" => "MySQL".to_string(),
            "sqlite" => "SQLite".to_string(),
            "oracle" => "Oracle".to_string(),
            other => other.to_string(),
        };

        let is_sqlite = config.db_type.trim().eq_ignore_ascii_case("sqlite");
        let host_port = if is_sqlite {
            if config.database.trim().is_empty() {
                ":memory:".to_string()
            } else {
                config.database.clone()
            }
        } else if config.port > 0 {
            format!("{}:{}", config.host, config.port)
        } else {
            config.host.clone()
        };

        let schema = if config.database.trim().is_empty() {
            "default".to_string()
        } else {
            config.database.clone()
        };

        Self {
            connection_id: connection_id.into(),
            connection_name: connection_name.into(),
            engine,
            host_port,
            schema,
            color,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectRequest {
    pub connection_id: String,
    pub connection_name: String,
    pub color: ConnectionColorTag,
    pub config: Config,
}

#[derive(Debug, Clone)]
pub struct SchemaRequest {
    pub connection_id: String,
    pub all_databases: bool,
    pub schema_name: Option<String>,
    pub target_database: Option<String>,
}

#[derive(Debug, Clone)]
pub struct QueryRequest {
    pub connection_id: String,
    pub sql: String,
    pub target_database: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TableDataRequest {
    pub connection_id: String,
    pub table_name: String,
    pub limit: i64,
    pub offset: i64,
    pub sort_column: String,
    pub sort_direction: String,
    pub filter: String,
    pub target_database: Option<String>,
}

#[derive(Clone)]
pub struct DatabaseEngine {
    runtime: Arc<Runtime>,
    pool: Arc<Mutex<Pool>>,
}

impl DatabaseEngine {
    /// Initializes the process-wide TLS crypto provider and multi-threaded Tokio runtime.
    pub fn new() -> Result<Self, String> {
        init_crypto();
        let runtime = Builder::new_multi_thread()
            .enable_all()
            .thread_name("table-view-db")
            .build()
            .map_err(|err| format!("Failed to initialize Tokio database runtime: {err}"))?;

        Ok(Self {
            runtime: Arc::new(runtime),
            pool: Arc::new(Mutex::new(Pool::new())),
        })
    }

    fn spawn_on_tokio<T, F, Fut>(&self, task: F) -> DbFuture<T>
    where
        T: Send + 'static,
        F: FnOnce(Arc<Mutex<Pool>>) -> Fut + Send + 'static,
        Fut: Future<Output = Result<T, String>> + Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        let pool = Arc::clone(&self.pool);

        self.runtime.spawn(async move {
            let outcome = task(pool).await;
            let _ = tx.send(outcome);
        });

        Box::pin(async move {
            rx.await
                .map_err(|err| format!("Database worker task was cancelled: {err}"))?
        })
    }

    /// Tests connectivity to a database without adding the connection to the active pool.
    pub fn test_connection(&self, config: Config) -> DbFuture<()> {
        self.spawn_on_tokio(move |_pool| async move {
            let mut driver = create_driver(&config.db_type).ok_or_else(|| {
                format!("Unsupported database driver type: {}", config.db_type)
            })?;
            driver.connect(&config).await?;
            let _ = driver.disconnect().await;
            Ok(())
        })
    }

    /// Connects to a database, registers the live driver in the pool, and returns connection metadata.
    pub fn connect(&self, request: ConnectRequest) -> DbFuture<ConnectionMetadata> {
        self.spawn_on_tokio(move |pool| async move {
            let mut driver = create_driver(&request.config.db_type).ok_or_else(|| {
                format!(
                    "Unsupported database driver type: {}",
                    request.config.db_type
                )
            })?;

            let conn_id = if request.connection_id.trim().is_empty() {
                format!("{}-default", request.config.db_type)
            } else {
                request.connection_id.clone()
            };

            driver.connect(&request.config).await?;

            let metadata = ConnectionMetadata::from_config(
                conn_id.clone(),
                request.connection_name,
                request.color,
                &request.config,
            );

            let mut guard = pool.lock().await;
            guard.remove_all_for_connection(&conn_id).await;
            guard.put(conn_id, driver, request.config).await;

            Ok(metadata)
        })
    }

    /// Closes and removes all pooled connections associated with `connection_id`.
    pub fn disconnect(&self, connection_id: String) -> DbFuture<()> {
        self.spawn_on_tokio(move |pool| async move {
            let mut guard = pool.lock().await;
            guard.remove_all_for_connection(&connection_id).await;
            Ok(())
        })
    }

    /// Introspects schemas, tables, views, and functions for an active connection.
    pub fn get_schema(&self, request: SchemaRequest) -> DbFuture<SchemaResult> {
        self.spawn_on_tokio(move |pool| async move {
            let target_db = normalize_optional(request.target_database.as_deref());
            let driver_arc =
                ensure_driver(&pool, &request.connection_id, target_db.as_deref()).await?;
            let schema_name = normalize_optional(request.schema_name.as_deref());
            let driver = driver_arc.read().await;
            driver
                .get_schema(request.all_databases, schema_name.as_deref())
                .await
        })
    }

    /// Executes an arbitrary SQL query on the pooled connection.
    pub fn execute_query(&self, request: QueryRequest) -> DbFuture<QueryResult> {
        self.spawn_on_tokio(move |pool| async move {
            let target_db = normalize_optional(request.target_database.as_deref());
            let driver_arc =
                ensure_driver(&pool, &request.connection_id, target_db.as_deref()).await?;
            let driver = driver_arc.read().await;
            driver.query(&request.sql).await
        })
    }

    /// Fetches paginated, sorted, and filtered rows from a table on the pooled connection.
    pub fn fetch_table_data(&self, request: TableDataRequest) -> DbFuture<TableDataResult> {
        self.spawn_on_tokio(move |pool| async move {
            let target_db = normalize_optional(request.target_database.as_deref());
            let driver_arc =
                ensure_driver(&pool, &request.connection_id, target_db.as_deref()).await?;
            let driver = driver_arc.read().await;
            driver
                .fetch_table_data(
                    &request.table_name,
                    request.limit,
                    request.offset,
                    &request.sort_column,
                    &request.sort_direction,
                    &request.filter,
                )
                .await
        })
    }

    /// Closes all active connections in the pool.
    pub fn close_all(&self) -> DbFuture<()> {
        self.spawn_on_tokio(move |pool| async move {
            let mut guard = pool.lock().await;
            guard.close_all().await;
            Ok(())
        })
    }
}

fn normalize_optional(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|trimmed| !trimmed.is_empty())
        .map(str::to_string)
}

async fn ensure_driver(
    pool: &Arc<Mutex<Pool>>,
    connection_id: &str,
    target_database: Option<&str>,
) -> Result<Arc<RwLock<Box<dyn DatabaseDriver>>>, String> {
    {
        let mut guard = pool.lock().await;
        if let Some(driver) = guard.get(connection_id, target_database) {
            return Ok(driver);
        }
    }

    log::info!(
        "db: driver not in pool, connecting outside lock for connection {}",
        connection_id
    );

    let base_config = {
        let guard = pool.lock().await;
        guard.get_config(connection_id).cloned()
    }
    .ok_or_else(|| format!("Connection config not found for {connection_id}"))?;

    let target_db_name = target_database
        .map(str::to_string)
        .unwrap_or_else(|| base_config.database.clone());

    let mut new_config = base_config;
    new_config.database = target_db_name.clone();

    let mut driver = create_driver(&new_config.db_type).ok_or_else(|| {
        format!("Unsupported database driver type: {}", new_config.db_type)
    })?;

    // Connect outside the global pool lock so concurrent connections do not stall.
    driver.connect(&new_config).await?;

    let mut guard = pool.lock().await;
    guard
        .put(connection_id.to_string(), driver, new_config)
        .await;
    guard
        .get(connection_id, Some(&target_db_name))
        .ok_or_else(|| format!("Pooled driver unavailable after connect for {connection_id}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_sqlite_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("table_view_{name}_{}_{nanos}.db", std::process::id()))
    }

    fn run_future<T>(fut: DbFuture<T>) -> Result<T, String> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("test runtime should build");
        rt.block_on(fut)
    }

    #[test]
    fn test_engine_sqlite_test_connection_connect_schema_query_and_disconnect() {
        let db_path = temp_sqlite_path("engine_lifecycle");
        std::fs::File::create(&db_path).expect("temp sqlite file should be created");
        let db_path_str = db_path.to_string_lossy().to_string();

        let engine = DatabaseEngine::new().expect("DatabaseEngine should initialize");
        let config = Config {
            db_type: "sqlite".to_string(),
            database: db_path_str.clone(),
            ..Default::default()
        };

        run_future(engine.test_connection(config.clone())).expect("test_connection should pass");

        let metadata = run_future(engine.connect(ConnectRequest {
            connection_id: "conn-1".to_string(),
            connection_name: "local-sqlite".to_string(),
            color: ConnectionColorTag::Teal,
            config,
        }))
        .expect("connect should succeed");
        assert_eq!(metadata.connection_id, "conn-1");
        assert_eq!(metadata.connection_name, "local-sqlite");
        assert_eq!(metadata.engine, "SQLite");
        assert_eq!(metadata.color, ConnectionColorTag::Teal);

        run_future(engine.execute_query(QueryRequest {
            connection_id: "conn-1".to_string(),
            sql: "CREATE TABLE items (id INTEGER PRIMARY KEY, label TEXT)".to_string(),
            target_database: None,
        }))
        .expect("CREATE TABLE should succeed");

        run_future(engine.execute_query(QueryRequest {
            connection_id: "conn-1".to_string(),
            sql: "INSERT INTO items VALUES (1, 'alpha')".to_string(),
            target_database: None,
        }))
        .expect("INSERT should succeed");

        let schema = run_future(engine.get_schema(SchemaRequest {
            connection_id: "conn-1".to_string(),
            all_databases: false,
            schema_name: None,
            target_database: None,
        }))
        .expect("get_schema should succeed");
        assert!(schema.tables.iter().any(|table| table.name == "items"));

        let query_result = run_future(engine.execute_query(QueryRequest {
            connection_id: "conn-1".to_string(),
            sql: "SELECT id, label FROM items".to_string(),
            target_database: None,
        }))
        .expect("SELECT should succeed");
        assert_eq!(query_result.row_count, 1);
        assert_eq!(query_result.fields.len(), 2);

        let table_data = run_future(engine.fetch_table_data(TableDataRequest {
            connection_id: "conn-1".to_string(),
            table_name: "items".to_string(),
            limit: 50,
            offset: 0,
            sort_column: String::new(),
            sort_direction: String::new(),
            filter: String::new(),
            target_database: None,
        }))
        .expect("fetch_table_data should succeed");
        assert_eq!(table_data.total_count, 1);
        assert_eq!(table_data.rows.len(), 1);

        run_future(engine.disconnect("conn-1".to_string())).expect("disconnect should succeed");

        let post_disconnect = run_future(engine.execute_query(QueryRequest {
            connection_id: "conn-1".to_string(),
            sql: "SELECT 1".to_string(),
            target_database: None,
        }));
        assert!(post_disconnect.is_err());

        run_future(engine.close_all()).expect("close_all should succeed");
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn test_engine_returns_structured_error_for_unsupported_driver_and_bad_sql() {
        let engine = DatabaseEngine::new().expect("DatabaseEngine should initialize");

        let unsupported = run_future(engine.test_connection(Config {
            db_type: "unknown-db".to_string(),
            ..Default::default()
        }));
        assert!(unsupported.is_err());

        run_future(engine.connect(ConnectRequest {
            connection_id: "mem-1".to_string(),
            connection_name: "mem".to_string(),
            color: ConnectionColorTag::Blue,
            config: Config {
                db_type: "sqlite".to_string(),
                database: ":memory:".to_string(),
                ..Default::default()
            },
        }))
        .expect("in-memory sqlite connect should succeed");

        let bad_sql = run_future(engine.execute_query(QueryRequest {
            connection_id: "mem-1".to_string(),
            sql: "SELECT * FROM nonexistent_table_xyz".to_string(),
            target_database: None,
        }));
        assert!(bad_sql.is_err());

        run_future(engine.close_all()).expect("close_all should succeed");
    }
}
