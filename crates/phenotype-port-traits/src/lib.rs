//! # Phenotype Port Traits
//!
//! Hexagonal architecture port interfaces for Phenotype.
//!
//! Defines the contracts between domain logic and infrastructure adapters:
//! - [`Repository`] — CRUD persistence
//! - [`EventPublisher`] / [`EventSubscriber`] — event-driven messaging
//! - [`Notifier`] — notification delivery
//! - [`CachePort`] — caching abstraction

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

/// Generic repository port for aggregate persistence.
#[async_trait]
pub trait Repository<T: Send + Sync>: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn get(&self, id: &str) -> Result<Option<T>, Self::Error>;
    async fn save(&self, id: &str, entity: &T) -> Result<(), Self::Error>;
    async fn delete(&self, id: &str) -> Result<(), Self::Error>;
    async fn list(&self, offset: usize, limit: usize) -> Result<Vec<T>, Self::Error>;
    async fn exists(&self, id: &str) -> Result<bool, Self::Error> {
        Ok(self.get(id).await?.is_some())
    }
}

/// Event publisher port — outbound events.
#[async_trait]
pub trait EventPublisher: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn publish(&self, topic: &str, payload: &serde_json::Value) -> Result<(), Self::Error>;
}

/// Event subscriber port — inbound events.
#[async_trait]
pub trait EventSubscriber: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn subscribe(
        &self,
        topic: &str,
        handler: Box<dyn Fn(serde_json::Value) + Send + Sync>,
    ) -> Result<(), Self::Error>;
}

/// Notification delivery port.
#[async_trait]
pub trait Notifier: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn notify(&self, recipient: &str, subject: &str, body: &str)
        -> Result<(), Self::Error>;
}

/// Cache port for key-value caching.
#[async_trait]
pub trait CachePort: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> Result<Option<T>, Self::Error>;
    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl_secs: Option<u64>,
    ) -> Result<(), Self::Error>;
    async fn delete(&self, key: &str) -> Result<(), Self::Error>;
}
