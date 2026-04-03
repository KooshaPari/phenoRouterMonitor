//! phenotype-logging
//!
//! Structured logging infrastructure for the Phenotype ecosystem.
//!
//! # Modules
//!
//! - `config`: Log configuration types (LogLevel, OutputFormat, LogConfig)
//! - `subscriber`: Tracing subscriber initialization
//! - `otel`: OpenTelemetry integration

use thiserror::Error;

pub mod config;
pub mod subscriber;
pub mod otel;

#[derive(Debug, Error)]
pub enum LoggingError {
    #[error("{0}")]
    Invalid(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("OpenTelemetry error: {0}")]
    OTel(String),
}

pub type Result<T> = std::result::Result<T, LoggingError>;

// Re-export main types
pub use config::{LogConfig, LogLevel, OutputFormat};
pub use subscriber::{init, init_default};
pub use otel::{init_with_otel, OTelError};
