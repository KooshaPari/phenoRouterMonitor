//! Driven ports - Interfaces for infrastructure concerns

use crate::domain::{Skill, SkillId, SkillManifest, SkillEvent};
use crate::SkillsError;
use std::path::PathBuf;
use std::fmt::Debug;

/// Port for persisting and retrieving skills
pub trait StoragePort: Send + Sync + Debug {
    /// Save a skill manifest
    fn save(&self, skill: &Skill) -> Result<(), SkillsError>;
    
    /// Load a skill by ID
    fn load(&self, skill_id: SkillId) -> Result<Option<Skill>, SkillsError>;
    
    /// Load a skill by name
    fn load_by_name(&self, name: &str) -> Result<Option<Skill>, SkillsError>;
    
    /// Delete a skill
    fn delete(&self, skill_id: SkillId) -> Result<(), SkillsError>;
    
    /// List all stored skills
    fn list(&self) -> Result<Vec<Skill>, SkillsError>;
}

/// Port for loading skill manifests
pub trait LoaderPort: Send + Sync + Debug {
    /// Load a manifest from a path
    fn load_manifest(&self, path: &PathBuf) -> Result<SkillManifest, SkillsError>;
    
    /// Validate a manifest
    fn validate(&self, manifest: &SkillManifest) -> Result<(), SkillsError>;
}

/// Port for sandbox execution
pub trait SandboxPort: Send + Sync + Debug {
    /// Execute a skill in the sandbox
    fn execute(&self, skill: &Skill, input: serde_json::Value) -> Result<serde_json::Value, SkillsError>;
    
    /// Check if the sandbox is available
    fn is_available(&self) -> bool;
}

/// Port for emitting events
pub trait EventPort: Send + Sync + Debug {
    /// Emit a skill event
    fn emit(&self, event: SkillEvent) -> Result<(), SkillsError>;
}
