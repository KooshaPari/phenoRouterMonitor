//! Sentry configuration for Phenotype

/// Sentry configuration
#[derive(Debug, Clone)]
pub struct SentryConfig {
    /// DSN for Sentry
    pub dsn: String,
    /// Environment
    pub environment: String,
}

impl SentryConfig {
    /// Create new Sentry config
    pub fn new(dsn: impl Into<String>, environment: impl Into<String>) -> Self {
        Self {
            dsn: dsn.into(),
            environment: environment.into(),
        }
    }
}
