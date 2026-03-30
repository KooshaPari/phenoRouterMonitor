//! EventStore trait — generic append-only event storage.

use chrono::{DateTime, Utc};

use crate::error::Result;
use crate::event::EventEnvelope;

/// JSON-backed envelope for stores that persist [`serde_json::Value`] payloads.
pub type JsonEnvelope = EventEnvelope<serde_json::Value>;

/// Generic event store for append-only event storage with hash chain support.
///
/// Implementations of this trait are responsible for:
/// - Appending events in order
/// - Maintaining sequence numbers
/// - Computing and verifying SHA-256 hashes
/// - Ensuring immutability of stored events
pub trait EventStore: Send + Sync {
    /// Append a new event; returns the assigned sequence number.
    fn append(&self, event: &JsonEnvelope, entity_type: &str, entity_id: &str) -> Result<i64>;

    /// Get all events for an entity.
    fn get_events(&self, entity_type: &str, entity_id: &str) -> Result<Vec<JsonEnvelope>>;

    /// Get events from a specific sequence onward (exclusive).
    fn get_events_since(
        &self,
        entity_type: &str,
        entity_id: &str,
        sequence: i64,
    ) -> Result<Vec<JsonEnvelope>>;

    /// Get events within a time range (inclusive).
    fn get_events_by_range(
        &self,
        entity_type: &str,
        entity_id: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<JsonEnvelope>>;

    /// Get the latest event sequence number for an entity (0 if none exist).
    fn get_latest_sequence(&self, entity_type: &str, entity_id: &str) -> Result<i64>;
    fn verify_chain(&self, entity_type: &str, entity_id: &str) -> Result<()>;
}
