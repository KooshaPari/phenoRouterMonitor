//! Outbound ports - interfaces for infrastructure adapters.
//!
//! Outbound ports define what the domain core needs from the outside world.
//! Adapters implement these traits to provide concrete functionality
//! (databases, caches, message brokers, etc.).

use async_trait::async_trait;
use std::time::Duration;

use crate::error::Result;
use crate::models::Entity;

/// Repository port for entity persistence.
///
/// This is the primary outbound port for data access.
/// All CRUD operations go through this abstraction.
#[async_trait]
pub trait Repository<E: Entity>: Send + Sync {
    /// Save an entity (insert or update).
    ///
    /// # Errors
    ///
    /// Returns `PortError::Connection` if the store is unreachable.
    /// Returns `PortError::Serialization` if entity cannot be serialized.
    async fn save(&self, entity: &E) -> Result<()>;

    /// Find an entity by its ID.
    ///
    /// # Errors
    ///
    /// Returns `PortError::NotFound` if entity doesn't exist.
    async fn find_by_id(&self, id: &str) -> Result<Option<E>>;

    /// Delete an entity by its ID.
    ///
    /// # Errors
    ///
    /// Returns `PortError::NotFound` if entity doesn't exist.
    async fn delete(&self, id: &str) -> Result<()>;
    /// List all entities with optional pagination.
    ///
    /// # Errors
    ///
    /// Returns `PortError::Connection` if the store is unreachable.
    async fn list(&self, offset: usize, limit: usize) -> Result<Vec<E>>;

    /// Count total entities.
    ///
    /// # Errors
    ///
    /// Returns `PortError::Connection` if the store is unreachable.
    async fn count(&self) -> Result<usize>;

    /// Check if an entity exists.
    ///
    /// Default implementation uses `find_by_id`.
    async fn exists(&self, id: &str) -> Result<bool> {
        Ok(self.find_by_id(id).await?.is_some())
    }
}

/// Unit of Work pattern for transactional operations.
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    /// Begin a new transaction.
    async fn begin(&mut self) -> Result<()>;

    /// Commit the current transaction.
    async fn commit(&mut self) -> Result<()>;

    /// Rollback the current transaction.
    async fn rollback(&mut self) -> Result<()>;
}

/// Cache port for key-value storage.
#[async_trait]
pub trait CachePort: Send + Sync {
    /// Get a value by key.
    ///
    /// Returns `None` if key doesn't exist.
    async fn get(&self, key: &str) -> Result<Option<String>>;

    /// Set a value with a TTL (time-to-live).
    ///
    /// When TTL expires, the key should be automatically removed.
    async fn set(&self, key: &str, value: &str, ttl: Duration) -> Result<()>;

    /// Set a value only if the key doesn't exist (NX - Not eXists).
    ///
    /// Returns `true` if the key was set, `false` if it already existed.
    async fn set_nx(&self, key: &str, value: &str, ttl: Duration) -> Result<bool>;

    /// Delete a key.
    async fn delete(&self, key: &str) -> Result<()>;

    /// Check if a key exists.
    async fn exists(&self, key: &str) -> Result<bool>;

    /// Set expiration on an existing key.
    async fn expire(&self, key: &str, ttl: Duration) -> Result<()>;

    /// Get the remaining TTL for a key.
    ///
    /// Returns `None` if key doesn't exist or has no TTL.
    async fn ttl(&self, key: &str) -> Result<Option<Duration>>;

    /// Ping the cache to check connectivity.
    async fn ping(&self) -> Result<()>;
}

/// Extended cache port with JSON serialization support.
#[async_trait]
pub trait CacheJsonPort: CachePort {
    /// Get a value and deserialize it from JSON.
    async fn get_json<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>>;

    /// Set a value after serializing it to JSON.
    async fn set_json<T: serde::Serialize + Send + Sync>(&self, key: &str, value: &T, ttl: Duration) -> Result<()>;
}

