//! phenotype-logging
//!
//! Structured logging infrastructure for the Phenotype ecosystem.
//!
//! # Modules
//!
//! - `config`: Log configuration types (LogLevel, OutputFormat, LogConfig)
//! - `subscriber`: Tracing subscriber initialization
//! - `otel`: OpenTelemetry integration (requires `otel` feature)

use thiserror::Error;

pub mod config;
pub mod subscriber;

#[cfg(feature = "otel")]
pub mod otel;

#[derive(Debug, Error)]
pub enum LoggingError {
    #[error("{0}")]
    Invalid(String),
    #[error("Configuration error: {0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, LoggingError>;

// Re-export main types
pub use config::{LogConfig, LogLevel, OutputFormat};
pub use subscriber::{init, init_default};

#[cfg(feature = "otel")]
pub use otel::{init_with_otel, OTelError};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loglevel_as_str() {
        assert_eq!(LogLevel::Trace.as_str(), "trace");
        assert_eq!(LogLevel::Debug.as_str(), "debug");
        assert_eq!(LogLevel::Info.as_str(), "info");
        assert_eq!(LogLevel::Warn.as_str(), "warn");
        assert_eq!(LogLevel::Error.as_str(), "error");
    }

    #[test]
    fn test_loglevel_display() {
        assert_eq!(format!("{}", LogLevel::Info), "info");
        assert_eq!(format!("{}", LogLevel::Error), "error");
    }

    #[test]
    fn test_loglevel_default() {
        let level: LogLevel = Default::default();
        assert!(matches!(level, LogLevel::Info));
    }

    #[test]
    fn test_output_format_default() {
        let format: OutputFormat = Default::default();
        // Default depends on debug_assertions
        // In tests (debug), it should be Pretty
        #[cfg(debug_assertions)]
        assert!(matches!(format, OutputFormat::Pretty));
        #[cfg(not(debug_assertions))]
        assert!(matches!(format, OutputFormat::Compact));
    }

    #[test]
    fn test_log_config_default() {
        let config = LogConfig::default();
        assert!(matches!(config.level, LogLevel::Info));
        assert!(config.include_timestamp);
        assert!(config.include_target);
        assert!(!config.include_thread_id);
    }

    #[test]
    fn test_loglevel_from_env() {
        assert!(matches!(LogLevel::from_env("info"), Some(LogLevel::Info)));
        assert!(matches!(LogLevel::from_env("INFO"), Some(LogLevel::Info)));
        assert!(matches!(LogLevel::from_env("debug"), Some(LogLevel::Debug)));
        assert!(matches!(LogLevel::from_env("warn"), Some(LogLevel::Warn)));
        assert!(matches!(
            LogLevel::from_env("warning"),
            Some(LogLevel::Warn)
        ));
        assert!(matches!(LogLevel::from_env("trace"), Some(LogLevel::Trace)));
        assert!(matches!(LogLevel::from_env("error"), Some(LogLevel::Error)));
        assert!(matches!(LogLevel::from_env("unknown"), None));
    }
}
