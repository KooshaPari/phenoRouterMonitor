//! Cycle views with assigned feature data.

use serde::{Deserialize, Serialize};

use super::{Cycle, WpProgressSummary};
use crate::domain::feature::Feature;
use crate::domain::state_machine::FeatureState;

/// View struct carrying a Cycle together with its assigned features and WP progress.
/// Populated by the storage/query layer in WP02.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleWithFeatures {
    pub cycle: Cycle,
    pub features: Vec<Feature>,
    /// Aggregate count of WPs per WpState across all assigned features.
    pub wp_progress: WpProgressSummary,
}

impl CycleWithFeatures {
    /// Return `true` when the cycle is safe to ship.
    ///
    /// Per FR-C07: all assigned Features must be in `Validated` or `Shipped` state.
    /// An empty feature list is treated as vacuously shippable (no blocking features).
    /// A Cycle with no assigned Features is a planning placeholder -- callers may
    /// apply an additional "at least one feature" guard at the service layer.
    pub fn is_shippable(&self) -> bool {
        self.features
            .iter()
            .all(|f| matches!(f.state, FeatureState::Validated | FeatureState::Shipped))
    }
}
