//! Skill registry - Central registration and management

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tracing::{debug, info, warn};

use crate::domain::{Skill, SkillId, SkillManifest, SkillEvent, SkillDependency, SkillVersion};
use crate::ports::{StoragePort, LoaderPort, EventPort};
use crate::SkillsError;

/// Errors that can occur during registration
#[derive(Debug, thiserror::Error)]
pub enum RegistrationError {
    #[error("skill already registered: {0}")]
    AlreadyRegistered(String),
    
    #[error("invalid manifest: {0}")]
    InvalidManifest(String),
    
    #[error("dependency not found: {0}")]
    DependencyNotFound(String),
    
    #[error("version conflict: {0}")]
    VersionConflict(String),
}

/// Errors that can occur during dependency resolution
#[derive(Debug, thiserror::Error)]
pub enum ResolutionError {
    #[error("circular dependency detected: {0}")]
    CircularDependency(String),
    
    #[error("unresolved dependency: {0}")]
    UnresolvedDependency(String),
    
    #[error("version mismatch: {0}")]
    VersionMismatch(String),
}

/// Central registry for skills
#[derive(Debug)]
pub struct SkillRegistry {
    skills: Arc<RwLock<HashMap<SkillId, Skill>>>,
    by_name: Arc<RwLock<HashMap<String, SkillId>>>,
    storage: Option<Box<dyn StoragePort>>,
    loader: Option<Box<dyn LoaderPort>>,
    event_port: Option<Box<dyn EventPort>>,
}

impl SkillRegistry {
    /// Create a new skill registry
    pub fn new() -> Self {
        Self {
            skills: Arc::new(RwLock::new(HashMap::new())),
            by_name: Arc::new(RwLock::new(HashMap::new())),
            storage: None,
            loader: None,
            event_port: None,
        }
    }
    
    /// Set the storage adapter
    pub fn with_storage(mut self, storage: Box<dyn StoragePort>) -> Self {
        self.storage = Some(storage);
        self
    }
    
    /// Set the loader adapter
    pub fn with_loader(mut self, loader: Box<dyn LoaderPort>) -> Self {
        self.loader = Some(loader);
        self
    }
    
    /// Set the event port
    pub fn with_event_port(mut self, event_port: Box<dyn EventPort>) -> Self {
        self.event_port = Some(event_port);
        self
    }
    
    /// Register a skill from a manifest
    pub fn register(&self, manifest: SkillManifest, path: PathBuf) -> Result<SkillId, RegistrationError> {
        let name = manifest.name.clone();
        
        // Check if already registered
        {
            let by_name = self.by_name.read().unwrap();
            if by_name.contains_key(&name) {
                return Err(RegistrationError::AlreadyRegistered(name));
            }
        }
        
        // Create skill
        let skill = Skill::new(manifest.clone(), path);
        let skill_id = skill.id;
        
        // Store skill
        {
            let mut skills = self.skills.write().unwrap();
            let mut by_name = self.by_name.write().unwrap();
            
            skills.insert(skill_id, skill);
            by_name.insert(name.clone(), skill_id);
        }
        
        // Emit event
        if let Some(ref event_port) = self.event_port {
            let event = SkillEvent::registered(skill_id, manifest);
            let _ = event_port.emit(event);
        }
        
        info!("Registered skill '{}' with ID {}", name, skill_id);
        Ok(skill_id)
    }
    
    /// Unregister a skill
    pub fn unregister(&self, skill_id: SkillId) -> Result<(), SkillsError> {
        let skill = {
            let mut skills = self.skills.write().unwrap();
            skills.remove(&skill_id).ok_or_else(|| SkillsError::NotFound(skill_id.to_string()))?
        };
        
        {
            let mut by_name = self.by_name.write().unwrap();
            by_name.remove(&skill.manifest.name);
        }
        
        // Emit event
        if let Some(ref event_port) = self.event_port {
            let event = SkillEvent::unregistered(skill_id);
            let _ = event_port.emit(event);
        }
        
        info!("Unregistered skill '{}' with ID {}", skill.manifest.name, skill_id);
        Ok(())
    }
    
    /// Get a skill by ID
    pub fn get(&self, skill_id: SkillId) -> Option<Skill> {
        let skills = self.skills.read().unwrap();
        skills.get(&skill_id).cloned()
    }
    
    /// Get a skill by name
    pub fn get_by_name(&self, name: &str) -> Option<Skill> {
        let by_name = self.by_name.read().unwrap();
        let skill_id = by_name.get(name)?;
        let skills = self.skills.read().unwrap();
        skills.get(skill_id).cloned()
    }
    
    /// List all registered skills
    pub fn list(&self) -> Vec<Skill> {
        let skills = self.skills.read().unwrap();
        skills.values().cloned().collect()
    }
    
    /// Activate a skill
    pub fn activate(&self, skill_id: SkillId) -> Result<(), SkillsError> {
        {
            let mut skills = self.skills.write().unwrap();
            let skill = skills.get_mut(&skill_id).ok_or_else(|| SkillsError::NotFound(skill_id.to_string()))?;
            skill.is_active = true;
        }
        
        if let Some(ref event_port) = self.event_port {
            let _ = event_port.emit(SkillEvent::Activated {
                skill_id,
                timestamp: chrono::Utc::now(),
            });
        }
        
        Ok(())
    }
    
    /// Deactivate a skill
    pub fn deactivate(&self, skill_id: SkillId) -> Result<(), SkillsError> {
        {
            let mut skills = self.skills.write().unwrap();
            let skill = skills.get_mut(&skill_id).ok_or_else(|| SkillsError::NotFound(skill_id.to_string()))?;
            skill.is_active = false;
        }
        
        if let Some(ref event_port) = self.event_port {
            let _ = event_port.emit(SkillEvent::Deactivated {
                skill_id,
                timestamp: chrono::Utc::now(),
            });
        }
        
        Ok(())
    }
    
    /// Resolve dependencies for a skill
    pub fn resolve_dependencies(&self, skill_id: SkillId) -> Result<Vec<SkillId>, ResolutionError> {
        let skill = self.get(skill_id).ok_or_else(|| ResolutionError::UnresolvedDependency(skill_id.to_string()))?;
        
        let mut resolved = Vec::new();
        
        for dep in skill.dependencies() {
            let dep_skill = self.get_by_name(&dep.name)
                .ok_or_else(|| ResolutionError::UnresolvedDependency(dep.name.clone()))?;
            
            // Check version compatibility
            let dep_version = dep_skill.version().to_string();
            if !dep.version_req.starts_with(&format!(">={}", dep_version)) && 
               !dep.version_req.contains(&dep_version) {
                return Err(ResolutionError::VersionMismatch(
                    format!("{} requires {}, found {}", dep.name, dep.version_req, dep_version)
                ));
            }
            
            resolved.push(dep_skill.id);
        }
        
        Ok(resolved)
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}
