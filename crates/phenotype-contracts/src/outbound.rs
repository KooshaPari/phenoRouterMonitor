//! Outbound (driven) port traits for hexagonal architecture.
//!
//! These traits define the interfaces that the domain requires from
//! infrastructure. Adapters on the driven side (e.g. databases, caches,
//! message brokers) implement these ports.
//!
//! Traces to: FR-PHENO-004

use crate::error::ContractError;
use crate::models::{DomainEntity, DomainEvent, EntityId};
use async_trait::async_trait;
use serde::Serialize;
use std::fmt::Debug;

/// A generic repository port for CRUD operations on domain entities.
#[async_trait]
pub trait Repository<Entity, Id>: Send + Sync + 'static
where
    Entity: DomainEntity<Id = Id> + Send + Sync,
    Id: EntityId,
{
    /// Persist a new or updated entity.
    async fn save(&self, entity: &Entity) -> Result<(), ContractError>;

    /// Retrieve an entity by its identifier, or `None` if not found.
    async fn find_by_id(&self, id: &Id) -> Result<Option<Entity>, ContractError>;

    /// Delete an entity by its identifier.
    async fn delete(&self, id: &Id) -> Result<(), ContractError>;

    /// Check whether an entity with the given id exists.
    async fn exists(&self, id: &Id) -> Result<bool, ContractError> {
        Ok(self.find_by_id(id).await?.is_some())
    }
}

/// A cache port for key-value storage with optional TTL.
#[async_trait]
pub trait CachePort<K, V>: Send + Sync + 'static
where
    K: Send + Sync + 'static,
    V: Send + Sync + 'static,
{
    /// Retrieve a cached value by key.
    async fn get(&self, key: &K) -> Result<Option<V>, ContractError>;

    /// Store a value under the given key.
    async fn set(&self, key: K, value: V) -> Result<(), ContractError>;

    /// Remove a cached value.
    async fn delete(&self, key: &K) -> Result<(), ContractError>;

    /// Check whether a key is present in the cache.
    async fn exists(&self, key: &K) -> Result<bool, ContractError> {
        Ok(self.get(key).await?.is_some())
    }
}

/// An event bus port for publishing and subscribing to domain events.
#[async_trait]
pub trait EventBus: Send + Sync + 'static {
    /// Publish a domain event.
    async fn publish<E: DomainEvent + Serialize + Send + Sync>(
        &self,
        event: &E,
    ) -> Result<(), ContractError>;

    /// Subscribe to events of a given type name.
    async fn subscribe(&self, event_type: &str) -> Result<(), ContractError>;
}

/// A secret manager port for retrieving sensitive configuration.
#[async_trait]
pub trait SecretManager: Send + Sync + 'static {
    /// Retrieve a secret by its key.
    async fn get_secret(&self, key: &str) -> Result<String, ContractError>;

    /// Store a secret.
    async fn set_secret(&self, key: &str, value: &str) -> Result<(), ContractError>;
}

/// A configuration loader port.
#[async_trait]
pub trait ConfigLoader: Send + Sync + 'static {
    /// The configuration type produced.
    type Config: Debug + Send + Sync + serde::de::DeserializeOwned + 'static;

    /// Load and return the configuration.
    async fn load(&self) -> Result<Self::Config, ContractError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Debug, Clone)]
    struct Item {
        id: String,
        name: String,
    }

    impl DomainEntity for Item {
        type Id = String;
        fn id(&self) -> &String {
            &self.id
        }
    }

    struct InMemoryRepo {
        store: Mutex<HashMap<String, Item>>,
    }

    impl InMemoryRepo {
        fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl Repository<Item, String> for InMemoryRepo {
        async fn save(&self, entity: &Item) -> Result<(), ContractError> {
            self.store
                .lock()
                .unwrap()
                .insert(entity.id().clone(), entity.clone());
            Ok(())
        }

        async fn find_by_id(&self, id: &String) -> Result<Option<Item>, ContractError> {
            Ok(self.store.lock().unwrap().get(id).cloned())
        }

        async fn delete(&self, id: &String) -> Result<(), ContractError> {
            self.store.lock().unwrap().remove(id);
            Ok(())
        }
    }

    struct InMemoryCache {
        store: Mutex<HashMap<String, String>>,
    }

    impl InMemoryCache {
        fn new() -> Self {
            Self {
                store: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl CachePort<String, String> for InMemoryCache {
        async fn get(&self, key: &String) -> Result<Option<String>, ContractError> {
            Ok(self.store.lock().unwrap().get(key).cloned())
        }

        async fn set(&self, key: String, value: String) -> Result<(), ContractError> {
            self.store.lock().unwrap().insert(key, value);
            Ok(())
        }

        async fn delete(&self, key: &String) -> Result<(), ContractError> {
            self.store.lock().unwrap().remove(key);
            Ok(())
        }
    }

    struct StubEventBus;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestEvent {
        msg: String,
    }

    impl DomainEvent for TestEvent {
        fn event_type(&self) -> &str {
            "TestEvent"
        }
    }

    #[async_trait]
    impl EventBus for StubEventBus {
        async fn publish<E: DomainEvent + Serialize + Send + Sync>(
            &self,
            _event: &E,
        ) -> Result<(), ContractError> {
            Ok(())
        }

        async fn subscribe(&self, _event_type: &str) -> Result<(), ContractError> {
            Ok(())
        }
    }

    // Traces to: FR-PHENO-004
    #[tokio::test]
    async fn repository_crud() {
        let repo = InMemoryRepo::new();
        let item = Item {
            id: "i-1".into(),
            name: "Widget".into(),
        };

        repo.save(&item).await.unwrap();
        let found = repo.find_by_id(&"i-1".into()).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Widget");

        assert!(repo.exists(&"i-1".into()).await.unwrap());

        repo.delete(&"i-1".into()).await.unwrap();
        assert!(!repo.exists(&"i-1".into()).await.unwrap());
    }

    // Traces to: FR-PHENO-004
    #[tokio::test]
    async fn cache_get_set_delete() {
        let cache = InMemoryCache::new();

        assert!(cache.get(&"k".into()).await.unwrap().is_none());
        cache.set("k".into(), "v".into()).await.unwrap();
        assert_eq!(cache.get(&"k".into()).await.unwrap().unwrap(), "v");
        assert!(cache.exists(&"k".into()).await.unwrap());

        cache.delete(&"k".into()).await.unwrap();
        assert!(!cache.exists(&"k".into()).await.unwrap());
    }

    // Traces to: FR-PHENO-004
    #[tokio::test]
    async fn event_bus_publish_subscribe() {
        let bus = StubEventBus;
        let event = TestEvent {
            msg: "hello".into(),
        };
        bus.publish(&event).await.unwrap();
        bus.subscribe("TestEvent").await.unwrap();
    }
}
