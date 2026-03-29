//! Integration tests for two-level cache adapter.
//! Tests critical functionality: L1/L2 coherency, TTL behavior, hit rates, concurrent access.

use phenotype_cache_adapter::{CacheAdapter, CacheError, L1Cache, L2Cache};
use serde_json::{json, Value as JsonValue};
use std::sync::Arc;
use std::time::Duration;

// ============================================================================
// L1 Cache Tests
// ============================================================================

#[test]
fn test_l1_hit() {
    let cache = L1Cache::new(10);
    let value = json!({ "data": "test_value" });
    cache.set("key1".to_string(), value.clone(), None).unwrap();

    let retrieved = cache.get("key1").unwrap();
    assert_eq!(retrieved, value);
}

#[test]
fn test_l1_miss() {
    let cache = L1Cache::new(10);
    let result = cache.get("nonexistent");

    assert!(result.is_err());
    assert!(matches!(result, Err(CacheError::KeyNotFound(_))));
}

#[test]
fn test_l1_overwrite() {
    let cache = L1Cache::new(10);
    let value1 = json!({ "version": 1 });
    let value2 = json!({ "version": 2 });

    cache.set("key1".to_string(), value1.clone(), None).unwrap();
    assert_eq!(cache.get("key1").unwrap(), value1);

    cache.set("key1".to_string(), value2.clone(), None).unwrap();
    assert_eq!(cache.get("key1").unwrap(), value2);
}

#[test]
fn test_l1_invalidate() {
    let cache = L1Cache::new(10);
    let value = json!({ "data": "test" });
    cache.set("key1".to_string(), value, None).unwrap();

    cache.invalidate("key1").unwrap();
    assert!(cache.get("key1").is_err());
}

#[test]
fn test_l1_clear() {
    let cache = L1Cache::new(10);
    cache.set("key1".to_string(), json!("value1"), None).unwrap();
    cache.set("key2".to_string(), json!("value2"), None).unwrap();
    cache.set("key3".to_string(), json!("value3"), None).unwrap();

    assert_eq!(cache.size(), 3);
    cache.clear();
    assert_eq!(cache.size(), 0);
}

#[test]
fn test_l1_ttl_expiry() {
    let cache = L1Cache::new(10);
    let value = json!({ "data": "expiring" });
    cache.set("key1".to_string(), value, Some(Duration::from_millis(50))).unwrap();

    // Should exist immediately
    assert!(cache.get("key1").is_ok());

    // Wait for expiry
    std::thread::sleep(Duration::from_millis(100));

    // Should be expired
    assert!(cache.get("key1").is_err());
}

#[test]
fn test_l1_size_limit() {
    let cache = L1Cache::new(3);

    cache.set("key1".to_string(), json!("value1"), None).unwrap();
    cache.set("key2".to_string(), json!("value2"), None).unwrap();
    cache.set("key3".to_string(), json!("value3"), None).unwrap();

    assert_eq!(cache.size(), 3);

    // Adding a 4th entry should evict one
    cache.set("key4".to_string(), json!("value4"), None).unwrap();

    // Size should still be 3 (one evicted, one added)
    assert_eq!(cache.size(), 3);
}

// ============================================================================
// L2 Cache Tests
// ============================================================================

#[test]
fn test_l2_hit() {
    let cache = L2Cache::new();
    let value = json!({ "persistent": true });
    cache.set("key1".to_string(), value.clone(), None).unwrap();

    let retrieved = cache.get("key1").unwrap();
    assert_eq!(retrieved, value);
}

#[test]
fn test_l2_miss() {
    let cache = L2Cache::new();
    let result = cache.get("nonexistent");

    assert!(result.is_err());
}

#[test]
fn test_l2_ttl_expiry() {
    let cache = L2Cache::new();
    let value = json!({ "data": "temporary" });
    cache.set("key1".to_string(), value, Some(Duration::from_millis(50))).unwrap();

    assert!(cache.get("key1").is_ok());

    std::thread::sleep(Duration::from_millis(100));

    assert!(cache.get("key1").is_err());
}

// ============================================================================
// Cache Adapter - Two-Level Integration Tests
// ============================================================================

