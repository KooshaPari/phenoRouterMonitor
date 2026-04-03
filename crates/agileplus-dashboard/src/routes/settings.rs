//! Settings POST handlers for plane, agent, and dashboard configuration.

use axum::{
    http::StatusCode,
    response::{Html, Response},
};

use crate::templates::ToastPartial;

use super::helpers::{env_or_none, render};
use super::{
    AgentConfig, AgentSettingsForm, Config, DashboardConfig, DashboardSettingsForm, PlaneConfig,
    PlaneSettingsForm,
};

pub async fn save_plane_settings(
    axum::Form(form): axum::Form<PlaneSettingsForm>,
) -> Response {
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

pub async fn save_agent_settings(
    axum::Form(form): axum::Form<AgentSettingsForm>,
) -> Response {
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
    axum::Form(form): axum::Form<DashboardSettingsForm>,
) -> Response {
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
    axum::Form(form): axum::Form<AgentSettingsForm>,
) -> Html<String> {
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

pub async fn test_plane_connection(
    axum::Form(form): axum::Form<PlaneSettingsForm>,
) -> Response {
    let is_valid = !form.api_url.trim().is_empty()
        && !form.api_key.trim().is_empty()
        && !form.workspace_slug.trim().is_empty()
        && form.api_url.starts_with("http");

    if is_valid {
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
