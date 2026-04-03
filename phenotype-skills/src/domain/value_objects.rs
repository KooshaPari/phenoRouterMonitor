//! Value objects - Immutable, equality-based objects

use serde::{Deserialize, Serialize};
use semver::Version;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Semantic version for skills
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillVersion(pub Version);

impl SkillVersion {
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self(Version::new(major, minor, patch))
    }
    
    pub fn parse(s: &str) -> anyhow::Result<Self> {
        Ok(Self(Version::parse(s)?))
    }
}

impl Default for SkillVersion {
    fn default() -> Self {
        Self::new(0, 1, 0)
    }
}

impl std::fmt::Display for SkillVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A dependency on another skill
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillDependency {
    pub name: String,
    pub version_req: String,
    pub optional: bool,
}

/// Skill manifest - defines a skill's metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillManifest {
    pub name: String,
    pub version: SkillVersion,
    pub description: String,
    pub author: String,
    pub license: String,
    pub entry_point: String,
    pub dependencies: Vec<SkillDependency>,
    pub metadata: SkillMetadata,
}

/// Additional metadata for skills
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub tags: Vec<String>,
    pub categories: Vec<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub attributes: HashMap<String, String>,
}

/// Execution mode for sandboxing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// In-process execution (no sandbox)
    InProcess,
    /// WASM sandbox (Tier 1)
    WASM,
    /// gVisor container (Tier 2)
    GVisor,
    /// Firecracker microVM (Tier 3)
    Firecracker,
}

impl Default for ExecutionMode {
    fn default() -> Self {
        ExecutionMode::WASM
    }
}

impl SkillManifest {
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let manifest: SkillManifest = toml::from_str(&content)?;
        Ok(manifest)
    }
    
    pub fn from_toml(content: &str) -> anyhow::Result<Self> {
        let manifest: SkillManifest = toml::from_str(content)?;
        Ok(manifest)
    }
}
