//! EventStore trait — generic append-only event storage.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
    ///
    /// The implementation should:
    /// 1. Compute the hash for this event based on the previous event's hash
    /// 2. Assign the next sequence number
    /// 3. Return the sequence number or an error
    fn append<T: Serialize + for<'de> Deserialize<'de>>(
        &self,
        event: &EventEnvelope<T>,
        event_type: &str,
    ) -> Result<i64>;

    /// Get all events for a given entity type and ID, in ascending sequence order.
    fn get_events<T: Serialize + for<'de> Deserialize<'de>>(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Vec<EventEnvelope<T>>>;

    /// Get events from a specific sequence onward (exclusive).
    fn get_events_since<T: Serialize + for<'de> Deserialize<'de>>(
        &self,
        entity_type: &str,
        entity_id: &str,
        sequence: i64,
    ) -> Result<Vec<EventEnvelope<T>>>;

    /// Get events within a time range (inclusive).
    fn get_events_by_range<T: Serialize + for<'de> Deserialize<'de>>(
        &self,
        entity_type: &str,
        entity_id: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<EventEnvelope<T>>>;

    /// Get the latest event sequence number for an entity (0 if none exist).
    fn get_latest_sequence(&self, entity_type: &str, entity_id: &str) -> Result<i64>;

    /// Verify the hash chain integrity for an entity.
    fn verify_chain(&self, entity_type: &str, entity_id: &str) -> Result<()>;
}
