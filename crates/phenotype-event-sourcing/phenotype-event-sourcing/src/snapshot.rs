//! Snapshot management for fast aggregate loading and memory efficiency.

use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};

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

/// A generic snapshot of an entity's state at a specific sequence number.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot<T: Serialize> {
    /// Entity type identifier
    pub entity_type: String,
    /// Entity ID
    pub entity_id: String,
    /// The serialized state at this snapshot
    pub state: T,
    /// The sequence number after applying all events up to this snapshot
    pub event_sequence: i64,
    /// When this snapshot was created
    pub created_at: DateTime<Utc>,
}

/// Determine whether a new snapshot should be created based on thresholds.
pub fn should_snapshot(
    config: &SnapshotConfig,
    current_sequence: i64,
    last_snapshot_sequence: i64,
    last_snapshot_time: Option<DateTime<Utc>>,
) -> bool {
    // Check event threshold
    if current_sequence - last_snapshot_sequence >= config.event_threshold {
        return true;
    }

    // Check time threshold
    if let Some(last_time) = last_snapshot_time {
        let elapsed = Utc::now().signed_duration_since(last_time);
        if elapsed > TimeDelta::seconds(config.time_threshold_secs as i64) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_snapshot_event_threshold() {
        let config = SnapshotConfig {
            event_threshold: 100,
            time_threshold_secs: 300,
        };
        assert!(should_snapshot(&config, 100, 0, None));
        assert!(!should_snapshot(&config, 50, 0, None));
    }

    #[test]
    fn should_snapshot_time_threshold() {
        let config = SnapshotConfig {
            event_threshold: 100,
            time_threshold_secs: 300,
        };
        let old = Utc::now() - TimeDelta::seconds(400);
        assert!(should_snapshot(&config, 50, 0, Some(old)));
        assert!(!should_snapshot(&config, 50, 0, Some(Utc::now())));
    }

    #[test]
    fn default_config() {
        let config = SnapshotConfig::default();
        assert_eq!(config.event_threshold, 100);
        assert_eq!(config.time_threshold_secs, 300);
    }
}
