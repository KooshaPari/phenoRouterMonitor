// Error types for bifrost-routing

use thiserror::Error;
use std::fmt;

#[derive(Error, Debug, Clone)]
pub enum BifrostError {
    #[error("provider error: {0}")]
    ProviderError(String),

    #[error("request invalid: {0}")]
    InvalidRequest(String),

    #[error("timeout: {provider} did not respond within {timeout_ms}ms")]
    Timeout { provider: String, timeout_ms: u64 },

    #[error("authentication failed for {provider}: {reason}")]
    AuthenticationError { provider: String, reason: String },

    #[error("rate limited by {provider}")]
    RateLimited { provider: String },

    #[error("insufficient credits/quota: {provider}")]
    InsufficientQuota { provider: String },

    #[error("unsupported model: {model} not available on {provider}")]
    UnsupportedModel { model: String, provider: String },

    #[error("all providers failed: {attempts} attempts made")]
    AllProvidersFailed { attempts: usize },

    #[error("routing error: {0}")]
    RoutingError(String),

    #[error("configuration error: {0}")]
    ConfigurationError(String),

    #[error("serialization error: {0}")]
    SerializationError(String),

    #[error("io error: {0}")]
    IoError(String),
}

impl BifrostError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            BifrostError::Timeout { .. }
                | BifrostError::RateLimited { .. }
                | BifrostError::AllProvidersFailed { .. }
        )
    }

    pub fn provider_name(&self) -> Option<String> {
        match self {
            BifrostError::ProviderError(msg) => Some(msg.clone()),
            BifrostError::Timeout { provider, .. } => Some(provider.clone()),
            BifrostError::AuthenticationError { provider, .. } => Some(provider.clone()),
            BifrostError::RateLimited { provider } => Some(provider.clone()),
            BifrostError::InsufficientQuota { provider } => Some(provider.clone()),
            BifrostError::UnsupportedModel { provider, .. } => Some(provider.clone()),
            _ => None,
        }
    }
}

pub type BifrostResult<T> = Result<T, BifrostError>;
