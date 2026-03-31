use std::env;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    response::Response,
    routing::{get, post},
    Router,
};
use crate::app_state::SharedState;
use crate::templates::{AgentSettingsPage, PlaneSettingsPage, ServicesSettingsPage, SettingsPage};
use crate::routes::{
    render, Config, PlaneConfig, AgentConfig, ServiceConfig,
    DashboardConfig, PlaneSettingsForm, AgentSettingsForm,
    ServiceSettingsForm, DashboardSettingsForm, default_service_enabled,
    env_or_none, plane_connection_checks, plane_health_endpoints,
    plane_api_key_hint, plane_sync_mode, percentage_coverage,
    DEFAULT_PLANE_API_URL, DEFAULT_PLANE_WEB_URL,
};

const DEFAULT_PLANE_API_URL: &str = "https://app.plane.so";
const DEFAULT_PLANE_WEB_URL: &str = "https://app.plane.so";

pub async fn settings_page() -> Response {
    render(SettingsPage)
}

pub async fn plane_settings_page(State(state): State<SharedState>) -> Response {
    let store = state.read().await;
    let plane_workspace = env_or_none("PLANE_WORKSPACE");
    let project_slug = env_or_none("PLANE_PROJECT").unwrap_or_else(|| "not configured".to_string());
    let plane_api_key = env_or_none("PLANE_API_KEY");
    let plane_api_url = env_or_none("PLANE_API_URL").unwrap_or_else(|| DEFAULT_PLANE_API_URL.to_string());
    let plane_web_url = env_or_none("PLANE_WEB_URL").unwrap_or_else(|| DEFAULT_PLANE_WEB_URL.to_string());
    let (connected, connection_status, mut config_warnings) = plane_connection_checks(&plane_api_key, &plane_workspace);

    let plane_health_endpoints = plane_health_endpoints(&store.health);
    let plane_health_healthy = plane_health_endpoints
        .iter()
        .all(|endpoint| endpoint.healthy && !endpoint.degraded);
    let plane_api_latency_ms = plane_health_endpoints
        .iter()
        .find(|endpoint| endpoint.name == "Plane API")
        .and_then(|endpoint| endpoint.latency_ms);

    if !connected {
        config_warnings.push("Plane sync disabled until required settings are provided".to_string());
    }
    if !plane_health_healthy {
        config_warnings.push("Plane API health check is not healthy".to_string());
    }

    let mapped_features = store
        .features
        .iter()
        .filter(|feature| feature.plane_issue_id.is_some())
        .count();
    let total_features = store.features.len();
    let mapped_work_packages = store
        .work_packages
        .values()
        .flatten()
        .filter(|wp| wp.plane_sub_issue_id.is_some())
        .count();
    let total_work_packages: usize = store.work_packages.values().map(Vec::len).sum();

    let connection_status_configured = !connection_status.is_empty();

    render(PlaneSettingsPage {
        workspace_name: plane_workspace.clone().unwrap_or_else(|| "Not configured".to_string()),
        workspace_slug: plane_workspace.unwrap_or_else(|| "not configured".to_string()),
        project_slug,
        plane_api_url: plane_api_url.trim_end_matches('/').to_string(),
        plane_web_url: plane_web_url.trim_end_matches('/').to_string(),
        plane_api_url_set: !plane_api_url.trim_end_matches('/').is_empty(),
        plane_web_url_set: !plane_web_url.trim_end_matches('/').is_empty(),
        plane_api_key_hint: plane_api_key_hint(&plane_api_key),
        plane_api_key_set: plane_api_key.is_some(),
        sync_enabled: connected,
        sync_mode: plane_sync_mode(),
        connected,
        connection_status: connection_status.clone(),
        connection_status_configured,
        plane_service_healthy: plane_health_healthy,
        plane_api_latency_ms,
        plane_health_endpoints,
        mapped_features_coverage: percentage_coverage(mapped_features, total_features),
        mapped_work_packages_coverage: percentage_coverage(mapped_work_packages, total_work_packages),
        mapped_features,
        mapped_work_packages,
        config_warnings,
    })
}

