// Policy loader for TOML configuration files.

use crate::error::PolicyEngineError;
use crate::policy::Policy;
use crate::result::Severity;
use crate::rule::{Rule, RuleType};
use serde::Deserialize;
use std::fs;
use std::path::Path;

fn default_priority() -> u32 { 100 }
fn default_severity_str() -> String { "Error".to_string() }
fn default_enabled() -> bool { true }

#[derive(Debug, Deserialize)]
struct RuleConfig {
    rule_type: String,
    fact: String,
    pattern: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default = "default_priority")]
    priority: u32,
    #[serde(default = "default_severity_str")]
    severity: String,
}

#[derive(Debug, Deserialize)]
struct PolicyConfig {
    name: String,
    #[serde(default)]
    description: Option<String>,
    rules: Vec<RuleConfig>,
    #[serde(default = "default_enabled")]
    enabled: bool,
}

#[derive(Debug, Deserialize)]
struct PoliciesConfigFile {
    policies: Vec<PolicyConfig>,
}

impl RuleConfig {
    fn to_rule(&self) -> Result<Rule, PolicyEngineError> {
        let rule_type = match self.rule_type.to_lowercase().as_str() {
            "allow" => RuleType::Allow,
            "deny" => RuleType::Deny,
            "require" => RuleType::Require,
            other => return Err(PolicyEngineError::RuleValidationError {
                message: format!("Invalid rule type: '{}'", other),
            }),
        };

        let severity = match self.severity.to_lowercase().as_str() {
            "info" => Severity::Info,
            "warning" => Severity::Warning,
            "error" => Severity::Error,
            other => return Err(PolicyEngineError::RuleValidationError {
                message: format!("Invalid severity: '{}'. Expected Info, Warning, or Error", other),
            }),
        };

        let _ = regex::Regex::new(&self.pattern).map_err(|e| {
            PolicyEngineError::RegexCompilationError {
                pattern: self.pattern.clone(),
                source: e,
            }
        })?;

        let mut rule = Rule::new(rule_type, &self.fact, &self.pattern)
            .with_priority(self.priority)
            .with_severity(severity);

        if let Some(desc) = &self.description {
            rule = rule.with_description(desc.clone());
        }

        Ok(rule)
    }
}

impl PolicyConfig {
    fn to_policy(&self) -> Result<Policy, PolicyEngineError> {
        let mut policy = Policy::new(&self.name).set_enabled(self.enabled);

        if let Some(desc) = &self.description {
            policy = policy.with_description(desc.clone());
        }

        for rule_config in &self.rules {
            let rule = rule_config.to_rule()?;
            policy = policy.add_rule(rule);
        }

        Ok(policy)
    }
}

impl PoliciesConfigFile {
    fn from_string(content: &str) -> Result<Self, PolicyEngineError> {
        Ok(toml::from_str(content)?)
    }

    fn from_file(path: &Path) -> Result<Self, PolicyEngineError> {
        let content = fs::read_to_string(path)?;
        Self::from_string(&content)
    }

    fn to_policies(&self) -> Result<Vec<Policy>, PolicyEngineError> {
        self.policies.iter().map(|p| p.to_policy()).collect()
    }
}

pub struct PolicyLoader;

impl PolicyLoader {
    pub fn from_file(path: &Path) -> Result<Vec<Policy>, PolicyEngineError> {
        let config = PoliciesConfigFile::from_file(path)?;
        config.to_policies()
    }

    pub fn from_string(content: &str) -> Result<Vec<Policy>, PolicyEngineError> {
        let config = PoliciesConfigFile::from_string(content)?;
        config.to_policies()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_config_to_rule_allow() {
        let config = RuleConfig {
            rule_type: "Allow".to_string(),
            fact: "status".to_string(),
            pattern: "^active$".to_string(),
            description: None,
            priority: 100,
            severity: "Error".to_string(),
        };
        let rule = config.to_rule().unwrap();
        assert_eq!(rule.rule_type, RuleType::Allow);
    }

    #[test]
    fn test_invalid_rule_type() {
        let config = RuleConfig {
            rule_type: "Invalid".to_string(),
            fact: "field".to_string(),
            pattern: ".*".to_string(),
            description: None,
            priority: 100,
            severity: "Error".to_string(),
        };
        assert!(config.to_rule().is_err());
    }

    #[test]
    fn test_invalid_severity() {
        let config = RuleConfig {
            rule_type: "Allow".to_string(),
            fact: "field".to_string(),
            pattern: ".*".to_string(),
            description: None,
            priority: 100,
            severity: "Invalid".to_string(),
        };
        assert!(config.to_rule().is_err());
    }

    #[test]
    fn test_policy_config_to_policy() {
        let config = PolicyConfig {
            name: "test".to_string(),
            description: Some("Test".to_string()),
            rules: vec![RuleConfig {
                rule_type: "Require".to_string(),
                fact: "email".to_string(),
                pattern: ".*".to_string(),
                description: None,
                priority: 100,
                severity: "Error".to_string(),
            }],
            enabled: true,
        };
        let policy = config.to_policy().unwrap();
        assert_eq!(policy.name, "test");
        assert_eq!(policy.rules.len(), 1);
    }

    #[test]
    fn test_policies_config_from_string() {
        let toml = r#"
[[policies]]
name = "test"
[[policies.rules]]
rule_type = "Allow"
fact = "status"
pattern = "^active$"
"#;
        let config = PoliciesConfigFile::from_string(toml).unwrap();
        assert_eq!(config.policies.len(), 1);
    }

    #[test]
    fn test_malformed_toml() {
        assert!(PoliciesConfigFile::from_string("not valid toml {{{{").is_err());
    }

    #[test]
    fn test_missing_required_fields() {
        let content = r#"[[policies]]
enabled = true"#;
        assert!(PoliciesConfigFile::from_string(content).is_err());
    }

    #[test]
    fn test_missing_rules_field() {
        let content = r#"[[policies]]
name = "no_rules""#;
        assert!(PoliciesConfigFile::from_string(content).is_err());
    }

    #[test]
    fn test_empty_rules_array() {
        let toml = r#"[[policies]]
name = "empty_rules"
rules = []
"#;
        let config = PoliciesConfigFile::from_string(toml).unwrap();
        assert!(config.policies[0].rules.is_empty());
    }

    #[test]
    fn test_invalid_regex_in_toml() {
        let toml = r#"
[[policies]]
name = "bad_regex"
[[policies.rules]]
rule_type = "Allow"
fact = "field"
pattern = "[invalid"
"#;
        let config = PoliciesConfigFile::from_string(toml).unwrap();
        assert!(config.to_policies().is_err());
    }

    #[test]
    fn test_policies_config_from_file() {
        use std::io::Write;
        let mut file = tempfile::NamedTempFile::new().unwrap();
        write!(file, r#"
[[policies]]
name = "from_file"
[[policies.rules]]
rule_type = "Allow"
fact = "field"
pattern = ".*"
"#).unwrap();
        let config = PoliciesConfigFile::from_file(file.path()).unwrap();
        assert_eq!(config.policies[0].name, "from_file");
    }
}
