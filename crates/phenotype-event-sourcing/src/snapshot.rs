//! Snapshot utilities for event sourcing.
//!
//! Snapshots store the aggregated state at a point in time, allowing event replays to begin
//! from the snapshot rather than from the very first event. This improves performance when
//! events are numerous.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::Result;

/// A snapshot of an entity's state at a specific sequence number.
///
/// Snapshots capture the full state of an aggregate root at a point in time,
/// eliminating the need to replay all events from the beginning. When loading
/// an entity, the latest snapshot is restored, and only events after the snapshot
/// are replayed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// The entity's ID (e.g., aggregate root ID)
    pub entity_id: String,

    /// The entity's type (e.g., "Order", "User")
    pub entity_type: String,

    /// The sequence number at which this snapshot was taken
    /// (i.e., the sequence of the last event included in the snapshot state)
    pub snapshot_at_sequence: i64,

    /// The timestamp when the snapshot was created
    pub snapshot_timestamp: DateTime<Utc>,

    /// The serialized state at this sequence number
    pub state: serde_json::Value,

    /// The hash of the event that triggered this snapshot (for verification)
    pub triggering_event_hash: String,

    /// Unique snapshot ID for auditing
    pub snapshot_id: Uuid,
}

impl Snapshot {
    /// Create a new snapshot.
    pub fn new(
        entity_id: impl Into<String>,
        entity_type: impl Into<String>,
        sequence: i64,
        state: serde_json::Value,
        triggering_event_hash: impl Into<String>,
    ) -> Self {
        Self {
            entity_id: entity_id.into(),
            entity_type: entity_type.into(),
            snapshot_at_sequence: sequence,
            snapshot_timestamp: Utc::now(),
            state,
            triggering_event_hash: triggering_event_hash.into(),
            snapshot_id: Uuid::new_v4(),
        }
    }
}

/// In-memory snapshot store for testing and lightweight use cases.
#[derive(Debug, Clone, Default)]
pub struct InMemorySnapshotStore {
    // Key: (entity_type, entity_id) -> Snapshot
    snapshots: std::sync::Arc<dashmap::DashMap<(String, String), Snapshot>>,
}

impl InMemorySnapshotStore {
    /// Create a new in-memory snapshot store.
    pub fn new() -> Self {
        Self {
            snapshots: std::sync::Arc::new(dashmap::DashMap::new()),
        }
    }

    /// Save a snapshot.
    pub fn save(&self, snapshot: Snapshot) -> Result<()> {
        let key = (snapshot.entity_type.clone(), snapshot.entity_id.clone());
        self.snapshots.insert(key, snapshot);
        Ok(())
    }

    /// Get the latest snapshot for an entity.
    pub fn get_latest(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Option<Snapshot>> {
        let key = (entity_type.to_string(), entity_id.to_string());
        Ok(self.snapshots.get(&key).as_deref().cloned())
    }

    /// Delete the snapshot for an entity.
    pub fn delete(&self, entity_type: &str, entity_id: &str) -> Result<()> {
        let key = (entity_type.to_string(), entity_id.to_string());
        self.snapshots.remove(&key);
        Ok(())
    }

    /// Get all snapshots (for audit/verification).
    pub fn get_all(&self) -> Result<Vec<Snapshot>> {
        Ok(self
            .snapshots
            .iter()
            .map(|entry| entry.value().clone())
            .collect())
    }

    /// Clear all snapshots.
    pub fn clear(&self) -> Result<()> {
        self.snapshots.clear();
        Ok(())
    }
}

/// Strategy for determining when to create snapshots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotStrategy {
    /// Create a snapshot every N events
    EventInterval(i64),
    /// Create a snapshot every N seconds
    TimeInterval(u64),
    /// Create a snapshot when the total event size exceeds N bytes
    SizeThreshold(usize),
    /// Manual snapshot only (caller decides when)
    Manual,
}

impl Default for SnapshotStrategy {
    fn default() -> Self {
        // Default: snapshot every 100 events
        SnapshotStrategy::EventInterval(100)
    }
}

