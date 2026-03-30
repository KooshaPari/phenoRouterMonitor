//! phenotype-cache-adapter
//!
//! Two-tier cache with L1 (LRU) and L2 (DashMap/Moka).

use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use lru::LruCache;
use moka::sync::Cache as MokaCache;
use phenotype_error_core::ErrorKind;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex, RwLock};

pub type Result<T> = std::result::Result<T, ErrorKind>;

/// Metrics hook for observability.
pub trait MetricsHook: Send + Sync + Debug {
    fn record_hit(&self, tier: &str);
    fn record_miss(&self, tier: &str);
}

#[derive(Clone, Serialize, Deserialize)]
struct CacheEntry<T> {
    value: T,
    expiry: Option<DateTime<Utc>>,
}

impl<T> CacheEntry<T> {
    fn is_expired(&self) -> bool {
        if let Some(expiry) = self.expiry {
            return expiry < Utc::now();
        }
        false
    }
}

/// Two-tier cache implementation.
pub struct TwoTierCache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + Debug + 'static,
    V: Clone + Serialize + DeserializeOwned + Send + Sync + Debug + 'static,
{
    l1: Arc<Mutex<LruCache<K, CacheEntry<V>>>>,
    l2: MokaCache<K, CacheEntry<V>>,
    metrics: Option<Arc<dyn MetricsHook>>,
}

impl<K, V> TwoTierCache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + Debug + 'static,
    V: Clone + Serialize + DeserializeOwned + Send + Sync + Debug + 'static,
{
    pub fn new(l1_cap: usize, l2_cap: u64) -> Self {
        Self {
            l1: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(l1_cap).unwrap_or(NonZeroUsize::new(100).unwrap())))),
            l2: MokaCache::builder()
                .max_capacity(l2_cap)
                .build(),
            metrics: None,
        }
    }

    pub fn with_metrics(mut self, metrics: Arc<dyn MetricsHook>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    pub fn get(&self, key: &K) -> Option<V> {
        // L1 Check
        {
            let mut l1 = self.l1.lock().unwrap();
            if let Some(entry) = l1.get(key) {
                if !entry.is_expired() {
                    if let Some(ref m) = self.metrics { m.record_hit("L1"); }
                    return Some(entry.value.clone());
                } else {
                    l1.pop(key);
                }
            }
        }
        if let Some(ref m) = self.metrics { m.record_miss("L1"); }

        // L2 Check
        if let Some(entry) = self.l2.get(key) {
            if !entry.is_expired() {
                if let Some(ref m) = self.metrics { m.record_hit("L2"); }
                // Backfill L1
                let mut l1 = self.l1.lock().unwrap();
                l1.put(key.clone(), entry.value.clone().into_entry(entry.expiry));
                return Some(entry.value);
            } else {
                self.l2.invalidate(key);
            }
        }
        if let Some(ref m) = self.metrics { m.record_miss("L2"); }

        None
    }

    pub fn insert(&self, key: K, value: V, ttl: Option<Duration>) {
        let expiry = ttl.map(|d| Utc::now() + d);
        let entry = CacheEntry { value: value.clone(), expiry };
        
        // Update both tiers
        let mut l1 = self.l1.lock().unwrap();
        l1.put(key.clone(), entry.clone());
        self.l2.insert(key, entry);
    }

    pub fn remove(&self, key: &K) {
        let mut l1 = self.l1.lock().unwrap();
        l1.pop(key);
        self.l2.invalidate(key);
    }
}

trait ToEntry<V> {
    fn into_entry(self, expiry: Option<DateTime<Utc>>) -> CacheEntry<V>;
}

impl<V> ToEntry<V> for V {
    fn into_entry(self, expiry: Option<DateTime<Utc>>) -> CacheEntry<V> {
        CacheEntry { value: self, expiry }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_roundtrip() {
        let cache = TwoTierCache::<String, String>::new(10, 100);
        cache.insert("foo".into(), "bar".into(), None);
        assert_eq!(cache.get(&"foo".into()), Some("bar".into()));
    }

    #[test]
    fn test_cache_expiry() {
        let cache = TwoTierCache::<String, String>::new(10, 100);
        cache.insert("foo".into(), "bar".into(), Some(Duration::milliseconds(-1)));
        assert_eq!(cache.get(&"foo".into()), None);
    }
}
