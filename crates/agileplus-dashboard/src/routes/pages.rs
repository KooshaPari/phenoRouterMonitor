use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, Response},
};

use agileplus_domain::domain::state_machine::FeatureState;

use crate::app_state::SharedState;
use crate::templates::{
    AgentSettingsPage, EventsPage, FeatureView, FeaturesPage, HomePage, PlaneHealthEndpointView,
    PlaneSettingsPage, ServicesSettingsPage, SettingsPage,
};

use super::helpers::{
    DEFAULT_PLANE_API_URL, DEFAULT_PLANE_WEB_URL, build_project_summaries, env_or_none,
    parse_bool_env, render, sample_events,
};

fn plane_api_key_hint(api_key: &Option<String>) -> String {
    match api_key {
        Some(key) => match (key.chars().next(), key.chars().rev().next()) {
            (Some(first), Some(last)) => format!("{first}••••••{last}"),
            _ => "Configured".to_string(),
        },
        None => "Not configured".to_string(),
    }
}

fn plane_health_endpoints(
    services: &[crate::app_state::ServiceHealth],
) -> Vec<PlaneHealthEndpointView> {
    services
        .iter()
        .filter(|service| service.name.contains("Plane") || service.name.starts_with("API"))
        .map(|service| PlaneHealthEndpointView {
            name: service.name.clone(),
            healthy: service.healthy,
            degraded: service.degraded,
            latency_ms: service.latency_ms,
            last_check_utc: service
                .last_check
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string(),
        })
        .collect()
}

fn plane_sync_mode() -> String {
    if parse_bool_env("PLANE_SYNC_BIDIRECTIONAL", false) {
        "Bidirectional".to_string()
    } else {
        "One-way".to_string()
    }
}

fn plane_connection_checks(
    api_key: &Option<String>,
    workspace: &Option<String>,
) -> (bool, String, Vec<String>) {
    let mut warnings = Vec::new();
    if api_key.is_none() {
        warnings.push("Missing PLANE_API_KEY; configure a valid Plane API key".to_string());
    }
    if workspace.is_none() {
        warnings.push("Missing PLANE_WORKSPACE; set workspace slug for Plane sync".to_string());
    }

    if warnings.is_empty() {
        (true, "Connected via PLANE_API_KEY".to_string(), warnings)
    } else if warnings.len() == 1 {
        let status = warnings[0].clone();
        (false, status, warnings)
    } else {
        (false, "Plane settings incomplete".to_string(), warnings)
    }
}

fn percentage_coverage(hit: usize, total: usize) -> String {
    if total == 0 {
        return "0/0 (0%)".to_string();
    }
    let ratio = (hit.saturating_mul(100)).saturating_div(total);
    format!("{hit}/{total} ({ratio}%)")
}

pub async fn root(State(state): State<SharedState>) -> Response {
    let store = state.read().await;
    let total_features = store.features.len();
    let active_features = store
        .features
        .iter()
        .filter(|feature| {
            !matches!(
                feature.state,
                FeatureState::Shipped | FeatureState::Retrospected
            )
        })
        .count();
    let shipped_features = store
        .features
        .iter()
        .filter(|feature| {
            matches!(
                feature.state,
                FeatureState::Shipped | FeatureState::Retrospected
            )
        })
        .count();
    let projects = build_project_summaries(&store);

    render(HomePage {
        total_features,
        active_features,
        shipped_features,
        projects,
    })
}

pub async fn home(State(state): State<SharedState>) -> Response {
    root(State(state)).await
}

