//! Policy rules - Allow, Deny, Require with pattern matching.

use crate::context::EvaluationContext;
use crate::error::PolicyEngineError;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Types of policy rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleType {
    /// Allow rule: value matching pattern is allowed.
    Allow,
    /// Deny rule: value matching pattern is denied.
    Deny,
    /// Require rule: fact must exist and match pattern.
    Require,
}

impl RuleType {
    /// Returns a string representation of the rule type.
    pub fn as_str(&self) -> &'static str {
        match self {
            RuleType::Allow => "Allow",
            RuleType::Deny => "Deny",
            RuleType::Require => "Require",
        }
    }
}

impl std::fmt::Display for RuleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A policy rule with pattern matching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    /// The type of rule (Allow, Deny, Require).
    pub rule_type: RuleType,
    /// The fact key to evaluate.
    pub fact: String,
    /// The regex pattern to match against the fact value.
    pub pattern: String,
    /// Optional human-readable description of the rule.
    pub description: Option<String>,
}

impl Rule {
    /// Creates a new rule.
    pub fn new(
        rule_type: RuleType,
        fact: impl Into<String>,
        pattern: impl Into<String>,
    ) -> Self {
        Self {
            rule_type,
            fact: fact.into(),
            pattern: pattern.into(),
            description: None,
        }
    }

    /// Sets the description of the rule.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Evaluates this rule against a context.
    ///
    /// Returns Ok(true) if the rule is satisfied, Ok(false) if violated.
    /// Returns Err if regex compilation or evaluation fails.
    pub fn evaluate(&self, context: &EvaluationContext) -> Result<bool, PolicyEngineError> {
        let regex = Regex::new(&self.pattern).map_err(|e| {
            PolicyEngineError::RegexCompilationError {
                pattern: self.pattern.clone(),
                source: e,
            }
        })?;

        let fact_value = context.get_string(&self.fact);

        match self.rule_type {
            RuleType::Allow => {
                // Allow: fact must match pattern, or fact not exist is OK
                match fact_value {
                    Some(value) => Ok(regex.is_match(&value)),
                    None => Ok(true), // Absence is allowed
                }
            }
            RuleType::Deny => {
                // Deny: fact must NOT match pattern
                match fact_value {
                    Some(value) => Ok(!regex.is_match(&value)),
                    None => Ok(true), // Absence is allowed (not denied)
                }
            }
            RuleType::Require => {
                // Require: fact must exist AND match pattern
                match fact_value {
                    Some(value) => Ok(regex.is_match(&value)),
                    None => Ok(false), // Missing fact fails Require
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_type_display() {
        assert_eq!(RuleType::Allow.as_str(), "Allow");
        assert_eq!(RuleType::Deny.as_str(), "Deny");
        assert_eq!(RuleType::Require.as_str(), "Require");
    }

    #[test]
    fn test_allow_rule_matching() {
        let rule = Rule::new(RuleType::Allow, "status", "^active$");
        let mut ctx = EvaluationContext::new();
        ctx.set_string("status", "active");

        assert!(rule.evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_allow_rule_non_matching() {
        let rule = Rule::new(RuleType::Allow, "status", "^active$");
        let mut ctx = EvaluationContext::new();
        ctx.set_string("status", "inactive");

        assert!(!rule.evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_allow_rule_missing_fact() {
        let rule = Rule::new(RuleType::Allow, "status", "^active$");
        let ctx = EvaluationContext::new();

        // Missing fact is allowed
        assert!(rule.evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_deny_rule_matching() {
        let rule = Rule::new(RuleType::Deny, "status", "^banned$");
        let mut ctx = EvaluationContext::new();
        ctx.set_string("status", "banned");

        // Deny fails when pattern matches
        assert!(!rule.evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_deny_rule_non_matching() {
        let rule = Rule::new(RuleType::Deny, "status", "^banned$");
        let mut ctx = EvaluationContext::new();
        ctx.set_string("status", "active");

        // Deny succeeds when pattern doesn't match
        assert!(rule.evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_require_rule_matching() {
        let rule = Rule::new(RuleType::Require, "email", "^[a-z]+@example\\.com$");
        let mut ctx = EvaluationContext::new();
        ctx.set_string("email", "user@example.com");

        assert!(rule.evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_require_rule_missing() {
        let rule = Rule::new(RuleType::Require, "email", "^[a-z]+@example\\.com$");
        let ctx = EvaluationContext::new();

        // Require fails when fact missing
        assert!(!rule.evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_rule_with_description() {
        let rule = Rule::new(RuleType::Allow, "role", "admin|user")
            .with_description("User must have valid role");

        assert_eq!(
            rule.description,
            Some("User must have valid role".to_string())
        );
    }

    #[test]
    fn test_invalid_regex() {
        let rule = Rule::new(RuleType::Allow, "field", "[invalid");
        let ctx = EvaluationContext::new();

        let result = rule.evaluate(&ctx);
        assert!(result.is_err());
    }
}
