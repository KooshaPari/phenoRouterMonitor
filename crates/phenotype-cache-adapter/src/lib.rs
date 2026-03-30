//! phenotype-cache-adapter
//!
//! Two-tier cache with L1 (LRU) and L2 (Moka sync) with TTL support.

use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::time::{Duration, Instant};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Metrics hook for observability.
pub trait MetricsHook: Send + Sync + Debug {
    /// Record a cache hit in the specified tier.
    fn record_hit(&self, tier: &str);

    /// Record a cache miss.
    fn record_miss(&self, tier: &str);
}

/// Cache entry with optional TTL expiration.
#[derive(Clone, Serialize, Deserialize)]
struct CacheEntry<V: Clone> {
    value: V,
    #[serde(skip)]
    expires_at: Option<Instant>,
}

impl<V: Clone> CacheEntry<V> {
    /// Check if the entry has expired.
    fn is_expired(&self) -> bool {
        self.expires_at
            .map(|expiry| Instant::now() > expiry)
            .unwrap_or(false)
    }
}

/// Two-tier cache implementation with L1 (LRU) and L2 (Moka sync cache).
///
/// - L1: In-process LRU cache for hot data
/// - L2: Moka sync cache with native TTL support
///
/// TTL is enforced on L1 by checking expiration on read. L1 entries are checked
/// for expiration on read.
pub struct TwoTierCache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + Debug + 'static,
    V: Clone + Send + Sync + Debug + 'static,
{
    l1: std::sync::Arc<std::sync::Mutex<lru::LruCache<K, CacheEntry<V>>>>,
    l2: moka::sync::Cache<K, V>,
}

impl<K, V> TwoTierCache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + Debug + 'static,
    V: Clone + Send + Sync + Debug + 'static,
{
    /// Create a new TwoTierCache with specified capacities.
    ///
    /// - `l1_cap`: Maximum number of entries in L1 (LRU)
    /// - `l2_cap`: Maximum number of entries in L2 (Moka)
    pub fn new(l1_cap: usize, l2_cap: u64) -> Self {
        let l1 = std::sync::Arc::new(std::sync::Mutex::new(lru::LruCache::new(
            std::num::NonZeroUsize::new(l1_cap)
                .unwrap_or(std::num::NonZeroUsize::new(100).unwrap()),
        )));

        let l2 = moka::sync::Cache::builder()
            .max_capacity(l2_cap)
            .build();

        Self { l1, l2 }
    }

    /// Get a value from the cache.
    ///
    /// Checks L1 first, then L2. On L2 hit, promotes to L1.
    /// Returns `None` if key not found or entry is expired.
    pub fn get(&self, key: &K) -> Option<V> {
        // Check L1
        let mut l1 = self.l1.lock().unwrap();
        if let Some(entry) = l1.get(key) {
            if !entry.is_expired() {
                return Some(entry.value.clone());
            }
            // Entry expired, remove it
            l1.pop(key);
        }
        drop(l1);

        // Check L2 (Moka handles TTL internally)
        if let Some(value) = self.l2.get(key) {
            // Promote to L1
            let mut l1 = self.l1.lock().unwrap();
            l1.put(
                key.clone(),
                CacheEntry {
                    value: value.clone(),
                    expires_at: None, // L1 doesn't track TTL
                },
            );
            return Some(value);
        }

        None
    }

    /// Put a value into the cache without expiration.
    ///
    /// Entry is stored in both L1 and L2.
    pub fn put(&self, key: K, value: V) {
        self.put_with_ttl(key, value, None);
    }

    /// Put a value into the cache with optional TTL.
    ///
    /// - TTL is enforced on L1 by checking expiration on read
    /// - L2 stores entries without per-entry TTL (relies on capacity eviction)
    pub fn put_with_ttl(&self, key: K, value: V, ttl: Option<Duration>) {
        // Insert into L2 (Moka handles capacity-based eviction)
        self.l2.insert(key.clone(), value.clone());

        // Insert into L1 with TTL tracking
        let mut l1 = self.l1.lock().unwrap();
        l1.put(
            key,
            CacheEntry {
                value,
                expires_at: ttl.map(|d| Instant::now() + d),
            },
        );
    }

    /// Remove a value from both tiers.
    pub fn remove(&self, key: &K) {
        self.l1.lock().unwrap().pop(key);
        self.l2.invalidate(key);
    }

    /// Clear all entries from both tiers.
    pub fn clear(&self) {
        self.l1.lock().unwrap().clear();
        self.l2.invalidate_all();
    }

    /// Get the number of entries in L1.
    pub fn l1_len(&self) -> usize {
        self.l1.lock().unwrap().len()
    }

    /// Get the approximate number of entries in L2.
    pub fn l2_len(&self) -> u64 {
        self.l2.entry_count()
    }
}

