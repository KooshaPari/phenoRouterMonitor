//! Error types for phenotype-monitor ecosystem

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MonitorError {
    #[error("Router error: {0}")]
    Router(String),
    
    #[error("Metrics error: {0}")]
    Metrics(String),
    
    #[error("Metering error: {0}")]
    Metering(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, MonitorError>;
