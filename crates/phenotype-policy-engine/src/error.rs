// Error types for policy engine operations.
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    PolicyNotFound,
    ConfigParseError,
    RegexCompilationError,
    SerializationError,
    RuleValidationError,
    IoError,
    Unknown,
}

#[derive(Error, Debug)]
pub enum PolicyEngineError {
    #[error("Policy not found: {name}")]
    PolicyNotFound { name: String },
    #[error("Failed to parse TOML")]
    ConfigParseError { #[source] source: toml::de::Error },
    #[error("Failed to compile regex pattern '{pattern}'")]
    RegexCompilationError { pattern: String, #[source] source: regex::Error },
    #[error("Serialization error")]
    SerializationError { #[source] source: serde_json::Error },
    #[error("Invalid rule configuration: {message}")]
    RuleValidationError { message: String },
    #[error("IO error")]
    IoError { #[source] source: std::io::Error },
}

impl PolicyEngineError {
    pub fn kind(&self) -> ErrorKind {
        match self {
            Self::PolicyNotFound { .. } => ErrorKind::PolicyNotFound,
            Self::ConfigParseError { .. } => ErrorKind::ConfigParseError,
            Self::RegexCompilationError { .. } => ErrorKind::RegexCompilationError,
            Self::SerializationError { .. } => ErrorKind::SerializationError,
            Self::RuleValidationError { .. } => ErrorKind::RuleValidationError,
            Self::IoError { .. } => ErrorKind::IoError,
        }
    }
}

impl From<regex::Error> for PolicyEngineError {
    fn from(source: regex::Error) -> Self {
        Self::RegexCompilationError { pattern: String::new(), source }
    }
}

impl From<toml::de::Error> for PolicyEngineError {
    fn from(source: toml::de::Error) -> Self {
        Self::ConfigParseError { source }
    }
}

impl From<std::io::Error> for PolicyEngineError {
    fn from(source: std::io::Error) -> Self {
        Self::IoError { source }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_not_found_error() {
        let err = PolicyEngineError::PolicyNotFound { name: "test".to_string() };
        assert_eq!(err.kind(), ErrorKind::PolicyNotFound);
    }

    #[test]
    fn serialization_error() {
        let err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let policy_err = PolicyEngineError::SerializationError { source: err };
        assert_eq!(policy_err.kind(), ErrorKind::SerializationError);
    }
}
