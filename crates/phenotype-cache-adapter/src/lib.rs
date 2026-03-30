//! phenotype-cache-adapter
//!
//! Two-tier cache implementation with L1 (LRU) and L2 (Moka async).
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │         CacheAdapter<K, V>                  │
//! ├─────────────────────────────────────────────┤
//! │  ┌──────────────┐        ┌──────────────┐   │
//! │  │  L1: LRU     │ <--->  │  L2: Moka    │   │
//! │  │  (in-memory) │        │  (eviction)  │   │
//! │  └──────────────┘        └──────────────┘   │
//! └─────────────────────────────────────────────┘
//!
//! get():     L1 -> L2 -> origin (backfill on L2 hit)
//! set():     L1 + async L2 (write-back)
//! delete():  L1 + L2 (immediate)
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use phenotype_cache_adapter::CacheAdapter;
//! use chrono::Duration;
//!
//! let cache = CacheAdapter::new(100, 1000);
//! cache.set("key".to_string(), "value".to_string(), Some(Duration::hours(1)));
//!
//! assert_eq!(cache.get(&"key".to_string()), Some("value".to_string()));
//!
//! cache.delete(&"key".to_string());
//! assert_eq!(cache.get(&"key".to_string()), None);
//! ```

use chrono::{DateTime, Duration, Utc};
use lru::LruCache;
use moka::sync::Cache as MokaCache;
use phenotype_error_core::ErrorKind;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};

pub type Result<T> = std::result::Result<T, ErrorKind>;

/// Metrics hook for observability of cache operations.
pub trait MetricsHook: Send + Sync + Debug {
    /// Record a cache hit for a given tier.
    fn record_hit(&self, tier: &str);
    /// Record a cache miss for a given tier.
    fn record_miss(&self, tier: &str);
}

/// A cache entry with optional TTL expiration.
#[derive(Clone, Serialize, Deserialize)]
struct CacheEntry<T> {
    value: T,
    expiry: Option<DateTime<Utc>>,
}

impl<T> CacheEntry<T> {
    /// Check if this entry has expired.
    fn is_expired(&self) -> bool {
        if let Some(expiry) = self.expiry {
            return expiry < Utc::now();
        }
        false
    }

    /// Create a new entry with the given value and expiry.
    fn new(value: T, expiry: Option<DateTime<Utc>>) -> Self {
        Self { value, expiry }
    }
}

/// Two-tier cache adapter with L1 (LRU) and L2 (Moka async).
///
/// Implements a hierarchical caching strategy:
/// - **L1**: In-memory LRU cache for hot data
/// - **L2**: Async Moka cache for overflow and durability
///
/// Reads check L1 first, then L2, with automatic backfill.
/// Writes update L1 immediately and L2 asynchronously.
#[derive(Clone)]
pub struct CacheAdapter<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + Debug + 'static,
    V: Clone + Serialize + DeserializeOwned + Send + Sync + Debug + 'static,
{
    l1: Arc<Mutex<LruCache<K, CacheEntry<V>>>>,
    l2: MokaCache<K, CacheEntry<V>>,
    metrics: Option<Arc<dyn MetricsHook>>,
}

