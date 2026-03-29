//! # Phenotype Test Infrastructure
//!
//! Generic test utilities consolidating 20+ test fixtures across the Phenotype ecosystem.
//! Provides in-memory stores, builders, temporary file contexts, and assertion helpers.

use dashmap::DashMap;
use phenotype_errors::{PhenotypeError, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Generic in-memory store for testing
///
/// Thread-safe, concurrent in-memory key-value store backed by DashMap.
/// Useful for mocking storage backends in tests.
///
/// # Example
///
/// ```ignore
/// let store = InMemoryStore::<String, i32>::new();
/// store.insert("counter".to_string(), 0);
/// assert_eq!(store.get(&"counter".to_string()), Some(0));
/// ```
pub struct InMemoryStore<K, V>
where
    K: Eq + std::hash::Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    data: Arc<DashMap<K, V>>,
}

impl<K, V> InMemoryStore<K, V>
where
    K: Eq + std::hash::Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Create a new empty in-memory store
    pub fn new() -> Self {
        Self {
            data: Arc::new(DashMap::new()),
        }
    }

    /// Insert a key-value pair
    pub fn insert(&self, key: K, value: V) {
        self.data.insert(key, value);
    }

    /// Get a value by key, returning a clone
    pub fn get(&self, key: &K) -> Option<V> {
        self.data.get(key).map(|v| v.value().clone())
    }

    /// Remove a value by key
    pub fn remove(&self, key: &K) -> Option<V> {
        self.data.remove(key).map(|(_, v)| v)
    }

    /// Check if a key exists
    pub fn contains(&self, key: &K) -> bool {
        self.data.contains_key(key)
    }

    /// Clear all entries
    pub fn clear(&self) {
        self.data.clear();
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if store is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Iterate over all entries (snapshot)
    pub fn iter<F>(&self, mut f: F)
    where
        F: FnMut(&K, &V),
    {
        for entry in self.data.iter() {
            f(entry.key(), entry.value());
        }
    }
}

impl<K, V> Default for InMemoryStore<K, V>
where
    K: Eq + std::hash::Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> Clone for InMemoryStore<K, V>
where
    K: Eq + std::hash::Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
        }
    }
}

/// Builder trait for test domain models
///
/// Provides a common interface for building test fixtures.
pub trait TestBuilder<T> {
    /// Create a builder with sensible defaults
    fn with_defaults() -> Self;

    /// Build the final value, validating constraints
    fn build(self) -> Result<T>;
}

/// Test context for managing temporary files
///
/// Automatically creates a temporary directory that is cleaned up on drop.
/// Provides helpers for creating test files.
pub struct TestContext {
    temp_dir: Option<tempfile::TempDir>,
}

impl TestContext {
    /// Create a new test context with a temporary directory
    pub fn new() -> Result<Self> {
        let temp_dir = tempfile::TempDir::new()
            .map_err(|e| PhenotypeError::Io(e.to_string()))?;
        Ok(Self {
            temp_dir: Some(temp_dir),
        })
    }

    /// Get the path to the temporary directory
    pub fn path(&self) -> Option<&Path> {
        self.temp_dir.as_ref().map(|d| d.path())
    }

    /// Create an empty file in the temporary directory
    pub fn create_file(&self, name: &str) -> Result<PathBuf> {
        let path = self
            .path()
            .ok_or_else(|| PhenotypeError::Internal("No temp directory".to_string()))?
            .join(name);

        std::fs::write(&path, "")
            .map_err(|e| PhenotypeError::Io(e.to_string()))?;

        Ok(path)
    }

    /// Write content to a file in the temporary directory
    pub fn write_file(&self, name: &str, content: &str) -> Result<PathBuf> {
        let path = self
            .path()
            .ok_or_else(|| PhenotypeError::Internal("No temp directory".to_string()))?
            .join(name);

        std::fs::write(&path, content)
            .map_err(|e| PhenotypeError::Io(e.to_string()))?;

        Ok(path)
    }

    /// Read content from a file in the temporary directory
    pub fn read_file(&self, name: &str) -> Result<String> {
        let path = self
            .path()
            .ok_or_else(|| PhenotypeError::Internal("No temp directory".to_string()))?
            .join(name);

        std::fs::read_to_string(&path)
            .map_err(|e| PhenotypeError::Io(e.to_string()))
    }
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new().unwrap_or(Self { temp_dir: None })
    }
}

/// Generate a unique test ID with a prefix
///
/// # Example
///
/// ```ignore
/// let id = test_id("test");
/// assert!(id.starts_with("test_"));
/// ```
pub fn test_id(prefix: &str) -> String {
    format!("{}_{}", prefix, uuid::Uuid::new_v4())
}