pub async fn agent_settings_page() -> Response {
    let config = Config::load().unwrap_or(Config {
        plane: None,
        agents: None,
        services: None,
        dashboard: None,
    });

    let agent_config = config.agents.unwrap_or_else(|| AgentConfig {
        pool_size: 6,
        retry_budget: 3,
        dispatch_mode: "balanced".to_string(),
        default_provider: "claude".to_string(),
    });

    render(AgentSettingsPage {
        agent_pool_size: agent_config.pool_size,
        retry_budget: agent_config.retry_budget,
        dispatch_mode: agent_config.dispatch_mode,
        default_provider: agent_config.default_provider,
    })
}

pub async fn services_settings_page(State(state): State<SharedState>) -> Response {
    let store = state.read().await;
    let config = Config::load().unwrap_or(Config {
        plane: None,
        agents: None,
        services: None,
        dashboard: None,
    });

    let configs: Vec<crate::templates::ServiceConfigView> = config
        .services
        .unwrap_or_default()
        .into_iter()
        .map(|s| crate::templates::ServiceConfigView {
            name: s.name,
            endpoint_url: s.endpoint_url,
        })
        .collect();

    render(ServicesSettingsPage {
        services: store.health.clone(),
        configs,
    })
}

pub async fn save_plane_settings(axum::Form(form): axum::Form<PlaneSettingsForm>) -> Response {
    let mut config = match Config::load() {
        Ok(c) => c,
        Err(_) => Config { plane: None, agents: None, services: None, dashboard: None },
    };

    config.plane = Some(PlaneConfig {
        api_url: form.api_url.trim().to_string(),
        api_key: form.api_key.trim().to_string(),
        workspace_slug: form.workspace_slug.trim().to_string(),
        project_slug: form.project_slug.trim().to_string(),
    });

    use crate::templates::ToastPartial;
    match config.save() {
        Ok(_) => render(ToastPartial { message: "Plane settings saved successfully".to_string(), success: true }),
        Err(e) => render(ToastPartial { message: format!("Failed to save settings: {}", e), success: false }),
    }
}

pub async fn save_agent_settings(axum::Form(form): axum::Form<AgentSettingsForm>) -> Response {
    let mut config = match Config::load() {
        Ok(c) => c,
        Err(_) => Config { plane: None, agents: None, services: None, dashboard: None },
    };

    config.agents = Some(AgentConfig {
        pool_size: form.pool_size,
        retry_budget: form.retry_budget,
        dispatch_mode: form.dispatch_mode.trim().to_string(),
        default_provider: form.default_provider.trim().to_string(),
    });

    use crate::templates::ToastPartial;
    match config.save() {
        Ok(_) => render(ToastPartial { message: "Agent settings saved successfully".to_string(), success: true }),
        Err(e) => render(ToastPartial { message: format!("Failed to save settings: {}", e), success: false }),
    }
}

pub async fn save_dashboard_settings(axum::Form(form): axum::Form<DashboardSettingsForm>) -> Response {
    let mut config = match Config::load() {
        Ok(c) => c,
        Err(_) => Config { plane: None, agents: None, services: None, dashboard: None },
    };

    config.dashboard = Some(DashboardConfig {
        theme: form.theme.trim().to_string(),
        log_level: form.log_level.trim().to_string(),
        data_directory: form.data_directory.trim().to_string(),
    });

    use crate::templates::ToastPartial;
    match config.save() {
        Ok(_) => render(ToastPartial { message: "Dashboard settings saved successfully".to_string(), success: true }),
        Err(e) => render(ToastPartial { message: format!("Failed to save settings: {}", e), success: false }),
    }
}

