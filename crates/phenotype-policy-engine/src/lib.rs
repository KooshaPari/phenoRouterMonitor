//! Generic policy evaluation engine for Phenotype.
//!
//! Supports rule-based policy decision-making with various operators and effects.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

/// Policy evaluation errors.
#[derive(Debug, Error)]
pub enum PolicyError {
    #[error("field not found in context: {0}")]
    FieldNotFound(String),
    #[error("type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },
    #[error("rule evaluation failed: {0}")]
    RuleEvaluationFailed(String),
}

/// Operators for rule comparison.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum Operator {
    Eq,
    NotEq,
    Contains,
    NotContains,
    Gt,
    Gte,
    Lt,
    Lte,
}

impl std::str::FromStr for Operator {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Eq" => Ok(Operator::Eq),
            "NotEq" => Ok(Operator::NotEq),
            "Contains" => Ok(Operator::Contains),
            "NotContains" => Ok(Operator::NotContains),
            "Gt" => Ok(Operator::Gt),
            "Gte" => Ok(Operator::Gte),
            "Lt" => Ok(Operator::Lt),
            "Lte" => Ok(Operator::Lte),
            _ => Err(format!("Unknown operator: {}", s)),
        }
    }
}

/// Policy effect: Allow or Deny.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum Effect {
    Allow,
    Deny,
}

/// Get the type name of a JSON value.
fn value_type_name(val: &Value) -> &'static str {
    match val {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// A single rule within a policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub field: String,
    pub operator: Operator,
    pub value: Value,
}

impl Rule {
    /// Create a new rule.
    pub fn new(field: impl Into<String>, operator: Operator, value: Value) -> Self {
        Self {
            field: field.into(),
            operator,
            value,
        }
    }

    /// Evaluate rule against context value.
    pub fn evaluate(&self, context_value: &Value) -> Result<bool, PolicyError> {
        match self.operator {
            Operator::Eq => Ok(context_value == &self.value),
            Operator::NotEq => Ok(context_value != &self.value),
            Operator::Contains => {
                let context_str = context_value
                    .as_str()
                    .ok_or_else(|| PolicyError::TypeMismatch {
                        expected: "string".to_string(),
                        actual: value_type_name(context_value).to_string(),
                    })?;
                let value_str = self.value.as_str().ok_or_else(|| PolicyError::TypeMismatch {
                    expected: "string".to_string(),
                    actual: value_type_name(&self.value).to_string(),
                })?;
                Ok(context_str.contains(value_str))
            }
            Operator::NotContains => {
                let context_str = context_value
                    .as_str()
                    .ok_or_else(|| PolicyError::TypeMismatch {
                        expected: "string".to_string(),
                        actual: value_type_name(context_value).to_string(),
                    })?;
                let value_str = self.value.as_str().ok_or_else(|| PolicyError::TypeMismatch {
                    expected: "string".to_string(),
                    actual: value_type_name(&self.value).to_string(),
                })?;
                Ok(!context_str.contains(value_str))
            }
            Operator::Gt => {
                let context_num = context_value.as_f64().ok_or_else(|| PolicyError::TypeMismatch {
                    expected: "number".to_string(),
                    actual: value_type_name(context_value).to_string(),
                })?;
                let value_num = self.value.as_f64().ok_or_else(|| PolicyError::TypeMismatch {
                    expected: "number".to_string(),
                    actual: value_type_name(&self.value).to_string(),
                })?;
                Ok(context_num > value_num)
            }
            Operator::Gte => {
                let context_num = context_value.as_f64().ok_or_else(|| PolicyError::TypeMismatch {
                    expected: "number".to_string(),
                    actual: value_type_name(context_value).to_string(),
                })?;
                let value_num = self.value.as_f64().ok_or_else(|| PolicyError::TypeMismatch {
                    expected: "number".to_string(),
                    actual: value_type_name(&self.value).to_string(),
                })?;
                Ok(context_num >= value_num)
            }
            Operator::Lt => {
                let context_num = context_value.as_f64().ok_or_else(|| PolicyError::TypeMismatch {
                    expected: "number".to_string(),
                    actual: value_type_name(context_value).to_string(),
                })?;
                let value_num = self.value.as_f64().ok_or_else(|| PolicyError::TypeMismatch {
                    expected: "number".to_string(),
                    actual: value_type_name(&self.value).to_string(),
                })?;
                Ok(context_num < value_num)
            }
            Operator::Lte => {
                let context_num = context_value.as_f64().ok_or_else(|| PolicyError::TypeMismatch {
                    expected: "number".to_string(),
                    actual: value_type_name(context_value).to_string(),
                })?;
                let value_num = self.value.as_f64().ok_or_else(|| PolicyError::TypeMismatch {
                    expected: "number".to_string(),
                    actual: value_type_name(&self.value).to_string(),
                })?;
                Ok(context_num <= value_num)
            }
        }
    }
}

/// A policy with rules and effect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub name: String,
    pub rules: Vec<Rule>,
    pub effect: Effect,
}

impl Policy {
    /// Create a new policy.
    pub fn new(name: impl Into<String>, effect: Effect) -> Self {
        Self {
            name: name.into(),
            rules: Vec::new(),
            effect,
        }
    }

