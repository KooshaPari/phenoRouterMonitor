// Policy rule definitions with cached regex compilation.
use crate::error::PolicyEngineError;
use crate::result::Severity;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleType {
    Allow,
    Deny,
    Require,
}

impl RuleType {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub rule_type: RuleType,
    pub fact: String,
    pub pattern: String,
    pub description: Option<String>,
    #[serde(default = "default_priority")]
    pub priority: u32,
    #[serde(skip)]
    pub severity: Severity,
    #[serde(skip)]
    _compiled_regex: Option<regex::Regex>,
}

fn default_priority() -> u32 {
    100
}

impl Rule {
    pub fn new(rule_type: RuleType, fact: impl Into<String>, pattern: impl Into<String>) -> Self {
        let pattern = pattern.into();
        Self {
            rule_type,
            fact: fact.into(),
            pattern: pattern.clone(),
            description: None,
            priority: 100,
            severity: Severity::Error,
            _compiled_regex: Regex::new(&pattern).ok(),
        }
    }

    pub fn with_description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }

    pub fn with_priority(mut self, p: u32) -> Self {
        self.priority = p;
        self
    }

    pub fn with_severity(mut self, s: Severity) -> Self {
        self.severity = s;
        self
    }

    pub fn evaluate(&self, context: &EvaluationContext) -> Result<bool, PolicyEngineError> {
        let regex = match &self._compiled_regex {
            Some(r) => r.clone(),
            None => {
                Regex::new(&self.pattern).map_err(|e| {
                    PolicyEngineError::RegexCompilationError {
                        pattern: self.pattern.clone(),
                        source: e,
                    }
                })?
            }
        };

        let fact_value = context.get_string(&self.fact);
        match self.rule_type {
            RuleType::Allow => Ok(fact_value.map_or(true, |v| regex.is_match(&v))),
            RuleType::Deny => Ok(fact_value.is_some_and(|v| !regex.is_match(v))),
            RuleType::Require => Ok(fact_value.is_some_and(|v| regex.is_match(v))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_type_display() {
        assert_eq!(RuleType::Allow.as_str(), "Allow");
    }

    #[test]
    fn test_allow_rule_matching() {
        let r = Rule::new(RuleType::Allow, "s", "^active$");
        let mut ctx = EvaluationContext::new();
        ctx.set_string("s", "active");
        assert!(r.evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_allow_rule_non_matching() {
        let r = Rule::new(RuleType::Allow, "s", "^active$");
        let mut ctx = EvaluationContext::new();
        ctx.set_string("s", "inactive");
        assert!(!r.evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_allow_rule_missing_fact() {
        let r = Rule::new(RuleType::Allow, "s", "^active$");
        assert!(r.evaluate(&EvaluationContext::new()).unwrap());
    }

    #[test]
    fn test_deny_rule_matching() {
        let r = Rule::new(RuleType::Deny, "s", "^banned$");
        let mut ctx = EvaluationContext::new();
        ctx.set_string("s", "banned");
        assert!(!r.evaluate(&ctx).unwrap());
    }

    #[test]
    fn test_require_rule_missing() {
        let r = Rule::new(RuleType::Require, "e", ".*");
        assert!(!r.evaluate(&EvaluationContext::new()).unwrap());
    }

    #[test]
    fn test_invalid_regex() {
        let r = Rule::new(RuleType::Allow, "f", "[invalid");
        assert!(r.evaluate(&EvaluationContext::new()).is_err());
    }

    #[test]
    fn test_rule_with_priority() {
        let r = Rule::new(RuleType::Allow, "r", "admin").with_priority(10);
        assert_eq!(r.priority, 10);
    }

    #[test]
    fn test_rule_default_priority() {
        assert_eq!(Rule::new(RuleType::Allow, "r", "admin").priority, 100);
    }

    #[test]
    fn test_rule_with_severity() {
        let r = Rule::new(RuleType::Require, "e", ".*").with_severity(Severity::Warning);
        assert_eq!(r.severity, Severity::Warning);
    }

    #[test]
    fn test_rule_default_severity() {
        assert_eq!(
            Rule::new(RuleType::Deny, "s", "banned").severity,
            Severity::Error
        );
    }
}
