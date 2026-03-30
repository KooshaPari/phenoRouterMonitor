// Policy evaluation result types.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize)]
pub enum Severity {
    Info = 0,
    #[default]
    Warning = 1,
    Error = 2,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Info => "Info",
            Severity::Warning => "Warning",
            Severity::Error => "Error",
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub policy_name: String,
    pub rule_type: String,
    pub pattern: String,
    pub severity: Severity,
    pub message: String,
}

impl Violation {
    pub fn new(
        policy_name: String,
        rule_type: String,
        pattern: &str,
        severity: Severity,
        message: String,
    ) -> Self {
        Self {
            policy_name,
            rule_type,
            pattern: pattern.to_string(),
            severity,
            message,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyResult {
    pub passed: bool,
    pub violations: Vec<Violation>,
    // Reserved for future extensibility (documented but unused)
    #[serde(skip)]
    pub _reserved: (),
}

impl PolicyResult {
    pub fn passed() -> Self {
        Self {
            passed: true,
            violations: Vec::new(),
            _reserved: (),
        }
    }
    pub fn with_violations(violations: Vec<Violation>) -> Self {
        let passed = violations.is_empty();
        Self {
            passed,
            violations,
            _reserved: (),
        }
    }
    pub fn add_violation(&mut self, violation: Violation) {
        self.passed = false;
        self.violations.push(violation);
    }
    pub fn summary(&self) -> String {
        if self.passed {
            "Policy passed".to_string()
        } else {
            format!("{} violation(s)", self.violations.len())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_violation_creation() {
        let v = Violation::new(
            "p".to_string(),
            "R".to_string(),
            ".",
            Severity::Error,
            "m".to_string(),
        );
        assert_eq!(v.policy_name, "p");
        assert_eq!(v.severity, Severity::Error);
    }
    #[test]
    fn test_policy_result_passed() {
        let r = PolicyResult::passed();
        assert!(r.passed);
        assert!(r.violations.is_empty());
    }
    #[test]
    fn test_policy_result_with_violations() {
        let v = Violation::new(
            "p".to_string(),
            "R".to_string(),
            ".",
            Severity::Error,
            "m".to_string(),
        );
        let r = PolicyResult::with_violations(vec![v]);
        assert!(!r.passed);
        assert_eq!(r.violations.len(), 1);
    }
    #[test]
    fn test_policy_result_summary() {
        assert_eq!(PolicyResult::passed().summary(), "Policy passed");
        let v = Violation::new(
            "p".to_string(),
            "R".to_string(),
            ".",
            Severity::Error,
            "m".to_string(),
        );
        assert_eq!(
            PolicyResult::with_violations(vec![v]).summary(),
            "1 violation(s)"
        );
    }
    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
    }
}
