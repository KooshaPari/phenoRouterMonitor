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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // --- Mock Repository ---

    #[derive(Debug, Clone)]
    struct MockRepo {
        store: std::sync::Arc<Mutex<HashMap<String, String>>>,
    }

    impl MockRepo {
        fn new() -> Self {
            Self {
                store: std::sync::Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[derive(Debug, thiserror::Error)]
    #[error("{0}")]
    struct MockError(String);

    #[async_trait]
    impl Repository<String> for MockRepo {
        type Error = MockError;

        async fn get(&self, id: &str) -> Result<Option<String>, Self::Error> {
            Ok(self.store.lock().unwrap().get(id).cloned())
        }

        async fn save(&self, id: &str, entity: &String) -> Result<(), Self::Error> {
            self.store
                .lock()
                .unwrap()
                .insert(id.to_string(), entity.clone());
            Ok(())
        }

        async fn delete(&self, id: &str) -> Result<(), Self::Error> {
            self.store.lock().unwrap().remove(id);
            Ok(())
        }

        async fn list(&self, offset: usize, limit: usize) -> Result<Vec<String>, Self::Error> {
            let store = self.store.lock().unwrap();
            let items: Vec<String> = store.values().cloned().collect();
            Ok(items.into_iter().skip(offset).take(limit).collect())
        }
    }

    // Traces to: FR-PHENO-020
    #[tokio::test]
    async fn repository_save_and_get() {
        let repo = MockRepo::new();
        repo.save("1", &"hello".to_string()).await.unwrap();
        let val = repo.get("1").await.unwrap();
        assert_eq!(val, Some("hello".to_string()));
    }

    // Traces to: FR-PHENO-020
    #[tokio::test]
    async fn repository_get_missing_returns_none() {
        let repo = MockRepo::new();
        let val = repo.get("nonexistent").await.unwrap();
        assert!(val.is_none());
    }

    // Traces to: FR-PHENO-020
    #[tokio::test]
    async fn repository_delete() {
        let repo = MockRepo::new();
        repo.save("1", &"data".to_string()).await.unwrap();
        repo.delete("1").await.unwrap();
        assert!(repo.get("1").await.unwrap().is_none());
    }

    // Traces to: FR-PHENO-020
    #[tokio::test]
    async fn repository_exists_default_impl() {
        let repo = MockRepo::new();
        assert!(!repo.exists("1").await.unwrap());
        repo.save("1", &"val".to_string()).await.unwrap();
        assert!(repo.exists("1").await.unwrap());
    }

    // Traces to: FR-PHENO-020
    #[tokio::test]
    async fn repository_list_with_pagination() {
        let repo = MockRepo::new();
        for i in 0..5 {
            repo.save(&i.to_string(), &format!("item-{i}"))
                .await
                .unwrap();
        }
        let page = repo.list(0, 3).await.unwrap();
        assert_eq!(page.len(), 3);
    }

    // --- Mock EventPublisher ---

    struct MockPublisher {
        published: Mutex<Vec<(String, serde_json::Value)>>,
    }

    impl MockPublisher {
        fn new() -> Self {
            Self {
                published: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl EventPublisher for MockPublisher {
        type Error = MockError;

        async fn publish(
            &self,
            topic: &str,
            payload: &serde_json::Value,
        ) -> Result<(), Self::Error> {
            self.published
                .lock()
                .unwrap()
                .push((topic.to_string(), payload.clone()));
            Ok(())
        }
    }

    // Traces to: FR-PHENO-021
    #[tokio::test]
    async fn event_publisher_publishes() {
        let pub_ = MockPublisher::new();
        let payload = serde_json::json!({"key": "value"});
        pub_.publish("topic.a", &payload).await.unwrap();
        let events = pub_.published.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0, "topic.a");
    }

    // --- Mock Notifier ---

    struct MockNotifier {
        sent: Mutex<Vec<(String, String, String)>>,
    }

    impl MockNotifier {
        fn new() -> Self {
            Self {
                sent: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl Notifier for MockNotifier {
        type Error = MockError;

        async fn notify(
            &self,
            recipient: &str,
            subject: &str,
            body: &str,
        ) -> Result<(), Self::Error> {
            self.sent.lock().unwrap().push((
                recipient.to_string(),
                subject.to_string(),
                body.to_string(),
            ));
            Ok(())
        }
    }

    // Traces to: FR-PHENO-022
    #[tokio::test]
    async fn notifier_sends() {
        let n = MockNotifier::new();
        n.notify("user@test.com", "Alert", "Server down")
            .await
            .unwrap();
        let sent = n.sent.lock().unwrap();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].0, "user@test.com");
        assert_eq!(sent[0].1, "Alert");
    }
}
