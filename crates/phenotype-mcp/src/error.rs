//! Error types for MCP protocol operations

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// MCP protocol error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    /// Invalid Request: The JSON sent is not a valid Request object
    #[serde(rename = "-32600")]
    InvalidRequest = -32600,
    /// Method not found: The method does not exist or is not available
    #[serde(rename = "-32601")]
    MethodNotFound = -32601,
    /// Invalid params: Invalid method parameter(s)
    #[serde(rename = "-32602")]
    InvalidParams = -32602,
    /// Internal error: Internal JSON-RPC error
    #[serde(rename = "-32603")]
    InternalError = -32603,
    /// Server error: Server error (reserved for implementation-defined server errors)
    #[serde(rename = "-32000")]
    ServerError = -32000,
    /// Parse error: Invalid JSON was received by the server
    #[serde(rename = "-32700")]
    ParseError = -32700,
}

/// Result type for MCP operations
pub type Result<T> = std::result::Result<T, Error>;

/// MCP error type
#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    #[error("method not found: {0}")]
    MethodNotFound(String),

    #[error("invalid params: {0}")]
    InvalidParams(String),

    #[error("internal error: {0}")]
    InternalError(String),

    #[error("server error: {0}")]
    ServerError(String),

    #[error("parse error: {0}")]
    ParseError(String),

    #[error("serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("anyhow error: {0}")]
    Other(#[from] anyhow::Error),
}

impl Error {
    /// Get the error code for this error
    pub fn code(&self) -> ErrorCode {
        match self {
            Error::InvalidRequest(_) => ErrorCode::InvalidRequest,
            Error::MethodNotFound(_) => ErrorCode::MethodNotFound,
            Error::InvalidParams(_) => ErrorCode::InvalidParams,
            Error::InternalError(_) => ErrorCode::InternalError,
            Error::ServerError(_) => ErrorCode::ServerError,
            Error::ParseError(_) => ErrorCode::ParseError,
            Error::SerializationError(_) => ErrorCode::ParseError,
            Error::Other(_) => ErrorCode::InternalError,
        }
    }

    /// Get error message
    pub fn message(&self) -> String {
        self.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(ErrorCode::InvalidRequest as i32, -32600);
        assert_eq!(ErrorCode::MethodNotFound as i32, -32601);
        assert_eq!(ErrorCode::InvalidParams as i32, -32602);
        assert_eq!(ErrorCode::InternalError as i32, -32603);
        assert_eq!(ErrorCode::ServerError as i32, -32000);
        assert_eq!(ErrorCode::ParseError as i32, -32700);
    }

    #[test]
    fn test_error_code_from_error() {
        let err = Error::MethodNotFound("test_method".into());
        assert_eq!(err.code(), ErrorCode::MethodNotFound);
    }
}
