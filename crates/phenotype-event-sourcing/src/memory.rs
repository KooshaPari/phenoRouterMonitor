#![allow(clippy::type_complexity)]
//! In-memory event store.

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::EventSourcingError;
use crate::event::EventEnvelope;

pub struct InMemoryEventStore<T> {
    events: Arc<RwLock<HashMap<String, HashMap<String, Vec<EventEnvelope<T>>>>>>,
}

impl<T> Default for InMemoryEventStore<T> {
    fn default() -> Self {
        Self {
            events: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<T> InMemoryEventStore<T> {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl<T: Clone + Send + Sync + Serialize + DeserializeOwned + 'static> crate::EventStore<T>
    for InMemoryEventStore<T>
{
    async fn append(
        &self,
        entity_type: &str,
        entity_id: &str,
        event: EventEnvelope<T>,
    ) -> Result<i64, EventSourcingError> {
        let mut store = self.events.write().await;
        let entity_events = store.entry(entity_type.to_string()).or_default();
        let seq = entity_events
            .entry(entity_id.to_string())
            .or_default()
            .len() as i64
            + 1;
        entity_events
            .get_mut(&entity_id.to_string())
            .unwrap()
            .push(event);
        Ok(seq)
    }

    async fn get_events(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Vec<EventEnvelope<T>>, EventSourcingError> {
        let store = self.events.read().await;
        Ok(store
            .get(entity_type)
            .and_then(|e| e.get(entity_id).cloned())
            .unwrap_or_default())
    }

    async fn get_sequence(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<i64, EventSourcingError> {
        let store = self.events.read().await;
        Ok(store
            .get(entity_type)
            .and_then(|e| e.get(entity_id))
            .map(|v| v.len() as i64)
            .unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EventStore as _;

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
