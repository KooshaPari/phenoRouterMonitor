//! In-memory event store.

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

use crate::error::{EventSourcingError, Result};
use crate::event::EventEnvelope;
use crate::store::EventStore;

pub struct InMemoryEventStore<T> {
    events: Arc<RwLock<HashMap<String, HashMap<String, Vec<EventEnvelope<T>>>>>,
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
impl<T: Send + Sync + Serialize + DeserializeOwned> EventStore for InMemoryEventStore<T> {
    async fn append(
        &self,
        entity_type: &str,
        entity_id: &str,
        event: EventEnvelope<T>,
    ) -> std::result::Result<EventEnvelope<T>, crate::Error> {
        let mut store = self.events.write().await;
        let entity_map = store.entry(entity_type.to_string()).or_insert_with(HashMap::new);
        let events = entity_map.entry(entity_id.to_string()).or_insert_with(Vec::new);
        let sequence = if events.is_empty() { 1 } else { events.len() as i64 + 1 };
        events.push(event.clone());
        Ok(event)
    }

    async fn get_events(&self, entity_type: &str, entity_id: &str) -> std::result::Result<Vec<EventEnvelope<T>>, crate::Error> {
        let store = self.events.read().await;
        Ok(store.get(entity_type).and_then(|m| m.get(entity_id)).cloned().unwrap_or_default())
    }

    async fn get_events_since(&self, entity_type: &str, entity_id: &str, sequence: i64) -> std::result::Result<Vec<EventEnvelope<T>>, crate::Error> {
        let events = self.get_events(entity_type, entity_id).await?;
        Ok(events.into_iter().filter(|e| e.metadata.sequence > sequence).collect())
    }

    async fn get_events_by_range(&self, entity_type: &str, entity_id: &str, from: chrono::DateTime<chrono::Utc>, to: chrono::DateTime<chrono::Utc>) -> std::result::Result<Vec<EventEnvelope<T>>, crate::Error> {
        let events = self.get_events(entity_type, entity_id).await?;
        Ok(events.into_iter().filter(|e| e.metadata.timestamp >= from && e.metadata.timestamp <= to).collect())
    }

    async fn get_latest_sequence(&self, entity_type: &str, entity_id: &str) -> std::result::Result<i64, crate::Error> {
        let store = self.events.read().await;
        Ok(store.get(entity_type).and_then(|m| m.get(entity_id)).map(|e| e.len() as i64).unwrap_or(0))
    }

    async fn verify_chain(&self, entity_type: &str, entity_id: &str) -> std::result::Result<bool, crate::Error> {
        Ok(true)
    }
}
