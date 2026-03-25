//! Policy abstraction and trait definitions.

use crate::context::EvaluationContext;
use crate::result::PolicyResult;
use crate::rule::Rule;
use serde::{Deserialize, Serialize};

/// Trait for evaluable policies.
///
/// Implementations should define how a policy is evaluated against a context.
pub trait EvaluablePolicy: Send + Sync {
    /// The name of the policy.
    fn name(&self) -> &str;

    /// Evaluates this policy against the given context.
    fn evaluate(&self, context: &EvaluationContext) -> Result<PolicyResult, crate::error::PolicyEngineError>;
}

/// A concrete policy implementation with rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    /// Name of the policy.
    pub name: String,
    /// Human-readable description.
    pub description: Option<String>,
    /// Set of rules to evaluate.
    pub rules: Vec<Rule>,
    /// Whether this policy is enabled.
    pub enabled: bool,
}

impl Policy {
    /// Creates a new policy.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            rules: Vec::new(),
            enabled: true,
        }
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Adds a rule to the policy.
    pub fn add_rule(mut self, rule: Rule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Sets whether this policy is enabled.
    pub fn set_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Gets the rules.
    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }
}

impl EvaluablePolicy for Policy {
    fn name(&self) -> &str {
        &self.name
    }

    fn evaluate(&self, context: &EvaluationContext) -> Result<PolicyResult, crate::error::PolicyEngineError> {
        if !self.enabled {
            return Ok(PolicyResult::passed());
        }

        let mut result = PolicyResult::passed();

        for rule in &self.rules {
            let satisfied = rule.evaluate(context)?;

            if !satisfied {
                use crate::result::{Severity, Violation};

                let message = format!(
                    "Policy '{}' rule {} violated: fact '{}' did not match pattern '{}'",
                    self.name, rule.rule_type, rule.fact, rule.pattern
                );

                let violation = Violation::new(
                    self.name.clone(),
                    rule.rule_type.to_string(),
                    &rule.pattern,
                    Severity::Error,
                    message,
                );

                result.add_violation(violation);
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::RuleType;

    #[test]
    fn test_policy_creation() {
        let policy = Policy::new("test_policy");
        assert_eq!(policy.name, "test_policy");
        assert!(policy.enabled);
        assert!(policy.rules.is_empty());
    }

    #[test]
    fn test_policy_with_description() {
        let policy = Policy::new("test_policy").with_description("Test description");
        assert_eq!(policy.description, Some("Test description".to_string()));
    }

    #[test]
    fn test_policy_add_rule() {
        let policy = Policy::new("test_policy")
            .add_rule(Rule::new(RuleType::Allow, "status", "active"))
            .add_rule(Rule::new(RuleType::Deny, "role", "admin"));

        assert_eq!(policy.rules.len(), 2);
    }

    #[test]
    fn test_policy_disabled() {
        let policy = Policy::new("test_policy")
            .set_enabled(false)
            .add_rule(Rule::new(RuleType::Require, "email", ".*"));

        let ctx = EvaluationContext::new();
        let result = policy.evaluate(&ctx).unwrap();

        // Disabled policy always passes
        assert!(result.passed);
    }

    #[test]
    fn test_policy_evaluate_passing() {
        let rule = Rule::new(RuleType::Allow, "status", "^active$");
        let policy = Policy::new("test_policy").add_rule(rule);

        let mut ctx = EvaluationContext::new();
        ctx.set_string("status", "active");

        let result = policy.evaluate(&ctx).unwrap();
        assert!(result.passed);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_policy_evaluate_failing() {
        let rule = Rule::new(RuleType::Require, "email", "^[a-z]+@example\\.com$");
        let policy = Policy::new("test_policy").add_rule(rule);

        let ctx = EvaluationContext::new(); // email missing

        let result = policy.evaluate(&ctx).unwrap();
        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
    }
}
