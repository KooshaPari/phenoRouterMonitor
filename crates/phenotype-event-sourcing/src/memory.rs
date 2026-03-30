//! In-memory [`EventStore`](crate::store::EventStore) implementation.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

use crate::error::EventSourcingError;
use crate::event::EventEnvelope;
use crate::store::EventStore;

/// Thread-safe storage for a single entity's events.
type EntityEvents<T> = HashMap<String, Vec<EventEnvelope<T>>>;

/// Storage mapping entity types to their entities.
type EventStoreInner<T> = HashMap<String, EntityEvents<T>>;

/// In-memory event store.
#[allow(clippy::type_complexity)]
pub struct InMemoryEventStore<T> {
    events: Arc<RwLock<EventStoreInner<T>>>,
}

impl<T> InMemoryEventStore<T> {
    /// Creates a new empty event store.
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<T> Default for InMemoryEventStore<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<T: Clone + Send + Sync + Serialize + DeserializeOwned + 'static> EventStore<T>
    for InMemoryEventStore<T>
{
    async fn append(
        &self,
        entity_type: &str,
        entity_id: &str,
        event: EventEnvelope<T>,
    ) -> Result<i64, EventSourcingError> {
        // Get or create the aggregate store for this entity type
        let aggregate_store = self
            .events
            .write()
            .await
            .entry(entity_type.to_string())
            .or_insert_with(HashMap::new)
            .clone();

        // Lock and update the entity's events
        let mut entity_events = aggregate_store.write().await;
        let seq = entity_events
            .entry(entity_id.to_string())
            .or_insert_with(Vec::new)
            .len() as i64
            + 1;
        entity_events
            .entry(entity_id.to_string())
            .or_insert_with(Vec::new)
            .push(event);
        Ok(seq)
    }

    async fn get_events(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Vec<EventEnvelope<T>>, EventSourcingError> {
        let store = self.events.read().await;
        let aggregate_store = match store.get(entity_type) {
            Some(s) => s.clone(),
            None => return Ok(Vec::new()),
        };
        let entity_events = aggregate_store.read().await;
        Ok(entity_events.get(entity_id).cloned().unwrap_or_default())
    }

    async fn get_sequence(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<i64, EventSourcingError> {
        let store = self.events.read().await;
        let aggregate_store = match store.get(entity_type) {
            Some(s) => s.clone(),
            None => return Ok(0),
        };
        let entity_events = aggregate_store.read().await;
        Ok(entity_events
            .get(entity_id)
            .map(|v| v.len() as i64)
            .unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn append_and_retrieve() {
        let store = InMemoryEventStore::<String>::new();
        let event = EventEnvelope::new("test".to_string(), "actor1".into());
        let seq = store.append("user", "123", event.clone()).await.unwrap();
        assert_eq!(seq, 1);
        let events = store.get_events("user", "123").await.unwrap();
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn sequence_increments() {
        let store = InMemoryEventStore::<String>::new();
        let e1 = EventEnvelope::new("event1".to_string(), "actor1".into());
        let e2 = EventEnvelope::new("event2".to_string(), "actor1".into());

        let s1 = store.append("entity", "id1", e1).await.unwrap();
        let s2 = store.append("entity", "id1", e2).await.unwrap();

        assert_eq!(s1, 1);
        assert_eq!(s2, 2);
    }
}
