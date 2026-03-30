//! Event types for event sourcing

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Event envelope wrapping an event with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// Event type
    pub event_type: String,
    /// Event data (JSON)
    pub data: String,
    /// Metadata
    pub metadata: EventMetadata,
}

/// Event metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Event ID
    pub id: String,
    /// Stream ID
    pub stream_id: String,
    /// Version
    pub version: u64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl EventEnvelope {
    /// Create a new event envelope
    pub fn new(event_type: impl Into<String>, data: impl Into<String>, stream_id: impl Into<String>, version: u64) -> Self {
        Self {
            event_type: event_type.into(),
            data: data.into(),
            metadata: EventMetadata {
                id: uuid::Uuid::new_v4().to_string(),
                stream_id: stream_id.into(),
                version,
                timestamp: Utc::now(),
            },
        }
    }
}
