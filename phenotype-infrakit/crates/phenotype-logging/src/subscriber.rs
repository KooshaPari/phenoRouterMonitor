//! Tracing subscriber initialization.

use crate::config::{LogConfig, LogLevel, OutputFormat};

/// Initialize the tracing subscriber with the given configuration.
pub fn init(config: LogConfig) {
    let env_filter = config
        .env_filter
        .as_deref()
        .map(tracing_subscriber::EnvFilter::try_from_default_env)
        .transpose()
        .unwrap_or_else(|_| {
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        });

    let layer = match config.format {
        OutputFormat::Pretty => {
            tracing_subscriber::fmt::layer()
                .with_target(config.include_target)
                .with_thread_ids(config.include_thread_id)
                .with_file(true)
                .with_line_number(true)
        }
        OutputFormat::Compact => {
            tracing_subscriber::fmt::layer()
                .with_target(config.include_target)
                .with_thread_ids(config.include_thread_id)
                .compact()
        }
        OutputFormat::Json => {
            tracing_subscriber::fmt::layer()
                .json()
                .with_target(config.include_target)
                .with_thread_ids(config.include_thread_id)
        }
    };

    tracing_subscriber::registry()
        .with(env_filter)
        .with(layer)
        .init();
}

/// Initialize with default configuration.
pub fn init_default() {
    init(LogConfig::default());
}

/// Initialize from environment variables.
pub fn init_from_env() {
    init(LogConfig::from_env());
}
