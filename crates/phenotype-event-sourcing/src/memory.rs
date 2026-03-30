//! In-memory event store.

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::EventSourcingError;
use crate::event::EventEnvelope;

/// Entity-level events collection (id -> events)
type EntityEvents<T> = HashMap<String, Vec<EventEnvelope<T>>>;

/// Aggregate-level events collection (entity_type -> EntityEvents)
type EventStoreInner<T> = HashMap<String, EntityEvents<T>>;

pub struct InMemoryEventStore<T> {
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
impl<T: Clone + Send + Sync + Serialize + DeserializeOwned + 'static> crate::store::EventStore<T>
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
            .entry(entity_type.to_string())
            .or_insert_with(|| Arc::new(RwLock::new(HashMap::new())))
            .clone();

        // Lock and update the entity's events
        let mut entity_events = aggregate_store.write().await;
        let seq = entity_events
            .entry(entity_id.to_string())
            .or_default()
            .len() as i64
            + 1;
        entity_events
            .entry(entity_id.to_string())
            .or_default()
            .push(event);
        Ok(seq)
    }

    async fn get_events(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Vec<EventEnvelope<T>>, EventSourcingError> {
        let aggregate_store = match self.events.get(entity_type) {
            Some(store) => store.clone(),
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
        let aggregate_store = match self.events.get(entity_type) {
            Some(store) => store.clone(),
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
    use crate::EventStore;

    #[tokio::test]
    async fn append_and_retrieve() {
        let store = InMemoryEventStore::<String>::new();
        let event = EventEnvelope::new("test".to_string(), "actor1".into());
        let seq = store.append("user", "123", event.clone()).await.unwrap();
        assert_eq!(seq, 1);
        let events = store.get_events("user", "123").await.unwrap();
        assert_eq!(events.len(), 1);
    }
}