#[test]
fn test_adapter_l1_hit() {
    let adapter = CacheAdapter::new(10);
    let value = json!({ "source": "direct" });
    adapter.set("key1".to_string(), value.clone(), None).unwrap();

    let retrieved = adapter.get("key1").unwrap();
    assert_eq!(retrieved, value);

    let stats = adapter.stats();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 0);
}

#[test]
fn test_adapter_l1_miss_l2_hit() {
    let adapter = CacheAdapter::new(1); // Small L1 to force eviction

    // Fill L1
    adapter.set("key1".to_string(), json!("value1"), None).unwrap();

    // Add another key that will evict key1 from L1
    adapter.set("key2".to_string(), json!("value2"), None).unwrap();

    // Retrieve key1 - should hit L2 and reload L1
    let retrieved = adapter.get("key1").unwrap();
    assert_eq!(retrieved, json!("value1"));

    let stats = adapter.stats();
    assert_eq!(stats.hits, 1);
    // L1 size is 1 (hits limit) - either key1 or key2 is in L1 after retrieval
    assert_eq!(stats.l1_size, 1);
    assert_eq!(stats.l2_size, 2); // Both in L2
}

#[test]
fn test_adapter_l2_miss_network_fetch() {
    let adapter = CacheAdapter::new(10);

    // Try to get a key that was never set
    let result = adapter.get("never_existed");
    assert!(result.is_err());

    let stats = adapter.stats();
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 1);
}

#[test]
fn test_adapter_ttl_expiry_l1() {
    let adapter = CacheAdapter::new(10);
    let value = json!({ "ttl": "short" });
    adapter.set("key1".to_string(), value, Some(Duration::from_millis(50))).unwrap();

    // Should exist in L1
    assert!(adapter.get("key1").is_ok());

    // Wait for expiry
    std::thread::sleep(Duration::from_millis(100));

    // Should be expired in both L1 and L2
    assert!(adapter.get("key1").is_err());
}

#[test]
fn test_adapter_cache_invalidation() {
    let adapter = CacheAdapter::new(10);
    let value = json!({ "data": "test" });

    adapter.set("key1".to_string(), value, None).unwrap();
    assert!(adapter.get("key1").is_ok());

    adapter.invalidate("key1").unwrap();

    // Should be removed from both L1 and L2
    assert!(adapter.get("key1").is_err());
}

#[test]
fn test_adapter_l1_l2_coherency() {
    let adapter = CacheAdapter::new(10);

    // Set initial value
    let value1 = json!({ "version": 1 });
    adapter.set("key1".to_string(), value1.clone(), None).unwrap();

    // Retrieve from L1
    let retrieved1 = adapter.get("key1").unwrap();
    assert_eq!(retrieved1, value1);

    // Update value
    let value2 = json!({ "version": 2 });
    adapter.set("key1".to_string(), value2.clone(), None).unwrap();

    // Retrieve should get updated value from both L1 and L2
    let retrieved2 = adapter.get("key1").unwrap();
    assert_eq!(retrieved2, value2);
}

#[test]
fn test_adapter_size_limits() {
    let adapter = CacheAdapter::new(5);

    for i in 0..10 {
        let key = format!("key{}", i);
        let value = json!({ "index": i });
        adapter.set(key, value, None).unwrap();
    }

    let stats = adapter.stats();
    // L1 should be at most 5
    assert!(stats.l1_size <= 5);
    // L2 should have all 10
    assert_eq!(stats.l2_size, 10);
}

