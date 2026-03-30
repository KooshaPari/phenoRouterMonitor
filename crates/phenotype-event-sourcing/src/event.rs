//! Event envelope for event sourcing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Event envelope wrapping domain events with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope<T> {
    /// Event ID
    pub id: Uuid,

    /// Event sequence number
    pub sequence: i64,

    /// Event timestamp
    pub timestamp: DateTime<Utc>,

    /// Type of entity this event applies to
    pub entity_type: String,

    /// Event payload
    pub payload: T,

    /// Actor who created the event
    pub actor: String,
}

impl<T> EventEnvelope<T> {
    /// Create a new event envelope
    pub fn new(payload: T, actor: impl Into<String>, entity_type: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            sequence: 1,
            timestamp: Utc::now(),
            entity_type: entity_type.into(),
            payload,
            actor: actor.into(),
        }
    }
}