    /// Add a rule to the policy.
    pub fn with_rule(mut self, rule: Rule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Evaluate all rules in the policy (AND logic).
    pub fn evaluate_rules(&self, context: &Value) -> Result<bool, PolicyError> {
        for rule in &self.rules {
            let context_value = context
                .get(&rule.field)
                .ok_or_else(|| PolicyError::FieldNotFound(rule.field.clone()))?;

            if !rule.evaluate(context_value)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

/// Policy decision result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow,
    Deny(String),
}

impl std::fmt::Display for PolicyDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicyDecision::Allow => write!(f, "Allow"),
            PolicyDecision::Deny(reason) => write!(f, "Deny: {}", reason),
        }
    }
}

/// Policy evaluation engine.
pub struct PolicyEngine;

impl PolicyEngine {
    /// Evaluate a list of policies against context.
    ///
    /// Returns Allow if any Allow policy matches.
    /// Returns Deny if any Deny policy matches and no Allow policy matches.
    pub fn evaluate(policies: &[Policy], context: &Value) -> PolicyDecision {
        let mut deny_reasons = Vec::new();

        for policy in policies {
            match policy.evaluate_rules(context) {
                Ok(true) => {
                    if policy.effect == Effect::Allow {
                        return PolicyDecision::Allow;
                    } else {
                        deny_reasons.push(format!("Policy '{}' denied access", policy.name));
                    }
                }
                Ok(false) => {
                    // Rules didn't match, continue to next policy
                }
                Err(e) => {
                    deny_reasons.push(format!("Policy '{}' error: {}", policy.name, e));
                }
            }
        }

        if deny_reasons.is_empty() {
            PolicyDecision::Deny("No matching policy found".to_string())
        } else {
            PolicyDecision::Deny(deny_reasons.join("; "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_eq() {
        let rule = Rule::new("role", Operator::Eq, Value::String("admin".to_string()));
        let context = serde_json::json!({ "role": "admin" });
        assert!(rule.evaluate(context.get("role").unwrap()).unwrap());
    }

    #[test]
    fn test_rule_not_eq() {
        let rule = Rule::new("role", Operator::NotEq, Value::String("guest".to_string()));
        let context = serde_json::json!({ "role": "admin" });
        assert!(rule.evaluate(context.get("role").unwrap()).unwrap());
    }

    #[test]
    fn test_rule_contains() {
        let rule = Rule::new(
            "email",
            Operator::Contains,
            Value::String("example.com".to_string()),
        );
        let context = serde_json::json!({ "email": "user@example.com" });
        assert!(rule.evaluate(context.get("email").unwrap()).unwrap());
    }

    #[test]
    fn test_rule_gt() {
        let rule = Rule::new("age", Operator::Gt, Value::Number(18.into()));
        let context = serde_json::json!({ "age": 25 });
        assert!(rule.evaluate(context.get("age").unwrap()).unwrap());
    }

    #[test]
    fn test_rule_lt() {
        let rule = Rule::new("age", Operator::Lt, Value::Number(18.into()));
        let context = serde_json::json!({ "age": 25 });
        assert!(!rule.evaluate(context.get("age").unwrap()).unwrap());
    }

    #[test]
    fn test_policy_allow() {
        let policy = Policy::new("admin_policy", Effect::Allow)
            .with_rule(Rule::new("role", Operator::Eq, Value::String("admin".to_string())));

        let context = serde_json::json!({ "role": "admin" });
        let decision = PolicyEngine::evaluate(&[policy], &context);
        assert_eq!(decision, PolicyDecision::Allow);
    }

    #[test]
    fn test_policy_deny() {
        let policy = Policy::new("admin_policy", Effect::Deny)
            .with_rule(Rule::new("role", Operator::Eq, Value::String("guest".to_string())));

        let context = serde_json::json!({ "role": "guest" });
        let decision = PolicyEngine::evaluate(&[policy], &context);
        assert!(matches!(decision, PolicyDecision::Deny(_)));
    }

    #[test]
    fn test_multiple_rules_all_must_match() {
        let policy = Policy::new("complex_policy", Effect::Allow)
            .with_rule(Rule::new("role", Operator::Eq, Value::String("admin".to_string())))
            .with_rule(Rule::new("age", Operator::Gte, Value::Number(18.into())));

        let context = serde_json::json!({ "role": "admin", "age": 25 });
        assert_eq!(
            PolicyEngine::evaluate(&[policy], &context),
            PolicyDecision::Allow
        );
    }

    #[test]
    fn test_multiple_rules_one_fails() {
        let policy = Policy::new("complex_policy", Effect::Allow)
            .with_rule(Rule::new("role", Operator::Eq, Value::String("admin".to_string())))
            .with_rule(Rule::new("age", Operator::Gte, Value::Number(18.into())));

        let context = serde_json::json!({ "role": "admin", "age": 15 });
        assert!(matches!(
            PolicyEngine::evaluate(&[policy], &context),
            PolicyDecision::Deny(_)
        ));
    }

    #[test]
    fn test_missing_field() {
        let policy = Policy::new("admin_policy", Effect::Allow)
            .with_rule(Rule::new("role", Operator::Eq, Value::String("admin".to_string())));

        let context = serde_json::json!({});
        assert!(matches!(
            PolicyEngine::evaluate(&[policy], &context),
            PolicyDecision::Deny(_)
        ));
    }
}
