use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_type: String,
    pub aggregate_id: String,
    pub payload: serde_json::Value,
    pub metadata: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event: Event,
    pub sequence: i64,
    pub timestamp: DateTime<Utc>,
    pub hash: String,
    pub id: Uuid,
}

impl Event {
    pub fn new(event_type: impl Into<String>, aggregate_id: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            event_type: event_type.into(),
            aggregate_id: aggregate_id.into(),
            payload,
            metadata: serde_json::Map::new(),
        }
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

impl EventEnvelope {
    pub fn new(event: Event, sequence: i64, hash: String) -> Self {
        Self {
            event,
            sequence,
            timestamp: Utc::now(),
            hash,
            id: Uuid::new_v4(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = Event::new("user.created", "user-1", serde_json::json!({"name": "test"}));
        assert_eq!(event.event_type, "user.created");
        assert_eq!(event.aggregate_id, "user-1");
    }

    #[test]
    fn test_event_with_metadata() {
        let event = Event::new("user.created", "user-1", serde_json::json!({}))
            .with_metadata("source", serde_json::json!("api"));
        assert_eq!(event.metadata.get("source").unwrap(), "api");
    }

    #[test]
    fn test_event_envelope() {
        let event = Event::new("test", "agg-1", serde_json::json!({}));
        let envelope = EventEnvelope::new(event, 1, "hash123".to_string());
        assert_eq!(envelope.sequence, 1);
        assert_eq!(envelope.hash, "hash123");
    }
}