<<<<<<< HEAD
=======
pub async fn dashboard_page(
    State(state): State<SharedState>,
    axum::extract::Query(query): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Response {
    use crate::templates::DashboardPage;

    let store = state.read().await;
    let filter = super::helpers::dashboard_filter_from_query(&query);
    let cards = super::helpers::build_kanban_cards(&store, filter);
    let (projects, active_project) = super::helpers::load_projects(&store);
    let active_filter = query.get("filter").cloned().unwrap_or_else(|| "all".into());
    render(DashboardPage {
        kanban_cards: cards,
        health: store.health.clone(),
        projects,
        active_project,
        active_filter,
    })
}

pub async fn hub_page() -> Response {
    use crate::templates::HubPage;
    render(HubPage {
        projects: vec![],
    })
}

>>>>>>> origin/main
pub async fn settings_page() -> Response {
    render(SettingsPage)
}

pub async fn features_page(State(state): State<SharedState>) -> Response {
    let store = state.read().await;
    let features = store
        .features
        .iter()
        .map(FeatureView::from_feature)
        .collect::<Vec<_>>();
    render(FeaturesPage { features })
}

pub async fn events_page() -> Response {
    render(EventsPage {
        events: sample_events(),
    })
}

pub async fn plane_settings_page(State(state): State<SharedState>) -> Response {
    let store = state.read().await;
    let plane_workspace = env_or_none("PLANE_WORKSPACE");
    let project_slug = env_or_none("PLANE_PROJECT").unwrap_or_else(|| "not configured".to_string());
    let plane_api_key = env_or_none("PLANE_API_KEY");
    let plane_api_url =
        env_or_none("PLANE_API_URL").unwrap_or_else(|| DEFAULT_PLANE_API_URL.to_string());
    let plane_web_url =
        env_or_none("PLANE_WEB_URL").unwrap_or_else(|| DEFAULT_PLANE_WEB_URL.to_string());
    let (connected, connection_status, mut config_warnings) =
        plane_connection_checks(&plane_api_key, &plane_workspace);

    let plane_health_endpoints = plane_health_endpoints(&store.health);
    let plane_health_healthy = plane_health_endpoints
        .iter()
        .all(|endpoint| endpoint.healthy && !endpoint.degraded);
    let plane_api_latency_ms = plane_health_endpoints
        .iter()
        .find(|endpoint| endpoint.name == "Plane API")
        .and_then(|endpoint| endpoint.latency_ms);

    if !connected {
        config_warnings
            .push("Plane sync disabled until required settings are provided".to_string());
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
        workspace_name: plane_workspace
            .clone()
            .unwrap_or_else(|| "Not configured".to_string()),
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
        mapped_work_packages_coverage: percentage_coverage(
            mapped_work_packages,
            total_work_packages,
        ),
        mapped_features,
        mapped_work_packages,
        config_warnings,
    })
}

pub async fn agent_settings_page() -> Response {
    render(AgentSettingsPage {
        agent_pool_size: 6,
        retry_budget: 3,
        dispatch_mode: "balanced".into(),
<<<<<<< HEAD
=======
        default_provider: "default".into(),
>>>>>>> origin/main
    })
}

