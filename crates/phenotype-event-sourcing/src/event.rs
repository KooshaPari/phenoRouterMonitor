//! Event types for phenotype-event-sourcing.

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
    pub entity_id: String,
    pub payload: T,
    pub actor: String,
    pub prev_hash: String,
    pub hash: String,
}

impl<T: Clone + Serialize> EventEnvelope<T> {
    pub fn new(
        entity_type: impl Into<String>,
        entity_id: impl Into<String>,
        payload: T,
        actor: impl Into<String>,
    ) -> Self {
        let id = Uuid::new_v4();
        let timestamp = Utc::now();
        let entity_type = entity_type.into();
        let entity_id = entity_id.into();
        let actor_str = actor.into();
        let sequence = 1;
        let prev_hash = ZERO_HASH.to_string();
        let payload_json = serde_json::to_value(&payload).unwrap_or_default();
        let hash = compute_hash(
            &id,
            timestamp,
            &entity_type,
            &entity_id,
            &payload_json,
            &actor_str,
            &prev_hash,
        )
        .unwrap_or_else(|_| "error".to_string());

        Self {
            id,
            sequence,
            timestamp,
            entity_type,
            entity_id,
            payload,
            actor: actor_str,
            prev_hash,
            hash,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_event_envelope() {
        let event = EventEnvelope::new("users", "user-123", "UserCreated", "system");
        assert_eq!(event.entity_type, "users");
        assert_eq!(event.entity_id, "user-123");
        assert_eq!(event.payload, "UserCreated");
        assert_eq!(event.actor, "system");
        assert_eq!(event.prev_hash, ZERO_HASH);
        assert!(!event.hash.is_empty());
        assert_eq!(event.sequence, 1);
    }

    #[test]
    fn event_roundtrip_json() {
        let event = EventEnvelope::new("orders", "order-456", "OrderPlaced", "user-789");
        let json = serde_json::to_string(&event).unwrap();
        let parsed: EventEnvelope<String> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.entity_type, event.entity_type);
        assert_eq!(parsed.entity_id, event.entity_id);
        assert_eq!(parsed.payload, event.payload);
        assert_eq!(parsed.actor, event.actor);
        assert_eq!(parsed.hash, event.hash);
    }
}
