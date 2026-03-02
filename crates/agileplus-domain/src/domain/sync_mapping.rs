//! Sync mapping domain type — tracks entity↔Plane.so issue mappings.
//!
//! Traceability: FR-006 / WP01-T003

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Direction of synchronization for a mapped entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncDirection {
    Push,
    Pull,
    Bidirectional,
}

impl std::fmt::Display for SyncDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Push => write!(f, "push"),
            Self::Pull => write!(f, "pull"),
            Self::Bidirectional => write!(f, "bidirectional"),
        }
    }
}

/// Mapping between a local AgilePlus entity and a Plane.so issue.
///
/// Unique by `(entity_type, entity_id)`. Tracks content hashes for
/// change detection and conflict counting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMapping {
    pub id: i64,
    pub entity_type: String,
    pub entity_id: i64,
    pub plane_issue_id: String,
    pub content_hash: String,
    pub last_synced_at: DateTime<Utc>,
    pub sync_direction: SyncDirection,
    pub conflict_count: i32,
}

impl SyncMapping {
    pub fn new(
        entity_type: impl Into<String>,
        entity_id: i64,
        plane_issue_id: impl Into<String>,
        content_hash: impl Into<String>,
    ) -> Self {
        Self {
            id: 0,
            entity_type: entity_type.into(),
            entity_id,
            plane_issue_id: plane_issue_id.into(),
            content_hash: content_hash.into(),
            last_synced_at: Utc::now(),
            sync_direction: SyncDirection::Bidirectional,
            conflict_count: 0,
        }
    }

    pub fn increment_conflict(&mut self) {
        self.conflict_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sync_mapping() {
        let m = SyncMapping::new("feature", 1, "plane-123", "abc123");
        assert_eq!(m.entity_type, "feature");
        assert_eq!(m.plane_issue_id, "plane-123");
        assert_eq!(m.sync_direction, SyncDirection::Bidirectional);
        assert_eq!(m.conflict_count, 0);
    }

    #[test]
    fn increment_conflict() {
        let mut m = SyncMapping::new("wp", 2, "plane-456", "def456");
        m.increment_conflict();
        m.increment_conflict();
        assert_eq!(m.conflict_count, 2);
    }

    #[test]
    fn sync_direction_display() {
        assert_eq!(SyncDirection::Push.to_string(), "push");
        assert_eq!(SyncDirection::Pull.to_string(), "pull");
        assert_eq!(SyncDirection::Bidirectional.to_string(), "bidirectional");
    }
}
