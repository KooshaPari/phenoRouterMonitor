use std::{net::SocketAddr, sync::Arc};

use agileplus_dashboard::{app_state::DashboardStore, routes};
use axum::Router;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();

    let port = std::env::var("AGILEPLUS_DASHBOARD_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3000);
    let state = Arc::new(tokio::sync::RwLock::new(DashboardStore::seeded()));

    let app: Router = routes::router(state)
        .nest_service("/static", ServeDir::new("templates/static"))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr).await?;
    info!(
        "agileplus-dashboard listening on http://{}",
        listener.local_addr()?
    );
    axum::serve(listener, app).await?;
    Ok(())
}
