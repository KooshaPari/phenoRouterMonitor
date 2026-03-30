//! Error types for the forgecode-fork subagent system

use thiserror::Error;

/// Result type for forgecode operations
pub type Result<T> = std::result::Result<T, ForgeError>;

/// Errors that can occur in the forgecode-fork subagent system
#[derive(Error, Debug)]
pub enum ForgeError {
    /// YAML parsing error
    #[error("YAML parsing error: {0}")]
    YamlError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Agent not found
    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    /// Invalid agent definition
    #[error("Invalid agent definition: {0}")]
    InvalidAgent(String),

    /// Discovery error
    #[error("Discovery error: {0}")]
    DiscoveryError(String),

    /// Registry error
    #[error("Registry error: {0}")]
    RegistryError(String),

    /// Schema validation error
    #[error("Schema validation error: {0}")]
    SchemaError(String),

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl From<serde_yaml::Error> for ForgeError {
    fn from(err: serde_yaml::Error) -> Self {
        ForgeError::YamlError(err.to_string())
    }
}

impl From<serde_json::Error> for ForgeError {
    fn from(err: serde_json::Error) -> Self {
        ForgeError::ConfigError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ForgeError::AgentNotFound("test-agent".to_string());
        assert_eq!(err.to_string(), "Agent not found: test-agent");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let forge_err = ForgeError::from(io_err);
        assert!(matches!(forge_err, ForgeError::IoError(_)));
    }
}
