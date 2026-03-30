//! Example API server for phenotype-router-monitor
//!
//! Run with: cargo run --example main -- --config examples/config.toml

use phenotype_router_api::ApiServer;
use phenotype_router_core::Router;
use phenotype_router_metrics::MetricsCollector;
use std::env;
use tracing::info;
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(true)
        .with_thread_ids(true)
        .with_level(true)
        .init();

    // Parse arguments
    let config_path = env::args()
        .nth(2)
        .unwrap_or_else(|| "examples/config.toml".to_string());

    info!("Loading configuration from: {}", config_path);

    // Load router configuration
    let router = Router::from_file(&config_path)
        .map_err(|e| anyhow::anyhow!("Failed to load router config: {}", e))?;

    info!("Loaded {} routes", router.routes_count());
    for route in router.routes() {
        info!(
            "  - Service: {} (backends: {})",
            route.service,
            route.pool.len()
        );
    }

    // Create metrics collector
    let metrics = MetricsCollector::new();

    // Create and run API server
    let server = ApiServer::new(router, metrics)
        .map_err(|e| anyhow::anyhow!("Failed to create API server: {}", e))?;

    let addr = server.addr();
    info!("Starting API server on http://{}", addr);
    info!("  - Health check: GET http://{}/ health", addr);
    info!("  - Metrics: GET http://{}/metrics", addr);
    info!("  - Metrics (JSON): GET http://{}/metrics/json", addr);
    info!("  - Router info: GET http://{}/router/info", addr);
    info!("  - Routes: GET http://{}/router/routes", addr);
    info!("  - Agents: GET http://{}/agents", addr);
    info!("  - Readiness: GET http://{}/ready", addr);

    server.run_with_shutdown().await?;

    Ok(())
}
