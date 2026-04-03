//! Compliance scanner for Phenotype infrastructure
//!
//! Provides compliance scanning functionality for security and policy enforcement.

use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::debug;

/// Severity levels for compliance findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "INFO"),
            Severity::Low => write!(f, "LOW"),
            Severity::Medium => write!(f, "MEDIUM"),
            Severity::High => write!(f, "HIGH"),
            Severity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// A compliance rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub description: String,
    pub severity: Severity,
}

/// A finding from a compliance scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub rule_id: String,
    pub severity: Severity,
    pub message: String,
}

/// Results of governance file scan
#[derive(Debug, Clone)]
pub struct GovernanceScan {
    pub has_codecov: bool,
    pub has_deny_toml: bool,
    pub has_pre_commit: bool,
    pub has_security_yml: bool,
    pub has_ci_yml: bool,
}

impl GovernanceScan {
    /// Calculate compliance score based on governance files
    pub fn compliance_score(&self) -> f32 {
        let mut score = 0.0;
        if self.has_codecov { score += 20.0; }
        if self.has_deny_toml { score += 20.0; }
        if self.has_pre_commit { score += 20.0; }
        if self.has_security_yml { score += 20.0; }
        if self.has_ci_yml { score += 20.0; }
        score
    }
}

/// Scanner for documentation completeness (health dashboard integration)
pub struct DocumentationScanner {
    required_files: Vec<&'static str>,
    max_age_days: u32,
}

impl DocumentationScanner {
    /// Create a new scanner with default settings
    pub fn new() -> Self {
        Self {
            required_files: vec![
                "CLAUDE.md",
                "README.md",
                "CONTRIBUTING.md",
                "LICENSE",
                "CHANGELOG.md",
            ],
            max_age_days: 90,
        }
    }

    /// Scan project for documentation completeness
    pub fn scan_documentation(&self, project_path: &Path) -> DocumentationScore {
        let mut findings = Vec::new();
        let mut present_count = 0;

        for file in &self.required_files {
            let file_path = project_path.join(file);
            if file_path.exists() {
                present_count += 1;
                debug!("Found documentation file: {}", file);

                // Check freshness (modified within max_age_days)
                if let Ok(metadata) = std::fs::metadata(&file_path) {
                    if let Ok(modified) = metadata.modified() {
                        let age = std::time::SystemTime::now()
                            .duration_since(modified)
                            .unwrap_or_default();
                        let days = age.as_secs() / (24 * 3600);
                        if days > self.max_age_days as u64 {
                            findings.push(DocFinding {
                                file: file.to_string(),
                                issue: format!("{} may be stale ({} days old)", file, days),
                            });
                        }
                    }
                }
            } else {
                findings.push(DocFinding {
                    file: file.to_string(),
                    issue: format!("Required file {} not found", file),
                });
            }
        }

        let score = (present_count as f32 / self.required_files.len() as f32) * 100.0;

        DocumentationScore {
            score,
            findings,
        }
    }

    /// Check for specific governance files
    pub fn scan_governance(&self, project_path: &Path) -> GovernanceScan {
        GovernanceScan {
            has_codecov: project_path.join("codecov.yml").exists(),
            has_deny_toml: project_path.join("deny.toml").exists(),
            has_pre_commit: project_path.join(".pre-commit-config.yaml").exists(),
            has_security_yml: project_path.join(".github/workflows/security.yml").exists(),
            has_ci_yml: project_path.join(".github/workflows/ci.yml").exists(),
        }
    }
}

impl Default for DocumentationScanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Score for documentation completeness
#[derive(Debug, Clone)]
pub struct DocumentationScore {
    pub score: f32,
    pub findings: Vec<DocFinding>,
}

/// A finding from documentation scan
#[derive(Debug, Clone)]
pub struct DocFinding {
    pub file: String,
    pub issue: String,
}

/// Main compliance scanner
pub struct ComplianceScanner {
    rules: Vec<Rule>,
}

impl ComplianceScanner {
    /// Create a new scanner
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Add a compliance rule
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    /// Scan a target against all rules
    pub async fn scan(&self, _path: &str) -> anyhow::Result<Vec<Finding>> {
        // Implement scan logic here
        Ok(Vec::new())
    }
}

impl Default for ComplianceScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_governance_scan_score() {
        let scan = GovernanceScan {
            has_codecov: true,
            has_deny_toml: true,
            has_pre_commit: true,
            has_security_yml: true,
            has_ci_yml: true,
        };
        assert_eq!(scan.compliance_score(), 100.0);

        let scan_partial = GovernanceScan {
            has_codecov: true,
            has_deny_toml: false,
            has_pre_commit: false,
            has_security_yml: true,
            has_ci_yml: false,
        };
        assert_eq!(scan_partial.compliance_score(), 40.0);
    }

    #[test]
    fn test_governance_scan_empty() {
        let scan = GovernanceScan {
            has_codecov: false,
            has_deny_toml: false,
            has_pre_commit: false,
            has_security_yml: false,
            has_ci_yml: false,
        };
        assert_eq!(scan.compliance_score(), 0.0);
    }

    #[test]
    fn test_compliance_scanner_new() {
        let scanner = ComplianceScanner::new();
        assert!(scanner.rules.is_empty());
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", Severity::Critical), "CRITICAL");
        assert_eq!(format!("{}", Severity::High), "HIGH");
        assert_eq!(format!("{}", Severity::Medium), "MEDIUM");
        assert_eq!(format!("{}", Severity::Low), "LOW");
        assert_eq!(format!("{}", Severity::Info), "INFO");
    }
}
