//! Policy engine - evaluates contexts against a set of policies.

use dashmap::DashMap;
use serde::{Deserialize, Serialize};

use crate::context::EvaluationContext;
use crate::error::PolicyEngineError;
use crate::policy::{EvaluablePolicy, Policy};
use crate::result::PolicyResult;

/// A thread-safe policy engine that evaluates contexts against multiple policies.
///
/// The engine maintains a set of policies and can evaluate an EvaluationContext
/// against all of them concurrently.
#[derive(Debug)]
pub struct PolicyEngine {
    policies: DashMap<String, Policy>,
}

impl PolicyEngine {
    /// Creates a new empty policy engine.
    pub fn new() -> Self {
        Self {
            policies: DashMap::new(),
        }
    }

    /// Creates a policy engine from a list of policies.
    pub fn with_policies(policies: Vec<Policy>) -> Self {
        let engine = Self::new();
        for policy in policies {
            engine.add_policy(policy);
        }
        engine
    }

    /// Adds a policy to the engine.
    pub fn add_policy(&self, policy: Policy) {
        self.policies.insert(policy.name.clone(), policy);
    }

    /// Removes a policy from the engine by name.
    pub fn remove_policy(&self, name: &str) -> Option<Policy> {
        self.policies.remove(name).map(|(_, p)| p)
    }

    /// Gets a policy by name.
    pub fn get_policy(&self, name: &str) -> Option<Policy> {
        self.policies.get(name).map(|p| p.clone())
    }

    /// Gets the number of policies in the engine.
    pub fn policy_count(&self) -> usize {
        self.policies.len()
    }

    /// Lists all policy names.
    pub fn policy_names(&self) -> Vec<String> {
        self.policies
            .iter()
            .map(|ref_multi| ref_multi.key().clone())
            .collect()
    }

    /// Evaluates a context against a single policy by name.
    pub fn evaluate_policy(
        &self,
        policy_name: &str,
        context: &EvaluationContext,
    ) -> Result<PolicyResult, PolicyEngineError> {
        let policy = self
            .get_policy(policy_name)
            .ok_or(PolicyEngineError::PolicyNotFound {
                name: policy_name.to_string(),
            })?;

        policy.evaluate(context)
    }

    /// Evaluates a context against all policies in the engine.
    ///
    /// Returns a combined result with all violations from all policies.
    pub fn evaluate_all(
        &self,
        context: &EvaluationContext,
    ) -> Result<PolicyResult, PolicyEngineError> {
        let mut combined_result = PolicyResult::passed();

        for policy_ref in self.policies.iter() {
            let policy = policy_ref.value();
            let result = policy.evaluate(context)?;

            // Merge violations
            for violation in result.violations {
                combined_result.add_violation(violation);
            }
        }

        Ok(combined_result)
    }

    /// Evaluates a context against a subset of policies.
    pub fn evaluate_subset(
        &self,
        policy_names: &[&str],
        context: &EvaluationContext,
    ) -> Result<PolicyResult, PolicyEngineError> {
        let mut combined_result = PolicyResult::passed();

        for name in policy_names {
            let result = self.evaluate_policy(name, context)?;

            // Merge violations
            for violation in result.violations {
                combined_result.add_violation(violation);
            }
        }

        Ok(combined_result)
    }

    /// Enables a policy.
    pub fn enable_policy(&self, name: &str) -> Result<(), PolicyEngineError> {
        if let Some(mut policy) = self.policies.get_mut(name) {
            policy.enabled = true;
            Ok(())
        } else {
            Err(PolicyEngineError::PolicyNotFound {
                name: name.to_string(),
            })
        }
    }

