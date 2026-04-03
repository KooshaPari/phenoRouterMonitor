//! In-memory event store implementation.

use std::collections::BTreeMap;
use std::sync::RwLock;

use crate::error::{EventSourcingError, Result};
use crate::event::EventEnvelope;
use crate::hash::{compute_hash, ZERO_HASH};
use crate::store::EventStore;

pub struct InMemoryEventStore {
    events: RwLock<BTreeMap<String, BTreeMap<String, Vec<StoredEvent>>>>,
}

#[derive(Clone, Debug)]
struct StoredEvent {
    sequence: i64,
    hash: String,
    prev_hash: String,
    payload_json: serde_json::Value,
    entity_type: String,
    entity_id: String,
    actor: String,
    timestamp: chrono::DateTime<chrono::Utc>,
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

    fn get_entity_key(entity_type: &str, entity_id: &str) -> String {
        format!("{}:{}", entity_type, entity_id)
    }
}

impl Default for InMemoryEventStore {
    fn default() -> Self {
        Self::new()
    }
}

impl EventStore for InMemoryEventStore {
    fn append(&self, event: &EventEnvelope<serde_json::Value>) -> Result<i64> {
        let key = Self::get_entity_key(&event.entity_type, &event.entity_id);
        let mut store = self
            .events
            .write()
            .map_err(|_| EventSourcingError::Internal("lock poisoned".into()))?;

        let entity_events = store.entry(key.clone()).or_insert_with(BTreeMap::new);
        let events = entity_events
            .entry(event.entity_id.clone())
            .or_insert_with(Vec::new);

        let sequence = if events.is_empty() {
            1
        } else {
            events.last().unwrap().sequence + 1
        };

        let prev_hash = if events.is_empty() {
            ZERO_HASH.to_string()
        } else {
            events.last().unwrap().hash.clone()
        };

        let hash = compute_hash(
            &event.id,
            event.timestamp,
            &event.entity_type,
            &event.entity_id,
            &event.payload,
            &event.actor,
            &prev_hash,
        )?;

        events.push(StoredEvent {
            sequence,
            hash,
            prev_hash,
            payload_json: event.payload.clone(),
            entity_type: event.entity_type.clone(),
            entity_id: event.entity_id.clone(),
            actor: event.actor.clone(),
            timestamp: event.timestamp,
            id: event.id,
        });

        Ok(sequence)
    }

    fn get_events(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Vec<EventEnvelope<serde_json::Value>>> {
        let key = Self::get_entity_key(entity_type, entity_id);
        let store = self
            .events
            .read()
            .map_err(|_| EventSourcingError::Internal("lock poisoned".into()))?;

        let events = store
            .get(&key)
            .and_then(|m| m.get(entity_id))
            .ok_or_else(|| {
                EventSourcingError::EntityNotFound(format!("{}/{}", entity_type, entity_id))
            })?;

        Ok(events
            .iter()
            .map(|se| EventEnvelope {
                id: se.id,
                sequence: se.sequence,
                timestamp: se.timestamp,
                entity_type: se.entity_type.clone(),
                entity_id: se.entity_id.clone(),
                payload: se.payload_json.clone(),
                actor: se.actor.clone(),
                prev_hash: se.prev_hash.clone(),
                hash: se.hash.clone(),
            })
            .collect())
    }

    fn get_latest_sequence(&self, entity_type: &str, entity_id: &str) -> Result<i64> {
        let key = Self::get_entity_key(entity_type, entity_id);
        let store = self
            .events
            .read()
            .map_err(|_| EventSourcingError::Internal("lock poisoned".into()))?;

        Ok(store
            .get(&key)
            .and_then(|m| m.get(entity_id))
            .and_then(|events| events.last().map(|e| e.sequence))
            .unwrap_or(0))
    }

    fn verify_chain(&self, entity_type: &str, entity_id: &str) -> Result<()> {
        let key = Self::get_entity_key(entity_type, entity_id);
        let store = self
            .events
            .read()
            .map_err(|_| EventSourcingError::Internal("lock poisoned".into()))?;

        let events = store
            .get(&key)
            .and_then(|m| m.get(entity_id))
            .ok_or_else(|| {
                EventSourcingError::EntityNotFound(format!("{}/{}", entity_type, entity_id))
            })?;

        let chain: Vec<(String, String)> = events
            .iter()
            .map(|e| (e.hash.clone(), e.prev_hash.clone()))
            .collect();

        crate::hash::verify_chain(&chain).map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn append_and_retrieve() {
        let store = InMemoryEventStore::new();
        let event = EventEnvelope::new(
            "orders",
            "order-1",
            serde_json::json!({"value": 42, "name": "test"}),
            "user1",
        );

        let seq = store.append(&event).unwrap();
        assert_eq!(seq, 1);

        let retrieved = store.get_events("orders", "order-1").unwrap();
        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].payload["value"], 42);
    }

    #[test]
    fn sequence_increments() {
        let store = InMemoryEventStore::new();
        let e1 = EventEnvelope::new(
            "orders",
            "order-1",
            serde_json::json!({"value": 1}),
            "user1",
        );
        let e2 = EventEnvelope::new(
            "orders",
            "order-1",
            serde_json::json!({"value": 2}),
            "user1",
        );

        let s1 = store.append(&e1).unwrap();
        let s2 = store.append(&e2).unwrap();

        assert_eq!(s1, 1);
        assert_eq!(s2, 2);
    }

    #[test]
    fn verify_chain() {
        let store = InMemoryEventStore::new();
        let e1 = EventEnvelope::new(
            "users",
            "user-1",
            serde_json::json!({"action": "created"}),
            "system",
        );
        let e2 = EventEnvelope::new(
            "users",
            "user-1",
            serde_json::json!({"action": "updated"}),
            "system",
        );

        store.append(&e1).unwrap();
        store.append(&e2).unwrap();

        store.verify_chain("users", "user-1").unwrap();
    }

    #[test]
    fn event_count() {
        let store = InMemoryEventStore::new();
        assert_eq!(store.event_count(), 0);

        let e1 = EventEnvelope::new("orders", "o1", serde_json::json!({}), "u1");
        let e2 = EventEnvelope::new("orders", "o2", serde_json::json!({}), "u1");

        store.append(&e1).unwrap();
        store.append(&e2).unwrap();

        assert_eq!(store.event_count(), 2);
    }
}
