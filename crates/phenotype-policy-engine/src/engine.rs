// Policy engine for managing and evaluating policies.

use crate::context::EvaluationContext;
use crate::error::PolicyEngineError;
use crate::policy::{EvaluablePolicy, Policy};
use crate::result::PolicyResult;
use dashmap::DashMap;
use std::sync::Arc;

// Thread-safe policy engine for managing and evaluating policies.
pub struct PolicyEngine {
    policies: Arc<DashMap<String, Policy>>,
}

impl PolicyEngine {
    pub fn new() -> Self {
        Self {
            policies: Arc::new(DashMap::new()),
        }
    }

    pub fn with_policies(policies: Vec<Policy>) -> Self {
        let engine = Self::new();
        for policy in policies {
            engine.add_policy(policy);
        }
        engine
    }

    pub fn add_policy(&self, policy: Policy) {
        self.policies.insert(policy.name().to_string(), policy);
    }

    pub fn remove_policy(&self, name: &str) -> Option<Policy> {
        self.policies.remove(name).map(|(_, p)| p)
    }

    pub fn get_policy(&self, name: &str) -> Option<std::sync::Arc<Policy>> {
        self.policies
            .get(name)
            .map(|p| p.value().clone())
            .map(Arc::new)
    }

    pub fn enable_policy(&self, name: &str) -> Result<(), PolicyEngineError> {
        match self.policies.get_mut(name) {
            Some(mut p) => {
                p.enabled = true;
                Ok(())
            }
            None => Err(PolicyEngineError::PolicyNotFound {
                name: name.to_string(),
            }),
        }
    }

    pub fn disable_policy(&self, name: &str) -> Result<(), PolicyEngineError> {
        match self.policies.get_mut(name) {
            Some(mut p) => {
                p.enabled = false;
                Ok(())
            }
            None => Err(PolicyEngineError::PolicyNotFound {
                name: name.to_string(),
            }),
        }
    }

    pub fn policy_names(&self) -> Vec<String> {
        self.policies.iter().map(|p| p.key().clone()).collect()
    }

    /// Evaluates all enabled policies and merges violations.
    pub fn evaluate_all(
        &self,
        context: &EvaluationContext,
    ) -> Result<PolicyResult, PolicyEngineError> {
        let mut result = PolicyResult::passed();

        for policy_ref in self.policies.iter() {
            let policy = policy_ref.value();
            // Skip disabled policies for efficiency
            if !policy.enabled {
                continue;
            }
            let policy_result = policy.evaluate(context)?;
            for violation in policy_result.violations {
                result.add_violation(violation);
            }
        }

        Ok(result)
    }

    /// Evaluates a single policy by name.
    pub fn evaluate_single(
        &self,
        name: &str,
        context: &EvaluationContext,
    ) -> Result<PolicyResult, PolicyEngineError> {
        let policy = self
            .get_policy(name)
            .ok_or_else(|| PolicyEngineError::PolicyNotFound {
                name: name.to_string(),
            })?;
        policy.evaluate(context)
    }

    /// Evaluates a subset of policies by name.
    pub fn evaluate_subset(
        &self,
        names: &[&str],
        context: &EvaluationContext,
    ) -> Result<PolicyResult, PolicyEngineError> {
        let mut result = PolicyResult::passed();

        for name in names {
            let policy =
                self.get_policy(name)
                    .ok_or_else(|| PolicyEngineError::PolicyNotFound {
                        name: name.to_string(),
                    })?;
            let policy_result = policy.evaluate(context)?;
            for violation in policy_result.violations {
                result.add_violation(violation);
            }
        }

        Ok(result)
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Tests for PolicyEngine.
/// Traces to: FR-POL-004
#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::{Rule, RuleType};

    // Traces to: FR-POL-004
    #[test]
    fn test_engine_new() {
        let engine = PolicyEngine::new();
        assert!(engine.policy_names().is_empty());
    }

    #[test]
    fn test_engine_add_policy() {
        let engine = PolicyEngine::new();
        engine.add_policy(Policy::new("test"));
        assert_eq!(engine.policy_names(), vec!["test"]);
    }

    // Traces to: FR-POL-004
    #[test]
    fn test_engine_remove_policy() {
        let engine = PolicyEngine::new();
        engine.add_policy(Policy::new("test"));
        let removed = engine.remove_policy("test");
        assert!(removed.is_some());
        assert!(engine.policy_names().is_empty());
    }

    #[test]
    fn test_engine_policy_not_found() {
        let engine = PolicyEngine::new();
        let err = engine.enable_policy("missing").unwrap_err();
        assert!(matches!(err, PolicyEngineError::PolicyNotFound { .. }));
    }

    // Traces to: FR-POL-004
    #[test]
    fn test_engine_enable_disable() {
        let engine = PolicyEngine::new();
        engine.add_policy(Policy::new("test"));
        assert!(engine.enable_policy("test").is_ok());
        assert!(engine.disable_policy("test").is_ok());
    }

    #[test]
    fn test_engine_evaluate_single_policy() {
        let engine = PolicyEngine::new();
        let policy = Policy::new("test").add_rule(Rule::new(RuleType::Require, "email", ".*"));
        engine.add_policy(policy);

        let mut ctx = EvaluationContext::new();
        ctx.set_string("email", "test@example.com");

        let result = engine.evaluate_single("test", &ctx).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_engine_evaluate_all() {
        let engine = PolicyEngine::new();
        engine.add_policy(Policy::new("p1").add_rule(Rule::new(RuleType::Require, "a", ".*")));
        engine.add_policy(Policy::new("p2").add_rule(Rule::new(RuleType::Require, "b", ".*")));

        let mut ctx = EvaluationContext::new();
        ctx.set_string("a", "1");
        ctx.set_string("b", "2");

        let result = engine.evaluate_all(&ctx).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_engine_evaluate_all_skips_disabled() {
        let engine = PolicyEngine::new();
        engine.add_policy(Policy::new("enabled").add_rule(Rule::new(RuleType::Require, "x", ".*")));
        let disabled_policy = Policy::new("disabled")
            .set_enabled(false)
            .add_rule(Rule::new(RuleType::Require, "y", ".*"));
        engine.add_policy(disabled_policy);

        let ctx = EvaluationContext::new(); // both x and y are missing

        let result = engine.evaluate_all(&ctx).unwrap();
        // Only the enabled policy should contribute violations
        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].policy_name, "enabled");
    }

    #[test]
    fn test_engine_evaluate_subset() {
        let engine = PolicyEngine::new();
        engine.add_policy(Policy::new("p1").add_rule(Rule::new(RuleType::Require, "a", ".*")));
        engine.add_policy(Policy::new("p2").add_rule(Rule::new(RuleType::Require, "b", ".*")));
        engine.add_policy(Policy::new("p3").add_rule(Rule::new(RuleType::Require, "c", ".*")));

        let ctx = EvaluationContext::new();

        let result = engine.evaluate_subset(&["p1", "p3"], &ctx).unwrap();
        assert!(!result.passed);
        assert_eq!(result.violations.len(), 2);
    }
}
