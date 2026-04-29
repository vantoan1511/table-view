use crate::drivers::DatabaseDriver;
use std::collections::{HashMap, VecDeque};

const MAX_SIZE: usize = 5;

/// Entry in the connection pool.
struct Entry {
    id: String,
    driver: Box<dyn DatabaseDriver>,
}

/// A thread-safe LRU connection pool for database drivers.
/// Maintains up to MAX_SIZE concurrent live connections keyed by connectionId.
/// When the pool is full, the least-recently-used connection is closed and evicted.
pub struct Pool {
    cache: HashMap<String, usize>, // id -> index in order
    order: VecDeque<String>,       // front = most recently used
    drivers: HashMap<String, Box<dyn DatabaseDriver>>,
}

impl Pool {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            order: VecDeque::new(),
            drivers: HashMap::new(),
        }
    }

    /// Get a reference to the driver for the given connectionId, updating its LRU position.
    /// Returns None if not found.
    pub fn get(&mut self, id: &str) -> Option<&dyn DatabaseDriver> {
        if self.drivers.contains_key(id) {
            // Move to front (most recently used)
            self.order.retain(|x| x != id);
            self.order.push_front(id.to_string());
            self.drivers.get(id).map(|d| d.as_ref())
        } else {
            None
        }
    }

    /// Get a mutable reference to the driver for the given connectionId.
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Box<dyn DatabaseDriver>> {
        if self.drivers.contains_key(id) {
            self.order.retain(|x| x != id);
            self.order.push_front(id.to_string());
            self.drivers.get_mut(id)
        } else {
            None
        }
    }

    /// Register a driver under the given connectionId.
    /// If the pool is already at MAX_SIZE, the LRU entry is evicted first.
    /// If an entry for id already exists it is replaced (old driver is disconnected).
    pub async fn put(&mut self, id: String, driver: Box<dyn DatabaseDriver>) {
        // Replace existing
        if let Some(mut old) = self.drivers.remove(&id) {
            if let Err(e) = old.disconnect().await {
                log::warn!("pool: error disconnecting replaced driver {}: {}", id, e);
            }
            self.order.retain(|x| x != &id);
        }

        // Evict LRU if at capacity
        if self.drivers.len() >= MAX_SIZE {
            if let Some(oldest_id) = self.order.pop_back() {
                log::info!("pool: evicting LRU connection {} (pool full)", oldest_id);
                if let Some(mut evicted) = self.drivers.remove(&oldest_id) {
                    if let Err(e) = evicted.disconnect().await {
                        log::warn!(
                            "pool: error disconnecting evicted driver {}: {}",
                            oldest_id,
                            e
                        );
                    }
                }
            }
        }

        self.order.push_front(id.clone());
        self.drivers.insert(id, driver);
    }

    /// Explicitly close and remove a connection from the pool.
    pub async fn remove(&mut self, id: &str) {
        if let Some(mut driver) = self.drivers.remove(id) {
            if let Err(e) = driver.disconnect().await {
                log::warn!("pool: error disconnecting removed driver {}: {}", id, e);
            }
            self.order.retain(|x| x != id);
        }
    }

    /// Disconnect every active driver in the pool.
    pub async fn close_all(&mut self) {
        for (id, mut driver) in self.drivers.drain() {
            if let Err(e) = driver.disconnect().await {
                log::warn!("pool: error closing driver {}: {}", id, e);
            }
        }
        self.order.clear();
    }
}
