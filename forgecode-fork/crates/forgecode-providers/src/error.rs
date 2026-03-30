//! Error types for forgecode-providers

use thiserror::Error;

/// Result type for forgecode-providers operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when working with LLM providers
#[derive(Error, Debug)]
pub enum Error {
    /// Missing required configuration field
    #[error("Missing required configuration: {field}")]
    MissingConfig { field: String },

    /// Invalid configuration value
    #[error("Invalid configuration for {field}: {reason}")]
    InvalidConfig { field: String, reason: String },

    /// API authentication failed
    #[error("Authentication failed: {reason}")]
    AuthenticationFailed { reason: String },

    /// API request failed
    #[error("API request failed: {reason}")]
    RequestFailed { reason: String },

    /// Invalid response from provider
    #[error("Invalid response from provider: {reason}")]
    InvalidResponse { reason: String },

    /// Provider not supported
    #[error("Provider not supported: {provider}")]
    UnsupportedProvider { provider: String },

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {retry_after}")]
    RateLimited { retry_after: Option<u64> },

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Serialization(e.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Network(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_config_error() {
        let err = Error::MissingConfig {
            field: "api_key".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Missing required configuration: api_key"
        );
    }

    #[test]
    fn test_invalid_config_error() {
        let err = Error::InvalidConfig {
            field: "temperature".to_string(),
            reason: "must be between 0 and 2".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Invalid configuration for temperature: must be between 0 and 2"
        );
    }

    #[test]
    fn test_authentication_error() {
        let err = Error::AuthenticationFailed {
            reason: "invalid api key".to_string(),
        };
        assert_eq!(err.to_string(), "Authentication failed: invalid api key");
    }

    #[test]
    fn test_rate_limited_error_with_retry() {
        let err = Error::RateLimited {
            retry_after: Some(60),
        };
        assert_eq!(err.to_string(), "Rate limit exceeded: Some(60)");
    }

    #[test]
    fn test_unsupported_provider_error() {
        let err = Error::UnsupportedProvider {
            provider: "gemini".to_string(),
        };
        assert_eq!(err.to_string(), "Provider not supported: gemini");
    }
}