pub async fn services_settings_page(State(state): State<SharedState>) -> Response {
    let store = state.read().await;
<<<<<<< HEAD
    render(ServicesSettingsPage {
        services: store.health.clone(),
=======
    let configs = store.health.iter().map(|h| crate::templates::ServiceConfigView {
        name: h.name.clone(),
        endpoint_url: format!("http://localhost:8080/health/{}", h.name),
    }).collect();
    render(ServicesSettingsPage {
        services: store.health.clone(),
        configs,
>>>>>>> origin/main
    })
}

pub async fn time_footer() -> Html<String> {
    Html(
        chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string(),
    )
}

pub async fn stream_placeholder() -> StatusCode {
    StatusCode::NO_CONTENT
}
<<<<<<< HEAD
=======

// ── Settings POST Handlers ─────────────────────────────────────────────────

pub async fn save_plane_settings(axum::Form(form): axum::Form<super::PlaneSettingsForm>) -> Response {
    use crate::templates::ToastPartial;
    use super::{Config, PlaneConfig};

    let mut config = match Config::load() {
        Ok(c) => c,
        Err(_) => Config {
            plane: None,
            agents: None,
            services: None,
            dashboard: None,
        },
    };

    config.plane = Some(PlaneConfig {
        api_url: form.api_url.trim().to_string(),
        api_key: form.api_key.trim().to_string(),
        workspace_slug: form.workspace_slug.trim().to_string(),
        project_slug: form.project_slug.trim().to_string(),
    });

    match config.save() {
        Ok(_) => render(ToastPartial {
            message: "Plane settings saved successfully".to_string(),
            success: true,
        }),
        Err(e) => render(ToastPartial {
            message: format!("Failed to save settings: {}", e),
            success: false,
        }),
    }
}

pub async fn save_agent_settings(axum::Form(form): axum::Form<super::AgentSettingsForm>) -> Response {
    use crate::templates::ToastPartial;
    use super::{Config, AgentConfig};

    let mut config = match Config::load() {
        Ok(c) => c,
        Err(_) => Config {
            plane: None,
            agents: None,
            services: None,
            dashboard: None,
        },
    };

    config.agents = Some(AgentConfig {
        pool_size: form.pool_size,
        retry_budget: form.retry_budget,
        dispatch_mode: form.dispatch_mode.trim().to_string(),
        default_provider: form.default_provider.trim().to_string(),
    });

    match config.save() {
        Ok(_) => render(ToastPartial {
            message: "Agent settings saved successfully".to_string(),
            success: true,
        }),
        Err(e) => render(ToastPartial {
            message: format!("Failed to save settings: {}", e),
            success: false,
        }),
    }
}

pub async fn save_dashboard_settings(
    axum::Form(form): axum::Form<super::DashboardSettingsForm>,
) -> Response {
    use crate::templates::ToastPartial;
    use super::{Config, DashboardConfig};

    let mut config = match Config::load() {
        Ok(c) => c,
        Err(_) => Config {
            plane: None,
            agents: None,
            services: None,
            dashboard: None,
        },
    };

    config.dashboard = Some(DashboardConfig {
        theme: form.theme.trim().to_string(),
        log_level: form.log_level.trim().to_string(),
        data_directory: form.data_directory.trim().to_string(),
    });

    match config.save() {
        Ok(_) => render(ToastPartial {
            message: "Dashboard settings saved successfully".to_string(),
            success: true,
        }),
        Err(e) => render(ToastPartial {
            message: format!("Failed to save settings: {}", e),
            success: false,
        }),
    }
}

pub async fn test_agent_connection(
    axum::Form(form): axum::Form<super::AgentSettingsForm>,
) -> Html<String> {
    use super::helpers::env_or_none;

    // Provider reachability check: validate that required env vars are present.
    let (ok, msg) = match form.default_provider.as_str() {
        "claude" => {
            let key = env_or_none("ANTHROPIC_API_KEY");
            if key.is_some() {
                (
                    true,
                    "Claude API key detected — connection likely valid".to_string(),
                )
            } else {
                (false, "ANTHROPIC_API_KEY not set".to_string())
            }
        }
        "gemini" => {
            let key = env_or_none("GEMINI_API_KEY").or_else(|| env_or_none("GOOGLE_API_KEY"));
            if key.is_some() {
                (
                    true,
                    "Gemini API key detected — connection likely valid".to_string(),
                )
            } else {
                (false, "GEMINI_API_KEY / GOOGLE_API_KEY not set".to_string())
            }
        }
        "local" => (
            true,
            "Local provider requires no external credentials".to_string(),
        ),
        other => (false, format!("Unknown provider: {}", other)),
    };

    let css = if ok { "text-green-400" } else { "text-red-400" };
    Html(format!(r#"<span class="{}">{}</span>"#, css, msg))
}

pub async fn test_plane_connection(axum::Form(form): axum::Form<super::PlaneSettingsForm>) -> Response {
    use crate::templates::ToastPartial;

    // Simple validation: check that required fields are filled and api_url looks like a URL
    let is_valid = !form.api_url.trim().is_empty()
        && !form.api_key.trim().is_empty()
        && !form.workspace_slug.trim().is_empty()
        && form.api_url.starts_with("http");

    if is_valid {
        // In a real implementation, you would make an HTTP request to verify connectivity
        render(ToastPartial {
            message: "Plane connection test passed (mock)".to_string(),
            success: true,
        })
    } else {
        render(ToastPartial {
            message: "Plane settings are incomplete or invalid".to_string(),
            success: false,
        })
    }
}
>>>>>>> origin/main
