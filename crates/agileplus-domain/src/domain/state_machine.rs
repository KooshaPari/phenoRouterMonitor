// Feature/work-package state machine — minimal types for WP04 governance/audit.
// Full implementation deferred to WP03.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

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

impl FromStr for FeatureState {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "created" => Ok(Self::Created),
            "specified" => Ok(Self::Specified),
            "researched" => Ok(Self::Researched),
            "planned" => Ok(Self::Planned),
            "implementing" => Ok(Self::Implementing),
            "validated" => Ok(Self::Validated),
            "shipped" => Ok(Self::Shipped),
            "retrospected" => Ok(Self::Retrospected),
            _ => Err(format!("unknown feature state: {s}")),
        }
    }
}

/// Result of a state transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionResult {
    pub transition: StateTransition,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl FeatureState {
    /// Ordered index for determining valid forward transitions.
    pub fn ordinal(self) -> u8 {
        match self {
            Self::Created => 0,
            Self::Specified => 1,
            Self::Researched => 2,
            Self::Planned => 3,
            Self::Implementing => 4,
            Self::Validated => 5,
            Self::Shipped => 6,
            Self::Retrospected => 7,
        }
    }

    /// All states in lifecycle order.
    const ALL: [FeatureState; 8] = [
        Self::Created,
        Self::Specified,
        Self::Researched,
        Self::Planned,
        Self::Implementing,
        Self::Validated,
        Self::Shipped,
        Self::Retrospected,
    ];

    /// Transition to a target state. Allows forward moves only.
    pub fn transition(self, target: FeatureState) -> Result<TransitionResult, crate::error::DomainError> {
        if target.ordinal() <= self.ordinal() {
            return Err(crate::error::DomainError::InvalidTransition {
                from: self.to_string(),
                to: target.to_string(),
                reason: format!("backward transition from {} to {} is not allowed", self, target),
            });
        }
        let skipped: Vec<FeatureState> = Self::ALL
            .iter()
            .copied()
            .filter(|s| s.ordinal() > self.ordinal() && s.ordinal() < target.ordinal())
            .collect();
        Ok(TransitionResult {
            transition: StateTransition {
                from: self,
                to: target,
                skipped,
            },
            timestamp: chrono::Utc::now(),
        })
    }
}
