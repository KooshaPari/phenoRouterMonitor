//! Cycle entity definition.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use super::CycleState;
use crate::error::DomainError;

/// A Cycle groups Features into a time-boxed delivery unit.
///
/// Traces to: FR-C01
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cycle {
    pub id: i64,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub state: CycleState,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    /// Optional Module scope; if set, only features owned/tagged to that Module may be assigned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module_scope_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Cycle {
    /// Create a new Cycle in `Draft` state.
    ///
    /// Returns `Err` if `end_date` is not strictly after `start_date`.
    pub fn new(
        name: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
        module_scope_id: Option<i64>,
    ) -> Result<Self, DomainError> {
        if end_date <= start_date {
            return Err(DomainError::Other(
                "end_date must be after start_date".to_string(),
            ));
        }
        let now = Utc::now();
        Ok(Self {
            id: 0,
            name: name.to_string(),
            description: None,
            state: CycleState::Draft,
            start_date,
            end_date,
            module_scope_id,
            created_at: now,
            updated_at: now,
        })
    }

    /// Transition this Cycle to `target`, updating `state` and `updated_at` on success.
    ///
    /// Note: the Review -> Shipped gate (all features validated) is enforced by the
    /// storage/service layer in WP02 and CLI in WP04 -- this method validates only
    /// the state graph edges.
    pub fn transition(&mut self, target: CycleState) -> Result<(), DomainError> {
        self.state.transition(target)?;
        self.state = target;
        self.updated_at = Utc::now();
        Ok(())
    }
}
