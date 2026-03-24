//! Generic event envelope with SHA-256 hash chain support.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Generic event envelope that works with any serializable event type.
///
/// The event is stored in a hash chain where each event includes:
/// - A unique ID
/// - A timestamp
/// - An event type discriminator (as a string)
/// - The serialized event payload
/// - The hash of the previous event in the chain
/// - The hash of this event (computed by the store)
/// - A monotonically increasing sequence number
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope<T: Serialize> {
    /// Unique event ID
    pub id: Uuid,

    /// Event timestamp
    pub timestamp: DateTime<Utc>,

    /// The actual event payload (generic type)
    pub payload: T,

    /// Actor who triggered this event (e.g., user ID, system name)
    pub actor: String,

    /// Hash of the previous event in the chain (32 bytes, hex-encoded for storage)
    pub prev_hash: String,

    /// Hash of this event (computed by store, hex-encoded, 32 bytes)
    pub hash: String,

    /// Monotonically increasing sequence number (per entity)
    pub sequence: i64,
}

impl<T: Serialize> EventEnvelope<T> {
    /// Create a new event envelope.
    /// Hash and sequence will be assigned by the store.
    pub fn new(payload: T, actor: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            payload,
            actor: actor.into(),
            prev_hash: "0".repeat(64), // Initial zero hash (32 bytes = 64 hex chars)
            hash: "".to_string(),      // Will be filled by store
            sequence: 0,               // Will be filled by store
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_event_envelope() {
        #[derive(Serialize)]
        struct TestPayload {
            value: i32,
        }

        let payload = TestPayload { value: 42 };
        let event = EventEnvelope::new(payload, "user-123");

        assert!(!event.id.is_nil());
        assert_eq!(event.actor, "user-123");
        assert_eq!(event.sequence, 0);
        assert_eq!(event.hash, "");
    }

    #[test]
    fn event_roundtrip_json() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct TestPayload {
            name: String,
            count: u32,
        }

        let payload = TestPayload {
            name: "test".to_string(),
            count: 100,
        };
        let event = EventEnvelope::new(payload, "actor");

        let json = serde_json::to_string(&event).unwrap();
        let decoded: EventEnvelope<TestPayload> = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.actor, "actor");
        assert_eq!(decoded.payload.name, "test");
        assert_eq!(decoded.payload.count, 100);
    }
}
