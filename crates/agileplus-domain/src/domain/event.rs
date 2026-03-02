//! Event sourcing domain types — immutable events with hash-chain integrity.
//!
//! Traceability: FR-020..FR-025 / WP01-T001

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::feature::hex_bytes;

/// An immutable domain event recording a state mutation.
///
/// Events are append-only and form a hash chain per entity stream
/// (partitioned by `entity_type` + `entity_id`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: i64,
    pub entity_type: String,
    pub entity_id: i64,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub actor: String,
    pub timestamp: DateTime<Utc>,
    #[serde(with = "hex_bytes")]
    pub prev_hash: [u8; 32],
    #[serde(with = "hex_bytes")]
    pub hash: [u8; 32],
    pub sequence: i64,
}

impl Event {
    /// Create a new event (id and hash will be set by the store).
    pub fn new(
        entity_type: impl Into<String>,
        entity_id: i64,
        event_type: impl Into<String>,
        payload: serde_json::Value,
        actor: impl Into<String>,
    ) -> Self {
        Self {
            id: 0,
            entity_type: entity_type.into(),
            entity_id,
            event_type: event_type.into(),
            payload,
            actor: actor.into(),
            timestamp: Utc::now(),
            prev_hash: [0u8; 32],
            hash: [0u8; 32],
            sequence: 0,
        }
    }
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {}:{} {} by {} (seq {})",
            self.timestamp.format("%Y-%m-%dT%H:%M:%S"),
            self.entity_type,
            self.entity_id,
            self.event_type,
            self.actor,
            self.sequence,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_event_defaults() {
        let e = Event::new("feature", 1, "state_transitioned", serde_json::json!({}), "system");
        assert_eq!(e.id, 0);
        assert_eq!(e.entity_type, "feature");
        assert_eq!(e.entity_id, 1);
        assert_eq!(e.prev_hash, [0u8; 32]);
        assert_eq!(e.sequence, 0);
    }

    #[test]
    fn event_serde_roundtrip() {
        let e = Event::new("wp", 5, "created", serde_json::json!({"title": "WP05"}), "agent");
        let json = serde_json::to_string(&e).unwrap();
        let e2: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(e2.entity_type, "wp");
        assert_eq!(e2.entity_id, 5);
    }

    #[test]
    fn event_display() {
        let e = Event::new("feature", 1, "created", serde_json::json!({}), "user");
        let s = e.to_string();
        assert!(s.contains("feature:1"));
        assert!(s.contains("created"));
    }
}
