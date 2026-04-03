//! Service management and configuration handlers.

use std::process::Command;

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use chrono::Utc;
use serde::Deserialize;

use crate::app_state::{ServiceHealth, SharedState};
use crate::templates::ToastPartial;

use super::helpers::render;
use super::{Config, ServiceConfig, ServiceSettingsForm};

// ── Service Configuration Types ────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ServiceConfigForm {
    pub endpoint_url: Option<String>,
    pub timeout_ms: Option<u64>,
    pub max_retries: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ServiceToggleBody {
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SingleServiceTestForm {
    pub name: String,
    pub endpoint_url: String,
}

// ── Service Restart ───────────────────────────────────────────────────────

const ALLOWED_RESTART_PROGRAMS: [&str; 4] = ["systemctl", "docker", "process-compose", "echo"];

fn is_restart_command_allowed(program: &str) -> bool {
    ALLOWED_RESTART_PROGRAMS.contains(&program)
}

fn validate_restart_command(cmd_line: &str) -> Result<(), String> {
    let mut parts: Vec<&str> = cmd_line.split_whitespace().collect();
    if parts.is_empty() {
        return Err("empty restart command".into());
    }

    let program = parts.remove(0);
    if !is_restart_command_allowed(program) {
        return Err(format!(
            "command '{}' is not in approved restart command registry: {:?}",
            program, ALLOWED_RESTART_PROGRAMS
        ));
    }

    Ok(())
}

fn build_restart_command(cmd_line: &str) -> Result<Command, String> {
    validate_restart_command(cmd_line)?;
    let mut parts = cmd_line.split_whitespace();
    let program = parts.next().ok_or("empty command")?;
    let mut cmd = Command::new(program);
    for arg in parts {
        cmd.arg(arg);
    }
    Ok(cmd)
}

pub async fn restart_service(
    State(_state): State<SharedState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let template = std::env::var("AGILEPLUS_SERVICE_RESTART_CMD")
        .unwrap_or_else(|_| "systemctl restart {}".to_string());

    if !template.contains("{}") {
        return axum::Json(serde_json::json!({
            "status": "error",
            "service": name,
            "error": "AGILEPLUS_SERVICE_RESTART_CMD must include '{}' placeholder",
        }));
    }

    let command_str = template.replace("{}", &name);

    let mut command = match build_restart_command(&command_str) {
        Ok(c) => c,
        Err(err) => {
            return axum::Json(serde_json::json!({
                "status": "error",
                "service": name,
                "command": command_str,
                "error": err,
            }));
        }
    };

    match command.output() {
        Ok(output) => {
            let success = output.status.success();
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            axum::Json(serde_json::json!({
                "status": if success { "ok" } else { "error" },
                "service": name,
                "command": command_str,
                "stdout": stdout,
                "stderr": stderr,
            }))
        }
        Err(err) => axum::Json(serde_json::json!({
            "status": "error",
            "service": name,
            "command": command_str,
            "error": err.to_string(),
        })),
    }
}

// ── Service Configuration ──────────────────────────────────────────────────

pub async fn patch_service_config(
    Path(name): Path<String>,
    axum::Form(form): axum::Form<ServiceConfigForm>,
) -> impl IntoResponse {
    let mut config = Config::load().unwrap_or(Config {
        plane: None,
        agents: None,
        services: None,
        dashboard: None,
    });

    let services = config.services.get_or_insert_with(Vec::new);
    if let Some(entry) = services.iter_mut().find(|s| s.name == name) {
        if let Some(url) = form.endpoint_url.filter(|u| !u.trim().is_empty()) {
            entry.endpoint_url = url;
        }
    } else if let Some(url) = form.endpoint_url.filter(|u| !u.trim().is_empty()) {
        services.push(ServiceConfig {
            name: name.clone(),
            endpoint_url: url,
            enabled: true,
            timeout_ms: form.timeout_ms,
            max_retries: form.max_retries,
        });
    }

    match config.save() {
        Ok(_) => render(ToastPartial {
            message: format!("Service '{}' configuration saved", name),
            success: true,
        }),
        Err(e) => render(ToastPartial {
            message: format!("Failed to save: {}", e),
            success: false,
        }),
    }
}

pub async fn toggle_service(
    State(state): State<SharedState>,
    Path(name): Path<String>,
    axum::Json(body): axum::Json<ServiceToggleBody>,
) -> impl IntoResponse {
    let enabled = body.enabled.unwrap_or(true);

    // Persist state in config file
    let mut config = Config::load().unwrap_or(Config {
        plane: None,
        agents: None,
        services: None,
        dashboard: None,
    });

    let services = config.services.get_or_insert_with(Vec::new);
    if let Some(entry) = services.iter_mut().find(|s| s.name == name) {
        entry.enabled = enabled;
    } else {
        services.push(ServiceConfig {
            name: name.clone(),
            endpoint_url: String::new(),
            enabled,
            timeout_ms: None,
            max_retries: None,
        });
    }

    if let Err(err) = config.save() {
        return axum::Json(serde_json::json!({
            "status": "error",
            "service": name,
            "enabled": enabled,
            "error": format!("Failed to save config: {}", err),
        }));
    }

    // Update in-memory health status for UI
    {
        let mut store = state.write().await;
        if let Some(item) = store.health.iter_mut().find(|s| s.name == name) {
            item.healthy = enabled;
            item.degraded = !enabled;
            item.last_check = Utc::now();
        } else {
            store.health.push(ServiceHealth {
                name: name.clone(),
                healthy: enabled,
                degraded: !enabled,
                latency_ms: None,
                last_check: Utc::now(),
            });
        }
    }

    axum::Json(serde_json::json!({
        "status": "ok",
        "service": name,
        "enabled": enabled,
    }))
}

// ── Service Connection Testing ─────────────────────────────────────────────

pub async fn test_service_connection(
    axum::Form(form): axum::Form<SingleServiceTestForm>,
) -> Response {
    let is_valid = !form.endpoint_url.trim().is_empty() && form.endpoint_url.starts_with("http");

    if is_valid {
        render(ToastPartial {
            message: format!("Connection to {} successful (mock)", form.name),
            success: true,
        })
    } else {
        render(ToastPartial {
            message: format!("Invalid endpoint for {}: {}", form.name, form.endpoint_url),
            success: false,
        })
    }
}

// ── Settings POST Handlers ─────────────────────────────────────────────────

pub async fn save_services_settings(axum::Form(form): axum::Form<ServiceSettingsForm>) -> Response {
    let mut config = match Config::load() {
        Ok(c) => c,
        Err(_) => Config {
            plane: None,
            agents: None,
            services: None,
            dashboard: None,
        },
    };

    let mut services = Vec::new();
    for (name, url) in form.names.into_iter().zip(form.endpoint_urls.into_iter()) {
        if !name.trim().is_empty() {
            services.push(ServiceConfig {
                name: name.trim().to_string(),
                endpoint_url: url.trim().to_string(),
                enabled: true,
                timeout_ms: None,
                max_retries: None,
            });
        }
    }
    config.services = Some(services);

    match config.save() {
        Ok(_) => render(ToastPartial {
            message: "Service settings saved successfully".to_string(),
            success: true,
        }),
        Err(e) => render(ToastPartial {
            message: format!("Failed to save settings: {}", e),
            success: false,
        }),
    }
}
