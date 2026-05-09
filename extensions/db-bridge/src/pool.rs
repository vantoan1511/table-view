use crate::drivers::{Config, DatabaseDriver};
use std::collections::{HashMap, VecDeque};
use tokio::sync::Mutex;
use std::sync::Arc;

const MAX_SIZE: usize = 5;

/// A thread-safe LRU connection pool for database drivers.
/// Maintains up to MAX_SIZE concurrent live connections keyed by connectionId.
/// When the pool is full, the least-recently-used connection is closed and evicted.
pub struct Pool {
    order: VecDeque<String>,       // front = most recently used (key is "connId:dbName")
    drivers: HashMap<String, Arc<Mutex<Box<dyn DatabaseDriver>>>>,
    /// Stored config per connectionId, used for spawning per-DB sub-connections.
    configs: HashMap<String, Config>,
}

impl Pool {
    pub fn new() -> Self {
        Self {
            order: VecDeque::new(),
            drivers: HashMap::new(),
            configs: HashMap::new(),
        }
    }

    /// Key format for the pool: "connectionId:database"
    fn make_key(id: &str, database: &str) -> String {
        format!("{}:{}", id, database)
    }

    /// Get an Arc to the driver for the given connectionId and optional target_database.
    /// If target_database is None, it uses the database name from the original config.
    pub fn get(&mut self, id: &str, target_database: Option<&str>) -> Option<Arc<Mutex<Box<dyn DatabaseDriver>>>> {
        let db = if let Some(db) = target_database {
            db.to_string()
        } else {
            self.configs.get(id)?.database.clone()
        };

        let key = Self::make_key(id, &db);

        if self.drivers.contains_key(&key) {
            self.order.retain(|x| x != &key);
            self.order.push_front(key.clone());
            self.drivers.get(&key).cloned()
        } else {
            None
        }
    }

    /// Register a driver under the given connectionId and its config.
    /// The driver is keyed by "connectionId:database" using the database from the config.
    pub async fn put(&mut self, id: String, driver: Box<dyn DatabaseDriver>, config: Config) {
        let key = Self::make_key(&id, &config.database);

        // Replace existing
        if let Some(old) = self.drivers.remove(&key) {
            let mut d = old.lock().await;
            if let Err(e) = d.disconnect().await {
                log::warn!("pool: error disconnecting replaced driver {}: {}", key, e);
            }
            self.order.retain(|x| x != &key);
        }

        // Evict LRU if at capacity
        if self.drivers.len() >= MAX_SIZE {
            if let Some(oldest_key) = self.order.pop_back() {
                log::info!("pool: evicting LRU connection {} (pool full)", oldest_key);
                if let Some(evicted_arc) = self.drivers.remove(&oldest_key) {
                    let mut evicted = evicted_arc.lock().await;
                    if let Err(e) = evicted.disconnect().await {
                        log::warn!(
                            "pool: error disconnecting evicted driver {}: {}",
                            oldest_key,
                            e
                        );
                    }
                }
            }
        }

        self.order.push_front(key.clone());
        self.configs.insert(id, config);
        self.drivers.insert(key, Arc::new(Mutex::new(driver)));
    }

    /// Retrieve the stored Config for a connectionId.
    pub fn get_config(&self, id: &str) -> Option<&Config> {
        self.configs.get(id)
    }

    /// Explicitly close and remove a connection from the pool for a specific database.
    pub async fn remove(&mut self, id: &str, database: &str) {
        let key = Self::make_key(id, database);
        if let Some(driver_arc) = self.drivers.remove(&key) {
            let mut driver = driver_arc.lock().await;
            if let Err(e) = driver.disconnect().await {
                log::warn!("pool: error disconnecting removed driver {}: {}", key, e);
            }
            self.order.retain(|x| x != &key);
        }
    }

    /// Disconnect every active driver in the pool.
    pub async fn close_all(&mut self) {
        for (key, driver_arc) in self.drivers.drain() {
            let mut driver = driver_arc.lock().await;
            if let Err(e) = driver.disconnect().await {
                log::warn!("pool: error closing driver {}: {}", key, e);
            }
        }
        self.order.clear();
        self.configs.clear();
    }

    /// Get a driver for the given connectionId and database.
    /// If a driver for that database doesn't exist, it creates one using the stored Config.
    pub async fn get_or_create(
        &mut self,
        id: &str,
        target_database: Option<&str>,
    ) -> Result<Arc<Mutex<Box<dyn DatabaseDriver>>>, String> {
        if let Some(driver) = self.get(id, target_database) {
            return Ok(driver);
        }

        // Need to create a new one
        let base_config = self
            .get_config(id)
            .ok_or_else(|| format!("connection config not found for {}", id))?
            .clone();

        let target_db = target_database
            .map(|s| s.to_string())
            .unwrap_or_else(|| base_config.database.clone());

        log::info!(
            "pool: spawning sub-connection for {} on database {}",
            id,
            target_db
        );

        let mut new_config = base_config.clone();
        new_config.database = target_db;

        let mut driver = crate::drivers::create_driver(&new_config.db_type)
            .ok_or_else(|| format!("unsupported driver type: {}", new_config.db_type))?;

        driver.connect(&new_config).await?;

        // We use new_config for put so it keys by the target_db,
        // but we want to KEEP the base_config in self.configs.
        self.put(id.to_string(), driver, new_config).await;
        // Restore base config as the template
        self.configs.insert(id.to_string(), base_config);

        self.get(id, target_database)
            .ok_or_else(|| "failed to retrieve driver after putting it in pool".to_string())
    }
}
