pub mod drivers;
pub mod pool;

pub use drivers::{
    create_driver, AlterOperation, ColumnInfo, Config, DatabaseDriver, DbIndex, ForeignKeyDef,
    QueryResult, SchemaDetails, SchemaObject, SchemaResult, TableColumn, TableConstraint,
    TableDataResult, TableInfo, TableRelation,
};
pub use pool::Pool;

/// Installs the default `rustls` ring crypto provider at most once per process.
pub fn init_crypto() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_exports_and_config_defaults() {
        init_crypto();
        let config = Config {
            db_type: "sqlite".to_string(),
            database: ":memory:".to_string(),
            ..Default::default()
        };

        assert_eq!(config.connection_timeout, 30);
        assert_eq!(config.oracle_connect_type, "serviceName");
        assert_eq!(config.oracle_role, "normal");
        assert!(!config.ssl);
        assert_eq!(config.port, 0);
    }

    #[tokio::test]
    async fn test_pool_default_and_sqlite_driver_in_memory() {
        init_crypto();
        let mut pool = Pool::default();
        let config = Config {
            db_type: "sqlite".to_string(),
            database: ":memory:".to_string(),
            ..Default::default()
        };

        let mut driver = create_driver("sqlite").expect("sqlite driver should exist");
        driver
            .connect(&config)
            .await
            .expect("in-memory sqlite should connect");

        let result = driver
            .query("SELECT 1 AS num")
            .await
            .expect("SELECT 1 AS num should succeed");
        assert_eq!(result.row_count, 1);
        assert_eq!(result.fields.len(), 1);
        assert_eq!(
            result.fields[0],
            ColumnInfo {
                name: "num".to_string(),
                data_type: result.fields[0].data_type.clone(),
                is_primary_key: false,
                is_nullable: true,
                display_name: None,
            }
        );

        pool.put("test-conn".to_string(), driver, config).await;
        assert!(pool.get("test-conn", None).is_some());
        pool.close_all().await;
        assert!(pool.get("test-conn", None).is_none());
    }
}
