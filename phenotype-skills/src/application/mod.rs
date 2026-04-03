//! Application layer - Use cases and business logic

pub mod registry;
pub mod dependency_resolver;

pub use registry::{SkillRegistry, RegistrationError, ResolutionError};
pub use dependency_resolver::{DependencyResolver, ResolutionGraph};
