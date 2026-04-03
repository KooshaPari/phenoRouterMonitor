//! AgilePlus Dashboard Library

pub mod health_scanner;
pub use health_scanner::{HealthScanner, HealthSummary};

use std::sync::Arc;
use tokio::sync::RwLock;

/// App state for Axum routes
#[derive(Clone)]
pub struct AppState {
    pub scanner: Arc<RwLock<HealthScanner>>,
}

/// Create router with health endpoints
pub fn create_router(state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/health/projects", axum::routing::get(list_projects))
        .with_state(state)
}

async fn list_projects(axum::extract::State(state): axum::extract::State<AppState>) -> axum::Json<serde_json::Value> {
    let scanner = state.scanner.read().await;
    axum::Json(serde_json::json!({
        "total_projects": scanner.health_summary().total_projects,
        "average_score": scanner.health_summary().average_score,
    }))
}
