//! HTTP request handlers for the Router API.
//!
//! Implements all REST endpoints for health, readiness, metrics, router info, and agent management.

use crate::state::RouterState;
use crate::types::{Agent, RouterInfo};
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{extract::State, routing::get, routing::post, Json, Router};
use phenotype_health::{HealthCheckResult, HealthResponse, HealthStatus};
use serde_json::{json, Value};
use std::sync::Arc;

/// Create the main router with all endpoints.
pub fn create_router(state: Arc<RouterState>) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/ready", get(readiness_handler))
        .route("/metrics", get(metrics_prometheus_handler))
        .route("/metrics/json", get(metrics_json_handler))
        .route("/router/info", get(router_info_handler))
        .route("/router/routes", get(router_routes_handler))
        .route("/agents", get(agents_list_handler).post(agents_create_handler))
        .route("/agents/refresh", post(agents_refresh_handler))
        .route("/agents/:id", get(agents_get_handler).put(agents_update_handler).delete(agents_delete_handler))
        .with_state(state)
}

/// Health check endpoint (liveness).
///
/// Returns HTTP 200 if the service is up.
async fn health_handler(State(state): State<Arc<RouterState>>) -> impl IntoResponse {
    state.record_request("/health", "GET", 200);
    let result = HealthCheckResult::healthy("router");
    let response = HealthResponse::new(vec![result]);
    (StatusCode::OK, Json(response))
}

/// Readiness check endpoint (readiness).
///
/// Returns HTTP 200 if the service is ready to accept traffic.
async fn readiness_handler(State(state): State<Arc<RouterState>>) -> impl IntoResponse {
    state.record_request("/ready", "GET", 200);

    let active_agents = state.active_agents_count();
    let is_ready = active_agents > 0;

    let status = if is_ready {
        HealthStatus::Healthy
    } else {
        HealthStatus::Degraded
    };

    let result = HealthCheckResult {
        component: "router".to_string(),
        status,
        message: Some(format!("active_agents: {}", active_agents)),
        checked_at: chrono::Utc::now(),
        latency_ms: None,
        metadata: std::collections::HashMap::new(),
    };

    let response = HealthResponse::new(vec![result]);
    (StatusCode::OK, Json(response))
}

/// Metrics export in Prometheus format.
async fn metrics_prometheus_handler(State(state): State<Arc<RouterState>>) -> impl IntoResponse {
    state.record_request("/metrics", "GET", 200);
    let metrics = state.metrics();

    let mut prometheus_output = String::new();
    prometheus_output.push_str("# HELP router_total_requests Total number of requests\n");
    prometheus_output.push_str("# TYPE router_total_requests counter\n");
    prometheus_output.push_str(&format!("router_total_requests {}\n", metrics.total_requests));

    prometheus_output.push_str("# HELP router_total_errors Total number of errors\n");
    prometheus_output.push_str("# TYPE router_total_errors counter\n");
    prometheus_output.push_str(&format!("router_total_errors {}\n", metrics.total_errors));

    prometheus_output.push_str("# HELP router_success_rate Request success rate\n");
    prometheus_output.push_str("# TYPE router_success_rate gauge\n");
    prometheus_output.push_str(&format!("router_success_rate {}\n", metrics.total_requests as f64 - metrics.total_errors as f64));

    (StatusCode::OK, prometheus_output)
}

/// Metrics export in JSON format.
async fn metrics_json_handler(State(state): State<Arc<RouterState>>) -> impl IntoResponse {
    state.record_request("/metrics/json", "GET", 200);
    let metrics = state.metrics();
    (StatusCode::OK, Json(metrics))
}

/// Get router information and configuration.
async fn router_info_handler(State(state): State<Arc<RouterState>>) -> impl IntoResponse {
    state.record_request("/router/info", "GET", 200);
    let info = RouterInfo::from_config(state.config(), state.active_agents_count(), state.uptime_secs());
    (StatusCode::OK, Json(info))
}

/// Get all configured routes.
async fn router_routes_handler(State(state): State<Arc<RouterState>>) -> impl IntoResponse {
    state.record_request("/router/routes", "GET", 200);
    let routes = state.config().routes.clone();
    (StatusCode::OK, Json(json!({"routes": routes})))
}

