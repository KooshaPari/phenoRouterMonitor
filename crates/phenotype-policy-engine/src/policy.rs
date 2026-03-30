// Policy abstraction and trait definitions.

use crate::context::EvaluationContext;
use crate::result::{PolicyResult, Violation};
use crate::rule::Rule;
use serde::{Deserialize, Serialize};

// Trait for evaluable policies.
pub trait EvaluablePolicy: Send + Sync {
    fn name(&self) -> &str;
    fn evaluate(
        &self,
        context: &EvaluationContext,
    ) -> Result<PolicyResult, crate::error::PolicyEngineError>;
}

// A concrete policy implementation with rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub name: String,
    pub description: Option<String>,
    pub rules: Vec<Rule>,
    pub enabled: bool,
}

impl Policy {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            rules: Vec::new(),
            enabled: true,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn add_rule(mut self, rule: Rule) -> Self {
        self.rules.push(rule);
        self
    }

    pub fn set_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn rules_by_priority(&self) -> Vec<&Rule> {
        let mut rules: Vec<&Rule> = self.rules.iter().collect();
        rules.sort_by_key(|r| r.priority);
        rules
    }

    pub fn evaluate_with_priority(
        &self,
        context: &EvaluationContext,
    ) -> Result<PolicyResult, crate::error::PolicyEngineError> {
        if !self.enabled {
            return Ok(PolicyResult::passed());
        }

        let mut result = PolicyResult::passed();
        let sorted_rules = self.rules_by_priority();

        for rule in sorted_rules {
            let satisfied = rule.evaluate(context)?;

            if !satisfied {
                let message = format!(
                    "Policy '{}' rule {} violated: fact '{}' did not match pattern '{}'",
                    self.name, rule.rule_type, rule.fact, rule.pattern
                );

                let violation = Violation::new(
                    self.name.clone(),
                    rule.rule_type.to_string(),
                    &rule.pattern,
                    rule.severity,
                    message,
                );

                result.add_violation(violation);
            }
        }

        Ok(result)
    }
}

impl EvaluablePolicy for Policy {
    fn name(&self) -> &str {
        &self.name
    }

    fn evaluate(
        &self,
        context: &EvaluationContext,
    ) -> Result<PolicyResult, crate::error::PolicyEngineError> {
        self.evaluate_with_priority(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::result::Severity;
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

    #[test]
    fn test_rules_by_priority() {
        let policy = Policy::new("test")
            .add_rule(Rule::new(RuleType::Allow, "a", ".*").with_priority(100))
            .add_rule(Rule::new(RuleType::Allow, "b", ".*").with_priority(1))
            .add_rule(Rule::new(RuleType::Allow, "c", ".*").with_priority(50));

        let sorted = policy.rules_by_priority();
        assert_eq!(sorted[0].fact, "b"); // priority 1
        assert_eq!(sorted[1].fact, "c"); // priority 50
        assert_eq!(sorted[2].fact, "a"); // priority 100
    }

    #[test]
    fn test_rules_same_priority_preserve_order() {
        let policy = Policy::new("test")
            .add_rule(Rule::new(RuleType::Allow, "first", ".*").with_priority(50))
            .add_rule(Rule::new(RuleType::Allow, "second", ".*").with_priority(50))
            .add_rule(Rule::new(RuleType::Allow, "third", ".*").with_priority(50));

        let sorted = policy.rules_by_priority();
        assert_eq!(sorted[0].fact, "first");
        assert_eq!(sorted[1].fact, "second");
        assert_eq!(sorted[2].fact, "third");
    }

    #[test]
    fn test_evaluate_with_priority_early_deny() {
        let policy = Policy::new("security")
            .add_rule(
                Rule::new(RuleType::Deny, "status", "^banned$")
                    .with_priority(1)
                    .with_description("Block banned users"),
            )
            .add_rule(
                Rule::new(RuleType::Require, "email", "^admin@")
                    .with_priority(10)
                    .with_description("Require admin email"),
            );

        let mut ctx = EvaluationContext::new();
        ctx.set_string("status", "banned");
        ctx.set_string("email", "user@example.com");

        let result = policy.evaluate(&ctx).unwrap();
        assert!(!result.passed);

        let deny_violation = result.violations.iter().find(|v| v.rule_type == "Deny");
        assert!(deny_violation.is_some());
    }

    // Traces to: FR-POL-001, FR-POL-002
    #[test]
    fn test_policy_violation_uses_rule_severity() {
        let policy = Policy::new("warning_policy").add_rule(
            Rule::new(RuleType::Require, "status", "^active$").with_severity(Severity::Warning),
        );

        let ctx = EvaluationContext::new(); // status missing

        let result = policy.evaluate(&ctx).unwrap();
        assert!(
            !result.passed,
            "Policy should fail when required fact is missing"
        );
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].severity, Severity::Warning);
    }
}
