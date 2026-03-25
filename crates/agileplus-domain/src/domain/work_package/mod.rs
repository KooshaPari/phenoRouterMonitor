use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::DomainError;

pub mod dependency;

pub use dependency::{DependencyGraph, DependencyType, WpDependency};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WpState {
    Planned,
    Doing,
    Review,
    Done,
    Blocked,
}

impl WpState {
    pub fn can_transition_to(&self, target: WpState) -> bool {
        matches!(
            (self, target),
            (WpState::Planned, WpState::Doing)
                | (WpState::Planned, WpState::Blocked)
                | (WpState::Doing, WpState::Review)
                | (WpState::Doing, WpState::Blocked)
                | (WpState::Review, WpState::Done)
                | (WpState::Review, WpState::Doing)
                | (WpState::Blocked, WpState::Planned)
                | (WpState::Blocked, WpState::Doing)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrState {
    Open,
    Review,
    ChangesRequested,
    Approved,
    Merged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkPackage {
    pub id: i64,
    pub feature_id: i64,
    pub title: String,
    pub state: WpState,
    pub sequence: i32,
    pub file_scope: Vec<String>,
    pub acceptance_criteria: String,
    pub agent_id: Option<String>,
    pub pr_url: Option<String>,
    pub pr_state: Option<PrState>,
    pub worktree_path: Option<String>,
    /// Plane.so sub-issue ID mapping.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plane_sub_issue_id: Option<String>,
    /// Git commit SHA at which this WP's worktree was branched from main.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_commit: Option<String>,
    /// Git commit SHA of the most recent commit on this WP's branch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_commit: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl WorkPackage {
    pub fn new(feature_id: i64, title: &str, sequence: i32, acceptance_criteria: &str) -> Self {
        let now = Utc::now();
        Self {
            id: 0,
            feature_id,
            title: title.to_string(),
            state: WpState::Planned,
            sequence,
            file_scope: Vec::new(),
            acceptance_criteria: acceptance_criteria.to_string(),
            agent_id: None,
            pr_url: None,
            pr_state: None,
            worktree_path: None,
            plane_sub_issue_id: None,
            base_commit: None,
            head_commit: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn transition(&mut self, target: WpState) -> Result<(), DomainError> {
        if !self.state.can_transition_to(target) {
            return Err(DomainError::InvalidTransition {
                from: format!("{:?}", self.state),
                to: format!("{:?}", target),
                reason: "transition not allowed".into(),
            });
        }
        self.state = target;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn has_file_overlap(&self, other: &WorkPackage) -> Vec<String> {
        self.file_scope
            .iter()
            .filter(|scope| other.file_scope.contains(scope))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests;
