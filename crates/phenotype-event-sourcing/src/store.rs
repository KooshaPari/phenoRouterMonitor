//! EventStore trait — generic append-only event storage.

use chrono::{DateTime, Utc};

use crate::error::Result;
use crate::event::EventEnvelope;

/// Generic event store for append-only event storage with hash chain support.
///
/// Implementations of this trait are responsible for:
/// - Appending events in order
/// - Maintaining sequence numbers
/// - Computing and verifying SHA-256 hashes
/// - Ensuring immutability of stored events
pub trait EventStore: Send + Sync {
    /// Append a new event; returns the assigned sequence number.
    fn append(
        &self,
        event: &EventEnvelope,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<i64>;

    /// Get all events for an entity.
    fn get_events(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Vec<EventEnvelope>>;

    /// Get events from a specific sequence onward (exclusive).
    fn get_events_since(
        &self,
        entity_type: &str,
        entity_id: &str,
        sequence: i64,
    ) -> Result<Vec<EventEnvelope>>;

    /// Get events within a time range (inclusive).
    fn get_events_by_range(
        &self,
        entity_type: &str,
        entity_id: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<EventEnvelope>>;

    /// Get the latest event sequence number for an entity (0 if none exist).
    fn get_latest_sequence(&self, entity_type: &str, entity_id: &str) -> Result<i64>;

    /// Verify the hash chain integrity for an entity.
    fn verify_chain(&self, entity_type: &str, entity_id: &str) -> Result<()>;
}
