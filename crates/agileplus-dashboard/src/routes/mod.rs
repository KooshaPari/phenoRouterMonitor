//! Axum route handlers for the dashboard.  (T077)
//!
//! Pattern: if the request carries `HX-Request: true`, return only the
//! relevant partial; otherwise return the full page layout.

use std::fs;
use std::path::PathBuf;

use axum::{
    Router,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use crate::app_state::SharedState;

// Re-export submodules and key types
pub mod api;
pub mod dashboard;
pub mod evidence;
pub mod helpers;
pub mod pages;
pub mod services;
#[cfg(test)]
pub mod tests;

// Re-export public types used by clients
pub use api::{AgentInfo, EvidenceArtifactJson, EvidenceGalleryJson, HealthStatus, ServiceHealthJson};

// ── Configuration Types ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaneConfig {
    pub api_url: String,
    pub api_key: String,
    pub workspace_slug: String,
    pub project_slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub pool_size: usize,
    pub retry_budget: usize,
    pub dispatch_mode: String,
    pub default_provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub name: String,
    pub endpoint_url: String,
    #[serde(default = "default_service_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub max_retries: Option<u32>,
}

fn default_service_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    pub theme: String,
    pub log_level: String,
    pub data_directory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub plane: Option<PlaneConfig>,
    pub agents: Option<AgentConfig>,
    pub services: Option<Vec<ServiceConfig>>,
    pub dashboard: Option<DashboardConfig>,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_path();
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let config = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Config {
                plane: None,
                agents: None,
                services: None,
                dashboard: None,
            })
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::config_path();
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }

    fn config_path() -> PathBuf {
        std::env::var("HOME")
            .ok()
            .map(|home| PathBuf::from(home).join(".agileplus/config.toml"))
            .unwrap_or_else(|| PathBuf::from(".agileplus/config.toml"))
    }
}

// ── Form Request Types ────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct PlaneSettingsForm {
    pub api_url: String,
    pub api_key: String,
    pub workspace_slug: String,
    pub project_slug: String,
}

#[derive(Debug, Deserialize)]
pub struct AgentSettingsForm {
    pub pool_size: usize,
    pub retry_budget: usize,
    pub dispatch_mode: String,
    pub default_provider: String,
}

#[derive(Debug, Deserialize)]
pub struct ServiceSettingsForm {
    pub names: Vec<String>,
    pub endpoint_urls: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct DashboardSettingsForm {
    pub theme: String,
    pub log_level: String,
    pub data_directory: String,
}

// ── Main Router Builder ────────────────────────────────────────────────────

pub fn router(state: SharedState) -> Router {
    Router::new()
        .route("/", get(pages::root))
        .route("/home", get(pages::home))
        .route("/dashboard", get(pages::dashboard_page))
        .route("/features", get(pages::features_page))
        .route("/events", get(pages::events_page))
        .route("/settings", get(pages::settings_page))
        .route("/settings/plane", get(pages::plane_settings_page))
        .route("/settings/agents", get(pages::agent_settings_page))
        .route("/settings/services", get(pages::services_settings_page))
        .route("/api/settings/services", post(services::save_services_settings))
        .route("/api/settings/services/test", post(services::test_service_connection))
        .route("/hub", get(pages::hub_page))
        .route("/api/settings/plane", post(pages::save_plane_settings))
        .route("/api/settings/plane/test", post(pages::test_plane_connection))
        .route("/api/settings/agents", post(pages::save_agent_settings))
        .route(
            "/api/settings/agents/test-connection",
            post(pages::test_agent_connection),
        )
        .route("/api/settings/dashboard", post(pages::save_dashboard_settings))
        .route(
            "/api/dashboard/services/{name}/restart",
            post(services::restart_service),
        )
        .route(
            "/api/dashboard/services/{name}/config",
            axum::routing::patch(services::patch_service_config),
        )
        .route(
            "/api/dashboard/services/{name}/toggle",
            post(services::toggle_service),
        )
        .route("/api/dashboard/kanban", get(dashboard::kanban_board))
        .route("/api/dashboard/features/{id}", get(dashboard::feature_detail))
        .route("/api/dashboard/features/{id}/work-packages", get(dashboard::wp_list))
        .route("/api/dashboard/features/{id}/events", get(dashboard::feature_events))
        .route("/api/dashboard/features/{id}/media", get(dashboard::feature_media))
        // HTML partial endpoints (HTMX-compatible)
        .route("/api/dashboard/health", get(dashboard::health_panel))
        .route("/api/dashboard/events", get(dashboard::event_timeline))
        .route("/api/dashboard/agents", get(dashboard::agent_activity))
        // JSON API endpoints (for polling from JavaScript templates)
        .route("/api/dashboard/agents.json", get(api::agents_json))
        .route("/api/dashboard/health.json", get(api::health_json))
        .route("/api/dashboard/projects", get(dashboard::project_switcher))
        .route(
            "/api/dashboard/projects/{id}/activate",
            post(dashboard::switch_project),
        )
        .route("/api/time", get(dashboard::time_footer))
        .route("/api/stream-placeholder", get(dashboard::stream_placeholder))
        .route(
            "/api/evidence/{feature_id}/{artifact_id}/content",
            get(evidence::evidence_content),
        )
        .route(
            "/api/evidence/{feature_id}/{artifact_id}/preview",
            get(evidence::evidence_preview),
        )
        .route(
            "/api/features/{id}/evidence",
            get(evidence::feature_evidence_list),
        )
        .route(
            "/api/features/{id}/evidence/generate",
            post(evidence::feature_evidence_generate),
        )
        .route(
            "/api/dashboard/features/{id}/evidence.json",
            get(evidence::feature_evidence_json),
        )
        .with_state(state)
}
