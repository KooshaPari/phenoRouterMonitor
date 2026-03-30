//! Cache port for temporary data storage.
//!
//! Caches provide fast access to frequently-used data with configurable lifetimes.
//! Implementations abstract the caching mechanism (Redis, in-memory, etc.).

use async_trait::async_trait;
use std::fmt::Debug;

/// Errors that can occur during cache operations.
#[derive(Debug, Clone)]
pub enum CacheError {
    NotFound,
    SerializationError(String),
    StorageError(String),
    InvalidKey(String),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::NotFound => write!(f, "Cache entry not found"),
            CacheError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            CacheError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            CacheError::InvalidKey(msg) => write!(f, "Invalid cache key: {}", msg),
        }
    }
}

impl std::error::Error for CacheError {}

/// Cache port for temporary data storage with TTL support.
///
/// # Example
///
/// ```ignore
/// impl CachePort for RedisCache {
///     type Error = CacheError;
///
///     async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, Self::Error> {
///         // Retrieve from Redis...
///     }
///
///     async fn set(&self, key: &str, value: &[u8], ttl_secs: Option<u64>) -> Result<(), Self::Error> {
///         // Store in Redis with optional TTL...
///     }
/// }
/// ```
#[async_trait]
pub trait CachePort: Send + Sync + Debug {
    /// Error type returned by cache operations.
    type Error: std::error::Error + Send + Sync + Debug;

    /// Retrieve a value by key.
    ///
    /// Returns `Ok(None)` if the key doesn't exist or has expired.
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, Self::Error>;

    /// Store a value with optional TTL.
    ///
    /// # Arguments
    ///
    /// - `key`: Cache key (opaque to the port, interpreted by implementation)
    /// - `value`: Raw bytes to store
    /// - `ttl_secs`: Optional time-to-live in seconds. `None` means no expiry.
    async fn set(&self, key: &str, value: &[u8], ttl_secs: Option<u64>) -> Result<(), Self::Error>;

    /// Delete a value by key.
    ///
    /// Returns success even if the key doesn't exist.
    async fn delete(&self, key: &str) -> Result<(), Self::Error>;

    /// Check if a key exists.
    async fn exists(&self, key: &str) -> Result<bool, Self::Error>;
}
