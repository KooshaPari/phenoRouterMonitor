//! Cycle and feature association types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Many-to-many assignment join between a Cycle and a Feature.
///
/// Traces to: FR-C03
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleFeature {
    pub cycle_id: i64,
    pub feature_id: i64,
    pub added_at: DateTime<Utc>,
}

impl CycleFeature {
    pub fn new(cycle_id: i64, feature_id: i64) -> Self {
        Self {
            cycle_id,
            feature_id,
            added_at: Utc::now(),
        }
    }
}
