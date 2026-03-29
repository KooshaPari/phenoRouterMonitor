//! Policy loading from TOML configuration files.

use crate::error::PolicyEngineError;
use crate::policy::Policy;
use crate::rule::{Rule, RuleType};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// TOML representation of a rule for loading.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleConfig {
    /// Type of rule: "Allow", "Deny", or "Require".
    pub r#type: String,
    /// The fact key to evaluate.
    pub fact: String,
    /// The regex pattern to match.
    pub pattern: String,
    /// Optional description.
    pub description: Option<String>,
}

impl RuleConfig {
    /// Converts this configuration to a Rule.
    pub fn to_rule(&self) -> Result<Rule, PolicyEngineError> {
        let rule_type = match self.r#type.to_lowercase().as_str() {
            "allow" => RuleType::Allow,
            "deny" => RuleType::Deny,
            "require" => RuleType::Require,
            invalid => {
                return Err(PolicyEngineError::InvalidConfiguration(format!(
                    "Invalid rule type: '{}'. Expected 'Allow', 'Deny', or 'Require'.",
                    invalid
                )))
            }
        };

        let mut rule = Rule::new(rule_type, self.fact.clone(), self.pattern.clone());
        if let Some(desc) = &self.description {
            rule = rule.with_description(desc.clone());
        }

        Ok(rule)
    }
}

/// TOML representation of a policy for loading.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    /// Name of the policy.
    pub name: String,
    /// Description of the policy.
    pub description: Option<String>,
    /// List of rules.
    pub rules: Vec<RuleConfig>,
    /// Whether the policy is enabled (default: true).
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

impl PolicyConfig {
    /// Converts this configuration to a Policy.
    pub fn to_policy(&self) -> Result<Policy, PolicyEngineError> {
        let mut policy = Policy::new(self.name.clone()).set_enabled(self.enabled);

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

/// Top-level configuration for loading policies from TOML.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoliciesConfigFile {
    /// Version of the configuration format.
    pub version: Option<String>,
    /// List of policies.
    pub policies: Vec<PolicyConfig>,
}

impl PoliciesConfigFile {
    /// Loads policies from a TOML file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, PolicyEngineError> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            PolicyEngineError::LoadError(format!("Failed to read policy file: {}", e))
        })?;

        Self::from_string(&content)
    }

    /// Parses policies from a TOML string.
    pub fn from_string(toml_str: &str) -> Result<Self, PolicyEngineError> {
        toml::from_str(toml_str)
            .map_err(|e| PolicyEngineError::SerializationError(format!("Failed to parse TOML: {}", e)))
    }

    /// Converts all configurations to Policy objects.
    pub fn to_policies(&self) -> Result<Vec<Policy>, PolicyEngineError> {
        self.policies.iter().map(|pc| pc.to_policy()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_rule_config_to_rule_allow() {
        let rule_config = RuleConfig {
            r#type: "Allow".to_string(),
            fact: "status".to_string(),
            pattern: "^active$".to_string(),
            description: Some("Status must be active".to_string()),
        };

        let rule = rule_config.to_rule().unwrap();
        assert_eq!(rule.rule_type, RuleType::Allow);
        assert_eq!(rule.fact, "status");
        assert_eq!(rule.description, Some("Status must be active".to_string()));
    }

    #[test]
    fn test_policy_config_to_policy() {
        let policy_config = PolicyConfig {
            name: "test_policy".to_string(),
            description: Some("A test policy".to_string()),
            rules: vec![RuleConfig {
                r#type: "Allow".to_string(),
                fact: "status".to_string(),
                pattern: "^active$".to_string(),
                description: None,
            }],
            enabled: true,
        };

        let policy = policy_config.to_policy().unwrap();
        assert_eq!(policy.name, "test_policy");
        assert_eq!(policy.rules.len(), 1);
        assert!(policy.enabled);
    }

    #[test]
    fn test_policies_config_from_string() {
        let toml_content = r#"
version = "1.0"

[[policies]]
name = "access_policy"
description = "Access control policy"
enabled = true

[[policies.rules]]
type = "Allow"
fact = "role"
pattern = "^(admin|user)$"
description = "User must have admin or user role"

[[policies.rules]]
type = "Deny"
fact = "status"
pattern = "^banned$"
"#;

        let config = PoliciesConfigFile::from_string(toml_content).unwrap();
        assert_eq!(config.policies.len(), 1);
        assert_eq!(config.policies[0].rules.len(), 2);
    }

    #[test]
    fn test_policies_config_from_file() {
        let mut tmpfile = NamedTempFile::new().unwrap();
        let toml_content = r#"
version = "1.0"

[[policies]]
name = "file_policy"
rules = []
enabled = true
"#;
        tmpfile.write_all(toml_content.as_bytes()).unwrap();
        tmpfile.flush().unwrap();

        let config = PoliciesConfigFile::from_file(tmpfile.path()).unwrap();
        assert_eq!(config.policies.len(), 1);
        assert_eq!(config.policies[0].name, "file_policy");
    }

    #[test]
    fn test_invalid_rule_type() {
        let rule_config = RuleConfig {
            r#type: "Invalid".to_string(),
            fact: "field".to_string(),
            pattern: ".*".to_string(),
            description: None,
        };

        let result = rule_config.to_rule();
        assert!(result.is_err());
    }

    #[test]
    fn test_policies_config_to_policies() {
        let policy_config = PoliciesConfigFile {
            version: Some("1.0".to_string()),
            policies: vec![
                PolicyConfig {
                    name: "policy1".to_string(),
                    description: None,
                    rules: vec![],
                    enabled: true,
                },
                PolicyConfig {
                    name: "policy2".to_string(),
                    description: None,
                    rules: vec![],
                    enabled: false,
                },
            ],
        };

        let policies = policy_config.to_policies().unwrap();
        assert_eq!(policies.len(), 2);
        assert_eq!(policies[0].name, "policy1");
        assert_eq!(policies[1].name, "policy2");
    }
}
