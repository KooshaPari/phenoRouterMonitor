//! # Cache Port
//!
//! Outbound port for caching operations.

use async_trait::async_trait;
use std::time::Duration;

/// Cache port for get/set/delete operations with TTL support.
#[async_trait]
pub trait CachePort: Send + Sync {
    /// Get a value from cache.
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError>;

    /// Set a value in cache with optional TTL.
    async fn set(
        &self,
        key: &str,
        value: Vec<u8>,
        ttl: Option<Duration>,
    ) -> Result<(), CacheError>;

    /// Delete a key from cache.
    async fn delete(&self, key: &str) -> Result<(), CacheError>;

    /// Check if a key exists.
    async fn exists(&self, key: &str) -> Result<bool, CacheError>;

    /// Clear all cache entries (use with caution).
    async fn clear(&self) -> Result<(), CacheError>;
}

/// Cache operation errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum CacheError {
    #[error("key not found: {0}")]
    NotFound(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("connection error: {0}")]
    Connection(String),

    #[error("timeout: {0}")]
    Timeout(String),

    #[error("operation failed: {0}")]
    Operation(String),
}
