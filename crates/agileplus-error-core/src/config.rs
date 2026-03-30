//! Configuration error type for loading and merging configurations.

use thiserror::Error;

/// Error type for configuration loading and merging operations.
#[derive(Debug, Clone, Error)]
pub enum ConfigError {
    #[error("config file not found: {0}")]
    FileNotFound(String),

    #[error("config parsing error: {0}")]
    ParseError(String),

    #[error("invalid config: {0}")]
    Invalid(String),

    #[error("missing config key: {0}")]
    MissingKey(String),

    #[error("config merge conflict: {0}")]
    MergeConflict(String),

    #[error("config validation error: {0}")]
    ValidationError(String),

    #[error("environment variable not set: {0}")]
    MissingEnvVar(String),

    #[error("config serialization error: {0}")]
    SerializationError(String),
}

impl ConfigError {
    pub fn file_not_found(path: impl Into<String>) -> Self { Self::FileNotFound(path.into()) }
    pub fn parse_error(msg: impl Into<String>) -> Self { Self::ParseError(msg.into()) }
    pub fn invalid(msg: impl Into<String>) -> Self { Self::Invalid(msg.into()) }
    pub fn missing_key(key: impl Into<String>) -> Self { Self::MissingKey(key.into()) }
    pub fn merge_conflict(msg: impl Into<String>) -> Self { Self::MergeConflict(msg.into()) }
    pub fn validation_error(msg: impl Into<String>) -> Self { Self::ValidationError(msg.into()) }
    pub fn missing_env_var(var: impl Into<String>) -> Self { Self::MissingEnvVar(var.into()) }
    pub fn serialization_error(msg: impl Into<String>) -> Self { Self::SerializationError(msg.into()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_not_found_error() {
        let err = ConfigError::file_not_found("config.toml");
        assert_eq!(err.to_string(), "config file not found: config.toml");
    }

    #[test]
    fn test_parse_error() {
        let err = ConfigError::parse_error("expected key=value");
        assert_eq!(err.to_string(), "config parsing error: expected key=value");
    }

    #[test]
    fn test_invalid_config_error() {
        let err = ConfigError::invalid("port must be between 1 and 65535");
        assert_eq!(err.to_string(), "invalid config: port must be between 1 and 65535");
    }

    #[test]
    fn test_missing_key_error() {
        let err = ConfigError::missing_key("database.url");
        assert_eq!(err.to_string(), "missing config key: database.url");
    }

    #[test]
    fn test_merge_conflict_error() {
        let err = ConfigError::merge_conflict("conflicting values for workers");
        assert_eq!(err.to_string(), "config merge conflict: conflicting values for workers");
    }

    #[test]
    fn test_validation_error() {
        let err = ConfigError::validation_error("invalid log level");
        assert_eq!(err.to_string(), "config validation error: invalid log level");
    }

    #[test]
    fn test_missing_env_var_error() {
        let err = ConfigError::missing_env_var("DATABASE_URL");
        assert_eq!(err.to_string(), "environment variable not set: DATABASE_URL");
    }

    #[test]
    fn test_serialization_error() {
        let err = ConfigError::serialization_error("failed to serialize to json");
        assert_eq!(err.to_string(), "config serialization error: failed to serialize to json");
    }
}