#[test]
fn test_adapter_concurrent_access() {
    let adapter = Arc::new(CacheAdapter::new(100));
    let mut handles = vec![];

    // Spawn threads for concurrent reads and writes
    for thread_id in 0..5 {
        let adapter_clone = Arc::clone(&adapter);
        let handle = std::thread::spawn(move || {
            for i in 0..20 {
                let key = format!("thread{}_key{}", thread_id, i);
                let value = json!({ "thread": thread_id, "index": i });

                adapter_clone.set(key.clone(), value.clone(), None).unwrap();

                // Read back
                let retrieved = adapter_clone.get(&key).unwrap();
                assert_eq!(retrieved, value);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let stats = adapter.stats();
    assert_eq!(stats.hits, 100); // 5 threads * 20 reads
}

#[test]
fn test_adapter_hit_rate_metrics() {
    let adapter = CacheAdapter::new(10);
    let value = json!({ "data": "test" });

    adapter.set("key1".to_string(), value.clone(), None).unwrap();

    // Generate hits and misses
    adapter.get("key1").unwrap();  // hit
    adapter.get("key1").unwrap();  // hit
    adapter.get("key1").unwrap();  // hit
    let _ = adapter.get("key2");   // miss
    let _ = adapter.get("key3");   // miss

    let stats = adapter.stats();
    assert_eq!(stats.hits, 3);
    assert_eq!(stats.misses, 2);
    assert!((stats.hit_rate() - 0.6).abs() < 0.01); // 3/5 = 0.6
}

#[test]
fn test_adapter_multiple_values() {
    let adapter = CacheAdapter::new(10);

    let values = vec![
        ("string_key", json!("string_value")),
        ("number_key", json!(42)),
        ("bool_key", json!(true)),
        ("array_key", json!([1, 2, 3])),
        ("object_key", json!({"nested": "object"})),
    ];

    for (key, value) in &values {
        adapter.set(key.to_string(), value.clone(), None).unwrap();
    }

    for (key, expected_value) in values {
        let retrieved = adapter.get(key).unwrap();
        assert_eq!(retrieved, expected_value);
    }
}

#[test]
fn test_adapter_stress_large_values() {
    let adapter = CacheAdapter::new(10);

    // Create a large JSON value
    let mut large_obj = serde_json::Map::new();
    for i in 0..100 {
        large_obj.insert(format!("field{}", i), json!(format!("value{}", i)));
    }
    let large_value = JsonValue::Object(large_obj);

    adapter.set("large_key".to_string(), large_value.clone(), None).unwrap();

    let retrieved = adapter.get("large_key").unwrap();
    assert_eq!(retrieved, large_value);
}

#[test]
fn test_adapter_stress_many_keys() {
    let adapter = CacheAdapter::new(1000);

    const NUM_KEYS: usize = 500;

    // Write phase
    for i in 0..NUM_KEYS {
        let key = format!("stress_key_{}", i);
        let value = json!({ "index": i });
        adapter.set(key, value, None).unwrap();
    }

    // Read phase
    for i in 0..NUM_KEYS {
        let key = format!("stress_key_{}", i);
        let retrieved = adapter.get(&key).unwrap();
        assert_eq!(retrieved["index"], i);
    }

    let stats = adapter.stats();
    assert_eq!(stats.hits, NUM_KEYS as u64);
    assert!(stats.l2_size >= NUM_KEYS);
}

#[test]
fn test_adapter_ttl_with_updates() {
    let adapter = CacheAdapter::new(10);
    let value = json!({ "data": "test" });

    // Set with short TTL
    adapter.set("key1".to_string(), value.clone(), Some(Duration::from_millis(100))).unwrap();
    assert!(adapter.get("key1").is_ok());

    // Wait half the TTL
    std::thread::sleep(Duration::from_millis(50));

    // Refresh with new TTL
    adapter.set("key1".to_string(), value.clone(), Some(Duration::from_millis(100))).unwrap();

    // Wait the original time again - should still exist
    std::thread::sleep(Duration::from_millis(75));
    assert!(adapter.get("key1").is_ok());

    // Wait for new TTL to expire
    std::thread::sleep(Duration::from_millis(50));
    assert!(adapter.get("key1").is_err());
}

#[test]
fn test_adapter_json_value_types() {
    let adapter = CacheAdapter::new(10);

    let test_cases = vec![
        ("null_val", json!(null)),
        ("bool_true", json!(true)),
        ("bool_false", json!(false)),
        ("int", json!(42)),
        ("float", json!(3.14)),
        ("string", json!("hello")),
        ("empty_array", json!([])),
        ("array", json!([1, "two", true])),
        ("empty_object", json!({})),
        ("object", json!({"key": "value"})),
    ];

    for (key, value) in test_cases {
        adapter.set(key.to_string(), value.clone(), None).unwrap();
        let retrieved = adapter.get(key).unwrap();
        assert_eq!(retrieved, value);
    }
}
