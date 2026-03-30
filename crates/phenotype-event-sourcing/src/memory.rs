//! In-memory event store for tests and development.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::RwLock;

use crate::error::{EventSourcingError, EventStoreError, Result};
use crate::event::EventEnvelope;
use crate::hash;
use crate::store::EventStore;

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
        self.events.write().unwrap().clear();
    }

    pub fn event_count(&self) -> usize {
        self.events
            .read()
            .unwrap()
            .values()
            .flat_map(|m| m.values())
            .map(|v| v.len())
            .sum()
    }
}

impl Default for InMemoryEventStore {
    fn default() -> Self {
        Self::new()
    }
}

impl EventStore for InMemoryEventStore {
    fn append<T: Serialize + for<'de> Deserialize<'de>>(
        &self,
        event: &EventEnvelope<T>,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<i64> {
        let mut store = self
            .events
            .write()
            .map_err(|_| EventStoreError::StorageError("lock poisoned".into()))?;

        let entity_map = store
            .entry(entity_type.to_string())
            .or_insert_with(BTreeMap::new);
        let events = entity_map.entry(entity_id.to_string()).or_insert_with(Vec::new);

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

        let payload_json = serde_json::to_value(&event.payload)?;

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

    fn get_events<T: Serialize + for<'de> Deserialize<'de>>(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Vec<EventEnvelope<T>>> {
        let store = self
            .events
            .read()
            .map_err(|_| EventStoreError::StorageError("lock poisoned".into()))?;

        let events = store
            .get(entity_type)
            .and_then(|m| m.get(entity_id))
            .ok_or_else(|| EventStoreError::NotFound(format!("{entity_type}/{entity_id}")))?;

        events
            .iter()
            .map(|se| {
                let payload: T = serde_json::from_value(se.payload_json.clone())?;
                Ok(EventEnvelope {
                    id: se.id,
                    timestamp: se.timestamp,
                    payload,
                    actor: se.actor.clone(),
                    prev_hash: se.prev_hash.clone(),
                    hash: se.hash.clone(),
                    sequence: se.sequence,
                })
            })
            .collect::<Result<Vec<_>, EventSourcingError>>()
    }

    fn get_events_since<T: Serialize + for<'de> Deserialize<'de>>(
        &self,
        entity_type: &str,
        entity_id: &str,
        sequence: i64,
    ) -> Result<Vec<EventEnvelope<T>>> {
        let store = self
            .events
            .read()
            .map_err(|_| EventStoreError::StorageError("lock poisoned".into()))?;

        let events = store
            .get(entity_type)
            .and_then(|m| m.get(entity_id))
            .ok_or_else(|| EventStoreError::NotFound(format!("{entity_type}/{entity_id}")))?;

        events
            .iter()
            .filter(|se| se.sequence > sequence)
            .map(|se| {
                let payload: T = serde_json::from_value(se.payload_json.clone())?;
                Ok(EventEnvelope {
                    id: se.id,
                    timestamp: se.timestamp,
                    payload,
                    actor: se.actor.clone(),
                    prev_hash: se.prev_hash.clone(),
                    hash: se.hash.clone(),
                    sequence: se.sequence,
                })
            })
            .collect::<Result<Vec<_>, EventSourcingError>>()
    }

    fn get_events_by_range<T: Serialize + for<'de> Deserialize<'de>>(
        &self,
        entity_type: &str,
        entity_id: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<EventEnvelope<T>>> {
        let store = self
            .events
            .read()
            .map_err(|_| EventStoreError::StorageError("lock poisoned".into()))?;

        let events = store
            .get(entity_type)
            .and_then(|m| m.get(entity_id))
            .ok_or_else(|| EventStoreError::NotFound(format!("{entity_type}/{entity_id}")))?;

        events
            .iter()
            .filter(|se| se.timestamp >= from && se.timestamp <= to)
            .map(|se| {
                let payload: T = serde_json::from_value(se.payload_json.clone())?;
                Ok(EventEnvelope {
                    id: se.id,
                    timestamp: se.timestamp,
                    payload,
                    actor: se.actor.clone(),
                    prev_hash: se.prev_hash.clone(),
                    hash: se.hash.clone(),
                    sequence: se.sequence,
                })
            })
            .collect::<Result<Vec<_>, EventSourcingError>>()
    }

    fn get_latest_sequence(&self, entity_type: &str, entity_id: &str) -> Result<i64> {
        let store = self
            .events
            .read()
            .map_err(|_| EventStoreError::StorageError("lock poisoned".into()))?;

        Ok(store
            .get(entity_type)
            .and_then(|m| m.get(entity_id))
            .and_then(|events| events.last().map(|e| e.sequence))
            .unwrap_or(0))
    }

    fn verify_chain(&self, entity_type: &str, entity_id: &str) -> Result<()> {
        let store = self
            .events
            .read()
            .map_err(|_| EventStoreError::StorageError("lock poisoned".into()))?;

        let events = store
            .get(entity_type)
            .and_then(|m| m.get(entity_id))
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
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestPayload {
        value: i32,
        name: String,
    }

    #[test]
    fn append_and_retrieve() {
        let store = InMemoryEventStore::new();
        let payload = TestPayload {
            value: 42,
            name: "test".to_string(),
        };
        let event = EventEnvelope::new(payload.clone(), "user1");
        let entity_id = "entity-1";

        let seq = store.append(&event, "TestEvent", entity_id).unwrap();
        assert_eq!(seq, 1);

        let retrieved = store.get_events::<TestPayload>("TestEvent", entity_id).unwrap();
        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].payload.value, 42);
    }

    #[test]
    fn sequence_increments() {
        let store = InMemoryEventStore::new();
        let e1 = EventEnvelope::new(
            TestPayload {
                value: 1,
                name: "a".to_string(),
            },
            "user1",
        );
        let e2 = EventEnvelope::new(
            TestPayload {
                value: 2,
                name: "b".to_string(),
            },
            "user1",
        );

        let s1 = store.append(&e1, "Event", "entity-1").unwrap();
        let s2 = store.append(&e2, "Event", "entity-1").unwrap();

        assert_eq!(s1, 1);
        assert_eq!(s2, 2);
    }
}
