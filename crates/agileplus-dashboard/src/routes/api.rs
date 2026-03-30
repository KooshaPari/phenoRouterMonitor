//! JSON API endpoints for the dashboard.

use axum::{extract::State, response::IntoResponse};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::app_state::SharedState;
use crate::process_detector;

// ── JSON API Response Types ────────────────────────────────────────────────

/// JSON response for GET /api/dashboard/agents (real-time agent detection)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub name: String,
    pub status: String,
    pub current_task: String,
    pub pid: Option<u32>,
    pub started_at: Option<String>,
    pub worktree: String,
    pub uptime: String,
}

/// JSON response for GET /api/dashboard/health (service health status)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub services: Vec<ServiceHealthJson>,
    pub timestamp: String,
    pub all_healthy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealthJson {
    pub name: String,
    pub healthy: bool,
    pub degraded: bool,
    pub latency_ms: Option<u64>,
    pub last_check: String,
}

/// JSON response for GET /api/dashboard/features/{id}/evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceGalleryJson {
    pub feature_id: String,
    pub artifacts: Vec<EvidenceArtifactJson>,
    pub generated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceArtifactJson {
    pub id: String,
    pub type_: String,
    pub title: String,
    pub path: String,
    pub url: String,
    pub created_at: String,
}

// ── JSON API Handlers ──────────────────────────────────────────────────────

/// JSON API: GET /api/dashboard/agents
/// Returns detected agent processes as JSON (polls every 5s from dashboard templates).
pub async fn agents_json(State(_state): State<SharedState>) -> impl IntoResponse {
    // Detect real agent processes
    let detected = process_detector::detect_agents();

    // Convert detected agents to JSON response
    let agents: Vec<AgentInfo> = detected
        .into_iter()
        .map(|agent| {
            let uptime = calculate_uptime(&agent.started_at);
            AgentInfo {
                name: agent.name,
                status: agent.status.clone(),
                current_task: agent.current_task,
                pid: Some(agent.pid),
                started_at: agent.started_at,
                worktree: agent.worktree.unwrap_or_default(),
                uptime,
            }
        })
        .collect();

    axum::Json(serde_json::json!({
        "agents": agents,
        "count": agents.len(),
        "timestamp": Utc::now().to_rfc3339(),
    }))
}

/// JSON API: GET /api/dashboard/health
/// Returns service health status as JSON (polls every 10s from dashboard templates).
pub async fn health_json(State(state): State<SharedState>) -> impl IntoResponse {
    let store = state.read().await;

    let services: Vec<ServiceHealthJson> = store
        .health
        .iter()
        .map(|service| ServiceHealthJson {
            name: service.name.clone(),
            healthy: service.healthy,
            degraded: service.degraded,
            latency_ms: service.latency_ms,
            last_check: service.last_check.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        })
        .collect();

    let all_healthy = services.iter().all(|s| s.healthy && !s.degraded);

    axum::Json(HealthStatus {
        services,
        timestamp: Utc::now().to_rfc3339(),
        all_healthy,
    })
}

// ── Helper Functions ───────────────────────────────────────────────────────

/// Calculate uptime string from the elapsed duration string produced by
/// `process_detector::get_process_start_time` (e.g. "5m", "1h 20m").
fn calculate_uptime(started_at: &Option<String>) -> String {
    match started_at {
        Some(s) => s.clone(),
        None => "unknown".to_string(),
    }
}
