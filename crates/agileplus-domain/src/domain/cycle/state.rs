//! Cycle lifecycle state and transitions.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::DomainError;

/// Lifecycle state for a Cycle.
///
/// Allowed transitions:
/// ```text
/// Draft   -> Active
/// Active  -> Review
/// Active  -> Draft      (revert)
/// Review  -> Shipped    (gate enforced in WP02/WP04, not here)
/// Review  -> Active     (changes requested)
/// Shipped -> Archived
/// ```
///
/// Traces to: FR-C02
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum CycleState {
    Draft,
    Active,
    Review,
    Shipped,
    Archived,
}

impl fmt::Display for CycleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Draft => "Draft",
                Self::Active => "Active",
                Self::Review => "Review",
                Self::Shipped => "Shipped",
                Self::Archived => "Archived",
            }
        )
    }
}

impl FromStr for CycleState {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Draft" => Ok(Self::Draft),
            "Active" => Ok(Self::Active),
            "Review" => Ok(Self::Review),
            "Shipped" => Ok(Self::Shipped),
            "Archived" => Ok(Self::Archived),
            other => Err(DomainError::Other(format!("unknown cycle state: {other}"))),
        }
    }
}

impl CycleState {
    /// Validate a transition from `self` to `target`.
    ///
    /// Returns `Ok(())` for allowed edges, `Err(NoOpTransition)` for self-to-self,
    /// and `Err(InvalidTransition)` for all other pairs.
    pub fn transition(self, target: CycleState) -> Result<(), DomainError> {
        if self == target {
            return Err(DomainError::NoOpTransition(self.to_string()));
        }
        let allowed = matches!(
            (self, target),
            (CycleState::Draft, CycleState::Active)
                | (CycleState::Active, CycleState::Review)
                | (CycleState::Active, CycleState::Draft)
                | (CycleState::Review, CycleState::Shipped)
                | (CycleState::Review, CycleState::Active)
                | (CycleState::Shipped, CycleState::Archived)
        );
        if allowed {
            Ok(())
        } else {
            Err(DomainError::InvalidTransition {
                from: self.to_string(),
                to: target.to_string(),
                reason: format!(
                    "transition from {self} to {target} is not a permitted edge in the cycle state graph"
                ),
            })
        }
    }
}
