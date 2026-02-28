// Feature/work-package state machine — minimal types for WP04 governance/audit.
// Full implementation deferred to WP03.

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureState {
    Created,
    Specified,
    Researched,
    Planned,
    Implementing,
    Validated,
    Shipped,
    Retrospected,
}

impl fmt::Display for FeatureState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Created => "created",
                Self::Specified => "specified",
                Self::Researched => "researched",
                Self::Planned => "planned",
                Self::Implementing => "implementing",
                Self::Validated => "validated",
                Self::Shipped => "shipped",
                Self::Retrospected => "retrospected",
            }
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    pub from: FeatureState,
    pub to: FeatureState,
    pub skipped: Vec<FeatureState>,
}

impl fmt::Display for StateTransition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}", self.from, self.to)
    }
}
