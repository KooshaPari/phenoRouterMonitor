use std::sync::{Arc, RwLock};

use crate::error::EventSourcingError;
use crate::event::{Event, EventEnvelope};
use crate::hash::compute_event_hash;
use crate::store::EventStore;
use crate::Result;

#[derive(Debug, Clone)]
struct StoredEnvelope {
    event: Event,
    sequence: i64,
    timestamp: chrono::DateTime<chrono::Utc>,
    hash: String,
    id: uuid::Uuid,
}

#[derive(Debug, Default)]
struct StoreInner {
    events: Vec<StoredEnvelope>,
}

#[derive(Debug, Default, Clone)]
pub struct MemoryEventStore {
    inner: Arc<RwLock<StoreInner>>,
}

impl MemoryEventStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl EventStore for MemoryEventStore {
    fn append(&self, event: Event) -> Result<EventEnvelope> {
        let mut store = self.inner.write().map_err(|e| EventSourcingError::Store(e.to_string()))?;
        let sequence = (store.events.len() as i64) + 1;
        let previous_hash = store.events.last().map(|e| e.hash.clone()).unwrap_or_default();
        let hash = compute_event_hash(
            &event.event_type,
            &event.aggregate_id,
            &event.payload,
            sequence,
            &previous_hash,
        )?;
        let envelope = EventEnvelope::new(event.clone(), sequence, hash.clone());
        store.events.push(StoredEnvelope {
            event,
            sequence,
            timestamp: envelope.timestamp,
            hash,
            id: envelope.id,
        });
        Ok(envelope)
    }

    fn get_events(&self, aggregate_id: &str) -> Result<Vec<EventEnvelope>> {
        let store = self.inner.read().map_err(|e| EventSourcingError::Store(e.to_string()))?;
        Ok(store.events.iter()
            .filter(|e| e.event.aggregate_id == aggregate_id)
            .map(|e| EventEnvelope {
                event: e.event.clone(),
                sequence: e.sequence,
                timestamp: e.timestamp,
                hash: e.hash.clone(),
                id: e.id,
            })
            .collect())
    }

    fn get_event_by_sequence(&self, sequence: i64) -> Result<EventEnvelope> {
        let store = self.inner.read().map_err(|e| EventSourcingError::Store(e.to_string()))?;
        let stored = store.events.get(sequence as usize - 1)
            .ok_or_else(|| EventSourcingError::EventNotFound(format!("sequence {}", sequence)))?;
        Ok(EventEnvelope {
            event: stored.event.clone(),
            sequence: stored.sequence,
            timestamp: stored.timestamp,
            hash: stored.hash.clone(),
            id: stored.id,
        })
    }

    fn get_last_sequence(&self) -> Result<i64> {
        let store = self.inner.read().map_err(|e| EventSourcingError::Store(e.to_string()))?;
        Ok(store.events.len() as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_and_retrieve() {
        let store = MemoryEventStore::new();
        let event = Event::new("test", "agg-1", serde_json::json!({}));
        let envelope = store.append(event).unwrap();
        assert_eq!(envelope.sequence, 1);

        let events = store.get_events("agg-1").unwrap();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn test_get_event_by_sequence() {
        let store = MemoryEventStore::new();
        store.append(Event::new("test", "agg-1", serde_json::json!({}))).unwrap();
        let event = store.get_event_by_sequence(1).unwrap();
        assert_eq!(event.sequence, 1);
    }

    #[test]
    fn test_get_last_sequence() {
        let store = MemoryEventStore::new();
        assert_eq!(store.get_last_sequence().unwrap(), 0);
        store.append(Event::new("test", "agg-1", serde_json::json!({}))).unwrap();
        assert_eq!(store.get_last_sequence().unwrap(), 1);
    }
}