/// Cache port for atomic counter operations.
#[async_trait]
pub trait CacheCounterPort: CachePort {
    /// Increment a counter by the given delta.
    ///
    /// Returns the new value.
    async fn incr(&self, key: &str, delta: i64) -> Result<i64>;

    /// Decrement a counter by the given delta.
    ///
    /// Returns the new value.
    async fn decr(&self, key: &str, delta: i64) -> Result<i64>;

    /// Get the current counter value.
    async fn get_counter(&self, key: &str) -> Result<Option<i64>>;
}

/// Cache port for distributed locking.
#[async_trait]
pub trait CacheLockPort: CachePort {
    /// Acquire a lock with the given TTL.
    ///
    /// Returns `true` if the lock was acquired, `false` otherwise.
    async fn lock(&self, key: &str, ttl: Duration) -> Result<bool>;

    /// Release a lock.
    async fn unlock(&self, key: &str) -> Result<()>;

    /// Extend a lock's TTL.
    ///
    /// Returns `true` if the lock was extended, `false` if it doesn't exist.
    async fn extend_lock(&self, key: &str, ttl: Duration) -> Result<bool>;
}

/// Domain event envelope with metadata.
#[derive(Debug, Clone)]
pub struct EventEnvelope<E: crate::models::DomainEvent> {
    /// The domain event.
    pub event: E,
    /// Event type identifier.
    pub event_type: &'static str,
    /// Aggregate ID that produced this event.
    pub aggregate_id: String,
    /// Timestamp when the event occurred.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Correlation ID for tracing distributed operations.
    pub correlation_id: Option<String>,
    /// Causation ID linking to the event that caused this one.
    pub causation_id: Option<String>,
}

impl<E: crate::models::DomainEvent> EventEnvelope<E> {
    /// Create a new event envelope.
    pub fn new(event: E) -> Self {
        Self {
            event_type: event.event_type(),
            aggregate_id: event.aggregate_id().to_string(),
            timestamp: event.timestamp(),
            event,
            correlation_id: None,
            causation_id: None,
        }
    }

    /// Add a correlation ID.
    pub fn with_correlation_id(mut self, id: String) -> Self {
        self.correlation_id = Some(id);
        self
    }

    /// Add a causation ID.
    pub fn with_causation_id(mut self, id: String) -> Self {
        self.causation_id = Some(id);
        self
    }
}

/// Event publisher port for publishing domain events.
#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publish a single domain event.
    ///
    /// # Errors
    ///
    /// Returns `PortError::Connection` if the event bus is unreachable.
    /// Returns `PortError::Serialization` if event cannot be serialized.
    async fn publish<E: crate::models::DomainEvent>(&self, envelope: EventEnvelope<E>) -> Result<()>;

    /// Publish multiple events in a batch.
    ///
    /// Should be atomic - either all events are published or none.
    async fn publish_batch<E: crate::models::DomainEvent>(&self, envelopes: Vec<EventEnvelope<E>>) -> Result<()>;
}

/// Event subscriber port for consuming domain events.
#[async_trait]
pub trait EventSubscriber<E: crate::models::DomainEvent>: Send + Sync {
    /// Subscribe to events with a handler.
    ///
    /// The handler will be called for each event received.
    async fn subscribe<F>(&self, handler: F) -> Result<()>
    where
        F: Fn(E) -> Result<()> + Send + Sync + 'static;

    /// Unsubscribe from events.
    async fn unsubscribe(&self) -> Result<()>;
}

/// Secret port for accessing sensitive data.
#[async_trait]
pub trait SecretPort: Send + Sync {
    /// Get a secret by name.
    ///
    /// Returns `None` if the secret doesn't exist.
    async fn get(&self, name: &str) -> Result<Option<String>>;

    /// Set a secret.
    async fn set(&self, name: &str, value: &str) -> Result<()>;

    /// Delete a secret.
    async fn delete(&self, name: &str) -> Result<()>;

    /// List all secret names.
    async fn list(&self) -> Result<Vec<String>>;
}