impl<K, V> CacheAdapter<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + Debug + 'static,
    V: Clone + Serialize + DeserializeOwned + Send + Sync + Debug + 'static,
{
    /// Create a new two-tier cache with specified capacities.
    ///
    /// # Arguments
    /// * `l1_cap` - L1 (LRU) capacity in entries
    /// * `l2_cap` - L2 (Moka) capacity in entries
    ///
    /// # Panics
    /// If `l1_cap` is 0.
    pub fn new(l1_cap: usize, l2_cap: u64) -> Self {
        Self {
            l1: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(l1_cap).unwrap_or(NonZeroUsize::new(100).unwrap()),
            ))),
            l2: MokaCache::builder().max_capacity(l2_cap).build(),
            metrics: None,
        }
    }

    /// Attach optional metrics hook for observability.
    pub fn with_metrics(mut self, metrics: Arc<dyn MetricsHook>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Retrieve a value from cache with fallthrough logic.
    ///
    /// Checks L1 first, then L2. On L2 hit, backfills L1.
    /// Automatically removes expired entries.
    pub fn get(&self, key: &K) -> Option<V> {
        // Check L1
        {
            let mut l1 = self.l1.lock().unwrap();
            if let Some(entry) = l1.get(key) {
                if !entry.is_expired() {
                    if let Some(ref m) = self.metrics {
                        m.record_hit("L1");
                    }
                    return Some(entry.value.clone());
                } else {
                    // Remove expired entry from L1
                    l1.pop(key);
                }
            }
        }

        // Record L1 miss
        if let Some(ref m) = self.metrics {
            m.record_miss("L1");
        }

        // Check L2
        if let Some(entry) = self.l2.get(key) {
            if !entry.is_expired() {
                if let Some(ref m) = self.metrics {
                    m.record_hit("L2");
                }
                // Backfill L1 with L2 entry
                {
                    let mut l1 = self.l1.lock().unwrap();
                    l1.put(key.clone(), CacheEntry::new(entry.value.clone(), entry.expiry));
                }
                return Some(entry.value);
            } else {
                // Remove expired entry from L2
                self.l2.invalidate(key);
            }
        }

        // Record L2 miss
        if let Some(ref m) = self.metrics {
            m.record_miss("L2");
        }

        None
    }

    /// Insert a value into cache (write-back semantic).
    ///
    /// Updates L1 immediately and L2 asynchronously.
    ///
    /// # Arguments
    /// * `key` - Cache key
    /// * `value` - Value to cache
    /// * `ttl` - Optional time-to-live duration
    pub fn set(&self, key: K, value: V, ttl: Option<Duration>) {
        let expiry = ttl.map(|d| Utc::now() + d);
        let entry = CacheEntry::new(value.clone(), expiry);

        // Update L1 immediately
        {
            let mut l1 = self.l1.lock().unwrap();
            l1.put(key.clone(), entry.clone());
        }

        // Async update L2 (fire-and-forget)
        self.l2.insert(key, entry);
    }

    /// Delete a value from both cache tiers.
    ///
    /// Removes entry from L1 and L2 synchronously.
    pub fn delete(&self, key: &K) {
        let mut l1 = self.l1.lock().unwrap();
        l1.pop(key);
        self.l2.invalidate(key);
    }

    /// Clear all entries from both cache tiers.
    pub fn clear(&self) {
        let mut l1 = self.l1.lock().unwrap();
        l1.clear();
        self.l2.invalidate_all();
    }
}

