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
#[derive(Default)]
pub enum LogFormat {
    /// Human-readable, multi-line output (default for development).
    #[default]
    Pretty,
    /// Single-line human-readable output.
    Compact,
    /// Machine-readable JSON (default for production / `LOG_FORMAT=json`).
    Json,
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
    let filter = EnvFilter::try_new(&config.level).unwrap_or_else(|_| EnvFilter::new("info"));

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
#[cfg(test)]
mod tests {
    use super::*;

    // FR-PHENO-015: Default config is INFO level with Pretty format
    #[test]
    fn test_default_config_is_info_pretty() {
        let cfg = LogConfig::default();
        assert_eq!(cfg.level, "info");
        assert_eq!(cfg.format, LogFormat::Pretty);
        assert!(cfg.show_target);
        assert!(!cfg.show_thread);
        assert!(!cfg.show_file);
    }

    // FR-PHENO-016: Builder pattern allows step-by-step configuration
    #[test]
    fn test_builder_overrides() {
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

    // FR-PHENO-017: Environment variables override defaults
    #[test]
    fn test_from_env_reads_vars() {
        std::env::set_var("RUST_LOG", "trace");
        std::env::set_var("LOG_FORMAT", "json");

        let cfg = LogConfig::from_env();
        assert_eq!(cfg.level, "trace");
        assert_eq!(cfg.format, LogFormat::Json);

        std::env::remove_var("RUST_LOG");
        std::env::remove_var("LOG_FORMAT");
    }

    // FR-PHENO-018: Builder preserves show_target flag
    #[test]
    fn test_builder_show_target() {
        let cfg = LogConfig::builder()
            .show_target(false)
            .build();

        assert!(!cfg.show_target);
    }

    // FR-PHENO-019: Builder preserves show_thread flag
    #[test]
    fn test_builder_show_thread() {
        let cfg = LogConfig::builder()
            .show_thread(true)
            .build();

        assert!(cfg.show_thread);
    }

    // FR-PHENO-020: Builder preserves show_file flag
    #[test]
    fn test_builder_show_file() {
        let cfg = LogConfig::builder()
            .show_file(true)
            .build();

        assert!(cfg.show_file);
    }

    // FR-PHENO-021: Multiple log levels can be configured
    #[test]
    fn test_custom_log_levels() {
        let levels = vec!["trace", "debug", "info", "warn", "error"];
        for level in levels {
            let cfg = LogConfig::builder().level(level).build();
            assert_eq!(cfg.level, level);
        }
    }

    // FR-PHENO-022: All log formats are supported
    #[test]
    fn test_all_log_formats() {
        let formats = vec![LogFormat::Pretty, LogFormat::Compact, LogFormat::Json];
        for fmt in formats {
            let cfg = LogConfig::builder().format(fmt).build();
            assert_eq!(cfg.format, fmt);
        }
    }

    // FR-PHENO-023: LOG_FORMAT env var accepts lowercase variants
    #[test]
    fn test_log_format_case_insensitive() {
        std::env::set_var("LOG_FORMAT", "JSON");
        let cfg = LogConfig::from_env();
        assert_eq!(cfg.format, LogFormat::Json);

        std::env::set_var("LOG_FORMAT", "Compact");
        let cfg = LogConfig::from_env();
        assert_eq!(cfg.format, LogFormat::Compact);

        std::env::set_var("LOG_FORMAT", "PRETTY");
        let cfg = LogConfig::from_env();
        assert_eq!(cfg.format, LogFormat::Pretty);

        std::env::remove_var("LOG_FORMAT");
    }

    // FR-PHENO-024: Invalid LOG_FORMAT falls back to default
    #[test]
    fn test_invalid_log_format_uses_default() {
        std::env::set_var("LOG_FORMAT", "invalid_format");
        let cfg = LogConfig::from_env();
        assert_eq!(cfg.format, LogFormat::default());
        std::env::remove_var("LOG_FORMAT");
    }

    // FR-PHENO-025: Builder chaining is fluent and idiomatic
    #[test]
    fn test_builder_fluent_chain() {
        let cfg = LogConfig::builder()
            .level("warn")
            .format(LogFormat::Compact)
            .show_target(true)
            .show_thread(false)
            .show_file(true)
            .build();

        assert_eq!(cfg.level, "warn");
        assert_eq!(cfg.format, LogFormat::Compact);
        assert!(cfg.show_target);
        assert!(!cfg.show_thread);
        assert!(cfg.show_file);
    }

    // FR-PHENO-026: LogFormat serialization round-trips correctly
    #[test]
    fn test_log_format_serde_roundtrip() {
        let formats = vec![LogFormat::Pretty, LogFormat::Compact, LogFormat::Json];
        for fmt in formats {
            let json = serde_json::to_string(&fmt).unwrap();
            let deserialized: LogFormat = serde_json::from_str(&json).unwrap();
            assert_eq!(fmt, deserialized);
        }
    }

    // FR-PHENO-027: LogConfig serialization includes all fields
    #[test]
    fn test_log_config_serde_roundtrip() {
        let cfg = LogConfig::builder()
            .level("debug")
            .format(LogFormat::Json)
            .show_thread(true)
            .show_file(true)
            .build();

        let json = serde_json::to_string(&cfg).unwrap();
        let deserialized: LogConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(cfg.level, deserialized.level);
        assert_eq!(cfg.format, deserialized.format);
        assert_eq!(cfg.show_target, deserialized.show_target);
        assert_eq!(cfg.show_thread, deserialized.show_thread);
        assert_eq!(cfg.show_file, deserialized.show_file);
    }

    // FR-PHENO-028: RUST_LOG env var with multiple directives
    #[test]
    fn test_rust_log_multiple_directives() {
        std::env::set_var("RUST_LOG", "info,phenotype=debug,tracing=warn");
        let cfg = LogConfig::from_env();
        assert_eq!(cfg.level, "info,phenotype=debug,tracing=warn");
        std::env::remove_var("RUST_LOG");
    }

    // FR-PHENO-029: Missing env vars use defaults
    #[test]
    fn test_missing_env_vars_use_defaults() {
        std::env::remove_var("RUST_LOG");
        std::env::remove_var("LOG_FORMAT");

        let cfg = LogConfig::from_env();
        assert_eq!(cfg.level, "info");
        assert_eq!(cfg.format, LogFormat::Pretty);
    }

    // FR-PHENO-030: Config fields are independent
    #[test]
    fn test_config_fields_independent() {
        let cfg1 = LogConfig::builder().level("debug").build();
        let cfg2 = LogConfig::builder().format(LogFormat::Json).build();

        assert_eq!(cfg1.level, "debug");
        assert_eq!(cfg1.format, LogFormat::Pretty);
        assert_eq!(cfg2.level, "info");
        assert_eq!(cfg2.format, LogFormat::Json);
    }

    // FR-PHENO-031: Default LogFormat is Pretty
    #[test]
    fn test_default_log_format() {
        assert_eq!(LogFormat::default(), LogFormat::Pretty);
    }

    // FR-PHENO-032: All display options can be toggled independently
    #[test]
    fn test_display_options_independent() {
        let cfg = LogConfig::builder()
            .show_target(false)
            .show_thread(false)
            .show_file(false)
            .build();

        assert!(!cfg.show_target);
        assert!(!cfg.show_thread);
        assert!(!cfg.show_file);

        let cfg = LogConfig::builder()
            .show_target(true)
            .show_thread(true)
            .show_file(true)
            .build();

        assert!(cfg.show_target);
        assert!(cfg.show_thread);
        assert!(cfg.show_file);
    }
}
