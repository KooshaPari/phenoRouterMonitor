//! Domain events - Events that occur in the skill lifecycle

use super::{SkillId, SkillManifest};
use chrono::{DateTime, Utc};

/// Events that can occur in the skill system
#[derive(Debug, Clone)]
pub enum SkillEvent {
    /// Skill was registered
    Registered {
        skill_id: SkillId,
        manifest: SkillManifest,
        timestamp: DateTime<Utc>,
    },
    /// Skill was unregistered
    Unregistered {
        skill_id: SkillId,
        timestamp: DateTime<Utc>,
    },
    /// Skill was activated
    Activated {
        skill_id: SkillId,
        timestamp: DateTime<Utc>,
    },
    /// Skill was deactivated
    Deactivated {
        skill_id: SkillId,
        timestamp: DateTime<Utc>,
    },
    /// Skill was updated (hot reload)
    Updated {
        skill_id: SkillId,
        old_manifest: SkillManifest,
        new_manifest: SkillManifest,
        timestamp: DateTime<Utc>,
    },
    /// Dependency was resolved
    DependencyResolved {
        skill_id: SkillId,
        dependency_name: String,
        resolved_version: String,
        timestamp: DateTime<Utc>,
    },
    /// Execution started
    ExecutionStarted {
        skill_id: SkillId,
        execution_id: String,
        timestamp: DateTime<Utc>,
    },
    /// Execution completed
    ExecutionCompleted {
        skill_id: SkillId,
        execution_id: String,
        success: bool,
        timestamp: DateTime<Utc>,
    },
}

impl SkillEvent {
    pub fn registered(skill_id: SkillId, manifest: SkillManifest) -> Self {
        Self::Registered {
            skill_id,
            manifest,
            timestamp: Utc::now(),
        }
    }
    
    pub fn unregistered(skill_id: SkillId) -> Self {
        Self::Unregistered {
            skill_id,
            timestamp: Utc::now(),
        }
    }
    
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::Registered { timestamp, .. } => *timestamp,
            Self::Unregistered { timestamp, .. } => *timestamp,
            Self::Activated { timestamp, .. } => *timestamp,
            Self::Deactivated { timestamp, .. } => *timestamp,
            Self::Updated { timestamp, .. } => *timestamp,
            Self::DependencyResolved { timestamp, .. } => *timestamp,
            Self::ExecutionStarted { timestamp, .. } => *timestamp,
            Self::ExecutionCompleted { timestamp, .. } => *timestamp,
        }
    }
}
