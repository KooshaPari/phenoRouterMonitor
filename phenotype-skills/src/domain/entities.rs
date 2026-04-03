//! Domain entities - Core business objects

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::path::PathBuf;

use super::value_objects::{SkillManifest, SkillVersion, ExecutionMode};

/// Unique identifier for a skill
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SkillId(pub Uuid);

impl SkillId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for SkillId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SkillId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A loaded skill instance
#[derive(Debug, Clone)]
pub struct Skill {
    pub id: SkillId,
    pub manifest: SkillManifest,
    pub path: PathBuf,
    pub execution_mode: ExecutionMode,
    pub is_active: bool,
}

impl Skill {
    pub fn new(manifest: SkillManifest, path: PathBuf) -> Self {
        Self {
            id: SkillId::new(),
            manifest,
            path,
            execution_mode: ExecutionMode::WASM,
            is_active: true,
        }
    }
    
    pub fn name(&self) -> &str {
        &self.manifest.name
    }
    
    pub fn version(&self) -> &SkillVersion {
        &self.manifest.version
    }
    
    pub fn dependencies(&self) -> &[super::value_objects::SkillDependency] {
        &self.manifest.dependencies
    }
}
