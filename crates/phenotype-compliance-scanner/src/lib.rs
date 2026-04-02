//! Phenotype Compliance Scanner
//!
//! Provides compliance scanning functionality for security and policy enforcement.

use phenotype_health::{DimensionScore, Finding, Severity as HealthSeverity};
use std::path::Path;
use tracing::debug;

/// Compliance check result
#[derive(Debug, Clone)]
pub struct ComplianceResult {
    pub rule_id: String,
    pub passed: bool,
    pub message: String,
    pub severity: Severity,
}

/// Severity levels for compliance violations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

/// Scanner for compliance checks
pub struct Scanner {
    rules: Vec<Box<dyn ComplianceRule>>,
}

/// Trait for compliance rules
pub trait ComplianceRule: Send + Sync {
    fn id(&self) -> &str;
    fn description(&self) -> &str;
    fn check(&self, target: &ScanTarget) -> anyhow::Result<ComplianceResult>;
}

/// Target to scan
#[derive(Debug, Clone)]
pub enum ScanTarget {
    File(String),
    Directory(String),
    Content(String),
}

impl Scanner {
    /// Create a new scanner
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Add a compliance rule
    pub fn add_rule(&mut self, rule: Box<dyn ComplianceRule>) {
        self.rules.push(rule);
    }

    /// Scan a target against all rules
    pub fn scan(&self, target: &ScanTarget) -> Vec<ComplianceResult> {
        self.rules
            .iter()
            .filter_map(|rule| match rule.check(target) {
                Ok(result) => Some(result),
                Err(_) => None,
            })
            .collect()
    }
}

impl Default for Scanner {
    fn default() -> Self {
        Self::new()
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
    pub fn scan_documentation(&self, project_path: &Path) -> DimensionScore {
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
                            findings.push(Finding {
                                severity: HealthSeverity::Warning,
                                message: format!("{} may be stale ({} days old)", file, days),
                                file_path: Some(file.to_string()),
                                line_number: None,
                            });
                        }
                    }
                }
            } else {
                findings.push(Finding {
                    severity: HealthSeverity::Error,
                    message: format!("Required file {} not found", file),
                    file_path: None,
                    line_number: None,
                });
            }
        }

        let score = (present_count as f32 / self.required_files.len() as f32) * 100.0;

        DimensionScore {
            score,
            target: 100.0,
            raw_value: present_count as f32,
            unit: "files_present".to_string(),
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
        if self.has_codecov { score += 20.0 }
        if self.has_deny_toml { score += 20.0 }
        if self.has_pre_commit { score += 20.0 }
        if self.has_security_yml { score += 20.0 }
        if self.has_ci_yml { score += 20.0 }
        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestRule;

    impl ComplianceRule for TestRule {
        fn id(&self) -> &str {
            "TEST-001"
        }

        fn description(&self) -> &str {
            "Test rule"
        }

        fn check(&self, _target: &ScanTarget) -> anyhow::Result<ComplianceResult> {
            Ok(ComplianceResult {
                rule_id: "TEST-001".to_string(),
                passed: true,
                message: "Test passed".to_string(),
                severity: Severity::Info,
            })
        }
    }

    #[test]
    fn test_scanner() {
        let mut scanner = Scanner::new();
        scanner.add_rule(Box::new(TestRule));
        let results = scanner.scan(&ScanTarget::Content("test".to_string()));
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);
    }

    #[test]
    fn test_documentation_scanner_empty() {
        let temp = tempfile::TempDir::new().unwrap();
        let scanner = DocumentationScanner::new();
        let score = scanner.scan_documentation(temp.path());

        assert_eq!(score.score, 0.0);
        assert_eq!(score.findings.len(), 5); // All required files missing
    }

    #[test]
    fn test_documentation_scanner_complete() {
        let temp = tempfile::TempDir::new().unwrap();
        let scanner = DocumentationScanner::new();

        // Create all required files
        for file in &scanner.required_files {
            std::fs::write(temp.path().join(file), "# Test").unwrap();
        }

        let score = scanner.scan_documentation(temp.path());
        assert_eq!(score.score, 100.0);
    }
}