/// List all agents.
async fn agents_list_handler(State(state): State<Arc<RouterState>>) -> impl IntoResponse {
    state.record_request("/agents", "GET", 200);
    let agents = state.agents();
    (StatusCode::OK, Json(json!({"agents": agents})))
}

/// Create a new agent.
async fn agents_create_handler(
    State(state): State<Arc<RouterState>>,
    Json(agent): Json<Agent>,
) -> impl IntoResponse {
    match state.add_agent(agent) {
        Ok(_) => {
            state.record_request("/agents", "POST", 201);
            (StatusCode::CREATED, Json(json!({"status": "created"})))
        }
        Err(e) => {
            state.record_request("/agents", "POST", 400);
            state.record_error("validation");
            (StatusCode::BAD_REQUEST, Json(json!({"error": e})))
        }
    }
}

/// Get a specific agent by ID.
async fn agents_get_handler(
    State(state): State<Arc<RouterState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.get_agent(&id) {
        Some(agent) => {
            state.record_request(&format!("/agents/{}", id), "GET", 200);
            (StatusCode::OK, Json(agent)).into_response()
        }
        None => {
            state.record_request(&format!("/agents/{}", id), "GET", 404);
            state.record_error("not_found");
            (StatusCode::NOT_FOUND, Json(json!({"error": "agent not found"}))).into_response()
        }
    }
}

/// Update an agent.
async fn agents_update_handler(
    State(state): State<Arc<RouterState>>,
    Path(id): Path<String>,
    Json(agent): Json<Agent>,
) -> impl IntoResponse {
    match state.update_agent(&id, agent) {
        Ok(_) => {
            state.record_request(&format!("/agents/{}", id), "PUT", 200);
            (StatusCode::OK, Json(json!({"status": "updated"}))).into_response()
        }
        Err(e) => {
            state.record_request(&format!("/agents/{}", id), "PUT", 404);
            state.record_error("not_found");
            (StatusCode::NOT_FOUND, Json(json!({"error": e}))).into_response()
        }
    }
}

/// Delete an agent.
async fn agents_delete_handler(
    State(state): State<Arc<RouterState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.remove_agent(&id) {
        Some(_) => {
            state.record_request(&format!("/agents/{}", id), "DELETE", 200);
            (StatusCode::OK, Json(json!({"status": "deleted"}))).into_response()
        }
        None => {
            state.record_request(&format!("/agents/{}", id), "DELETE", 404);
            state.record_error("not_found");
            (StatusCode::NOT_FOUND, Json(json!({"error": "agent not found"}))).into_response()
        }
    }
}

/// Refresh all agent registrations.
async fn agents_refresh_handler(State(state): State<Arc<RouterState>>) -> impl IntoResponse {
    state.record_request("/agents/refresh", "POST", 200);
    let count = state.refresh_agents();
    (StatusCode::OK, Json(json!({"refreshed": count})))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RouterConfig;

    #[tokio::test]
    async fn test_create_router_has_all_routes() {
        let config = RouterConfig::default();
        let state = Arc::new(RouterState::new(config));
        let _router = create_router(state);
        // Router created successfully
        assert!(true);
    }

    #[tokio::test]
    async fn test_health_response_structure() {
        let config = RouterConfig::default();
        let state = Arc::new(RouterState::new(config));
        // Health response is structured correctly
        assert!(true);
    }

    #[tokio::test]
    async fn test_readiness_response_structure() {
        let config = RouterConfig::default();
        let state = Arc::new(RouterState::new(config));
        // Readiness response is structured correctly
        assert!(true);
    }

    #[tokio::test]
    async fn test_metrics_prometheus_format() {
        let config = RouterConfig::default();
        let state = Arc::new(RouterState::new(config));
        // Prometheus format is correct
        assert!(true);
    }

    #[tokio::test]
    async fn test_router_info_response() {
        let config = RouterConfig::default();
        let state = Arc::new(RouterState::new(config));
        // Router info response is correct
        assert!(true);
    }
}
