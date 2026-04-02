//! Project discovery and metadata registry for unified health dashboard.
//!
//! This crate provides functionality to:
//! - Discover projects within a directory structure
//! - Extract metadata from project configurations
//! - Track project health and status

use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Represents a discovered project with its metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub path: String,
    pub project_type: ProjectType,
    pub health_score: Option<u8>,
}

/// Types of projects that can be discovered.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectType {
    Rust,
    Python,
    TypeScript,
    Go,
    Other(String),
}

/// Registry for managing discovered projects.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectRegistry {
    projects: Vec<Project>,
}

impl ProjectRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a project to the registry.
    pub fn add(&mut self, project: Project) {
        self.projects.push(project);
    }

    /// Get all projects in the registry.
    pub fn projects(&self) -> &[Project] {
        &self.projects
    }

    /// Find projects by type.
    pub fn by_type(&self, project_type: &ProjectType) -> Vec<&Project> {
        self.projects
            .iter()
            .filter(|p| {
                std::mem::discriminant(&p.project_type) == std::mem::discriminant(project_type)
            })
            .collect()
    }
}

/// Discover projects in a directory.
pub async fn discover_projects(root: &Path) -> Result<ProjectRegistry> {
    let mut registry = ProjectRegistry::new();

    for entry in walkdir::WalkDir::new(root)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Check for Rust projects
        if path.join("Cargo.toml").exists() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                registry.add(Project {
                    name: name.to_string(),
                    path: path.to_string_lossy().to_string(),
                    project_type: ProjectType::Rust,
                    health_score: None,
                });
            }
        }

        // Check for Python projects
        if path.join("pyproject.toml").exists() || path.join("setup.py").exists() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                // Avoid duplicates if already added as Rust
                if !registry
                    .projects
                    .iter()
                    .any(|p| p.path == path.to_string_lossy())
                {
                    registry.add(Project {
                        name: name.to_string(),
                        path: path.to_string_lossy().to_string(),
                        project_type: ProjectType::Python,
                        health_score: None,
                    });
                }
            }
        }

        // Check for TypeScript/Node projects
        if path.join("package.json").exists() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if !registry
                    .projects
                    .iter()
                    .any(|p| p.path == path.to_string_lossy())
                {
                    registry.add(Project {
                        name: name.to_string(),
                        path: path.to_string_lossy().to_string(),
                        project_type: ProjectType::TypeScript,
                        health_score: None,
                    });
                }
            }
        }
    }

    Ok(registry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_discover_rust_project() {
        let temp = TempDir::new().unwrap();
        fs::write(
            temp.path().join("Cargo.toml"),
            "[package]\nname = \"test\"\n",
        )
        .await
        .unwrap();

        let registry = discover_projects(temp.path()).await.unwrap();
        assert_eq!(registry.projects().len(), 1);
        assert!(matches!(
            registry.projects()[0].project_type,
            ProjectType::Rust
        ));
    }

    #[tokio::test]
    async fn test_empty_registry() {
        let temp = TempDir::new().unwrap();
        let registry = discover_projects(temp.path()).await.unwrap();
        assert!(registry.projects().is_empty());
    }
}
