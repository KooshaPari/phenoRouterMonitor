//! TOML loader adapter

use std::path::PathBuf;
use tracing::{debug, error};

use crate::domain::SkillManifest;
use crate::ports::LoaderPort;
use crate::SkillsError;

/// TOML-based manifest loader
#[derive(Debug)]
pub struct TomlLoader;

impl TomlLoader {
    pub fn new() -> Self {
        Self
    }
}

impl LoaderPort for TomlLoader {
    fn load_manifest(&self, path: &PathBuf) -> Result<SkillManifest, SkillsError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| SkillsError::Io(e))?;
        
        let manifest: SkillManifest = toml::from_str(&content)
            .map_err(|e| SkillsError::Serialization(e.to_string()))?;
        
        debug!("Loaded manifest for '{}' from {:?}", manifest.name, path);
        Ok(manifest)
    }
    
    fn validate(&self, manifest: &SkillManifest) -> Result<(), SkillsError> {
        // Validate required fields
        if manifest.name.is_empty() {
            return Err(SkillsError::Serialization("Skill name cannot be empty".to_string()));
        }
        
        if manifest.entry_point.is_empty() {
            return Err(SkillsError::Serialization("Entry point cannot be empty".to_string()));
        }
        
        debug!("Validated manifest for '{}'", manifest.name);
        Ok(())
    }
}

impl Default for TomlLoader {
    fn default() -> Self {
        Self::new()
    }
}
