//! phenotype-cache-adapter
//!
//! Two-tier cache with L1 (LRU) and L2 (DashMap).

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;

/// Metrics hook for observability.
pub trait MetricsHook: Send + Sync + Debug {
    fn record_hit(&self, tier: &str);
    fn record_miss(&self, tier: &str);
}

/// Default no-op metrics hook.
#[derive(Debug, Default, Clone)]
pub struct NoOpMetrics;

impl MetricsHook for NoOpMetrics {
    fn record_hit(&self, _tier: &str) {}
    fn record_miss(&self, _tier: &str) {}
}

#[derive(Clone, Serialize, Deserialize)]
struct CacheEntry<V: Clone> {
    value: V,
}

impl<V: Clone> CacheEntry<V> {
    fn new(value: V) -> Self {
        Self { value }
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
    pub fn with_metrics(l1_capacity: usize, l2_capacity: usize, metrics: impl MetricsHook + 'static) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two_tier_cache_basic() {
        let cache: TwoTierCache<String, String> = TwoTierCache::new(10, 100);
        cache.put("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
    }

    #[test]
    fn test_two_tier_cache_l1_promotion() {
        let cache: TwoTierCache<String, i32> = TwoTierCache::new(2, 100);
        cache.put("a".to_string(), 1);
        cache.put("b".to_string(), 2);
        assert_eq!(cache.get(&"a".to_string()), Some(1));
        assert_eq!(cache.l1_len(), 1);
    }

    #[test]
    fn test_two_tier_cache_eviction() {
        let cache: TwoTierCache<String, String> = TwoTierCache::new(2, 100);
        cache.put("k1".to_string(), "v1".to_string());
        cache.put("k2".to_string(), "v2".to_string());
        cache.put("k3".to_string(), "v3".to_string());
        // k1 should be evicted from L1 but still in L2
        assert_eq!(cache.l1_len(), 2);
        assert!(cache.l2_len() >= 3);
    }

    #[test]
    fn test_two_tier_cache_remove() {
        let cache: TwoTierCache<String, String> = TwoTierCache::new(10, 100);
        cache.put("key".to_string(), "value".to_string());
        assert!(cache.get(&"key".to_string()).is_some());
        cache.remove(&"key".to_string());
        assert!(cache.get(&"key".to_string()).is_none());
    }

    #[test]
    fn test_two_tier_cache_clear() {
        let cache: TwoTierCache<String, String> = TwoTierCache::new(10, 100);
        cache.put("k1".to_string(), "v1".to_string());
        cache.put("k2".to_string(), "v2".to_string());
        cache.clear();
        assert_eq!(cache.l1_len(), 0);
        assert_eq!(cache.l2_len(), 0);
    }
}
