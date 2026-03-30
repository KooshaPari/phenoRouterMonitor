//! Outbound (driven) ports: Repository, CachePort, SecretPort, EventPublisher, EventSubscriber.
//!
//! These are implemented by infrastructure adapters (Postgres, Redis, Vault, Kafka, etc.).

use std::time::Duration;

use async_trait::async_trait;

use crate::domain::DomainEvent;
use crate::error::PortError;

// ---------------------------------------------------------------------------
// Repository
// ---------------------------------------------------------------------------

/// Generic repository for entity persistence.
///
/// # Type Parameters
///
/// * `E` -- entity type
/// * `I` -- identifier type
#[async_trait]
pub trait Repository<E, I>: Send + Sync
where
    E: Send + Sync,
    I: Send + Sync,
{
    /// Find an entity by its identifier.
    async fn find_by_id(&self, id: I) -> Result<Option<E>, PortError>;

    /// Persist an entity (insert or update).
    async fn save(&self, entity: E) -> Result<(), PortError>;

    /// Delete an entity by its identifier.
    async fn delete(&self, id: I) -> Result<(), PortError>;

    /// Return all entities.
    async fn find_all(&self) -> Result<Vec<E>, PortError>;
}

/// Unit-of-work for coordinating transactional persistence across repositories.
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    /// Begin a transaction.
    async fn begin(&self) -> Result<(), PortError>;
    /// Commit the transaction.
    async fn commit(&self) -> Result<(), PortError>;
    /// Roll back the transaction.
    async fn rollback(&self) -> Result<(), PortError>;
}

// ---------------------------------------------------------------------------
// CachePort
// ---------------------------------------------------------------------------

/// Cache port for key-value caching.
///
/// Consolidates `phenotype_contracts::ports::outbound::cache::CachePort` and
/// `harness_cache::ports::CachePort`.
#[async_trait]
pub trait CachePort: Send + Sync {
    /// Get a cached value.
    async fn get(&self, key: &str) -> Result<Option<String>, PortError>;
    /// Set a value with a TTL.
    async fn set(&self, key: &str, value: &str, ttl: Duration) -> Result<(), PortError>;
    /// Set only if the key does not already exist (atomic).
    async fn set_nx(&self, key: &str, value: &str, ttl: Duration) -> Result<bool, PortError>;
    /// Delete a key.
    async fn delete(&self, key: &str) -> Result<(), PortError>;
    /// Check whether a key exists.
    async fn exists(&self, key: &str) -> Result<bool, PortError>;
    /// Update the TTL on an existing key.
    async fn expire(&self, key: &str, ttl: Duration) -> Result<(), PortError>;
    /// Get remaining TTL for a key.
    async fn ttl(&self, key: &str) -> Result<Option<Duration>, PortError>;
    /// Connectivity check.
    async fn ping(&self) -> Result<(), PortError>;
}

/// JSON-aware cache extension.
#[async_trait]
pub trait CacheJsonPort: CachePort {
    /// Deserialize a cached JSON value.
    async fn get_json<T: serde::de::DeserializeOwned + Send>(
        &self,
        key: &str,
    ) -> Result<Option<T>, PortError>;

    /// Serialize and cache a value as JSON.
    async fn set_json<T: serde::Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> Result<(), PortError>;
}

/// Atomic counter operations on a cache.
#[async_trait]
pub trait CacheCounterPort: CachePort {
    /// Increment by `delta`, returning the new value.
    async fn incr(&self, key: &str, delta: i64) -> Result<i64, PortError>;
    /// Decrement by `delta`, returning the new value.
    async fn decr(&self, key: &str, delta: i64) -> Result<i64, PortError>;
}

/// Distributed-lock extension for a cache.
#[async_trait]
pub trait CacheLockPort: CachePort {
    /// Attempt to acquire a lock. Returns `true` on success.
    async fn acquire_lock(&self, key: &str, ttl: Duration) -> Result<bool, PortError>;
    /// Release a held lock.
    async fn release_lock(&self, key: &str) -> Result<(), PortError>;
    /// Extend the TTL on a held lock.
    async fn extend_lock(&self, key: &str, ttl: Duration) -> Result<(), PortError>;
}

// ---------------------------------------------------------------------------
// SecretPort
// ---------------------------------------------------------------------------

/// Secret management port.
#[async_trait]
pub trait SecretPort: Send + Sync {
    /// Retrieve a secret value.
    async fn get(&self, key: &str) -> Result<Option<String>, PortError>;
    /// Store a secret value.
    async fn set(&self, key: &str, value: &str) -> Result<(), PortError>;
    /// Delete a secret.
    async fn delete(&self, key: &str) -> Result<(), PortError>;
    /// List secret keys at a path.
    async fn list(&self, path: &str) -> Result<Vec<String>, PortError>;
}

/// Secret with version history.
#[async_trait]
pub trait VersionedSecretPort: SecretPort {
    /// Get a specific version.
    async fn get_version(&self, key: &str, version: u64) -> Result<Option<String>, PortError>;
    /// List available versions.
    async fn list_versions(&self, key: &str) -> Result<Vec<u64>, PortError>;
}

/// Automatic secret rotation.
#[async_trait]
pub trait SecretRotator: Send + Sync {
    /// Rotate a secret, returning the new value.
    async fn rotate(&self, key: &str) -> Result<String, PortError>;
    /// Enable periodic auto-rotation.
    async fn enable_auto_rotation(
        &self,
        key: &str,
        interval: Duration,
    ) -> Result<(), PortError>;
}

// ---------------------------------------------------------------------------
// EventPublisher / EventSubscriber
// ---------------------------------------------------------------------------

/// Publish domain events to a message broker.
#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publish a single event.
    async fn publish(&self, topic: &str, event: &dyn DomainEvent) -> Result<(), PortError>;

    /// Create a topic/stream if it does not exist.
    async fn create_topic(&self, topic: &str) -> Result<(), PortError>;
}

/// Subscribe to domain events on a message broker.
#[async_trait]
pub trait EventSubscriber: Send + Sync {
    /// Subscribe to a topic.
    async fn subscribe(&self, topic: &str) -> Result<(), PortError>;
    /// Unsubscribe from a topic.
    async fn unsubscribe(&self, topic: &str) -> Result<(), PortError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Verify trait-object safety for key traits.
    fn _assert_repo_object_safe(_: &dyn Repository<String, String>) {}
    fn _assert_cache_object_safe(_: &dyn CachePort) {}
    fn _assert_secret_object_safe(_: &dyn SecretPort) {}
    fn _assert_publisher_object_safe(_: &dyn EventPublisher) {}
    fn _assert_subscriber_object_safe(_: &dyn EventSubscriber) {}
}
