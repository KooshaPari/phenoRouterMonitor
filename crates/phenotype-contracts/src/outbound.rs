//! Outbound ports (driven side) - interfaces for accessing external services.

use crate::error;
use std::collections::HashMap;

/// Repository port for persisting and retrieving domain entities.
pub trait Repository: Send + Sync {
    type Entity: Send + Sync;
    type Id: Clone + Send + Sync;

    fn save(&self, id: Self::Id, entity: Self::Entity) -> error::Result<()>;
    fn get(&self, id: &Self::Id) -> error::Result<Self::Entity>;
    fn delete(&self, id: &Self::Id) -> error::Result<()>;
    fn list(&self) -> error::Result<Vec<Self::Entity>>;
}

/// Cache port for storing and retrieving cached values.
pub trait CachePort: Send + Sync {
    type Key: Clone + Send + Sync;
    type Value: Clone + Send + Sync;

    fn get(&self, key: &Self::Key) -> error::Result<Option<Self::Value>>;
    fn set(&self, key: Self::Key, value: Self::Value) -> error::Result<()>;
    fn invalidate(&self, key: &Self::Key) -> error::Result<()>;
}

/// Event bus port for publishing domain events.
pub trait EventBus: Send + Sync {
    type Event: Clone + Send + Sync;

    fn publish(&self, event: Self::Event) -> error::Result<()>;
    fn publish_batch(&self, events: Vec<Self::Event>) -> error::Result<()>;
}

/// Secret manager port for secure credential storage and retrieval.
pub trait SecretManager: Send + Sync {
    fn get(&self, name: &str) -> error::Result<String>;
    fn set(&self, name: String, value: String) -> error::Result<()>;
    fn delete(&self, name: &str) -> error::Result<()>;
}

/// Configuration loader port.
pub trait ConfigLoader: Send + Sync {
    fn load(&self) -> error::Result<HashMap<String, String>>;
}