    /// Disables a policy.
    pub fn disable_policy(&self, name: &str) -> Result<(), PolicyEngineError> {
        if let Some(mut policy) = self.policies.get_mut(name) {
            policy.enabled = false;
            Ok(())
        } else {
            Err(PolicyEngineError::PolicyNotFound {
                name: name.to_string(),
            })
        }
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for policy engine persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEngineConfig {
    /// Policies to load.
    pub policies: Vec<Policy>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rule::{Rule, RuleType};

    #[test]
    fn test_engine_new() {
        let engine = PolicyEngine::new();
        assert_eq!(engine.policy_count(), 0);
    }

    #[test]
    fn test_engine_add_policy() {
        let engine = PolicyEngine::new();
        let policy = Policy::new("test_policy");
        engine.add_policy(policy);

        assert_eq!(engine.policy_count(), 1);
        assert!(engine.get_policy("test_policy").is_some());
    }

    #[test]
    fn test_engine_remove_policy() {
        let engine = PolicyEngine::new();
        let policy = Policy::new("test_policy");
        engine.add_policy(policy);

        assert!(engine.remove_policy("test_policy").is_some());
        assert_eq!(engine.policy_count(), 0);
    }

    #[test]
    fn test_engine_with_policies() {
        let policies = vec![
            Policy::new("policy1"),
            Policy::new("policy2"),
            Policy::new("policy3"),
        ];
        let engine = PolicyEngine::with_policies(policies);

        assert_eq!(engine.policy_count(), 3);
    }

    #[test]
    fn test_engine_policy_names() {
        let engine = PolicyEngine::new();
        engine.add_policy(Policy::new("policy1"));
        engine.add_policy(Policy::new("policy2"));

        let names = engine.policy_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"policy1".to_string()));
        assert!(names.contains(&"policy2".to_string()));
    }

    #[test]
    fn test_engine_evaluate_single_policy() {
        let engine = PolicyEngine::new();
        let rule = Rule::new(RuleType::Allow, "status", "^active$");
        let policy = Policy::new("status_policy").add_rule(rule);
        engine.add_policy(policy);

        let mut ctx = EvaluationContext::new();
        ctx.set_string("status", "active");

        let result = engine.evaluate_policy("status_policy", &ctx).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_engine_evaluate_all() {
        let engine = PolicyEngine::new();
        engine.add_policy(Policy::new("policy1").add_rule(Rule::new(
            RuleType::Allow,
            "status",
            "^active$",
        )));
        engine.add_policy(Policy::new("policy2").add_rule(Rule::new(
            RuleType::Deny,
            "role",
            "^admin$",
        )));

        let mut ctx = EvaluationContext::new();
        ctx.set_string("status", "active");
        ctx.set_string("role", "user");

        let result = engine.evaluate_all(&ctx).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_engine_evaluate_subset() {
        let engine = PolicyEngine::new();
        engine.add_policy(Policy::new("policy1").add_rule(Rule::new(
            RuleType::Allow,
            "status",
            "^active$",
        )));
        engine.add_policy(Policy::new("policy2").add_rule(Rule::new(
            RuleType::Deny,
            "role",
            "^admin$",
        )));
        engine.add_policy(Policy::new("policy3"));

        let mut ctx = EvaluationContext::new();
        ctx.set_string("status", "active");

        let result = engine.evaluate_subset(&["policy1"], &ctx).unwrap();
        assert!(result.passed);
    }

    #[test]
    fn test_engine_enable_disable() {
        let engine = PolicyEngine::new();
        let policy = Policy::new("test_policy")
            .set_enabled(true)
            .add_rule(Rule::new(RuleType::Require, "email", ".*"));
        engine.add_policy(policy);

        engine.disable_policy("test_policy").unwrap();
        let disabled_policy = engine.get_policy("test_policy").unwrap();
        assert!(!disabled_policy.enabled);

        engine.enable_policy("test_policy").unwrap();
        let enabled_policy = engine.get_policy("test_policy").unwrap();
        assert!(enabled_policy.enabled);
    }

    #[test]
    fn test_engine_policy_not_found() {
        let engine = PolicyEngine::new();
        let ctx = EvaluationContext::new();

        let result = engine.evaluate_policy("nonexistent", &ctx);
        assert!(result.is_err());
    }
}
