//! Outbound ports (driven side) - interfaces for accessing external services.
//!
//! These are the interfaces that adapters implement to provide access to
//! external systems (databases, caches, event buses, etc.).

use crate::Result;
use std::collections::HashMap;

/// Repository port for persisting and retrieving domain entities.
pub trait Repository: Send + Sync {
    type Entity: Send + Sync;
    type Id: Clone + Send + Sync;

    /// Saves an entity.
    fn save(&self, id: Self::Id, entity: Self::Entity) -> Result<()>;

    /// Retrieves an entity by ID.
    fn get(&self, id: &Self::Id) -> Result<Self::Entity>;

    /// Deletes an entity by ID.
    fn delete(&self, id: &Self::Id) -> Result<()>;

    /// Lists all entities.
    fn list(&self) -> Result<Vec<Self::Entity>>;
}

/// Cache port for storing and retrieving cached values.
pub trait CachePort: Send + Sync {
    type Key: Clone + Send + Sync;
    type Value: Clone + Send + Sync;

    /// Gets a value from cache.
    fn get(&self, key: &Self::Key) -> Result<Option<Self::Value>>;

    /// Sets a value in cache.
    fn set(&self, key: Self::Key, value: Self::Value) -> Result<()>;

    /// Invalidates a cache entry.
    fn invalidate(&self, key: &Self::Key) -> Result<()>;
}

/// Event bus port for publishing and subscribing to domain events.
pub trait EventBus: Send + Sync {
    type Event: Clone + Send + Sync;

    /// Publishes an event to the bus.
    fn publish(&self, event: Self::Event) -> Result<()>;

    /// Publishes multiple events.
    fn publish_batch(&self, events: Vec<Self::Event>) -> Result<()>;
}

/// Secret manager port for secure credential storage and retrieval.
pub trait SecretManager: Send + Sync {
    /// Retrieves a secret by name.
    fn get(&self, name: &str) -> Result<String>;

    /// Stores a secret.
    fn set(&self, name: String, value: String) -> Result<()>;

    /// Deletes a secret.
    fn delete(&self, name: &str) -> Result<()>;
}

/// Configuration loader port.
pub trait ConfigLoader: Send + Sync {
    /// Loads configuration and returns as a map.
    fn load(&self) -> Result<HashMap<String, String>>;
}
