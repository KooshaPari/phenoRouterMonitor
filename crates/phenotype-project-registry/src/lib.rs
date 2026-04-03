//! Project discovery and registry for unified health dashboard
//!
//! Provides functionality to discover projects across the repos shelf
//! and manage their metadata for health tracking.

pub mod project;
pub mod health_registry;

pub use project::{ProjectMetadata, ProjectRegistry, ProjectType};
pub use health_registry::{HealthDashboardRegistry, HealthDashboardConfig};

use std::path::Path;

/// Discover all projects in the given root directory
pub fn discover_projects(root: &Path) -> ProjectRegistry {
    ProjectRegistry::discover(root)
}

/// Discover health dashboard configurations in the given root directory
pub async fn discover_health_configs(root: &Path) -> anyhow::Result<HealthDashboardRegistry> {
    HealthDashboardRegistry::discover_in(root).await
}
