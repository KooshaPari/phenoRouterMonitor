//! Health dashboard configuration registry.
//!
//! Provides registry functionality for health dashboard configurations
//! discovered across projects.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Configuration for a health dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthDashboardConfig {
    pub project_name: String,
    pub path: PathBuf,
    pub enabled_checks: Vec<String>,
    pub thresholds: ThresholdConfig,
}

/// Threshold configuration for health checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    pub min_documentation_score: f32,
    pub min_test_coverage: f32,
    pub max_critical_vulnerabilities: u32,
    pub max_outdated_dependencies: u32,
}

impl Default for ThresholdConfig {
    fn default() -> Self {
        Self {
            min_documentation_score: 70.0,
            min_test_coverage: 60.0,
            max_critical_vulnerabilities: 0,
            max_outdated_dependencies: 5,
        }
    }
}

/// Registry for health dashboard configurations
#[derive(Debug, Clone, Default)]
pub struct HealthDashboardRegistry {
    configs: HashMap<String, HealthDashboardConfig>,
}

impl HealthDashboardRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
        }
    }

    /// Discover health dashboard configurations in the given root directory
    pub fn discover_in(&self, root: &Path) -> Self {
        let mut registry = Self::new();

        // Look for phenotype-health.toml files
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let config_path = path.join("phenotype-health.toml");
                    if config_path.exists() {
                        if let Ok(config) = Self::parse_config(&config_path) {
                            registry
                                .configs
                                .insert(config.project_name.clone(), config);
                        }
                    }
                }
            }
        }

        registry
    }

    /// Parse a health configuration from a file
    fn parse_config(path: &Path) -> anyhow::Result<HealthDashboardConfig> {
        let content = std::fs::read_to_string(path)?;
        let config: HealthDashboardConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Get a configuration by project name
    pub fn get(&self, name: &str) -> Option<&HealthDashboardConfig> {
        self.configs.get(name)
    }

    /// Get all configurations
    pub fn all(&self) -> &HashMap<String, HealthDashboardConfig> {
        &self.configs
    }

    /// Number of configurations in the registry
    pub fn len(&self) -> usize {
        self.configs.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.configs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_empty_registry() {
        let registry = HealthDashboardRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_threshold_config_default() {
        let config = ThresholdConfig::default();
        assert_eq!(config.min_documentation_score, 70.0);
        assert_eq!(config.min_test_coverage, 60.0);
        assert_eq!(config.max_critical_vulnerabilities, 0);
        assert_eq!(config.max_outdated_dependencies, 5);
    }
}
