//! Logging configuration types.

use serde::{Deserialize, Serialize};

/// Log level configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    /// Parse from environment variable value.
    pub fn from_env(var: &str) -> Option<Self> {
        match var.to_lowercase().as_str() {
            "trace" => Some(Self::Trace),
            "debug" => Some(Self::Debug),
            "info" => Some(Self::Info),
            "warn" | "warning" => Some(Self::Warn),
            "error" => Some(Self::Error),
            _ => None,
        }
    }

    /// Get the corresponding tracing Level.
    pub fn to_tracing_level(self) -> tracing::Level {
        match self {
            Self::Trace => tracing::Level::TRACE,
            Self::Debug => tracing::Level::DEBUG,
            Self::Info => tracing::Level::INFO,
            Self::Warn => tracing::Level::WARN,
            Self::Error => tracing::Level::ERROR,
        }
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

/// Output format for logs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Human-readable format for development.
    Pretty,
    /// Compact format for production.
    Compact,
    /// JSON format for log aggregation.
    Json,
}

impl Default for OutputFormat {
    fn default() -> Self {
        #[cfg(debug_assertions)]
        return Self::Pretty;
        #[cfg(not(debug_assertions))]
        return Self::Compact;
    }
}

/// Logging configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// Minimum log level.
    pub level: LogLevel,
    /// Output format.
    pub format: OutputFormat,
    /// Whether to include timestamps.
    pub include_timestamp: bool,
    /// Whether to include target/module names.
    pub include_target: bool,
    /// Whether to include thread names.
    pub include_thread_id: bool,
    /// Custom env filter (e.g., "info,my_crate=debug").
    pub env_filter: Option<String>,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::default(),
            format: OutputFormat::default(),
            include_timestamp: true,
            include_target: true,
            include_thread_id: false,
            env_filter: None,
        }
    }
}

impl LogConfig {
    /// Load from environment variables.
    pub fn from_env() -> Self {
        let level = std::env::var("RUST_LOG")
            .ok()
            .and_then(|v| LogLevel::from_env(&v))
            .unwrap_or_default();

        let format = std::env::var("LOG_FORMAT")
            .ok()
            .and_then(|v| match v.to_lowercase().as_str() {
                "pretty" | "human" => Some(OutputFormat::Pretty),
                "compact" | "short" => Some(OutputFormat::Compact),
                "json" => Some(OutputFormat::Json),
                _ => None,
            })
            .unwrap_or_default();

        Self {
            level,
            format,
            ..Default::default()
        }
    }
}
