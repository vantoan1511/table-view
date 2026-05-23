use crate::drivers::{Config, DatabaseDriver};
use std::collections::{HashMap, VecDeque};
use tokio::sync::RwLock;
use std::sync::Arc;

const MAX_SIZE: usize = 5;

/// A thread-safe LRU connection pool for database drivers.
/// Maintains up to MAX_SIZE concurrent live connections keyed by connectionId.
/// When the pool is full, the least-recently-used connection is closed and evicted.
pub struct Pool {
    order: VecDeque<String>,       // front = most recently used (key is "connId:dbName")
    drivers: HashMap<String, Arc<RwLock<Box<dyn DatabaseDriver>>>>,
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
    pub fn get(&mut self, id: &str, target_database: Option<&str>) -> Option<Arc<RwLock<Box<dyn DatabaseDriver>>>> {
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

    pub async fn put(&mut self, id: String, driver: Box<dyn DatabaseDriver>, config: Config) {
        let key = Self::make_key(&id, &config.database);

        // Replace existing
        if let Some(old) = self.drivers.remove(&key) {
            tokio::spawn(async move {
                let mut d = old.write().await;
                if let Err(e) = d.disconnect().await {
                    log::warn!("pool: error disconnecting replaced driver: {}", e);
                }
            });
            self.order.retain(|x| x != &key);
        }

        // Evict LRU if at capacity
        if self.drivers.len() >= MAX_SIZE {
            if let Some(oldest_key) = self.order.pop_back() {
                log::info!("pool: evicting LRU connection {} (pool full)", oldest_key);
                if let Some(evicted_arc) = self.drivers.remove(&oldest_key) {
                    tokio::spawn(async move {
                        let mut evicted = evicted_arc.write().await;
                        if let Err(e) = evicted.disconnect().await {
                            log::warn!(
                                "pool: error disconnecting evicted driver: {}",
                                e
                            );
                        }
                    });
                }
            }
        }

        self.order.push_front(key.clone());
        self.configs.entry(id).or_insert(config);
        self.drivers.insert(key, Arc::new(RwLock::new(driver)));
    }

    /// Retrieve the stored Config for a connectionId.
    pub fn get_config(&self, id: &str) -> Option<&Config> {
        self.configs.get(id)
    }

    /// Explicitly close and remove a connection from the pool for a specific database.
    pub async fn remove(&mut self, id: &str, database: &str) {
        let key = Self::make_key(id, database);
        if let Some(driver_arc) = self.drivers.remove(&key) {
            let mut driver = driver_arc.write().await;
            if let Err(e) = driver.disconnect().await {
                log::warn!("pool: error disconnecting removed driver {}: {}", key, e);
            }
            self.order.retain(|x| x != &key);
        }
    }

    /// Explicitly close and remove all connections from the pool for a given connectionId.
    pub async fn remove_all_for_connection(&mut self, id: &str) {
        let prefix = format!("{}:", id);
        let mut keys_to_remove = Vec::new();
        for key in self.drivers.keys() {
            if key.starts_with(&prefix) {
                keys_to_remove.push(key.clone());
            }
        }
        for key in keys_to_remove {
            if let Some(driver_arc) = self.drivers.remove(&key) {
                let mut driver = driver_arc.write().await;
                if let Err(e) = driver.disconnect().await {
                    log::warn!("pool: error disconnecting removed driver {}: {}", key, e);
                }
                self.order.retain(|x| x != &key);
            }
        }
        self.configs.remove(id);
    }

    /// Disconnect every active driver in the pool.
    pub async fn close_all(&mut self) {
        for (key, driver_arc) in self.drivers.drain() {
            let mut driver = driver_arc.write().await;
            if let Err(e) = driver.disconnect().await {
                log::warn!("pool: error closing driver {}: {}", key, e);
            }
        }
        self.order.clear();
        self.configs.clear();
    }

}
