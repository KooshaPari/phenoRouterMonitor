//! Agent configuration traits and structures

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::Result;

/// Core trait for agent configuration
///
/// Defines the interface that all agent configurations must implement.
/// This allows agents to be loaded from YAML without Rust code changes.
#[async_trait]
pub trait SubagentConfig: Send + Sync + std::fmt::Debug {
    /// Unique identifier for the agent
    fn id(&self) -> &str;

    /// Human-readable name for the agent
    fn name(&self) -> &str;

    /// Description of what the agent does
    fn description(&self) -> &str;

    /// Instruction/prompt for the agent
    fn instruction(&self) -> &str;

    /// JSON Schema for expected input
    fn input_schema(&self) -> &serde_json::Value;

    /// JSON Schema for expected output
    fn output_schema(&self) -> &serde_json::Value;

    /// Get agent metadata as a map
    fn metadata(&self) -> HashMap<String, String>;

    /// Validate the configuration
    fn validate(&self) -> Result<()>;

    /// Initialize the agent (async setup)
    async fn initialize(&self) -> Result<()> {
        Ok(())
    }

    /// Get agent tags/categories
    fn tags(&self) -> Vec<&str> {
        vec![]
    }

    /// Check if agent is enabled
    fn is_enabled(&self) -> bool {
        true
    }
}

/// YAML-loadable agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent unique identifier
    pub id: String,

    /// Agent name
    pub name: String,

    /// Agent description
    pub description: String,

    /// Agent instruction/prompt
    pub instruction: String,

    /// Expected input JSON schema
    pub input_schema: serde_json::Value,

    /// Expected output JSON schema
    pub output_schema: serde_json::Value,

    /// Optional metadata key-value pairs
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    /// Optional tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,

    /// Whether the agent is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Version of the agent configuration
    #[serde(default = "default_version")]
    pub version: String,

    /// Custom agent properties (extensible)
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

fn default_enabled() -> bool {
    true
}

fn default_version() -> String {
    "1.0.0".to_string()
}

#[async_trait]
impl SubagentConfig for AgentConfig {
    fn id(&self) -> &str {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn instruction(&self) -> &str {
        &self.instruction
    }

    fn input_schema(&self) -> &serde_json::Value {
        &self.input_schema
    }

    fn output_schema(&self) -> &serde_json::Value {
        &self.output_schema
    }

    fn metadata(&self) -> HashMap<String, String> {
        self.metadata.clone()
    }

    fn validate(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(crate::error::ForgeError::InvalidAgent(
                "Agent ID cannot be empty".to_string(),
            ));
        }

        if self.name.is_empty() {
            return Err(crate::error::ForgeError::InvalidAgent(
                "Agent name cannot be empty".to_string(),
            ));
        }

        if self.instruction.is_empty() {
            return Err(crate::error::ForgeError::InvalidAgent(
                "Agent instruction cannot be empty".to_string(),
            ));
        }

        // Validate that IDs follow naming convention (alphanumeric + hyphens)
        if !self.id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(crate::error::ForgeError::InvalidAgent(
                format!("Agent ID '{}' contains invalid characters", self.id),
            ));
        }

        Ok(())
    }

    fn tags(&self) -> Vec<&str> {
        self.tags.iter().map(|s| s.as_str()).collect()
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_config_creation() {
        let config = AgentConfig {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            description: "A test agent".to_string(),
            instruction: "Do something".to_string(),
            input_schema: serde_json::json!({"type": "object"}),
            output_schema: serde_json::json!({"type": "object"}),
            metadata: HashMap::new(),
            tags: vec!["test".to_string()],
            enabled: true,
            version: "1.0.0".to_string(),
            extra: serde_json::json!({}),
        };

        assert_eq!(config.id(), "test-agent");
        assert_eq!(config.name(), "Test Agent");
        assert!(config.is_enabled());
    }

    #[test]
    fn test_agent_validation() {
        let valid_config = AgentConfig {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            description: "A test agent".to_string(),
            instruction: "Do something".to_string(),
            input_schema: serde_json::json!({}),
            output_schema: serde_json::json!({}),
            metadata: HashMap::new(),
            tags: vec![],
            enabled: true,
            version: "1.0.0".to_string(),
            extra: serde_json::json!({}),
        };

        assert!(valid_config.validate().is_ok());
    }

    #[test]
    fn test_agent_validation_empty_id() {
        let invalid_config = AgentConfig {
            id: String::new(),
            name: "Test Agent".to_string(),
            description: "A test agent".to_string(),
            instruction: "Do something".to_string(),
            input_schema: serde_json::json!({}),
            output_schema: serde_json::json!({}),
            metadata: HashMap::new(),
            tags: vec![],
            enabled: true,
            version: "1.0.0".to_string(),
            extra: serde_json::json!({}),
        };

        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_agent_validation_invalid_id_chars() {
        let invalid_config = AgentConfig {
            id: "test@agent".to_string(),
            name: "Test Agent".to_string(),
            description: "A test agent".to_string(),
            instruction: "Do something".to_string(),
            input_schema: serde_json::json!({}),
            output_schema: serde_json::json!({}),
            metadata: HashMap::new(),
            tags: vec![],
            enabled: true,
            version: "1.0.0".to_string(),
            extra: serde_json::json!({}),
        };

        assert!(invalid_config.validate().is_err());
    }
}
