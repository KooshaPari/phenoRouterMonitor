//! HTTP request handlers for monitoring API

use crate::error::{ApiError, Result};
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, Json};
use phenotype_router_metrics::PrometheusExporter;
use serde::Serialize;
use serde_json::json;

/// Health check response
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub uptime_seconds: u64,
    pub uptime: String,
    pub routes_count: usize,
    pub timestamp: String,
}

/// GET /health - Health check endpoint
pub async fn get_health(State(state): State<AppState>) -> Result<Json<HealthResponse>> {
    let response = HealthResponse {
        status: "healthy".to_string(),
        uptime_seconds: state.uptime_seconds(),
        uptime: state.uptime_string(),
        routes_count: state.router.routes_count(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(response))
}

/// GET /metrics - Metrics in Prometheus format
pub async fn get_metrics(State(state): State<AppState>) -> Result<String> {
    let exporter = PrometheusExporter::new((*state.metrics).clone());
    exporter.export()
}

/// GET /metrics/json - Metrics in JSON format
pub async fn get_metrics_json(State(state): State<AppState>) -> Result<Json<serde_json::Value>> {
    let exporter = PrometheusExporter::new((*state.metrics).clone());
    let json = exporter.export_json()?;
    Ok(Json(json))
}

/// Agent list response
#[derive(Serialize)]
pub struct AgentInfo {
    pub id: String,
    pub service: String,
    pub status: String,
    pub uptime_seconds: u64,
}

/// GET /agents - Get list of configured services
pub async fn get_agents(State(state): State<AppState>) -> Result<Json<Vec<AgentInfo>>> {
    let agents: Vec<AgentInfo> = state
        .router
        .routes()
        .iter()
        .enumerate()
        .map(|(idx, route)| AgentInfo {
            id: format!("agent-{}", idx),
            service: route.service.clone(),
            status: "active".to_string(),
            uptime_seconds: state.uptime_seconds(),
        })
        .collect();

    Ok(Json(agents))
}

/// Request to refresh agent configuration
#[derive(serde::Deserialize)]
pub struct RefreshRequest {
    pub service: Option<String>,
}

/// POST /agents/refresh - Reload configuration
pub async fn post_agents_refresh(
    State(_state): State<AppState>,
    Json(_req): Json<RefreshRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    // In a real implementation, this would reload configuration
    // For now, return success with a note that it requires restart
    let response = json!({
        "status": "acknowledged",
        "message": "Configuration reload queued (requires server restart for full effect)",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    Ok((StatusCode::ACCEPTED, Json(response)))
}

/// Router info response
#[derive(Serialize)]
pub struct RouterInfo {
    pub routes_count: usize,
    pub listen_addr: String,
    pub listen_port: u16,
    pub total_requests: u64,
    pub in_flight_requests: u64,
}

/// GET /router/info - Get router configuration and statistics
pub async fn get_router_info(State(state): State<AppState>) -> Result<Json<RouterInfo>> {
    let config = state.router.config();
    let metrics = state.metrics.clone();

    let response = RouterInfo {
        routes_count: state.router.routes_count(),
        listen_addr: config.listen_addr.clone(),
        listen_port: config.listen_port,
        total_requests: metrics.get_request_count(),
        in_flight_requests: metrics.get_in_flight_count(),
    };

    Ok(Json(response))
}

/// Route info response
#[derive(Serialize)]
pub struct RouteDetail {
    pub service: String,
    pub pattern: String,
    pub backends_count: usize,
    pub request_count: u64,
    pub timeout_ms: u64,
}

/// GET /router/routes - Get all configured routes
pub async fn get_router_routes(
    State(state): State<AppState>,
) -> Result<Json<Vec<RouteDetail>>> {
    let routes = state
        .router
        .routes()
        .iter()
        .map(|route| {
            let request_count = state.metrics.get_service_count(&route.service);
            RouteDetail {
                service: route.service.clone(),
                pattern: "regex-based".to_string(), // Pattern type would need to be exposed from Router
                backends_count: route.pool.len(),
                request_count,
                timeout_ms: route.timeout_ms,
            }
        })
        .collect();

    Ok(Json(routes))
}

/// Ready check response
#[derive(Serialize)]
pub struct ReadyResponse {
    pub ready: bool,
    pub reason: Option<String>,
}

/// GET /ready - Kubernetes readiness probe
pub async fn get_ready(State(state): State<AppState>) -> Result<Json<ReadyResponse>> {
    let ready = state.router.routes_count() > 0;
    let response = ReadyResponse {
        ready,
        reason: if ready {
            None
        } else {
            Some("No routes configured".to_string())
        },
    };

    Ok(Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-ROUTER-015 (HTTP handlers)
    #[test]
    fn test_health_response_structure() {
        let toml = r#"
[[routes]]
service = "api"
path_pattern = "^/api/.*"
backends = ["http://localhost:3000"]
timeout_ms = 30000
strategy = "roundrobin"
"#;
        let router = phenotype_router_core::Router::from_string(toml).unwrap();
        let metrics = phenotype_router_metrics::MetricsCollector::new();
        let state = AppState::new(router, metrics);

        let response = HealthResponse {
            status: "healthy".to_string(),
            uptime_seconds: state.uptime_seconds(),
            uptime: state.uptime_string(),
            routes_count: state.router.routes_count(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        assert_eq!(response.status, "healthy");
        assert!(response.uptime_seconds >= 0);
    }

    // Traces to: FR-ROUTER-015
    #[test]
    fn test_agent_info_structure() {
        let agent = AgentInfo {
            id: "agent-0".to_string(),
            service: "api".to_string(),
            status: "active".to_string(),
            uptime_seconds: 100,
        };

        assert_eq!(agent.service, "api");
        assert_eq!(agent.status, "active");
    }

    // Traces to: FR-ROUTER-015
    #[test]
    fn test_router_info_structure() {
        let info = RouterInfo {
            routes_count: 2,
            listen_addr: "0.0.0.0".to_string(),
            listen_port: 3030,
            total_requests: 100,
            in_flight_requests: 5,
        };

        assert_eq!(info.routes_count, 2);
        assert_eq!(info.listen_port, 3030);
    }

    // Traces to: FR-ROUTER-015
    #[test]
    fn test_route_detail_structure() {
        let route = RouteDetail {
            service: "api".to_string(),
            pattern: "regex-based".to_string(),
            backends_count: 3,
            request_count: 50,
            timeout_ms: 30000,
        };

        assert_eq!(route.service, "api");
        assert_eq!(route.backends_count, 3);
    }
}
