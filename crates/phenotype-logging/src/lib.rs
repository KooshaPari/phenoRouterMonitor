//! Structured logging for the Phenotype ecosystem.
//!
//! Wraps: tracing 0.1 + tracing-subscriber 0.3
//!
//! # Quick Start
//!
//! ```no_run
//! phenotype_logging::init_logging();
//! phenotype_logging::info!("service started");
//! ```

use tracing_subscriber::{fmt, EnvFilter};

// Re-export tracing macros for convenience.
pub use tracing::{debug, error, info, instrument, trace, warn};
// Re-export core tracing types consumers commonly need.
pub use tracing::{span, Level, Span};

/// Log output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// Human-readable, multi-line output (default for development).
    Pretty,
    /// Single-line human-readable output.
    Compact,
    /// Machine-readable JSON (default for production / `LOG_FORMAT=json`).
    Json,
}

impl Default for LogFormat {
    fn default() -> Self {
        Self::Pretty
    }
}

/// Configuration for the logging subsystem.
///
/// Use [`LogConfig::builder()`] for ergonomic construction or
/// [`LogConfig::from_env()`] to derive settings from environment variables.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct LogConfig {
    /// Directive string compatible with [`EnvFilter`] (e.g. `"info,my_crate=debug"`).
    pub level: String,
    /// Output format.
    pub format: LogFormat,
    /// Show the target (module path) in log lines.
    pub show_target: bool,
    /// Show the thread name / id.
    pub show_thread: bool,
    /// Show file name and line number.
    pub show_file: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".into(),
            format: LogFormat::default(),
            show_target: true,
            show_thread: false,
            show_file: false,
        }
    }
}

impl LogConfig {
    /// Return a [`LogConfigBuilder`] for step-by-step construction.
    pub fn builder() -> LogConfigBuilder {
        LogConfigBuilder(Self::default())
    }

    /// Build a config from environment variables.
    ///
    /// | Variable    | Maps to         | Example          |
    /// |-------------|-----------------|------------------|
    /// | `RUST_LOG`  | `level`         | `debug,hyper=warn` |
    /// | `LOG_FORMAT`| `format`        | `json`, `compact`, `pretty` |
    pub fn from_env() -> Self {
        let mut cfg = Self::default();

        if let Ok(v) = std::env::var("RUST_LOG") {
            cfg.level = v;
        }

        if let Ok(v) = std::env::var("LOG_FORMAT") {
            match v.to_lowercase().as_str() {
                "json" => cfg.format = LogFormat::Json,
                "compact" => cfg.format = LogFormat::Compact,
                "pretty" => cfg.format = LogFormat::Pretty,
                _ => {} // keep default
            }
        }

        cfg
    }
}

/// Builder for [`LogConfig`].
pub struct LogConfigBuilder(LogConfig);

impl LogConfigBuilder {
    pub fn level(mut self, level: impl Into<String>) -> Self {
        self.0.level = level.into();
        self
    }

    pub fn format(mut self, format: LogFormat) -> Self {
        self.0.format = format;
        self
    }

    pub fn show_target(mut self, v: bool) -> Self {
        self.0.show_target = v;
        self
    }

    pub fn show_thread(mut self, v: bool) -> Self {
        self.0.show_thread = v;
        self
    }

    pub fn show_file(mut self, v: bool) -> Self {
        self.0.show_file = v;
        self
    }

    pub fn build(self) -> LogConfig {
        self.0
    }
}

/// Initialise logging with sensible defaults (INFO level, pretty format).
///
/// Respects `RUST_LOG` and `LOG_FORMAT` environment variables when set.
///
/// # Panics
///
/// Panics if a global subscriber has already been set.
pub fn init_logging() {
    init_logging_with_config(LogConfig::from_env());
}

/// Initialise logging with an explicit [`LogConfig`].
///
/// # Panics
///
/// Panics if a global subscriber has already been set.
pub fn init_logging_with_config(config: LogConfig) {
    let filter = EnvFilter::try_new(&config.level)
        .unwrap_or_else(|_| EnvFilter::new("info"));

    match config.format {
        LogFormat::Pretty => {
            let sub = fmt::Subscriber::builder()
                .with_env_filter(filter)
                .with_target(config.show_target)
                .with_thread_names(config.show_thread)
                .with_file(config.show_file)
                .with_line_number(config.show_file)
                .pretty()
                .finish();
            tracing::subscriber::set_global_default(sub)
                .expect("failed to set global tracing subscriber");
        }
        LogFormat::Compact => {
            let sub = fmt::Subscriber::builder()
                .with_env_filter(filter)
                .with_target(config.show_target)
                .with_thread_names(config.show_thread)
                .with_file(config.show_file)
                .with_line_number(config.show_file)
                .compact()
                .finish();
            tracing::subscriber::set_global_default(sub)
                .expect("failed to set global tracing subscriber");
        }
        LogFormat::Json => {
            let sub = fmt::Subscriber::builder()
                .with_env_filter(filter)
                .with_target(config.show_target)
                .with_thread_names(config.show_thread)
                .with_file(config.show_file)
                .with_line_number(config.show_file)
                .json()
                .finish();
            tracing::subscriber::set_global_default(sub)
                .expect("failed to set global tracing subscriber");
        }
    }
}

/// Create a [`Span`] at INFO level with a given name.
///
/// Returns the span so the caller can `.enter()` or `.in_scope(|| ...)` it.
///
/// ```no_run
/// let span = phenotype_logging::context_span("handle_request");
/// let _guard = span.enter();
/// phenotype_logging::info!("processing");
/// ```
pub fn context_span(name: &'static str) -> Span {
    tracing::info_span!("context", otel.name = name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_info_pretty() {
        let cfg = LogConfig::default();
        assert_eq!(cfg.level, "info");
        assert_eq!(cfg.format, LogFormat::Pretty);
        assert!(cfg.show_target);
        assert!(!cfg.show_thread);
        assert!(!cfg.show_file);
    }

    #[test]
    fn builder_overrides() {
        let cfg = LogConfig::builder()
            .level("debug")
            .format(LogFormat::Json)
            .show_thread(true)
            .show_file(true)
            .build();

        assert_eq!(cfg.level, "debug");
        assert_eq!(cfg.format, LogFormat::Json);
        assert!(cfg.show_thread);
        assert!(cfg.show_file);
    }

    #[test]
    fn from_env_reads_vars() {
        std::env::set_var("RUST_LOG", "trace");
        std::env::set_var("LOG_FORMAT", "json");

        let cfg = LogConfig::from_env();
        assert_eq!(cfg.level, "trace");
        assert_eq!(cfg.format, LogFormat::Json);

        std::env::remove_var("RUST_LOG");
        std::env::remove_var("LOG_FORMAT");
    }
}
