//! AgilePlus Dashboard

pub mod health_scanner;
pub mod routes;

pub use health_scanner::{HealthScanner, HealthSummary};
pub use routes::{AppState, create_router};

use std::sync::Arc;
use tokio::sync::RwLock;

pub async fn start_dashboard(root_path: impl Into<String>, port: u16) -> anyhow::Result<()> {
    let scanner = Arc::new(RwLock::new(HealthScanner::new(root_path, 24)));
    let state = AppState { scanner };
    let app = create_router(state);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
