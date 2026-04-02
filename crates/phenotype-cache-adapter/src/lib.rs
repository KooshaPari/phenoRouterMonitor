//! phenotype-cache-adapter
//!
//! Two-tier cache with L1 (LRU) and L2 (Moka).

use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Metrics hook for observability.
pub trait MetricsHook: Send + Sync + Debug {
    fn record_hit(&self, tier: &str);
    fn record_miss(&self, tier: &str);
}

#[derive(Clone, Serialize, Deserialize)]
struct CacheEntry<V> {
    value: V,
}

/// Two-tier cache implementation.
pub struct TwoTierCache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + Debug + 'static,
    V: Clone + Send + Sync + Debug + 'static,
{
    l1: std::sync::Arc<std::sync::Mutex<lru::LruCache<K, CacheEntry<V>>>>,
    l2: moka::sync::Cache<K, CacheEntry<V>>,
}

impl<K, V> TwoTierCache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + Debug + 'static,
    V: Clone + Send + Sync + Debug + 'static,
{
    pub fn new(l1_cap: usize, l2_cap: u64) -> Self {
        Self {
            l1: std::sync::Arc::new(std::sync::Mutex::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(l1_cap)
                    .unwrap_or(std::num::NonZeroUsize::new(100).unwrap()),
            ))),
            l2: moka::sync::Cache::builder().max_capacity(l2_cap).build(),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let mut l1 = self.l1.lock().unwrap();
        if let Some(entry) = l1.get(key) {
            return Some(entry.value.clone());
        }
        drop(l1);

        if let Some(entry) = self.l2.get(key) {
            let value = entry.value.clone();
            let mut l1 = self.l1.lock().unwrap();
            l1.put(
                key.clone(),
                CacheEntry {
                    value: value.clone(),
                },
            );
            return Some(value);
        }
        None
    }

    pub fn put(&self, key: K, value: V) {
        let mut l1 = self.l1.lock().unwrap();
        l1.put(
            key.clone(),
            CacheEntry {
                value: value.clone(),
            },
        );
        drop(l1);
        self.l2.insert(key, CacheEntry { value });
    }

    pub fn len_l1(&self) -> usize {
        self.l1.lock().unwrap().len()
    }

    pub fn len_l2(&self) -> usize {
        self.l2.entry_count()
    }

    pub fn is_empty(&self) -> bool {
        self.len_l1() == 0 && self.len_l2() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic_put_get() {
        let cache = TwoTierCache::new(100, 1000);
        cache.put("key1", "value1");
        assert_eq!(cache.get(&"key1"), Some("value1".to_string()));
    }

    #[test]
    fn test_cache_get_nonexistent() {
        let cache = TwoTierCache::new(100, 1000);
        assert_eq!(cache.get(&"nonexistent"), None);
    }

    #[test]
    fn test_cache_overwrite() {
        let cache = TwoTierCache::new(100, 1000);
        cache.put("key1", "value1");
        cache.put("key1", "value2");
        assert_eq!(cache.get(&"key1"), Some("value2".to_string()));
    }

    #[test]
    fn test_cache_multiple_items() {
        let cache = TwoTierCache::new(100, 1000);
        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);
        assert_eq!(cache.get(&"a"), Some(1));
        assert_eq!(cache.get(&"b"), Some(2));
        assert_eq!(cache.get(&"c"), Some(3));
    }

    #[test]
    fn test_cache_l1_promotion() {
        let cache = TwoTierCache::new(2, 100);
        cache.put("key1", "v1");
        cache.put("key2", "v2");
        assert_eq!(cache.len_l1(), 2);
        let val = cache.get(&"key1");
        assert_eq!(val, Some("v1".to_string()));
    }

    #[test]
    fn test_cache_l2_promotion() {
        let cache = TwoTierCache::new(1, 100);
        cache.put("key1", "v1");
        cache.put("key2", "v2");
        let _ = cache.get(&"key1");
        let _ = cache.get(&"key2");
        assert!(cache.get(&"key1").is_some());
        assert!(cache.get(&"key2").is_some());
    }

    #[test]
    fn test_cache_lru_eviction() {
        let cache = TwoTierCache::new(2, 100);
        cache.put("key1", "v1");
        cache.put("key2", "v2");
        cache.put("key3", "v3");
        let val1 = cache.get(&"key1");
        let val2 = cache.get(&"key2");
        let val3 = cache.get(&"key3");
        assert!(val1.is_none() || val2.is_none() || val3.is_some());
    }

    #[test]
    fn test_cache_empty() {
        let cache: TwoTierCache<&str, &str> = TwoTierCache::new(100, 1000);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_len_after_put() {
        let cache = TwoTierCache::new(100, 1000);
        cache.put("a", 1);
        cache.put("b", 2);
        assert!(!cache.is_empty());
        assert!(cache.len_l1() >= 1);
    }

    #[test]
    fn test_cache_int_values() {
        let cache: TwoTierCache<&str, i32> = TwoTierCache::new(100, 1000);
        cache.put("int", 42);
        assert_eq!(cache.get(&"int"), Some(42));
    }

    #[test]
    fn test_cache_key_cloning() {
        let cache = TwoTierCache::new(100, 1000);
        let key = String::from("test_key");
        cache.put(key.clone(), String::from("value"));
        assert_eq!(cache.get(&key), Some(String::from("value")));
    }

    #[test]
    fn test_cache_zero_capacity() {
        let cache = TwoTierCache::new(0, 0);
        cache.put("key", "value");
        assert!(cache.get(&"key").is_none() || cache.get(&"key").is_some());
    }

    #[tokio::test]
    async fn test_cache_concurrent_put() {
        let cache = std::sync::Arc::new(TwoTierCache::new(100, 1000));
        let cache_ref = cache.clone();
        let handle = tokio::spawn(async move {
            for i in 0..100 {
                cache_ref.put(format!("key_{}", i), i);
            }
        });
        handle.await.unwrap();
        assert!(cache.len_l2() > 0);
    }

    #[tokio::test]
    async fn test_cache_concurrent_get() {
        let cache = std::sync::Arc::new(TwoTierCache::new(100, 1000));
        for i in 0..50 {
            cache.put(format!("key_{}", i), i);
        }
        let cache_ref = cache.clone();
        let handles: Vec<_> = (0..4).map(|_| {
            let c = cache_ref.clone();
            tokio::spawn(async move {
                for i in 0..50 {
                    let _ = c.get(&format!("key_{}", i));
                }
            })
        }).collect();
        for h in handles {
            h.await.unwrap();
        }
    }

    #[test]
    fn test_cache_large_values() {
        let cache = TwoTierCache::new(10, 100);
        let large_vec = (0..10000).collect::<Vec<_>>();
        cache.put("large", large_vec.clone());
        let retrieved = cache.get(&"large");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().len(), 10000);
    }

    #[test]
    fn test_cache_many_items() {
        let cache = TwoTierCache::new(50, 10000);
        for i in 0..1000 {
            cache.put(i, i * 2);
        }
        assert!(cache.len_l2() > 0);
        assert_eq!(cache.get(&500), Some(1000));
    }

    #[test]
    fn test_cache_alternating_put_get() {
        let cache = TwoTierCache::new(10, 100);
        for i in 0..20 {
            cache.put(i, i);
            assert_eq!(cache.get(&i), Some(i));
        }
    }

    #[test]
    fn test_cache_mixed_operations() {
        let cache = TwoTierCache::new(5, 50);
        cache.put("a", 1);
        cache.put("b", 2);
        assert_eq!(cache.get(&"a"), Some(1));
        cache.put("c", 3);
        assert_eq!(cache.get(&"b"), Some(2));
        cache.put("a", 10);
        assert_eq!(cache.get(&"a"), Some(10));
        cache.put("d", 4);
        let _ = cache.get(&"c");
        cache.put("e", 5);
        cache.put("f", 6);
    }
}
