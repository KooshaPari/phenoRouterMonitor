//! Outbound (driven) port traits.
//!
//! These define the interfaces that infrastructure adapters must implement
//! (databases, caches, secret stores, event buses, health probes).

use std::time::Duration;

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

// ---------------------------------------------------------------------------
// Repository
// ---------------------------------------------------------------------------

/// Generic persistence port.
#[async_trait]
pub trait Repository<T, Id>
where
    T: Send + Sync + 'static,
    Id: Send + Sync + 'static,
{
    type Error: std::error::Error + Send + Sync + 'static;

    async fn find_by_id(&self, id: &Id) -> Result<Option<T>, Self::Error>;
    async fn save(&self, entity: &T) -> Result<(), Self::Error>;
    async fn delete(&self, id: &Id) -> Result<(), Self::Error>;
    async fn exists(&self, id: &Id) -> Result<bool, Self::Error>;
}

// ---------------------------------------------------------------------------
// CachePort
// ---------------------------------------------------------------------------

/// Key-value cache port.
#[async_trait]
pub trait CachePort<K, V>
where
    K: Send + Sync + 'static,
    V: Send + Sync + 'static,
{
    type Error: std::error::Error + Send + Sync + 'static;

    async fn get(&self, key: &K) -> Result<Option<V>, Self::Error>;
    async fn set(&self, key: &K, value: &V) -> Result<(), Self::Error>;
    async fn set_with_ttl(&self, key: &K, value: &V, ttl: Duration) -> Result<(), Self::Error>;
    async fn delete(&self, key: &K) -> Result<(), Self::Error>;
    async fn exists(&self, key: &K) -> Result<bool, Self::Error>;
}

// ---------------------------------------------------------------------------
// EventBus
// ---------------------------------------------------------------------------

/// Publish-subscribe event bus port.
#[async_trait]
pub trait EventBus {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn publish<E>(&self, topic: &str, event: &E) -> Result<(), Self::Error>
    where
        E: Serialize + Send + Sync + 'static;

    async fn subscribe<E, F>(&self, topic: &str, handler: F) -> Result<(), Self::Error>
    where
        E: DeserializeOwned + Send + 'static,
        F: Fn(E) + Send + Sync + 'static;
}

// ---------------------------------------------------------------------------
// SecretPort
// ---------------------------------------------------------------------------

/// Secret/credential store port.
#[async_trait]
pub trait SecretPort {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn get_secret(&self, key: &str) -> Result<Option<String>, Self::Error>;
    async fn set_secret(&self, key: &str, value: &str) -> Result<(), Self::Error>;
    async fn delete_secret(&self, key: &str) -> Result<(), Self::Error>;
}

// ---------------------------------------------------------------------------
// HealthCheck
// ---------------------------------------------------------------------------

/// Health status reported by a component.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}

/// Health probe port.
#[async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check(&self) -> HealthStatus;
}
