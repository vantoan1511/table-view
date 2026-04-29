use crate::drivers::DatabaseDriver;
use std::collections::{HashMap, VecDeque};
use tokio::sync::Mutex;
use std::sync::Arc;

const MAX_SIZE: usize = 5;

/// A thread-safe LRU connection pool for database drivers.
/// Maintains up to MAX_SIZE concurrent live connections keyed by connectionId.
/// When the pool is full, the least-recently-used connection is closed and evicted.
pub struct Pool {
    order: VecDeque<String>,       // front = most recently used
    drivers: HashMap<String, Arc<Mutex<Box<dyn DatabaseDriver>>>>,
}

impl Pool {
    pub fn new() -> Self {
        Self {
            order: VecDeque::new(),
            drivers: HashMap::new(),
        }
    }

    /// Get an Arc to the driver for the given connectionId.
    pub fn get(&mut self, id: &str) -> Option<Arc<Mutex<Box<dyn DatabaseDriver>>>> {
        if self.drivers.contains_key(id) {
            self.order.retain(|x| x != id);
            self.order.push_front(id.to_string());
            self.drivers.get(id).cloned()
        } else {
            None
        }
    }

    /// Register a driver under the given connectionId.
    /// If the pool is already at MAX_SIZE, the LRU entry is evicted first.
    /// If an entry for id already exists it is replaced (old driver is disconnected).
    pub async fn put(&mut self, id: String, driver: Box<dyn DatabaseDriver>) {
        // Replace existing
        if let Some(old) = self.drivers.remove(&id) {
            let mut d = old.lock().await;
            if let Err(e) = d.disconnect().await {
                log::warn!("pool: error disconnecting replaced driver {}: {}", id, e);
            }
            self.order.retain(|x| x != &id);
        }

        // Evict LRU if at capacity
        if self.drivers.len() >= MAX_SIZE {
            if let Some(oldest_id) = self.order.pop_back() {
                log::info!("pool: evicting LRU connection {} (pool full)", oldest_id);
                if let Some(evicted_arc) = self.drivers.remove(&oldest_id) {
                    let mut evicted = evicted_arc.lock().await;
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
        self.drivers.insert(id, Arc::new(Mutex::new(driver)));
    }

    /// Explicitly close and remove a connection from the pool.
    #[allow(dead_code)]
    pub async fn remove(&mut self, id: &str) {
        if let Some(driver_arc) = self.drivers.remove(id) {
            let mut driver = driver_arc.lock().await;
            if let Err(e) = driver.disconnect().await {
                log::warn!("pool: error disconnecting removed driver {}: {}", id, e);
            }
            self.order.retain(|x| x != id);
        }
    }

    /// Disconnect every active driver in the pool.
    pub async fn close_all(&mut self) {
        for (id, driver_arc) in self.drivers.drain() {
            let mut driver = driver_arc.lock().await;
            if let Err(e) = driver.disconnect().await {
                log::warn!("pool: error closing driver {}: {}", id, e);
            }
        }
        self.order.clear();
    }
}
