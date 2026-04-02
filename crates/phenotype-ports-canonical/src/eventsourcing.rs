//! Event sourcing ports and types.
//!
//! Provides append-only event storage with hash chain verification.

use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::error::Result;

/// Event envelope for storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope<T> {
    /// Sequence number in the event stream.
    pub sequence: i64,
    /// Entity type (aggregate type).
    pub entity_type: String,
    /// Entity ID (aggregate ID).
    pub entity_id: String,
    /// Event payload.
    pub payload: T,
    /// Timestamp when the event was recorded.
    pub recorded_at: chrono::DateTime<chrono::Utc>,
    /// Hash chain for verification.
    pub hash: String,
    /// Previous event hash (for chain verification).
    pub previous_hash: Option<String>,
}

impl<T> EventEnvelope<T> {
    /// Create a new event envelope.
    pub fn new(
        sequence: i64,
        entity_type: String,
        entity_id: String,
        payload: T,
        hash: String,
    ) -> Self {
        Self {
            sequence,
            entity_type,
            entity_id,
            payload,
            recorded_at: chrono::Utc::now(),
            hash,
            previous_hash: None,
        }
    }

    /// Set the previous hash for chain verification.
    pub fn with_previous_hash(mut self, hash: String) -> Self {
        self.previous_hash = Some(hash);
        self
    }
}

/// Synchronous event store trait.
///
/// Basic append-only event storage for synchronous contexts.
pub trait EventStore: Send + Sync {
    /// Append an event to the store.
    ///
    /// Returns the sequence number of the appended event.
    ///
    /// # Errors
    ///
    /// Returns an error if the event cannot be appended (e.g., duplicate sequence).
    fn append(&self, event: &EventEnvelope<serde_json::Value>) -> Result<i64>;

    /// Get all events for an entity.
    ///
    /// Events are returned in sequence order.
    fn get_events(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Vec<EventEnvelope<serde_json::Value>>>;

    /// Get the latest sequence number for an entity.
    ///
    /// Returns 0 if no events exist for the entity.
    fn get_latest_sequence(&self, entity_type: &str, entity_id: &str) -> Result<i64>;

    /// Verify the hash chain for an entity.
    ///
    /// Returns `true` if the chain is valid, `false` otherwise.
    fn verify_chain(&self, entity_type: &str, entity_id: &str) -> Result<bool>;
}

/// Asynchronous event store trait.
///
/// Full-featured event store for async contexts with generic payload support.
#[async_trait]
pub trait AsyncEventStore<T>: Send + Sync
where
    T: Send + Sync + Serialize + for<'de> Deserialize<'de> + 'static,
{
    /// Append an event to the store.
    ///
    /// Returns the sequence number of the appended event.
    async fn append(&self, event: &EventEnvelope<T>) -> Result<i64>;

    /// Get all events for an entity.
    ///
    /// Events are returned in sequence order.
    async fn get(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Vec<EventEnvelope<T>>>;

    /// Get the latest sequence number for an entity.
    ///
    /// Returns `None` if no events exist for the entity.
    async fn get_latest_sequence(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Option<i64>>;

    /// Verify the hash chain for an entity.
    ///
    /// Returns `true` if the chain is valid, `false` otherwise.
    async fn verify_chain(&self, entity_type: &str, entity_id: &str) -> Result<bool>;

    /// Count events for an entity type.
    async fn count(&self, entity_type: &str) -> Result<usize>;
}

/// Snapshot configuration.
#[derive(Debug, Clone)]
pub struct SnapshotConfig {
    /// Number of events between snapshots.
    pub events_threshold: usize,
    /// Age threshold for snapshots.
    pub age_threshold: std::time::Duration,
    /// Whether to compress snapshots.
    pub compress: bool,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            events_threshold: 100,
            age_threshold: std::time::Duration::from_secs(3600),
            compress: true,
        }
    }
}

/// Snapshot for aggregate state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot<T> {
    /// Entity type.
    pub entity_type: String,
    /// Entity ID.
    pub entity_id: String,
    /// Sequence number of the last event included in this snapshot.
    pub sequence: i64,
    /// Snapshot payload (aggregate state).
    pub state: T,
    /// Timestamp when the snapshot was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Hash of the snapshot for integrity verification.
    pub hash: String,
}

impl<T> Snapshot<T> {
    /// Create a new snapshot.
    pub fn new(
        entity_type: String,
        entity_id: String,
        sequence: i64,
        state: T,
    ) -> Self {
        Self {
            entity_type,
            entity_id,
            sequence,
            state,
            created_at: chrono::Utc::now(),
            hash: String::new(), // Would be computed in real implementation
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_envelope_new() {
        let envelope = EventEnvelope::new(
            1,
            "Order".into(),
            "order-123".into(),
            serde_json::json!({"type": "created"}),
            "hash123".into(),
        );

        assert_eq!(envelope.sequence, 1);
        assert_eq!(envelope.entity_type, "Order");
        assert_eq!(envelope.entity_id, "order-123");
    }

    #[test]
    fn event_envelope_with_previous_hash() {
        let envelope = EventEnvelope::new(
            2,
            "Order".into(),
            "order-123".into(),
            serde_json::json!({"type": "updated"}),
            "hash456".into(),
        )
        .with_previous_hash("hash123".into());

        assert_eq!(envelope.previous_hash.as_deref(), Some("hash123"));
    }

    #[test]
    fn snapshot_new() {
        let snapshot = Snapshot::new(
            "Order".into(),
            "order-123".into(),
            100,
            serde_json::json!({"status": "completed"}),
        );

        assert_eq!(snapshot.sequence, 100);
        assert_eq!(snapshot.entity_id, "order-123");
    }

    #[test]
    fn snapshot_config_default() {
        let config = SnapshotConfig::default();
        assert_eq!(config.events_threshold, 100);
        assert!(config.compress);
    }
}