pub async fn save_services_settings(axum::Form(form): axum::Form<ServiceSettingsForm>) -> Response {
    let mut config = match Config::load() {
        Ok(c) => c,
        Err(_) => Config { plane: None, agents: None, services: None, dashboard: None },
    };

    let mut services = Vec::new();
    for (name, url) in form.names.into_iter().zip(form.endpoint_urls.into_iter()) {
        if !name.trim().is_empty() {
            services.push(ServiceConfig {
                name: name.trim().to_string(),
                endpoint_url: url.trim().to_string(),
                enabled: default_service_enabled(),
                timeout_ms: None,
                max_retries: None,
            });
        }
    }
    config.services = Some(services);

    use crate::templates::ToastPartial;
    match config.save() {
        Ok(_) => render(ToastPartial { message: "Service settings saved successfully".to_string(), success: true }),
        Err(e) => render(ToastPartial { message: format!("Failed to save settings: {}", e), success: false }),
    }
}

#[derive(serde::Deserialize)]
pub struct SingleServiceTestForm {
    pub name: String,
    pub endpoint_url: String,
}

pub async fn test_service_connection(
    axum::Form(form): axum::Form<SingleServiceTestForm>,
) -> Response {
    use crate::templates::ToastPartial;
    let is_valid = !form.endpoint_url.trim().is_empty() && form.endpoint_url.starts_with("http");

    if is_valid {
        render(ToastPartial { message: format!("Connection to {} successful (mock)", form.name), success: true })
    } else {
        render(ToastPartial { message: format!("Invalid endpoint for {}: {}", form.name, form.endpoint_url), success: false })
    }
}

pub async fn test_plane_connection(axum::Form(form): axum::Form<PlaneSettingsForm>) -> Response {
    use crate::templates::ToastPartial;
    let is_valid = !form.api_url.trim().is_empty()
        && !form.api_key.trim().is_empty()
        && !form.workspace_slug.trim().is_empty()
        && form.api_url.starts_with("http");

    if is_valid {
        render(ToastPartial { message: "Plane connection test passed (mock)".to_string(), success: true })
    } else {
        render(ToastPartial { message: "Plane settings are incomplete or invalid".to_string(), success: false })
    }
}

#[derive(serde::Deserialize)]
pub struct AgentTestConnectionForm {
    pub provider: String,
}

pub async fn test_agent_connection(
    axum::Form(form): axum::Form<AgentTestConnectionForm>,
) -> impl IntoResponse {
    let (ok, msg) = match form.provider.as_str() {
        "claude" => {
            let key = env_or_none("ANTHROPIC_API_KEY");
            if key.is_some() {
                (true, "Claude API key detected — connection likely valid".to_string())
            } else {
                (false, "ANTHROPIC_API_KEY not set".to_string())
            }
        }
        "gemini" => {
            let key = env_or_none("GEMINI_API_KEY").or_else(|| env_or_none("GOOGLE_API_KEY"));
            if key.is_some() {
                (true, "Gemini API key detected — connection likely valid".to_string())
            } else {
                (false, "GEMINI_API_KEY / GOOGLE_API_KEY not set".to_string())
            }
        }
        "local" => (true, "Local provider requires no external credentials".to_string()),
        other => (false, format!("Unknown provider: {}", other)),
    };

    let css = if ok { "text-green-400" } else { "text-red-400" };
    axum::response::Html(format!(r#"<span class="{}">{}</span>"#, css, msg)).into_response()
}

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/settings", get(settings_page))
        .route("/settings/plane", get(plane_settings_page))
        .route("/settings/agents", get(agent_settings_page))
        .route("/settings/services", get(services_settings_page))
        .route("/api/settings/plane", post(save_plane_settings))
        .route("/api/settings/plane/test", post(test_plane_connection))
        .route("/api/settings/agents", post(save_agent_settings))
        .route("/api/settings/agents/test-connection", post(test_agent_connection))
        .route("/api/settings/dashboard", post(save_dashboard_settings))
        .route("/api/settings/services", post(save_services_settings))
        .route("/api/settings/services/test", post(test_service_connection))
}