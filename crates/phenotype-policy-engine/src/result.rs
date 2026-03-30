//! Policy evaluation results and violation types.

use serde::{Deserialize, Serialize};

/// Severity levels for policy violations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Informational message, no action required.
    Info,
    /// Warning - recommended action to comply.
    Warning,
    /// Error - violation of policy, must be addressed.
    Error,
}

impl Severity {
    /// Returns a string representation of the severity.
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Info => "info",
            Severity::Warning => "warning",
            Severity::Error => "error",
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A policy violation - a rule that was not satisfied.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    /// The name of the policy that was violated.
    pub policy: String,
    /// The rule type that was violated (Allow, Deny, Require).
    pub rule_type: String,
    /// The condition/pattern that failed.
    pub condition: String,
    /// The severity of this violation.
    pub severity: Severity,
    /// A detailed message about the violation.
    pub message: String,
}

impl Violation {
    /// Creates a new violation.
    pub fn new(
        policy: impl Into<String>,
        rule_type: impl Into<String>,
        condition: impl Into<String>,
        severity: Severity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            policy: policy.into(),
            rule_type: rule_type.into(),
            condition: condition.into(),
            severity,
            message: message.into(),
        }
    }
}

/// The result of policy evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyResult {
    /// Whether all policies were satisfied (no error-level violations).
    pub passed: bool,
    /// List of all violations.
    pub violations: Vec<Violation>,
    /// Metadata about evaluation.
    pub metadata: serde_json::Value,
}

impl PolicyResult {
    /// Creates a new passing policy result.
    pub fn passed() -> Self {
        Self {
            passed: true,
            violations: Vec::new(),
            metadata: serde_json::json!({}),
        }
    }

    /// Creates a new policy result with violations.
    pub fn with_violations(violations: Vec<Violation>) -> Self {
        let passed = violations.iter().all(|v| v.severity != Severity::Error);
        Self {
            passed,
            violations,
            metadata: serde_json::json!({}),
        }
    }

    /// Adds a violation to the result.
    pub fn add_violation(&mut self, violation: Violation) {
        if violation.severity == Severity::Error {
            self.passed = false;
        }
        self.violations.push(violation);
    }

    /// Gets all error-level violations.
    pub fn errors(&self) -> Vec<&Violation> {
        self.violations
            .iter()
            .filter(|v| v.severity == Severity::Error)
            .collect()
    }

    /// Gets all warning-level violations.
    pub fn warnings(&self) -> Vec<&Violation> {
        self.violations
            .iter()
            .filter(|v| v.severity == Severity::Warning)
            .collect()
    }

    /// Gets all info-level violations.
    pub fn infos(&self) -> Vec<&Violation> {
        self.violations
            .iter()
            .filter(|v| v.severity == Severity::Info)
            .collect()
    }

    /// Returns true if there are any error-level violations.
    pub fn has_errors(&self) -> bool {
        !self.errors().is_empty()
    }

    /// Returns true if there are any warning-level violations.
    pub fn has_warnings(&self) -> bool {
        !self.warnings().is_empty()
    }

    /// Returns a summary string.
    pub fn summary(&self) -> String {
        let error_count = self.errors().len();
        let warning_count = self.warnings().len();
        let info_count = self.infos().len();

        format!(
            "Policy evaluation: {} errors, {} warnings, {} infos",
            error_count, warning_count, info_count
        )
    }
}

impl Default for PolicyResult {
    fn default() -> Self {
        Self::passed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
    }

    #[test]
    fn test_violation_creation() {
        let v = Violation::new(
            "test_policy",
            "Deny",
            "pattern",
            Severity::Error,
            "Test message",
        );
        assert_eq!(v.policy, "test_policy");
        assert_eq!(v.severity, Severity::Error);
    }

    #[test]
    fn test_policy_result_passed() {
        let result = PolicyResult::passed();
        assert!(result.passed);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_policy_result_with_violations() {
        let violations = vec![Violation::new(
            "test_policy",
            "Deny",
            "pattern",
            Severity::Error,
            "Error violation",
        )];
        let result = PolicyResult::with_violations(violations);
        assert!(!result.passed);
        assert_eq!(result.errors().len(), 1);
    }

    #[test]
    fn test_policy_result_summary() {
        let mut result = PolicyResult::passed();
        result.add_violation(Violation::new("p1", "Deny", "pat1", Severity::Error, "e1"));
        result.add_violation(Violation::new(
            "p1",
            "Allow",
            "pat2",
            Severity::Warning,
            "w1",
        ));

        let summary = result.summary();
        assert!(summary.contains("1 errors"));
        assert!(summary.contains("1 warnings"));
    }
}
