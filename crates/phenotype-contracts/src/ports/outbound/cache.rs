//! # Cache Port
//!
//! Outbound port for caching operations.
//!
//! ## Interface
//!
//! ```rust,ignore
//! use async_trait::async_trait;
//! use phenotype_contracts::ports::outbound::cache::CachePort;
//!
//! #[async_trait]
//! impl CachePort for RedisCacheAdapter {
//!     async fn get(&self, key: &str) -> Result<Option<String>, Error>;
//!     async fn set(&self, key: &str, value: &str, ttl: Duration) -> Result<(), Error>;
//!     async fn delete(&self, key: &str) -> Result<(), Error>;
//!     async fn exists(&self, key: &str) -> Result<bool, Error>;
//!     async fn expire(&self, key: &str, ttl: Duration) -> Result<(), Error>;
//!     async fn ttl(&self, key: &str) -> Result<Option<Duration>, Error>;
//! }
//! ```
//!
//! ## Extended Ports
//!
//! For specialized caching needs, extend the base port:
//!
//! - [`CacheJsonPort`] - JSON serialization support
//! - [`CacheCounterPort`] - Atomic counter operations
//! - [`CacheLockPort`] - Distributed locking

use async_trait::async_trait;
use std::time::Duration;

use super::Error;

/// Cache port interface.
///
/// Core caching operations that adapters must implement.
#[async_trait]
pub trait CachePort: Send + Sync {
    /// Get a value by key
    async fn get(&self, key: &str) -> Result<Option<String>, Error>;

    /// Set a value with TTL
    async fn set(&self, key: &str, value: &str, ttl: Duration) -> Result<(), Error>;

    /// Set a value only if key doesn't exist (atomic)
    async fn set_nx(&self, key: &str, value: &str, ttl: Duration) -> Result<bool, Error>;

    /// Delete a key
    async fn delete(&self, key: &str) -> Result<(), Error>;

    /// Check if key exists
    async fn exists(&self, key: &str) -> Result<bool, Error>;

    /// Set expiration on a key
    async fn expire(&self, key: &str, ttl: Duration) -> Result<(), Error>;

    /// Get TTL for a key
    async fn ttl(&self, key: &str) -> Result<Option<Duration>, Error>;

    /// Ping the cache
    async fn ping(&self) -> Result<(), Error>;

    /// Close the connection
    async fn close(&self) -> Result<(), Error>;
}

/// Extended cache port for JSON values.
///
/// Provides JSON serialization support.
#[async_trait]
pub trait CacheJsonPort: CachePort {
    /// Get and deserialize JSON value
    async fn get_json<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Error>;

    /// Serialize and set JSON value
    async fn set_json<T: serde::Serialize>(&self, key: &str, value: &T, ttl: Duration) -> Result<(), Error>;
}

/// Extended cache port for atomic counters.
#[async_trait]
pub trait CacheCounterPort: CachePort {
    /// Increment counter
    async fn incr(&self, key: &str, delta: i64) -> Result<i64, Error>;

    /// Decrement counter
    async fn decr(&self, key: &str, delta: i64) -> Result<i64, Error>;

    /// Get counter value
    async fn get_counter(&self, key: &str) -> Result<i64, Error>;
}

/// Extended cache port for distributed locks.
#[async_trait]
pub trait CacheLockPort: CachePort {
    /// Acquire a lock
    async fn acquire_lock(&self, key: &str, ttl: Duration) -> Result<bool, Error>;

    /// Release a lock
    async fn release_lock(&self, key: &str) -> Result<(), Error>;

    /// Extend lock TTL
    async fn extend_lock(&self, key: &str, ttl: Duration) -> Result<(), Error>;
}
