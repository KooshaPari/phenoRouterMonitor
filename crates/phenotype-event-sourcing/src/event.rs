//! Event envelope for event sourcing.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Event envelope wrapping domain events with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope<T> {
    /// Event ID
    pub id: Uuid,
    
    /// Event payload
    pub payload: T,
    
    /// Actor who created the event
    pub actor: String,
    
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
}

impl<T> EventEnvelope<T> {
    /// Create a new event envelope
    pub fn new(payload: T, actor: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            payload,
            actor: actor.into(),
            timestamp: Utc::now(),
        }
    }
}
