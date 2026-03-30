//! Snapshot types for event sourcing

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Snapshot of aggregate state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Aggregate ID
    pub aggregate_id: String,
    /// Aggregate type
    pub aggregate_type: String,
    /// Version at snapshot time
    pub version: u64,
    /// State as JSON
    pub state: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}