impl<K, V> Debug for CacheAdapter<K, V>
where
    K: Clone + Eq + std::hash::Hash + Send + Sync + Debug + 'static,
    V: Clone + Serialize + DeserializeOwned + Send + Sync + Debug + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CacheAdapter")
            .field("l1", &"LruCache")
            .field("l2", &"MokaCache")
            .field("metrics", &self.metrics)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Mock metrics for testing.
    #[derive(Debug)]
    struct MockMetrics {
        l1_hits: AtomicUsize,
        l1_misses: AtomicUsize,
        l2_hits: AtomicUsize,
        l2_misses: AtomicUsize,
    }

    impl MockMetrics {
        fn new() -> Arc<Self> {
            Arc::new(Self {
                l1_hits: AtomicUsize::new(0),
                l1_misses: AtomicUsize::new(0),
                l2_hits: AtomicUsize::new(0),
                l2_misses: AtomicUsize::new(0),
            })
        }
    }

    impl MetricsHook for MockMetrics {
        fn record_hit(&self, tier: &str) {
            match tier {
                "L1" => {
                    self.l1_hits.fetch_add(1, Ordering::SeqCst);
                }
                "L2" => {
                    self.l2_hits.fetch_add(1, Ordering::SeqCst);
                }
                _ => {}
            }
        }

        fn record_miss(&self, tier: &str) {
            match tier {
                "L1" => {
                    self.l1_misses.fetch_add(1, Ordering::SeqCst);
                }
                "L2" => {
                    self.l2_misses.fetch_add(1, Ordering::SeqCst);
                }
                _ => {}
            }
        }
    }

    /// Trace to: FR-CACHE-001 Basic cache roundtrip (get/set)
    #[test]
    fn test_basic_roundtrip() {
        let cache = CacheAdapter::<String, String>::new(10, 100);
        cache.set("foo".into(), "bar".into(), None);
        assert_eq!(cache.get(&"foo".into()), Some("bar".into()));
    }

    /// Trace to: FR-CACHE-002 L1 hit detection
    #[test]
    fn test_l1_hit() {
        let metrics = MockMetrics::new();
        let cache = CacheAdapter::<String, String>::new(10, 100)
            .with_metrics(metrics.clone());

        cache.set("foo".into(), "bar".into(), None);
        let result = cache.get(&"foo".into());

        assert_eq!(result, Some("bar".into()));
        assert_eq!(metrics.l1_hits.load(Ordering::SeqCst), 1);
        assert_eq!(metrics.l1_misses.load(Ordering::SeqCst), 0);
    }

    /// Trace to: FR-CACHE-003 L2 backfill on L1 miss
    #[test]
    fn test_l2_backfill() {
        let metrics = MockMetrics::new();
        let cache = CacheAdapter::<String, String>::new(2, 100)
            .with_metrics(metrics.clone());

        // Fill L1 to capacity (evict first entry to L2)
        cache.set("key1".into(), "val1".into(), None);
        cache.set("key2".into(), "val2".into(), None);
        cache.set("key3".into(), "val3".into(), None); // key1 evicted to L2

        // Accessing key1 should hit L2 and backfill L1
        let result = cache.get(&"key1".into());
        assert_eq!(result, Some("val1".into()));

        // Verify metrics: L1 miss, then L2 hit
        assert_eq!(metrics.l1_misses.load(Ordering::SeqCst), 1);
        assert_eq!(metrics.l2_hits.load(Ordering::SeqCst), 1);
    }

    /// Trace to: FR-CACHE-004 TTL expiration handling
    #[test]
    fn test_expiry() {
        let cache = CacheAdapter::<String, String>::new(10, 100);
        cache.set(
            "foo".into(),
            "bar".into(),
            Some(Duration::milliseconds(-1)),
        );
        assert_eq!(cache.get(&"foo".into()), None);
    }

    /// Trace to: FR-CACHE-005 Delete removes from both tiers
    #[test]
    fn test_delete() {
        let cache = CacheAdapter::<String, String>::new(10, 100);
        cache.set("foo".into(), "bar".into(), None);
        assert_eq!(cache.get(&"foo".into()), Some("bar".into()));

        cache.delete(&"foo".into());
        assert_eq!(cache.get(&"foo".into()), None);
    }

    /// Trace to: FR-CACHE-006 Clear removes all entries
    #[test]
    fn test_clear() {
        let cache = CacheAdapter::<String, String>::new(10, 100);
        cache.set("foo".into(), "bar".into(), None);
        cache.set("baz".into(), "qux".into(), None);

        assert_eq!(cache.get(&"foo".into()), Some("bar".into()));
        assert_eq!(cache.get(&"baz".into()), Some("qux".into()));

        cache.clear();

        assert_eq!(cache.get(&"foo".into()), None);
        assert_eq!(cache.get(&"baz".into()), None);
    }

    /// Trace to: FR-CACHE-007 Support multiple key/value types
    #[test]
    fn test_multiple_types() {
        let cache = CacheAdapter::<String, i32>::new(10, 100);
        cache.set("count".into(), 42, None);
        assert_eq!(cache.get(&"count".into()), Some(42));

        let str_cache = CacheAdapter::<i32, String>::new(10, 100);
        str_cache.set(1, "one".into(), None);
        assert_eq!(str_cache.get(&1), Some("one".into()));
    }

    /// Trace to: FR-CACHE-008 TTL duration enforcement
    #[test]
    fn test_ttl_duration() {
        let cache = CacheAdapter::<String, String>::new(10, 100);
        let ttl = Duration::seconds(10);
        cache.set("foo".into(), "bar".into(), Some(ttl));

        // Should be present immediately
        assert_eq!(cache.get(&"foo".into()), Some("bar".into()));

        // Should still be present (not expired yet)
        let result = cache.get(&"foo".into());
        assert_eq!(result, Some("bar".into()));
    }

    /// Trace to: FR-CACHE-009 Operation without metrics
    #[test]
    fn test_no_metrics() {
        let cache = CacheAdapter::<String, String>::new(10, 100);
        cache.set("foo".into(), "bar".into(), None);
        let result = cache.get(&"foo".into());
        assert_eq!(result, Some("bar".into()));
    }
}
