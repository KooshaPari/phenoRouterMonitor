//! Serialization error type for (de)serialization failures.

use thiserror::Error;

/// Error type for serialization and deserialization operations.
#[derive(Debug, Clone, Error)]
pub enum SerializationError {
    #[error("json error: {0}")]
    Json(String),

    #[error("toml error: {0}")]
    Toml(String),

    #[error("serialization error: {0}")]
    Internal(String),

    #[error("invalid encoding: {0}")]
    InvalidEncoding(String),
}

impl SerializationError {
    pub fn json(msg: impl Into<String>) -> Self { Self::Json(msg.into()) }
    pub fn toml(msg: impl Into<String>) -> Self { Self::Toml(msg.into()) }
    pub fn internal(msg: impl Into<String>) -> Self { Self::Internal(msg.into()) }
    pub fn invalid_encoding(msg: impl Into<String>) -> Self { Self::InvalidEncoding(msg.into()) }
}

impl From<serde_json::Error> for SerializationError {
    fn from(e: serde_json::Error) -> Self { Self::Json(e.to_string()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_error() { let err = SerializationError::json("invalid utf-8"); assert_eq!(err.to_string(), "json error: invalid utf-8"); }

    #[test]
    fn test_toml_error() { let err = SerializationError::toml("missing value"); assert_eq!(err.to_string(), "toml error: missing value"); }

    #[test]
    fn test_internal_error() { let err = SerializationError::internal("custom serializer failed"); assert_eq!(err.to_string(), "serialization error: custom serializer failed"); }

    #[test]
    fn test_invalid_encoding_error() { let err = SerializationError::invalid_encoding("not utf-8"); assert_eq!(err.to_string(), "invalid encoding: not utf-8"); }

    #[test]
    fn test_from_serde_json_error() {
        let serde_err = serde_json::from_str::<i32>("not json").unwrap_err();
        let ser_err: SerializationError = serde_err.into();
        matches!(ser_err, SerializationError::Json(_));
    }
}
