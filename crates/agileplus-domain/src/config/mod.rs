//! Application configuration — TOML schema, parsing, and env-var overrides.
//!
//! Traceability: FR-032 / WP15-T089

mod agents;
mod api;
mod core;
mod credentials;
mod loader;
mod telemetry;

#[cfg(test)]
mod tests;

pub use agents::AgentConfig;
pub use api::ApiConfig;
pub use core::CoreConfig;
pub use credentials::{CredentialBackend, CredentialConfig};
pub use loader::ConfigError;
pub use telemetry::TelemetryConfig;

use serde::{Deserialize, Serialize};

/// Top-level configuration for AgilePlus.
///
/// All sections have defaults so a missing `config.toml` (or missing sections
/// within it) never causes a parse error.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub core: CoreConfig,
    #[serde(default)]
    pub credentials: CredentialConfig,
    #[serde(default)]
    pub telemetry: TelemetryConfig,
    #[serde(default)]
    pub api: ApiConfig,
    #[serde(default)]
    pub agents: AgentConfig,
}
