//! Event types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::hash::{compute_hash, ZERO_HASH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope<T> {
    pub id: Uuid,
    pub sequence: i64,
    pub timestamp: DateTime<Utc>,
    pub entity_type: String,
    pub payload: T,
    pub actor: String,
    pub prev_hash: String,
    pub hash: String,
}

impl<T: Clone + Serialize> EventEnvelope<T> {
    pub fn new(payload: T, actor: String) -> Self {
        let id = Uuid::new_v4();
        let timestamp = Utc::now();
        let entity_type = "Event".to_string();
        let sequence = 1;
        let prev_hash = ZERO_HASH.to_string();
        let hash = compute_hash(
            &id, timestamp, &entity_type,
            &serde_json::to_value(&payload).unwrap_or_default(),
            &actor, &prev_hash,
        ).unwrap_or_else(|_| "error".to_string());
        Self { id, sequence, timestamp, entity_type, payload, actor, prev_hash, hash }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_event_envelope() {
        let event = EventEnvelope::new("test".to_string(), "actor1".to_string());
        assert_eq!(event.actor, "actor1");
        assert_eq!(event.prev_hash, ZERO_HASH);
        assert!(!event.hash.is_empty());
    }

    #[test]
    fn event_roundtrip_json() {
        let event = EventEnvelope::new("test".to_string(), "actor".to_string());
        let json = serde_json::to_string(&event).unwrap();
        let parsed: EventEnvelope<String> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.actor, event.actor);
        assert_eq!(parsed.payload, event.payload);
    }
}