/// Determine if a snapshot should be created based on the strategy and current state.
pub fn should_snapshot(
    strategy: SnapshotStrategy,
    current_sequence: i64,
    last_snapshot_sequence: i64,
    last_snapshot_time: DateTime<Utc>,
    event_bytes_since_snapshot: usize,
) -> bool {
    match strategy {
        SnapshotStrategy::EventInterval(interval) => {
            current_sequence - last_snapshot_sequence >= interval
        }
        SnapshotStrategy::TimeInterval(seconds) => {
            let elapsed = Utc::now()
                .signed_duration_since(last_snapshot_time)
                .num_seconds() as u64;
            elapsed >= seconds
        }
        SnapshotStrategy::SizeThreshold(threshold) => event_bytes_since_snapshot >= threshold,
        SnapshotStrategy::Manual => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_snapshot() {
        let state = serde_json::json!({"count": 42, "name": "test"});
        let snapshot = Snapshot::new(
            "entity-123",
            "Order",
            100,
            state.clone(),
            "abc123def456",
        );

        assert_eq!(snapshot.entity_id, "entity-123");
        assert_eq!(snapshot.entity_type, "Order");
        assert_eq!(snapshot.snapshot_at_sequence, 100);
        assert_eq!(snapshot.state, state);
        assert_eq!(snapshot.triggering_event_hash, "abc123def456");
        assert!(!snapshot.snapshot_id.is_nil());
    }

    #[test]
    fn in_memory_snapshot_store_save_and_retrieve() {
        let store = InMemorySnapshotStore::new();
        let state = serde_json::json!({"value": 123});
        let snapshot = Snapshot::new("entity-1", "Order", 50, state, "hash123");

        store.save(snapshot.clone()).unwrap();

        let retrieved = store.get_latest("Order", "entity-1").unwrap();
        assert!(retrieved.is_some());

        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.entity_id, "entity-1");
        assert_eq!(retrieved.snapshot_at_sequence, 50);
    }

    #[test]
    fn in_memory_snapshot_store_delete() {
        let store = InMemorySnapshotStore::new();
        let state = serde_json::json!({"value": 123});
        let snapshot = Snapshot::new("entity-1", "User", 25, state, "hash456");

        store.save(snapshot).unwrap();
        assert!(store.get_latest("User", "entity-1").unwrap().is_some());

        store.delete("User", "entity-1").unwrap();
        assert!(store.get_latest("User", "entity-1").unwrap().is_none());
    }

    #[test]
    fn snapshot_strategy_event_interval() {
        let strategy = SnapshotStrategy::EventInterval(100);
        assert!(should_snapshot(strategy, 150, 50, Utc::now(), 0));
        assert!(!should_snapshot(strategy, 140, 50, Utc::now(), 0));
    }

    #[test]
    fn snapshot_strategy_size_threshold() {
        let strategy = SnapshotStrategy::SizeThreshold(1000);
        assert!(should_snapshot(strategy, 50, 25, Utc::now(), 1500));
        assert!(!should_snapshot(strategy, 50, 25, Utc::now(), 500));
    }

    #[test]
    fn snapshot_strategy_manual() {
        let strategy = SnapshotStrategy::Manual;
        assert!(!should_snapshot(strategy, 150, 50, Utc::now(), 10000));
    }

    #[test]
    fn get_all_snapshots() {
        let store = InMemorySnapshotStore::new();
        let snap1 = Snapshot::new("e1", "Order", 50, serde_json::json!({}), "h1");
        let snap2 = Snapshot::new("e2", "User", 100, serde_json::json!({}), "h2");

        store.save(snap1).unwrap();
        store.save(snap2).unwrap();

        let all = store.get_all().unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn clear_snapshots() {
        let store = InMemorySnapshotStore::new();
        store
            .save(Snapshot::new("e1", "Order", 50, serde_json::json!({}), "h1"))
            .unwrap();
        assert!(!store.get_all().unwrap().is_empty());

        store.clear().unwrap();
        assert!(store.get_all().unwrap().is_empty());
    }
}
