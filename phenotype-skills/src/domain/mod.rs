//! Domain layer - Core business logic for skills
//!
//! Contains entities, value objects, and domain events.

pub mod entities;
pub mod value_objects;
pub mod events;

pub use entities::{Skill, SkillId};
pub use value_objects::{SkillManifest, SkillVersion, SkillDependency, SkillMetadata, ExecutionMode};
pub use events::SkillEvent;
