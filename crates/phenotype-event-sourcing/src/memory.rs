//! In-memory [`EventStore`](crate::store::EventStore) with SHA-256 hash chain.

use chrono::{DateTime, Utc};
use std::collections::BTreeMap;
use std::sync::RwLock;

use crate::error::{EventStoreError, Result};
use crate::hash;
use crate::store::{EventStore, JsonEnvelope};

/// Nested map: `entity_type` → `entity_id` → append-only event list.
pub struct InMemoryEventStore {
    events: RwLock<BTreeMap<String, BTreeMap<String, Vec<StoredEvent>>>>,
}

#[derive(Clone, Debug)]
struct StoredEvent {
    sequence: i64,
    hash: String,
    prev_hash: String,
    payload_json: serde_json::Value,
    actor: String,
    timestamp: DateTime<Utc>,
    id: uuid::Uuid,
}

impl InMemoryEventStore {
    pub fn new() -> Self {
        Self {
            events: RwLock::new(BTreeMap::new()),
        }
    }

    pub fn clear(&self) {
        if let Ok(mut g) = self.events.write() {
            g.clear();
        }
    }

    pub fn event_count(&self) -> usize {
        self.events
            .read()
            .map(|store| {
                store
                    .values()
                    .flat_map(|m| m.values())
                    .map(|v| v.len())
                    .sum()
            })
            .unwrap_or(0)
    }
}

impl Default for InMemoryEventStore {
    fn default() -> Self {
        Self::new()
    }
}

impl EventStore for InMemoryEventStore {
    fn append(&self, event: &JsonEnvelope, entity_type: &str, entity_id: &str) -> Result<i64> {
        let mut store = self
            .events
            .write()
            .map_err(|_| EventStoreError::StorageError("lock poisoned".into()))?;

        let entity_map = store
            .entry(entity_type.to_string())
            .or_insert_with(BTreeMap::new);
        let events = entity_map
            .entry(entity_id.to_string())
            .or_insert_with(Vec::new);

        let sequence = if events.is_empty() {
            1
        } else {
            events.last().unwrap().sequence + 1
        };
        let prev_hash = if events.is_empty() {
            "0".repeat(64)
        } else {
            events.last().unwrap().hash.clone()
        };

        let payload_json = event.payload.clone();

        let hash = hash::compute_hash(
            &event.id,
            event.timestamp,
            entity_type,
            &payload_json,
            &event.actor,
            &prev_hash,
        )?;

        events.push(StoredEvent {
            sequence,
            hash,
            prev_hash,
            payload_json,
            actor: event.actor.clone(),
            timestamp: event.timestamp,
            id: event.id,
        });

        Ok(sequence)
    }

    fn get_events(&self, entity_type: &str, entity_id: &str) -> Result<Vec<JsonEnvelope>> {
        let store = self
            .events
            .read()
            .map_err(|_| EventStoreError::StorageError("lock poisoned".into()))?;

        let entity_map = store
            .get(entity_type)
            .ok_or_else(|| EventStoreError::NotFound(format!("{entity_type}/{entity_id}")))?;
        let events = entity_map
            .get(entity_id)
            .ok_or_else(|| EventStoreError::NotFound(format!("{entity_type}/{entity_id}")))?;

        Ok(events
            .iter()
            .map(|se| JsonEnvelope {
                id: se.id,
                timestamp: se.timestamp,
                payload: se.payload_json.clone(),
                actor: se.actor.clone(),
                prev_hash: se.prev_hash.clone(),
                hash: se.hash.clone(),
                sequence: se.sequence,
            })
            .collect())
    }

    fn get_events_since(
        &self,
        entity_type: &str,
        entity_id: &str,
        sequence: i64,
    ) -> Result<Vec<JsonEnvelope>> {
        Ok(self
            .get_events(entity_type, entity_id)?
            .into_iter()
            .filter(|e| e.sequence > sequence)
            .collect())
    }

    fn get_events_by_range(
        &self,
        entity_type: &str,
        entity_id: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<JsonEnvelope>> {
        Ok(self
            .get_events(entity_type, entity_id)?
            .into_iter()
            .filter(|e| e.timestamp >= from && e.timestamp <= to)
            .collect())
    }

    fn get_latest_sequence(&self, entity_type: &str, entity_id: &str) -> Result<i64> {
        let store = self
            .events
            .read()
            .map_err(|_| EventStoreError::StorageError("lock poisoned".into()))?;

        let seq: Option<i64> = match store.get(entity_type) {
            Some(m) => m
                .get(entity_id)
                .and_then(|events: &Vec<StoredEvent>| events.last().map(|e| e.sequence)),
            None => None,
        };
        Ok(seq.unwrap_or(0))
    }

    fn verify_chain(&self, entity_type: &str, entity_id: &str) -> Result<()> {
        let store = self
            .events
            .read()
            .map_err(|_| EventStoreError::StorageError("lock poisoned".into()))?;

        let entity_map = store
            .get(entity_type)
            .ok_or_else(|| EventStoreError::NotFound(format!("{entity_type}/{entity_id}")))?;
        let events = entity_map
            .get(entity_id)
            .ok_or_else(|| EventStoreError::NotFound(format!("{entity_type}/{entity_id}")))?;

        let chain: Vec<(String, String)> = events
            .iter()
            .map(|e| (e.hash.clone(), e.prev_hash.clone()))
            .collect();

        hash::verify_chain(&chain)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn append_and_retrieve() {
        let store = InMemoryEventStore::new();
        let event = JsonEnvelope::new(
            json!({
                "value": 42,
                "name": "test"
            }),
            "user1",
        );
        let entity_id = "entity-1";

        let seq = store.append(&event, "TestEvent", entity_id).unwrap();
        assert_eq!(seq, 1);

        let retrieved = store.get_events("TestEvent", entity_id).unwrap();
        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].payload["value"], 42);
    }

    #[test]
    fn sequence_increments() {
        let store = InMemoryEventStore::new();
        let e1 = JsonEnvelope::new(json!({"value": 1, "name": "a"}), "user1");
        let e2 = JsonEnvelope::new(json!({"value": 2, "name": "b"}), "user1");

        let s1 = store.append(&e1, "Event", "entity-1").unwrap();
        let s2 = store.append(&e2, "Event", "entity-1").unwrap();

        assert_eq!(s1, 1);
        assert_eq!(s2, 2);
    }
}
