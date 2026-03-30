//! Parse error type for JSON/TOML/YAML parsing failures.

use thiserror::Error;

/// Error type for serialization format parsing (JSON, TOML, YAML).
#[derive(Debug, Clone, Error)]
pub enum ParseError {
    #[error("json parsing error: {0}")]
    Json(String),

    #[error("toml parsing error: {0}")]
    Toml(String),

    #[error("yaml parsing error: {0}")]
    Yaml(String),

    #[error("invalid format: {0}")]
    InvalidFormat(String),

    #[error("unexpected eof")]
    UnexpectedEof,

    #[error("invalid encoding: {0}")]
    InvalidEncoding(String),
}

impl ParseError {
    /// Create a JSON parse error.
    pub fn json(msg: impl Into<String>) -> Self {
        Self::Json(msg.into())
    }

    /// Create a TOML parse error.
    pub fn toml(msg: impl Into<String>) -> Self {
        Self::Toml(msg.into())
    }

    /// Create a YAML parse error.
    pub fn yaml(msg: impl Into<String>) -> Self {
        Self::Yaml(msg.into())
    }

    /// Create an invalid format error.
    pub fn invalid_format(msg: impl Into<String>) -> Self {
        Self::InvalidFormat(msg.into())
    }

    /// Create an invalid encoding error.
    pub fn invalid_encoding(msg: impl Into<String>) -> Self {
        Self::InvalidEncoding(msg.into())
    }
}

// Conversion from serde_json::Error
impl From<serde_json::Error> for ParseError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_error_creation() {
        let err = ParseError::json("invalid json");
        assert_eq!(err.to_string(), "json parsing error: invalid json");
    }

    #[test]
    fn test_json_error_from_serde() {
        let serde_err = serde_json::from_str::<i32>("not json").unwrap_err();
        let parse_err: ParseError = serde_err.into();
        matches!(parse_err, ParseError::Json(_));
    }

    #[test]
    fn test_toml_error_creation() {
        let err = ParseError::toml("malformed toml");
        assert_eq!(err.to_string(), "toml parsing error: malformed toml");
    }

    #[test]
    fn test_yaml_error_creation() {
        let err = ParseError::yaml("bad yaml");
        assert_eq!(err.to_string(), "yaml parsing error: bad yaml");
    }

    #[test]
    fn test_invalid_format_error() {
        let err = ParseError::invalid_format("unknown format");
        assert_eq!(err.to_string(), "invalid format: unknown format");
    }

    #[test]
    fn test_invalid_encoding_error() {
        let err = ParseError::invalid_encoding("utf-8");
        assert_eq!(err.to_string(), "invalid encoding: utf-8");
    }

    #[test]
    fn test_unexpected_eof_error() {
        let err = ParseError::UnexpectedEof;
        assert_eq!(err.to_string(), "unexpected eof");
    }
}
