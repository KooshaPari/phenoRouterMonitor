//! Configuration for the compliance scanner.
//!
//! All hardcoded defaults are defined here once.  They can be overridden by:
//!
//! 1. A TOML config file pointed to by `SCANNER_CONFIG_PATH` (env var)
//! 2. Individual env vars (listed below per field)
//! 3. Programmatic construction via `ScannerConfig::default()` or builder.

use serde::{Deserialize, Serialize};
use std::path::Path;

// ---------------------------------------------------------------------------
// Top-level config
// ---------------------------------------------------------------------------

/// Complete compliance-scanner configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ScannerConfig {
    #[serde(default)]
    pub documentation: DocumentationConfig,

    #[serde(default)]
    pub governance: GovernanceConfig,

    #[serde(default)]
    pub compliance: ComplianceConfig,
}

impl ScannerConfig {
    /// Load configuration from the path specified by the `SCANNER_CONFIG_PATH`
    /// environment variable, or return sensible defaults.
    pub fn from_env() -> Self {
        match std::env::var("SCANNER_CONFIG_PATH") {
            Ok(path) => Self::from_file(Path::new(&path)).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Load configuration from a TOML file, merging with defaults.
    pub fn from_file(path: &Path) -> Result<Self, crate::ConfigError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| crate::ConfigError::ReadFailed {
                path: path.to_string_lossy().to_string(),
                source: e,
            })?;
        let mut config: Self =
            toml::from_str(&content).map_err(|e| crate::ConfigError::ParseFailed {
                path: path.to_string_lossy().to_string(),
                source: e,
            })?;

        // Allow env-var overrides after file load
        config.apply_env_overrides();
        Ok(config)
    }

    /// Override individual fields via environment variables.
    ///
    /// | Env var                              | Field                          |
    /// |--------------------------------------|--------------------------------|
    /// | `SCANNER_MAX_AGE_DAYS`              | `documentation.max_age_days`   |
    /// | `SCANNER_GOVERNANCE_WEIGHT`         | `governance.weight`            |
    /// | `SCANNER_COMPLIANCE_DEFAULT_WEIGHT` | `compliance.default_score_weight` |
    fn apply_env_overrides(&mut self) {
        if let Ok(val) = std::env::var("SCANNER_MAX_AGE_DAYS") {
            if let Ok(days) = val.parse::<u32>() {
                self.documentation.max_age_days = days;
            }
        }
        if let Ok(val) = std::env::var("SCANNER_GOVERNANCE_WEIGHT") {
            if let Ok(w) = val.parse::<f32>() {
                self.governance.weight = w;
            }
        }
        if let Ok(val) = std::env::var("SCANNER_COMPLIANCE_DEFAULT_WEIGHT") {
            if let Ok(w) = val.parse::<f32>() {
                self.compliance.default_score_weight = w;
            }
        }
    }
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            documentation: DocumentationConfig::default(),
            governance: GovernanceConfig::default(),
            compliance: ComplianceConfig::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Sub-configs
// ---------------------------------------------------------------------------

/// Documentation scanning config.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DocumentationConfig {
    /// File names that every project is expected to have.
    #[serde(default = "default_required_files")]
    pub required_files: Vec<String>,

    /// Maximum age in days before a doc file is considered stale.
    #[serde(default = "default_max_age_days")]
    pub max_age_days: u32,
}

impl Default for DocumentationConfig {
    fn default() -> Self {
        Self {
            required_files: default_required_files(),
            max_age_days: default_max_age_days(),
        }
    }
}

fn default_required_files() -> Vec<String> {
    vec![
        "CLAUDE.md".into(),
        "README.md".into(),
        "CONTRIBUTING.md".into(),
        "LICENSE".into(),
        "CHANGELOG.md".into(),
    ]
}

fn default_max_age_days() -> u32 {
    90
}

/// Governance-file scanning config.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GovernanceConfig {
    /// Relative file paths (from project root) to check for governance
    /// compliance.
    #[serde(default = "default_governance_file_paths")]
    pub file_paths: Vec<String>,

    /// Score weight per governance file present (total = n_files × weight)
    #[serde(default = "default_governance_weight")]
    pub weight: f32,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            file_paths: default_governance_file_paths(),
            weight: default_governance_weight(),
        }
    }
}

fn default_governance_file_paths() -> Vec<String> {
    vec![
        "codecov.yml".into(),
        "deny.toml".into(),
        ".pre-commit-config.yaml".into(),
        ".github/workflows/security.yml".into(),
        ".github/workflows/ci.yml".into(),
    ]
}

fn default_governance_weight() -> f32 {
    20.0
}

/// General compliance scoring config.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ComplianceConfig {
    /// Default weight per rule for scoring.
    #[serde(default = "default_compliance_weight")]
    pub default_score_weight: f32,
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            default_score_weight: default_compliance_weight(),
        }
    }
}

fn default_compliance_weight() -> f32 {
    20.0
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

/// Convenience builder for programmatic construction.
#[derive(Debug, Default)]
pub struct ScannerConfigBuilder {
    documentation: Option<DocumentationConfig>,
    governance: Option<GovernanceConfig>,
    compliance: Option<ComplianceConfig>,
}

impl ScannerConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_documentation(mut self, cfg: DocumentationConfig) -> Self {
        self.documentation = Some(cfg);
        self
    }

    pub fn with_governance(mut self, cfg: GovernanceConfig) -> Self {
        self.governance = Some(cfg);
        self
    }

    pub fn with_compliance(mut self, cfg: ComplianceConfig) -> Self {
        self.compliance = Some(cfg);
        self
    }

    pub fn build(self) -> ScannerConfig {
        ScannerConfig {
            documentation: self.documentation.unwrap_or_default(),
            governance: self.governance.unwrap_or_default(),
            compliance: self.compliance.unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = ScannerConfig::default();
        assert_eq!(cfg.documentation.max_age_days, 90);
        assert_eq!(cfg.documentation.required_files.len(), 5);
        assert_eq!(cfg.governance.weight, 20.0);
        assert_eq!(cfg.governance.file_paths.len(), 5);
        assert_eq!(cfg.compliance.default_score_weight, 20.0);
    }

    #[test]
    fn test_config_builder() {
        let cfg = ScannerConfigBuilder::new()
            .with_documentation(DocumentationConfig {
                required_files: vec!["README.md".into()],
                max_age_days: 30,
            })
            .build();
        assert_eq!(cfg.documentation.max_age_days, 30);
        assert_eq!(cfg.documentation.required_files, vec!["README.md"]);
    }

    #[test]
    fn test_config_from_toml() {
        let toml_str = r#"
[documentation]
required_files = ["README.md", "LICENSE"]
max_age_days = 45

[governance]
file_paths = ["codecov.yml", "deny.toml"]
weight = 25.0

[compliance]
default_score_weight = 10.0
"#;
        let config: ScannerConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.documentation.max_age_days, 45);
        assert_eq!(config.documentation.required_files.len(), 2);
        assert_eq!(config.governance.weight, 25.0);
        assert_eq!(config.governance.file_paths.len(), 2);
        assert_eq!(config.compliance.default_score_weight, 10.0);
    }

    #[test]
    fn test_env_override() {
        unsafe { std::env::set_var("SCANNER_MAX_AGE_DAYS", "120") };
        let cfg = ScannerConfig::from_env();
        assert_eq!(cfg.documentation.max_age_days, 120);
    }
}
