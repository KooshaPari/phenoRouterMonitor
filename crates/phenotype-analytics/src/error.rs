//! Error types

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum AnalyticsError {
    #[error("tracking failed: {0}")]
    TrackingFailed(String),
    #[error("backend error: {0}")]
    BackendError(String),
    #[error("config error: {0}")]
    ConfigError(String),
}

pub type Result<T> = std::result::Result<T, AnalyticsError>;
