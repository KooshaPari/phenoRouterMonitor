//! Mock infrastructure for testing
//!
//! This module provides common mocking utilities for creating test doubles
//! and stub implementations.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// ============================================================================
// MockStorage - In-memory storage for mocking
// ============================================================================

/// Thread-safe in-memory storage for mock implementations
///
/// This can be used to create simple in-memory mocks of services
/// or data stores during testing.
#[derive(Debug, Default, Clone)]
pub struct MockStorage {
    data: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl MockStorage {
    /// Create a new empty mock storage
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Insert a value
    pub fn insert(&self, key: impl Into<String>, value: impl Into<Vec<u8>>) {
        let mut data = self.data.write().unwrap();
        data.insert(key.into(), value.into());
    }

    /// Get a value
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let data = self.data.read().unwrap();
        data.get(key).cloned()
    }

    /// Check if a key exists
    pub fn contains_key(&self, key: &str) -> bool {
        let data = self.data.read().unwrap();
        data.contains_key(key)
    }

    /// Remove a value
    pub fn remove(&self, key: &str) -> Option<Vec<u8>> {
        let mut data = self.data.write().unwrap();
        data.remove(key)
    }

    /// Clear all values
    pub fn clear(&self) {
        let mut data = self.data.write().unwrap();
        data.clear();
    }

    /// Get the number of stored items
    pub fn len(&self) -> usize {
        let data = self.data.read().unwrap();
        data.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        let data = self.data.read().unwrap();
        data.is_empty()
    }

    /// Get all keys
    pub fn keys(&self) -> Vec<String> {
        let data = self.data.read().unwrap();
        data.keys().cloned().collect()
    }

    /// Get all values
    pub fn values(&self) -> Vec<Vec<u8>> {
        let data = self.data.read().unwrap();
        data.values().cloned().collect()
    }
}

impl MockStorage {
    /// Insert a serializable value as JSON
    pub fn insert_json<T: serde::Serialize>(&self, key: impl Into<String>, value: &T) -> Result<(), serde_json::Error> {
        let json = serde_json::to_vec(value)?;
        self.insert(key, json);
        Ok(())
    }

    /// Get a deserialized value
    pub fn get_json<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<Result<T, serde_json::Error>> {
        self.get(key).map(|data| serde_json::from_slice(&data))
    }
}

// ============================================================================
// MockCallTracker - Track method calls
// ============================================================================

/// Track method calls for verification in tests
#[derive(Debug, Default, Clone)]
pub struct MockCallTracker {
    calls: Arc<RwLock<Vec<MockCall>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Represents a single method call
pub struct MockCall {
    pub method: String,
    pub args: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl MockCallTracker {
    /// Create a new call tracker
    pub fn new() -> Self {
        Self {
            calls: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Record a call
    pub fn record(&self, method: impl Into<String>, args: Vec<String>) {
        let call = MockCall {
            method: method.into(),
            args,
            timestamp: chrono::Utc::now(),
        };
        let mut calls = self.calls.write().unwrap();
        calls.push(call);
    }

    /// Get all recorded calls
    pub fn calls(&self) -> Vec<MockCall> {
        let calls = self.calls.read().unwrap();
        calls.clone()
    }

    /// Get calls for a specific method
    pub fn calls_for(&self, method: &str) -> Vec<MockCall> {
        let calls = self.calls.read().unwrap();
        calls.iter().filter(|c| c.method == method).cloned().collect()
    }

    /// Count calls for a specific method
    pub fn call_count(&self, method: &str) -> usize {
        let calls = self.calls.read().unwrap();
        calls.iter().filter(|c| c.method == method).count()
    }

    /// Clear all recorded calls
    pub fn clear(&self) {
        let mut calls = self.calls.write().unwrap();
        calls.clear();
    }

    /// Verify a method was called at least once
    pub fn was_called(&self, method: &str) -> bool {
        self.call_count(method) > 0
    }

    /// Verify a method was called exactly n times
    pub fn was_called_times(&self, method: &str, n: usize) -> bool {
        self.call_count(method) == n
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_storage_insert_get() {
        let storage = MockStorage::new();
        storage.insert("key", b"value");
        assert_eq!(storage.get("key"), Some(b"value".to_vec()));
    }

    #[test]
    fn test_mock_storage_contains_key() {
        let storage = MockStorage::new();
        storage.insert("key", b"value");
        assert!(storage.contains_key("key"));
        assert!(!storage.contains_key("other"));
    }

    #[test]
    fn test_mock_storage_remove() {
        let storage = MockStorage::new();
        storage.insert("key", b"value");
        assert_eq!(storage.remove("key"), Some(b"value".to_vec()));
        assert!(!storage.contains_key("key"));
    }

    #[test]
    fn test_mock_storage_clear() {
        let storage = MockStorage::new();
        storage.insert("key1", b"value1");
        storage.insert("key2", b"value2");
        storage.clear();
        assert!(storage.is_empty());
    }

    #[test]
    fn test_mock_storage_keys_values() {
        let storage = MockStorage::new();
        storage.insert("key1", b"value1");
        storage.insert("key2", b"value2");
        assert_eq!(storage.keys().len(), 2);
        assert_eq!(storage.values().len(), 2);
    }

    #[test]
    fn test_mock_storage_json() {
        let storage = MockStorage::new();
        let data = serde_json::json!({"name": "test", "value": 42});
        storage.insert_json("data", &data).unwrap();
        let retrieved: serde_json::Value = storage.get_json("data").unwrap().unwrap();
        assert_eq!(retrieved["name"], "test");
    }

    #[test]
    fn test_mock_call_tracker_record() {
        let tracker = MockCallTracker::new();
        tracker.record("method1", vec!["arg1".to_string()]);
        assert!(tracker.was_called("method1"));
        assert_eq!(tracker.call_count("method1"), 1);
    }

    #[test]
    fn test_mock_call_tracker_calls_for() {
        let tracker = MockCallTracker::new();
        tracker.record("method1", vec!["arg1".to_string()]);
        tracker.record("method2", vec!["arg2".to_string()]);
        tracker.record("method1", vec!["arg3".to_string()]);

        let method1_calls = tracker.calls_for("method1");
        assert_eq!(method1_calls.len(), 2);
        assert_eq!(method1_calls[0].method, "method1");
    }

    #[test]
    fn test_mock_call_tracker_was_called_times() {
        let tracker = MockCallTracker::new();
        tracker.record("method1", vec![]);
        tracker.record("method1", vec![]);
        tracker.record("method1", vec![]);

        assert!(tracker.was_called_times("method1", 3));
        assert!(!tracker.was_called_times("method1", 2));
    }

    #[test]
    fn test_mock_call_tracker_clear() {
        let tracker = MockCallTracker::new();
        tracker.record("method1", vec![]);
        tracker.clear();
        assert!(!tracker.was_called("method1"));
    }
}