/// Assertion helper to verify error message contains text
///
/// # Example
///
/// ```ignore
/// let error = PhenotypeError::NotFound("resource".to_string());
/// assert_error_contains(&error, "resource");
/// ```
pub fn assert_error_contains(error: &PhenotypeError, text: &str) {
    let error_str = error.to_string();
    assert!(
        error_str.contains(text),
        "Error '{}' does not contain '{}'",
        error_str,
        text
    );
}

/// Assertion helper to verify error is a specific variant
///
/// # Example
///
/// ```ignore
/// let error = PhenotypeError::NotFound("test".to_string());
/// assert_error_variant(&error, "NotFound");
/// ```
pub fn assert_error_variant(error: &PhenotypeError, variant: &str) {
    let error_str = format!("{:?}", error);
    assert!(
        error_str.starts_with(variant),
        "Error variant doesn't match. Expected {}, got {}",
        variant,
        error_str
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_store_insert_get() {
        let store = InMemoryStore::<String, i32>::new();
        store.insert("key1".to_string(), 42);
        assert_eq!(store.get(&"key1".to_string()), Some(42));
    }

    #[test]
    fn test_in_memory_store_remove() {
        let store = InMemoryStore::<String, String>::new();
        store.insert("key1".to_string(), "value1".to_string());
        assert!(store.remove(&"key1".to_string()).is_some());
        assert!(store.is_empty());
    }

    #[test]
    fn test_in_memory_store_contains() {
        let store = InMemoryStore::<String, i32>::new();
        store.insert("key1".to_string(), 42);
        assert!(store.contains(&"key1".to_string()));
        assert!(!store.contains(&"key2".to_string()));
    }

    #[test]
    fn test_in_memory_store_clear() {
        let store = InMemoryStore::<String, i32>::new();
        store.insert("key1".to_string(), 42);
        store.insert("key2".to_string(), 100);
        assert_eq!(store.len(), 2);
        store.clear();
        assert!(store.is_empty());
    }

    #[test]
    fn test_in_memory_store_len() {
        let store = InMemoryStore::<String, i32>::new();
        assert_eq!(store.len(), 0);
        store.insert("key1".to_string(), 42);
        assert_eq!(store.len(), 1);
        store.insert("key2".to_string(), 100);
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn test_in_memory_store_clone() {
        let store = InMemoryStore::<String, i32>::new();
        store.insert("key1".to_string(), 42);
        let store2 = store.clone();
        assert_eq!(store2.get(&"key1".to_string()), Some(42));
    }

    #[test]
    fn test_in_memory_store_iter() {
        let store = InMemoryStore::<String, i32>::new();
        store.insert("key1".to_string(), 42);
        store.insert("key2".to_string(), 100);
        let mut count = 0;
        store.iter(|_, _| {
            count += 1;
        });
        assert_eq!(count, 2);
    }

    #[test]
    fn test_test_context_new() {
        let ctx = TestContext::new().unwrap();
        assert!(ctx.path().is_some());
    }

    #[test]
    fn test_test_context_create_file() {
        let ctx = TestContext::new().unwrap();
        let path = ctx.create_file("test.txt").unwrap();
        assert!(path.exists());
    }

    #[test]
    fn test_test_context_write_file() {
        let ctx = TestContext::new().unwrap();
        let path = ctx.write_file("test.txt", "hello world").unwrap();
        assert!(path.exists());
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn test_test_context_read_file() {
        let ctx = TestContext::new().unwrap();
        ctx.write_file("test.txt", "hello world").unwrap();
        let content = ctx.read_file("test.txt").unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn test_test_context_default() {
        let _ctx = TestContext::default();
        // Just verify it doesn't panic
    }

    #[test]
    fn test_id_generation() {
        let id1 = test_id("test");
        let id2 = test_id("test");
        assert_ne!(id1, id2);
        assert!(id1.starts_with("test_"));
        assert!(id2.starts_with("test_"));
    }

    #[test]
    fn test_assert_error_contains() {
        let error = PhenotypeError::NotFound("resource".to_string());
        assert_error_contains(&error, "resource");
    }

    #[test]
    #[should_panic]
    fn test_assert_error_contains_fails() {
        let error = PhenotypeError::NotFound("resource".to_string());
        assert_error_contains(&error, "not_present");
    }

    #[test]
    fn test_assert_error_variant() {
        let error = PhenotypeError::NotFound("test".to_string());
        assert_error_variant(&error, "NotFound");
    }

    #[test]
    #[should_panic]
    fn test_assert_error_variant_fails() {
        let error = PhenotypeError::NotFound("test".to_string());
        assert_error_variant(&error, "Conflict");
    }
}