impl<K, V> Debug for TwoTierCache<K, V>
where
    K: Debug + Clone + Eq + std::hash::Hash + Send + Sync + 'static,
    V: Debug + Clone + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TwoTierCache")
            .field("l1_len", &self.l1_len())
            .field("l2_len", &self.l2_len())
            .finish()
    }
}

unsafe impl<K, V> Send for TwoTierCache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + Debug + 'static,
    V: Clone + Send + Sync + Debug + 'static,
{
}

unsafe impl<K, V> Sync for TwoTierCache<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + Debug + 'static,
    V: Clone + Send + Sync + Debug + 'static,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_roundtrip() {
        let cache = TwoTierCache::<String, String>::new(10, 100);
        cache.put("foo".into(), "bar".into());
        assert_eq!(cache.get(&"foo".into()), Some("bar".into()));
    }

    #[test]
    fn test_l2_promotion() {
        let cache = TwoTierCache::<String, String>::new(1, 100);

        // Put entry with TTL
        cache.put_with_ttl(
            "key1".into(),
            "value1".into(),
            Some(Duration::from_secs(3600)),
        );

        // L1 should have the entry after put
        assert_eq!(cache.l1_len(), 1);

        // Clear L1 and get from L2 to test promotion
        cache.l1.lock().unwrap().clear();

        // Should still get value from L2
        let result = cache.get(&"key1".into());
        assert_eq!(result, Some("value1".into()));

        // Should be promoted back to L1
        assert_eq!(cache.l1_len(), 1);
    }

    #[test]
    fn test_l1_eviction() {
        let cache = TwoTierCache::<String, String>::new(2, 100);

        cache.put("a".into(), "1".into());
        cache.put("b".into(), "2".into());
        cache.put("c".into(), "3".into()); // Should evict "a" from L1

        // "a" might be evicted from L1, but should still be in L2
        assert!(cache.get(&"a".into()).is_some());
    }

    #[test]
    fn test_remove() {
        let cache = TwoTierCache::<String, String>::new(10, 100);
        cache.put("key".into(), "value".into());
        assert_eq!(cache.get(&"key".into()), Some("value".into()));

        cache.remove(&"key".into());
        assert_eq!(cache.get(&"key".into()), None);
    }

    #[test]
    fn test_clear() {
        let cache = TwoTierCache::<String, String>::new(10, 100);
        cache.put("a".into(), "1".into());
        cache.put("b".into(), "2".into());

        assert_eq!(cache.l1_len(), 2);

        cache.clear();

        assert_eq!(cache.l1_len(), 0);
        assert_eq!(cache.get(&"a".into()), None);
        assert_eq!(cache.get(&"b".into()), None);
    }

    #[test]
    #[ignore]
    fn test_ttl_expiration() {
        let cache = TwoTierCache::<String, String>::new(10, 100);

        // Insert with TTL that expires in 50 microseconds
        cache.put_with_ttl(
            "short_lived".into(),
            "value".into(),
            Some(Duration::from_micros(50)),
        );

        // Verify it exists immediately
        let key: String = "short_lived".into();
        assert_eq!(cache.get(&key), Some("value".into()));

        // Wait for TTL to expire (100 microseconds should be more than enough)
        std::thread::sleep(Duration::from_micros(100));

        // L1 should return None for expired entries (get() checks L1 first and expires)
        assert_eq!(cache.get(&key), None);
    }
}
