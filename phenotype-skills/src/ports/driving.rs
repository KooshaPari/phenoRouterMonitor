//! Driving ports - Interfaces for user-facing concerns

use crate::domain::{Skill, SkillId, SkillManifest};
use crate::SkillsError;

/// Port for API interactions
pub trait ApiPort: Send + Sync {
    /// Register a skill via API
    fn register(&self, manifest: SkillManifest) -> Result<SkillId, SkillsError>;
    
    /// Get skill by ID
    fn get(&self, skill_id: SkillId) -> Result<Option<Skill>, SkillsError>;
    
    /// List all skills
    fn list(&self) -> Result<Vec<Skill>, SkillsError>;
    
    /// Unregister a skill
    fn unregister(&self, skill_id: SkillId) -> Result<(), SkillsError>;
    
    /// Execute a skill
    fn execute(&self, skill_id: SkillId, input: serde_json::Value) -> Result<serde_json::Value, SkillsError>;
}

/// Port for CLI interactions
pub trait CliPort: Send + Sync {
    /// Execute a CLI command
    fn execute(&self, args: Vec<String>) -> Result<(), SkillsError>;
}