/// Versioned secret port for secrets with version history.
#[async_trait]
pub trait VersionedSecretPort: SecretPort {
    /// Get a specific version of a secret.
    async fn get_version(&self, name: &str, version: u32) -> Result<Option<String>>;

    /// Get the latest version of a secret.
    ///
    /// Returns the value and version number.
    async fn get_latest(&self, name: &str) -> Result<Option<(String, u32)>>;

    /// Set a new version of a secret.
    ///
    /// Returns the new version number.
    async fn set_versioned(&self, name: &str, value: &str) -> Result<u32>;

    /// List all versions of a secret.
    async fn list_versions(&self, name: &str) -> Result<Vec<u32>>;

    /// Delete a specific version.
    async fn delete_version(&self, name: &str, version: u32) -> Result<()>;
}

/// Secret rotator port for automated secret rotation.
#[async_trait]
pub trait SecretRotator: Send + Sync {
    /// Rotate a secret (create new version, optionally disable old).
    ///
    /// Returns the new version number.
    async fn rotate(&self, name: &str) -> Result<u32>;

    /// Check if rotation is needed based on secret age.
    ///
    /// `max_age_days` - maximum age before rotation is recommended.
    async fn needs_rotation(&self, name: &str, max_age_days: u32) -> Result<bool>;
}

/// Policy port for rule-based evaluation.
#[async_trait]
pub trait PolicyPort: Send + Sync {
    /// Evaluate a policy against the given context.
    ///
    /// Returns `true` if policy is satisfied, `false` otherwise.
    async fn evaluate(&self, policy_name: &str, context: &serde_json::Value) -> Result<bool>;

    /// Get all available policy names.
    async fn list_policies(&self) -> Result<Vec<String>>;
}

/// Configuration port for accessing configuration values.
#[async_trait]
pub trait ConfigPort: Send + Sync {
    /// Get a configuration value by key.
    ///
    /// Returns `None` if the key doesn't exist.
    async fn get(&self, key: &str) -> Result<Option<String>>;

    /// Get a typed configuration value.
    async fn get_typed<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>>;

    /// Check if a key exists.
    async fn exists(&self, key: &str) -> Result<bool>;

    /// Get all configuration keys.
    async fn keys(&self) -> Result<Vec<String>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_envelope_new() {
        use crate::models::DomainEvent;

        #[derive(Debug, Clone)]
        struct TestEvent {
            id: String,
            ts: chrono::DateTime<chrono::Utc>,
        }

        impl DomainEvent for TestEvent {
            fn event_type(&self) -> &'static str {
                "test.event"
            }

            fn aggregate_id(&self) -> &str {
                &self.id
            }

            fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
                self.ts
            }
        }

        let event = TestEvent {
            id: "agg-1".into(),
            ts: chrono::Utc::now(),
        };

        let envelope = EventEnvelope::new(event);
        assert_eq!(envelope.event_type, "test.event");
        assert_eq!(envelope.aggregate_id, "agg-1");
        assert!(envelope.correlation_id.is_none());
    }

    #[test]
    fn event_envelope_with_metadata() {
        use crate::models::DomainEvent;

        #[derive(Debug, Clone)]
        struct TestEvent {
            id: String,
            ts: chrono::DateTime<chrono::Utc>,
        }

        impl DomainEvent for TestEvent {
            fn event_type(&self) -> &'static str {
                "test.event"
            }

            fn aggregate_id(&self) -> &str {
                &self.id
            }

            fn timestamp(&self) -> chrono::DateTime<chrono::Utc> {
                self.ts
            }
        }

        let event = TestEvent {
            id: "agg-1".into(),
            ts: chrono::Utc::now(),
        };

        let envelope = EventEnvelope::new(event)
            .with_correlation_id("corr-123".into())
            .with_causation_id("cause-456".into());

        assert_eq!(envelope.correlation_id.as_deref(), Some("corr-123"));
        assert_eq!(envelope.causation_id.as_deref(), Some("cause-456"));
    }
}
