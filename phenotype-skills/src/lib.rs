//! Phenotype Skills - Modular Skill System for Agent Orchestration
//!
//! This crate provides a comprehensive framework for building extensible
//! agent capabilities with hot-reloading, versioning, and dependency management.
//!
//! # Example
//!
//! ```
//! use phenotype_skills::{SkillRegistry, SkillManifest};
//!
//! let registry = SkillRegistry::new();
//! let manifest = SkillManifest::from_file("./my-skill.toml");
//! ```

pub mod domain;
pub mod application;
pub mod ports;
pub mod adapters;
pub mod runtime;

pub use domain::{Skill, SkillId, SkillManifest, SkillVersion, SkillDependency};
pub use application::{SkillRegistry, RegistrationError, ResolutionError};
pub use ports::{StoragePort, LoaderPort, SandboxPort};

use thiserror::Error;

/// Errors that can occur in the skill system
#[derive(Error, Debug)]
pub enum SkillsError {
    #[error("skill not found: {0}")]
    NotFound(String),
    
    #[error("version conflict: {0} requires {1}, but {2} is loaded")]
    VersionConflict(String, String, String),
    
    #[error("dependency resolution failed: {0}")]
    DependencyResolution(String),
    
    #[error("sandbox error: {0}")]
    Sandbox(String),
    
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("serialization error: {0}")]
    Serialization(String),
}

/// Result type for skill operations
pub type Result<T> = std::result::Result<T, SkillsError>;

/// Initialize the skill system with tracing
pub fn init() {
    tracing_subscriber::fmt::init();
}
