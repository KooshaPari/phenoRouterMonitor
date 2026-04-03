//! File system storage adapter

use std::path::{Path, PathBuf};
use std::fs;
use tracing::{debug, error};

use crate::domain::{Skill, SkillId, SkillManifest};
use crate::ports::StoragePort;
use crate::SkillsError;

/// File system storage for skills
#[derive(Debug)]
pub struct FileSystemStorage {
    base_path: PathBuf,
}

impl FileSystemStorage {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }
    
    fn skill_path(&self, skill_id: SkillId) -> PathBuf {
        self.base_path.join(format!("{}.toml", skill_id))
    }
    
    fn ensure_dir(&self) -> Result<(), SkillsError> {
        if !self.base_path.exists() {
            fs::create_dir_all(&self.base_path)?;
        }
        Ok(())
    }
}

impl StoragePort for FileSystemStorage {
    fn save(&self, skill: &Skill) -> Result<(), SkillsError> {
        self.ensure_dir()?;
        let path = self.skill_path(skill.id);
        
        let content = toml::to_string(&skill.manifest)
            .map_err(|e| SkillsError::Serialization(e.to_string()))?;
        
        fs::write(&path, content)?;
        debug!("Saved skill {} to {:?}", skill.id, path);
        
        Ok(())
    }
    
    fn load(&self, skill_id: SkillId) -> Result<Option<Skill>, SkillsError> {
        let path = self.skill_path(skill_id);
        
        if !path.exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(&path)?;
        let manifest: SkillManifest = toml::from_str(&content)
            .map_err(|e| SkillsError::Serialization(e.to_string()))?;
        
        let skill = Skill::new(manifest, path);
        Ok(Some(skill))
    }
    
    fn load_by_name(&self, name: &str) -> Result<Option<Skill>, SkillsError> {
        // Search all .toml files for matching name
        if !self.base_path.exists() {
            return Ok(None);
        }
        
        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                let content = fs::read_to_string(&path)?;
                if let Ok(manifest) = toml::from_str::<SkillManifest>(&content) {
                    if manifest.name == name {
                        let skill = Skill::new(manifest, path);
                        return Ok(Some(skill));
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    fn delete(&self, skill_id: SkillId) -> Result<(), SkillsError> {
        let path = self.skill_path(skill_id);
        
        if path.exists() {
            fs::remove_file(&path)?;
            debug!("Deleted skill {} from {:?}", skill_id, path);
        }
        
        Ok(())
    }
    
    fn list(&self) -> Result<Vec<Skill>, SkillsError> {
        let mut skills = Vec::new();
        
        if !self.base_path.exists() {
            return Ok(skills);
        }
        
        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                let content = fs::read_to_string(&path)?;
                if let Ok(manifest) = toml::from_str::<SkillManifest>(&content) {
                    let skill = Skill::new(manifest, path);
                    skills.push(skill);
                }
            }
        }
        
        Ok(skills)
    }
}
