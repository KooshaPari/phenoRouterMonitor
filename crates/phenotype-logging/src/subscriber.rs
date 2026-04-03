//! Tracing subscriber initialization.

use crate::config::{LogConfig, OutputFormat};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Initialize the tracing subscriber with the given configuration.
pub fn init(config: LogConfig) {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(config.level.as_str()));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(config.include_target)
        .with_thread_ids(config.include_thread_id);

    match config.format {
        OutputFormat::Pretty => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt_layer.pretty())
                .init();
        }
        OutputFormat::Compact => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt_layer.compact())
                .init();
        }
        OutputFormat::Json => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(fmt_layer.json())
                .init();
        }
    }
}

/// Initialize with default configuration.
pub fn init_default() {
    init(LogConfig::default());
}

/// Initialize from environment variables.
pub fn init_from_env() {
    init(LogConfig::from_env());
}
