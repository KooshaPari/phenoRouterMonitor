//! In-memory event store

use crate::{EventEnvelope, Result, hash::{ZERO_HASH, compute_hash}};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::RwLock;
use crate::store::EventStore;

pub struct InMemoryEventStore {
    events: RwLock<HashMap<String, HashMap<String, Vec<EventEnvelope>>>>,
}

impl InMemoryEventStore {
    pub fn new() -> Self {
        Self { events: RwLock::new(HashMap::new()) }
    }
}

impl Default for InMemoryEventStore {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl EventStore for InMemoryEventStore {
    async fn append(&self, entity_type: &str, entity_id: &str, event: EventEnvelope) -> Result<EventEnvelope> {
        let mut store = self.events.write().unwrap();
        let entity_map = store.entry(entity_type.to_string()).or_insert_with(HashMap::new);
        let events = entity_map.entry(entity_id.to_string()).or_insert_with(Vec::new);
        let sequence = if events.is_empty() { 1 } else { events.last().unwrap().metadata.sequence + 1 };
        let prev_hash = if events.is_empty() { ZERO_HASH.to_string() } else { events.last().unwrap().metadata.hash.clone() };
        let mut new_event = event;
        new_event.metadata.sequence = sequence;
        new_event.metadata.prev_hash = prev_hash;
        let hash_input = format!("{:?}{:?}{}", new_event.metadata.id, new_event.metadata.sequence, new_event.payload);
        new_event.metadata.hash = compute_hash(&hash_input);
        events.push(new_event.clone());
        Ok(new_event)
    }

    async fn get_events(&self, entity_type: &str, entity_id: &str) -> Result<Vec<EventEnvelope>> {
        let store = self.events.read().unwrap();
        Ok(store.get(entity_type).and_then(|m| m.get(entity_id)).cloned().unwrap_or_default())
    }

    async fn get_events_since(&self, entity_type: &str, entity_id: &str, sequence: i64) -> Result<Vec<EventEnvelope>> {
        let events = self.get_events(entity_type, entity_id).await?;
        Ok(events.into_iter().filter(|e| e.metadata.sequence > sequence).collect())
    }

    async fn get_events_by_range(&self, entity_type: &str, entity_id: &str, from: DateTime<Utc>, to: DateTime<Utc>) -> Result<Vec<EventEnvelope>> {
        let events = self.get_events(entity_type, entity_id).await?;
        Ok(events.into_iter().filter(|e| e.metadata.timestamp >= from && e.metadata.timestamp <= to).collect())
    }

    async fn get_latest_sequence(&self, entity_type: &str, entity_id: &str) -> Result<i64> {
        let store = self.events.read().unwrap();
        Ok(store.get(entity_type).and_then(|m| m.get(entity_id)).and_then(|e| e.last()).map(|e| e.metadata.sequence).unwrap_or(0))
    }

    async fn verify_chain(&self, entity_type: &str, entity_id: &str) -> Result<bool> {
        let events = self.get_events(entity_type, entity_id).await?;
        let mut expected_prev = ZERO_HASH.to_string();
        for event in &events {
            if event.metadata.prev_hash != expected_prev {
                return Ok(false);
            }
            expected_prev = event.metadata.hash.clone();
        }
        Ok(true)
    }
}
