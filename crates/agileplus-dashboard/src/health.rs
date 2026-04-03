use axum::{
    extract::State,
    response::IntoResponse,
    routing::get,
    Router,
};
use chrono::Utc;
use crate::app_state::SharedState;
use crate::templates::HealthPanelPartial;
use crate::routes::{render, ServiceHealthJson, HealthStatus};

pub async fn health_panel(State(state): State<SharedState>) -> Response {
    let store = state.read().await;
    render(HealthPanelPartial {
        services: store.health.clone(),
    })
}

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

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/api/dashboard/health", get(health_panel))
        .route("/api/dashboard/health.json", get(health_json))
}