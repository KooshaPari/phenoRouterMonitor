use axum::{
    extract::{Path, State},
    response::Html,
    routing::get,
    Router,
};
use std::sync::Arc;

use crate::health_scanner::HealthScanner;

#[derive(Clone)]
pub struct AppState {
    pub scanner: Arc<tokio::sync::RwLock<HealthScanner>>,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health/projects", get(list_projects))
        .route("/health/projects/:name", get(get_project))
        .route("/health/summary", get(health_summary))
        .route("/health/scan", get(trigger_scan))
        .with_state(state)
}

fn page(title: &str, content: &str) -> Html<String> {
    Html(format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>{}</title>
    <style>
        body {{ font-family: system-ui, sans-serif; max-width: 1200px; margin: 0 auto; padding: 20px; }}
        h1 {{ color: #333; }}
        .stats-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin: 20px 0; }}
        .stat-card {{ background: #f5f5f5; padding: 20px; border-radius: 8px; text-align: center; }}
        .stat-value {{ font-size: 2em; font-weight: bold; color: #2563eb; }}
        .stat-label {{ color: #666; margin-top: 5px; }}
        .health-table {{ width: 100%; border-collapse: collapse; margin: 20px 0; }}
        .health-table th, .health-table td {{ padding: 12px; text-align: left; border-bottom: 1px solid #ddd; }}
        .health-table th {{ background: #f5f5f5; font-weight: bold; }}
        .band-excellent {{ color: #16a34a; font-weight: bold; }}
        .band-good {{ color: #3b82f6; font-weight: bold; }}
        .band-fair {{ color: #eab308; font-weight: bold; }}
        .band-poor {{ color: #f97316; font-weight: bold; }}
        .band-critical {{ color: #dc2626; font-weight: bold; }}
        .last-scan {{ color: #666; font-style: italic; }}
        .error {{ color: #dc2626; padding: 20px; background: #fee2e2; border-radius: 8px; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    {}
</body>
</html>"#,
        title, title, content
    ))
}

async fn list_projects(State(state): State<AppState>) -> Html<String> {
    let scanner = state.scanner.read().await;
    let summary = scanner.health_summary();
    
    let mut table_rows = String::new();
    for (name, health) in scanner.projects() {
        let band_class = format!("band-{}", format!("{:?}", health.band).to_lowercase());
        table_rows.push_str(&format!(
            r#"<tr><td>{}</td><td>{:.0}%</td><td class="{}">{:?}</td><td>{} dimensions</td></tr>"#,
            name, health.overall_score, band_class, health.band, health.dimensions.len()
        ));
    }
    
    let content = format!(
        r#"
        <div class="stats-grid">
            <div class="stat-card"><div class="stat-value">{}</div><div class="stat-label">Total Projects</div></div>
            <div class="stat-card"><div class="stat-value">{:.1}</div><div class="stat-label">Average Score</div></div>
        </div>
        <table class="health-table">
            <thead><tr><th>Project</th><th>Score</th><th>Band</th><th>Status</th></tr></thead>
            <tbody>{}</tbody>
        </table>
        {}
        "#,
        summary.total_projects,
        summary.average_score,
        table_rows,
        summary.last_scan.map(|t| format!(r#"<p class="last-scan">Last scan: {}</p>"#, t.format("%Y-%m-%d %H:%M:%S UTC"))).unwrap_or_default()
    );
    
    page("Project Health Dashboard", &content)
}

async fn get_project(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> Html<String> {
    let scanner = state.scanner.read().await;
    
    match scanner.get_project(&name) {
        Some(health) => {
            let band_class = format!("band-{}", format!("{:?}", health.band).to_lowercase());
            let mut dimensions = String::new();
            for dim in &health.dimensions {
                dimensions.push_str(&format!(
                    r#"<li><strong>{}:</strong> {:.0}% (weight: {}) - {} findings</li>"#,
                    dim.dimension, dim.score, dim.weight, dim.findings.len()
                ));
            }
            
            let content = format!(
                r#"
                <div style="text-align: center; padding: 30px; background: #f5f5f5; border-radius: 12px; margin: 20px 0;">
                    <div style="font-size: 4em; font-weight: bold; {}">{:.0}%</div>
                    <div style="font-size: 1.5em; margin-top: 10px;" class="{}">{:?}</div>
                </div>
                <h2>Dimensions</h2>
                <ul>{}</ul>
                <p>{} total findings</p>
                <p><a href="/health/projects">← Back to all projects</a></p>
                "#,
                band_class, health.overall_score, band_class, health.band,
                dimensions, health.findings_count
            );
            page(&format!("Health: {}", health.repo_name), &content)
        }
        None => {
            let content = format!(
                r#"<div class="error"><h2>Project Not Found</h2><p>Project '{}' was not found.</p><p><a href="/health/projects">← Back to all projects</a></p></div>"#,
                name
            );
            page("Error", &content)
        }
    }
}

async fn health_summary(State(state): State<AppState>) -> Html<String> {
    let scanner = state.scanner.read().await;
    let summary = scanner.health_summary();
    
    let get_count = |band: phenotype_health::HealthBand| {
        summary.by_band.get(&band).copied().unwrap_or(0)
    };
    
    let content = format!(
        r#"
        <div class="stats-grid">
            <div class="stat-card band-excellent"><div class="stat-value">{}</div><div class="stat-label">Excellent</div></div>
            <div class="stat-card band-good"><div class="stat-value">{}</div><div class="stat-label">Good</div></div>
            <div class="stat-card band-fair"><div class="stat-value">{}</div><div class="stat-label">Fair</div></div>
            <div class="stat-card band-poor"><div class="stat-value">{}</div><div class="stat-label">Poor</div></div>
            <div class="stat-card band-critical"><div class="stat-value">{}</div><div class="stat-label">Critical</div></div>
        </div>
        <div class="stat-card" style="margin-top: 20px;">
            <div class="stat-value">{:.1}%</div>
            <div class="stat-label">Average Score</div>
        </div>
        {}
        <p style="margin-top: 20px;"><a href="/health/projects">View all projects</a> | <a href="/health/scan">Run new scan</a></p>
        "#,
        get_count(phenotype_health::HealthBand::Excellent),
        get_count(phenotype_health::HealthBand::Good),
        get_count(phenotype_health::HealthBand::Fair),
        get_count(phenotype_health::HealthBand::Poor),
        get_count(phenotype_health::HealthBand::Critical),
        summary.average_score,
        summary.last_scan.map(|t| format!(r#"<p class="last-scan">Last scan: {}</p>"#, t.format("%Y-%m-%d %H:%M:%S UTC"))).unwrap_or_default()
    );
    
    page("Health Summary", &content)
}

async fn trigger_scan(State(state): State<AppState>) -> Html<String> {
    let mut scanner = state.scanner.write().await;
    
    match scanner.scan_all().await {
        Ok(results) => {
            let mut rows = String::new();
            for health in &results {
                let band_class = format!("band-{}", format!("{:?}", health.band).to_lowercase());
                rows.push_str(&format!(
                    r#"<tr><td class="{}">{:?}</td><td>{}</td><td>{:.0}%</td></tr>"#,
                    band_class, health.band, health.repo_name, health.overall_score
                ));
            }
            
            let content = format!(
                r#"
                <p style="color: #16a34a; font-size: 1.2em;">✓ Scan Complete - Scanned {} projects</p>
                <table class="health-table">
                    <thead><tr><th>Band</th><th>Project</th><th>Score</th></tr></thead>
                    <tbody>{}</tbody>
                </table>
                <p><a href="/health/summary">View Summary</a> | <a href="/health/projects">View All Projects</a></p>
                "#,
                results.len(), rows
            );
            page("Scan Complete", &content)
        }
        Err(e) => {
            let content = format!(
                r#"<div class="error"><h2>Scan Failed</h2><p>{}</p><p><a href="/health/projects">← Back</a></p></div>"#,
                e
            );
            page("Error", &content)
        }
    }
}
