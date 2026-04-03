//! Project metadata and discovery

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::debug;

/// Type of project based on detected files
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectType {
    Rust,
    TypeScript,
    Python,
    Go,
    Mixed,
    Unknown,
}

impl ProjectType {
    /// Detect project type from directory contents
    pub fn detect(path: &Path) -> Self {
        let entries: Vec<_> = std::fs::read_dir(path)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .map(|d| d.file_name().to_string_lossy().to_string())
            .collect();

        let has_cargo_toml = entries.iter().any(|e| e == "Cargo.toml");
        let has_package_json = entries.iter().any(|e| e == "package.json");
        let has_pyproject_toml = entries.iter().any(|e| e == "pyproject.toml");
        let has_go_mod = entries.iter().any(|e| e == "go.mod");

        let detected = [
            (has_cargo_toml, ProjectType::Rust),
            (has_package_json, ProjectType::TypeScript),
            (has_pyproject_toml, ProjectType::Python),
            (has_go_mod, ProjectType::Go),
        ];

        let count = detected.iter().filter(|(found, _)| *found).count();
        
        if count > 1 {
            ProjectType::Mixed
        } else if count == 1 {
            detected.into_iter().find(|(found, _)| *found).map(|(_, t)| t).unwrap()
        } else {
            ProjectType::Unknown
        }
    }
}

/// Metadata about a discovered project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub name: String,
    pub path: PathBuf,
    pub project_type: ProjectType,
    pub has_claude_md: bool,
    pub has_readme: bool,
    pub has_contributing: bool,
    pub has_codecov: bool,
    pub has_deny_toml: bool,
    pub has_license: bool,
    pub discovered_at: DateTime<Utc>,
    pub last_modified: Option<DateTime<Utc>>,
}

impl ProjectMetadata {
    /// Scan a directory and extract project metadata
    pub fn scan(path: &Path) -> anyhow::Result<Self> {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let project_type = ProjectType::detect(path);
        
        let has_claude_md = path.join("CLAUDE.md").exists();
        let has_readme = path.join("README.md").exists();
        let has_contributing = path.join("CONTRIBUTING.md").exists();
        let has_codecov = path.join("codecov.yml").exists();
        let has_deny_toml = path.join("deny.toml").exists();
        let has_license = path.join("LICENSE").exists();

        debug!("Scanned project: {} at {:?}", name, path);

        Ok(Self {
            name,
            path: path.to_path_buf(),
            project_type,
            has_claude_md,
            has_readme,
            has_contributing,
            has_codecov,
            has_deny_toml,
            has_license,
            discovered_at: Utc::now(),
            last_modified: None,
        })
    }

    /// Calculate documentation completeness score (0-100)
    pub fn doc_score(&self) -> f32 {
        let mut score = 0.0;
        if self.has_claude_md { score += 25.0; }
        if self.has_readme { score += 25.0; }
        if self.has_contributing { score += 25.0; }
        if self.has_license { score += 25.0; }
        score
    }

    /// Calculate governance score (0-100)
    pub fn governance_score(&self) -> f32 {
        let mut score = 0.0;
        if self.has_codecov { score += 50.0; }
        if self.has_deny_toml { score += 50.0; }
        score
    }
}

/// Registry of discovered projects
#[derive(Debug, Clone, Default)]
pub struct ProjectRegistry {
    pub projects: HashMap<String, ProjectMetadata>,
}

impl ProjectRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Discover all projects in a directory
    pub fn discover(root: &Path) -> Self {
        let mut registry = Self::new();
        
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Ok(metadata) = ProjectMetadata::scan(&path) {
                        registry.projects.insert(metadata.name.clone(), metadata);
                    }
                }
            }
        }
        
        debug!("Discovered {} projects", registry.projects.len());
        registry
    }

    /// Get a project by name
    pub fn get(&self, name: &str) -> Option<&ProjectMetadata> {
        self.projects.get(name)
    }

    /// Get all projects
    pub fn all(&self) -> Vec<&ProjectMetadata> {
        self.projects.values().collect()
    }

    /// Count of discovered projects
    pub fn len(&self) -> usize {
        self.projects.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.projects.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_empty_registry() {
        let registry = ProjectRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_detect_rust_project() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("Cargo.toml"), "[package]").unwrap();
        
        let project_type = ProjectType::detect(temp_dir.path());
        assert_eq!(project_type, ProjectType::Rust);
    }

    #[test]
    fn test_detect_typescript_project() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("package.json"), "{}").unwrap();
        
        let project_type = ProjectType::detect(temp_dir.path());
        assert_eq!(project_type, ProjectType::TypeScript);
    }

    #[test]
    fn test_detect_mixed_project() {
        let temp_dir = TempDir::new().unwrap();
        fs::write(temp_dir.path().join("Cargo.toml"), "[package]").unwrap();
        fs::write(temp_dir.path().join("package.json"), "{}").unwrap();
        
        let project_type = ProjectType::detect(temp_dir.path());
        assert_eq!(project_type, ProjectType::Mixed);
    }

    #[test]
    fn test_doc_score_calculation() {
        let metadata = ProjectMetadata {
            name: "test".to_string(),
            path: PathBuf::from("/tmp/test"),
            project_type: ProjectType::Rust,
            has_claude_md: true,
            has_readme: true,
            has_contributing: true,
            has_codecov: false,
            has_deny_toml: false,
            has_license: true,
            discovered_at: Utc::now(),
            last_modified: None,
        };
        
        assert_eq!(metadata.doc_score(), 100.0);
    }
}
