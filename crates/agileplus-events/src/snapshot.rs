//! Snapshot management for fast aggregate loading.

use agileplus_domain::domain::event::Event;
use agileplus_domain::domain::snapshot::Snapshot;
use async_trait::async_trait;
use chrono::{TimeDelta, Utc};

use crate::store::EventStore;

#[derive(Debug, thiserror::Error)]
pub enum SnapshotError {
    #[error("Snapshot not found for {entity_type}:{entity_id}")]
    NotFound { entity_type: String, entity_id: i64 },
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Invalid snapshot: {0}")]
    Invalid(String),
}

#[derive(Clone, Debug)]
pub struct SnapshotConfig {
    /// Create snapshot after this many events since the last snapshot.
    pub event_threshold: i64,
    /// Create snapshot after this many seconds since the last snapshot.
    pub time_threshold_secs: u64,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            event_threshold: 100,
            time_threshold_secs: 300,
        }
    }
}

#[async_trait]
pub trait SnapshotStore: Send + Sync {
    /// Save a snapshot.
    async fn save(&self, snapshot: &Snapshot) -> Result<(), SnapshotError>;

    /// Load the most recent snapshot for an entity.
    async fn load(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<Option<Snapshot>, SnapshotError>;

    /// Delete snapshots older than a given sequence.
    async fn delete_before(
        &self,
        entity_type: &str,
        entity_id: i64,
        sequence: i64,
    ) -> Result<(), SnapshotError>;
}

/// Determine whether a new snapshot should be created.
pub fn should_snapshot(
    config: &SnapshotConfig,
    current_sequence: i64,
    last_snapshot_sequence: i64,
    last_snapshot_time: Option<chrono::DateTime<Utc>>,
) -> bool {
    if current_sequence - last_snapshot_sequence >= config.event_threshold {
        return true;
    }
    if let Some(last_time) = last_snapshot_time {
        let elapsed = Utc::now().signed_duration_since(last_time);
        if elapsed > TimeDelta::seconds(config.time_threshold_secs as i64) {
            return true;
        }
    }
    false
}

/// State loaded from a snapshot plus events to replay.
pub struct LoadedState {
    pub snapshot: Option<Snapshot>,
    pub events_to_replay: Vec<Event>,
}

impl LoadedState {
    /// Load the latest snapshot and any events since it.
    pub async fn load<SS: SnapshotStore, ES: EventStore>(
        snapshot_store: &SS,
        event_store: &ES,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<Self, SnapshotError> {
        let snapshot = snapshot_store.load(entity_type, entity_id).await?;

        let events_to_replay = if let Some(ref snap) = snapshot {
            event_store
                .get_events_since(entity_type, entity_id, snap.event_sequence)
                .await
                .map_err(|e| SnapshotError::StorageError(e.to_string()))?
        } else {
            event_store
                .get_events(entity_type, entity_id)
                .await
                .map_err(|e| SnapshotError::StorageError(e.to_string()))?
        };

        Ok(Self {
            snapshot,
            events_to_replay,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_snapshot_event_threshold() {
        let config = SnapshotConfig { event_threshold: 100, time_threshold_secs: 300 };
        assert!(should_snapshot(&config, 100, 0, None));
        assert!(!should_snapshot(&config, 50, 0, None));
    }

    #[test]
    fn should_snapshot_time_threshold() {
        let config = SnapshotConfig { event_threshold: 100, time_threshold_secs: 300 };
        let old = Utc::now() - TimeDelta::seconds(400);
        assert!(should_snapshot(&config, 50, 0, Some(old)));
        assert!(!should_snapshot(&config, 50, 0, Some(Utc::now())));
    }
}
