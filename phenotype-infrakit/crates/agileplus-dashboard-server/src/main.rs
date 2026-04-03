//! Dashboard Server - Simplified version using phenotype crates directly

use std::path::Path;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3001);

    let root = std::env::var("ROOT")
        .unwrap_or_else(|_| "/Users/kooshapari/CodeProjects/Phenotype/repos".to_string());

    tracing::info!("Starting Health Dashboard on port {}", port);
    tracing::info!("Scanning projects in: {}", root);

    // Run initial scan
    let projects = phenotype_project_registry::discover_projects(Path::new(&root));
    tracing::info!("Found {} projects", projects.len());

    let scanner = phenotype_compliance_scanner::DocumentationScanner::new();
    let mut health_results = Vec::new();

    for project in &projects {
        let compliance = scanner.scan(&project.path);
        let score = compliance.score;
        let doc_dimension = phenotype_health::DimensionScore::new("documentation", 15.0, score);
        
        let mut health = phenotype_health::ProjectHealth {
            repo_name: project.name.clone(),
            language: format!("{:?}", project.project_type),
            overall_score: 0.0,
            band: phenotype_health::HealthBand::Unknown,
            dimensions: vec![doc_dimension],
            findings_count: compliance.missing.len(),
        };
        health.compute_overall_score();
        
        tracing::info!("{}: {:.0}% ({:?})", project.name, health.overall_score, health.band);
        health_results.push(health);
    }

    // Create basic Axum app
    let app = axum::Router::new()
        .route("/health/projects", axum::routing::get(|| async move {
            axum::Json(serde_json::json!({
                "status": "ok",
                "projects": health_results.len(),
                "message": "Health dashboard active"
            }))
        }))
        .route("/health", axum::routing::get(|| async move {
            axum::Json(serde_json::json!({
                "status": "healthy",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }))
        }));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("Dashboard ready at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
