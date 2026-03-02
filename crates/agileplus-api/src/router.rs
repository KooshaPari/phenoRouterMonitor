//! axum router factory and HTTP server startup.
//!
//! Traceability: WP15-T085

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Json;
use axum::routing::get;
use axum::{Router, middleware};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use agileplus_domain::credentials::CredentialStore;
use agileplus_domain::ports::{observability::ObservabilityPort, storage::StoragePort, vcs::VcsPort};

type BoxError = Box<dyn std::error::Error + Send + Sync>;

use crate::responses::HealthResponse;
use crate::routes::{audit, features, governance, work_packages};
use crate::state::AppState;

/// Build the axum [`Router`] with all routes, middleware, and shared state.
///
/// Route groups:
/// - `/health`, `/info`          — public, no auth required
/// - `/api/v1/features`          — feature CRUD
/// - `/api/v1/work-packages`     — work package access
/// - `/api/v1/features/:slug/*`  — governance, audit (nested)
pub fn create_router<S, V, O>(state: AppState<S, V, O>) -> Router
where
    S: StoragePort + Send + Sync + Clone + 'static,
    V: VcsPort + Send + Sync + Clone + 'static,
    O: ObservabilityPort + Send + Sync + Clone + 'static,
{
    let creds: Arc<dyn CredentialStore> = state.credentials.clone();

    // Public routes — no auth middleware.
    let public = Router::new()
        .route("/health", get(health_handler))
        .route("/info", get(info_handler));

    // Protected routes — all require a valid X-API-Key header.
    let protected = Router::new()
        .nest("/api/v1/features", features::routes::<S, V, O>())
        .nest("/api/v1/work-packages", work_packages::routes::<S, V, O>())
        // Governance and audit are nested under /api/v1/features/:slug
        .nest("/api/v1/features", governance::routes::<S, V, O>())
        .nest("/api/v1/features", audit::routes::<S, V, O>())
        .layer(middleware::from_fn_with_state(
            creds,
            crate::middleware::auth::validate_api_key,
        ))
        .with_state(state);

    Router::new()
        .merge(public)
        .merge(protected)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse::ok())
}

async fn info_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "name": "agileplus-api",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// Start the HTTP API server, binding to `addr`.
///
/// This function runs until the server shuts down. It is intended to be
/// spawned alongside the gRPC server via `tokio::spawn` or `tokio::select!`.
pub async fn start_api<S, V, O>(addr: SocketAddr, state: AppState<S, V, O>) -> Result<(), BoxError>
where
    S: StoragePort + Send + Sync + Clone + 'static,
    V: VcsPort + Send + Sync + Clone + 'static,
    O: ObservabilityPort + Send + Sync + Clone + 'static,
{
    let app = create_router(state);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("HTTP API listening on {addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
