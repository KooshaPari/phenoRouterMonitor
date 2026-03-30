use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum BifrostError {
    #[error("router not found: {0}")]
    RouterNotFound(String),
    #[error("no providers available")]
    NoProvidersAvailable,
    #[error("routing failed: {0}")]
    RoutingFailed(String),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
}

pub type Result<T> = std::result::Result<T, BifrostError>;
