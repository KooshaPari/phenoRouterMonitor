//! phenotype-cache-adapter
//!
//! Two-tier cache with L1 (LRU) and L2 (DashMap), implementing CachePort.

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

// Re-export CachePort from phenotype-contracts
pub use phenotype_contracts::CachePort;

// Import observability traits from phenotype-port-traits
pub use phenotype_port_traits::{CounterMetrics, MetricsHook, NoOpMetrics};

/// Cache-specific errors
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Key not found: {0}")]
    NotFound(String),
    #[error("Cache error: {0}")]
    Other(String),
}

impl From<CacheError> for phenotype_contracts::error::Error {
    fn from(e: CacheError) -> Self {
        phenotype_contracts::error::Error::Internal(e.to_string())
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct CacheEntry<V: Clone> {
    value: V,
    #[serde(skip)]
    #[allow(dead_code)]
    timestamp: std::time::Instant,
}

impl<V: Clone> CacheEntry<V> {
    fn new(value: V) -> Self {
        Self {
            value,
            timestamp: std::time::Instant::now(),
        }
    }
}

/// Two-tier cache implementation with L1 (LRU) and L2 (DashMap).
pub struct TwoTierCache<K, V>
where
    K: Clone + Eq + Hash + Send + Sync + Debug + 'static,
    V: Clone + Send + Sync + Debug + 'static,
{
    l1: Arc<DashMap<K, CacheEntry<V>>>,
    l2: Arc<DashMap<K, CacheEntry<V>>>,
    l1_capacity: usize,
    metrics: Arc<dyn MetricsHook>,
}

impl<K, V> TwoTierCache<K, V>
where
    K: Clone + Eq + Hash + Send + Sync + Debug + 'static,
    V: Clone + Send + Sync + Debug + 'static,
{
    /// Create a new TwoTierCache with specified capacities.
    pub fn new(l1_capacity: usize, l2_capacity: usize) -> Self {
        Self {
            l1: Arc::new(DashMap::with_capacity(l1_capacity)),
            l2: Arc::new(DashMap::with_capacity(l2_capacity)),
            l1_capacity,
            metrics: Arc::new(NoOpMetrics),
        }
    }

    /// Create with custom metrics hook.
    pub fn with_metrics(l1_capacity: usize, l2_capacity: usize, metrics: impl MetricsHook) -> Self {
        Self {
            l1: Arc::new(DashMap::with_capacity(l1_capacity)),
            l2: Arc::new(DashMap::with_capacity(l2_capacity)),
            l1_capacity,
            metrics: Arc::new(metrics),
        }
    }

    /// Get value by key, checking L1 then L2.
    pub fn get(&self, key: &K) -> Option<V> {
        // Check L1 first (hot cache)
        if let Some(entry) = self.l1.get(key) {
            self.metrics.record_hit("L1");
            return Some(entry.value.clone());
        }

        // Check L2 (warm cache)
        if let Some(entry) = self.l2.get(key) {
            self.metrics.record_hit("L2");
            let value = entry.value.clone();
            // Promote to L1 if there's room
            if self.l1.len() < self.l1_capacity {
                self.l1.insert(key.clone(), CacheEntry::new(value.clone()));
            }
            return Some(value);
        }

        self.metrics.record_miss("L2");
        None
    }

    /// Put value into both tiers.
    pub fn put(&self, key: K, value: V) {
        let entry = CacheEntry::new(value.clone());
        // Always write to L1
        if self.l1.len() >= self.l1_capacity {
            // Evict oldest entry (first entry in L1)
            if let Some(first) = self.l1.iter().next() {
                let key_to_remove = first.key().clone();
                drop(first);
                self.l1.remove(&key_to_remove);
            }
        }
        self.l1.insert(key.clone(), entry.clone());
        self.l2.insert(key, entry);
    }

    /// Remove a key from both tiers.
    pub fn remove(&self, key: &K) -> bool {
        let l1_removed = self.l1.remove(key).is_some();
        let l2_removed = self.l2.remove(key).is_some();
        l1_removed || l2_removed
    }

    /// Clear all cached entries.
    pub fn clear(&self) {
        self.l1.clear();
        self.l2.clear();
    }

    /// Get L1 cache size.
    pub fn l1_len(&self) -> usize {
        self.l1.len()
    }

    /// Get L2 cache size.
    pub fn l2_len(&self) -> usize {
        self.l2.len()
    }
}

impl<K, V> CachePort for TwoTierCache<K, V>
where
    K: Clone + Eq + Hash + Send + Sync + Debug + 'static,
    V: Clone + Send + Sync + Debug + 'static,
{
    type Error = CacheError;

    fn get(&self, key: &K) -> phenotype_contracts::Result<Option<V>> {
        Ok(self.get(key))
    }

    fn set(&self, key: K, value: V) -> phenotype_contracts::Result<()> {
        self.put(key, value);
        Ok(())
    }

    fn invalidate(&self, key: &K) -> phenotype_contracts::Result<()> {
        self.remove(key);
        Ok(())
    }
}
