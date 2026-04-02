//! Cache port for caching operations.

use async_trait::async_trait;
use std::time::Duration;

/// Cache port for key-value caching operations.
#[async_trait]
pub trait CachePort: Send + Sync {
    /// Get a value by key.
    async fn get(&self, key: &str) -> Result<Option<String>, CacheError>;

    /// Set a value with TTL.
    async fn set(&self, key: &str, value: &str, ttl: Duration) -> Result<(), CacheError>;

    /// Set a value only if the key does not exist (NX).
    async fn set_nx(&self, key: &str, value: &str, ttl: Duration) -> Result<bool, CacheError>;

    /// Delete a key.
    async fn delete(&self, key: &str) -> Result<(), CacheError>;

    /// Check if a key exists.
    async fn exists(&self, key: &str) -> Result<bool, CacheError>;

    /// Set expiration on a key.
    async fn expire(&self, key: &str, ttl: Duration) -> Result<(), CacheError>;

    /// Get time-to-live for a key.
    async fn ttl(&self, key: &str) -> Result<Option<Duration>, CacheError>;

    /// Ping the cache to check connectivity.
    async fn ping(&self) -> Result<(), CacheError>;

    /// Close the cache connection.
    async fn close(&self) -> Result<(), CacheError>;
}

/// Cache port with JSON serialization support.
#[async_trait]
pub trait CacheJsonPort: Send + Sync {
    /// Get a value and deserialize it.
    async fn get_json<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>, CacheError>;

    /// Set a value after serializing it.
    async fn set_json<T: serde::Serialize>(&self, key: &str, value: &T, ttl: Duration) -> Result<(), CacheError>;
}

/// Cache port for atomic counter operations.
#[async_trait]
pub trait CacheCounterPort: Send + Sync {
    /// Increment a counter.
    async fn incr(&self, key: &str, delta: i64) -> Result<i64, CacheError>;

    /// Decrement a counter.
    async fn decr(&self, key: &str, delta: i64) -> Result<i64, CacheError>;

    /// Get the current value of a counter.
    async fn get_counter(&self, key: &str) -> Result<Option<i64>, CacheError>;
}

/// Cache port for distributed locking.
#[async_trait]
pub trait CacheLockPort: Send + Sync {
    /// Acquire a lock with a timeout.
    async fn lock(&self, key: &str, ttl: Duration) -> Result<bool, CacheError>;

    /// Release a lock.
    async fn unlock(&self, key: &str) -> Result<(), CacheError>;

    /// Extend a lock's TTL.
    async fn extend_lock(&self, key: &str, ttl: Duration) -> Result<bool, CacheError>;
}

/// Cache errors.
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("connection error: {0}")]
    Connection(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("key not found: {0}")]
    NotFound(String),

    #[error("operation failed: {0}")]
    OperationFailed(String),

    #[error("timeout")]
    Timeout,

    #[error("internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_error_connection_display() {
        let err = CacheError::Connection("refused".into());
        assert_eq!(err.to_string(), "connection error: refused");
    }

    #[test]
    fn cache_error_serialization_display() {
        let err = CacheError::Serialization("invalid utf8".into());
        assert_eq!(err.to_string(), "serialization error: invalid utf8");
    }

    #[test]
    fn cache_error_not_found_display() {
        let err = CacheError::NotFound("session:abc".into());
        assert_eq!(err.to_string(), "key not found: session:abc");
    }

    #[test]
    fn cache_error_operation_failed_display() {
        let err = CacheError::OperationFailed("OOM".into());
        assert_eq!(err.to_string(), "operation failed: OOM");
    }

    #[test]
    fn cache_error_timeout_display() {
        let err = CacheError::Timeout;
        assert_eq!(err.to_string(), "timeout");
    }

    #[test]
    fn cache_error_internal_display() {
        let err = CacheError::Internal("unexpected".into());
        assert_eq!(err.to_string(), "internal error: unexpected");
    }

    #[test]
    fn cache_error_debug() {
        let err = CacheError::Timeout;
        let debug = format!("{:?}", err);
        assert!(debug.contains("Timeout"));
    }
}
