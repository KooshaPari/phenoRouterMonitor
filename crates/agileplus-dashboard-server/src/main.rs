//! Dashboard Server Binary

use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let args: Vec<String> = env::args().collect();
    let port = args.iter()
        .position(|a| a == "--port")
        .and_then(|i| args.get(i + 1))
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let root = args.iter()
        .position(|a| a == "--root")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_else(|| "/Users/kooshapari/CodeProjects/Phenotype/repos".to_string());

    tracing::info!("Starting dashboard server on port {}", port);
    tracing::info!("Scanning projects in: {}", root);

    let scanner = std::sync::Arc::new(tokio::sync::RwLock::new(
        agileplus_dashboard::HealthScanner::new(&root, 24)
    ));

    {
        tracing::info!("Running initial health scan...");
        let mut guard = scanner.write().await;
        match guard.scan_all().await {
            Ok(results) => tracing::info!("Initial scan complete: {} projects", results.len()),
            Err(e) => tracing::error!("Initial scan failed: {}", e),
        }
    }

    let state = agileplus_dashboard::AppState { scanner: scanner.clone() };
    let app = agileplus_dashboard::create_router(state);
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    tracing::info!("Dashboard ready at http://0.0.0.0:{}", port);
    tracing::info!("Routes:");
    tracing::info!("  GET /health/projects     - List all projects");
    tracing::info!("  GET /health/projects/:name - Project details");
    tracing::info!("  GET /health/summary      - Health summary");
    tracing::info!("  GET /health/scan         - Trigger scan");

    axum::serve(listener, app).await?;
    Ok(())
}
