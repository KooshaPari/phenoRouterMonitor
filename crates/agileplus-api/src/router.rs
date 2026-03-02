//! axum router factory and HTTP server startup.
//!
//! Route layout:
//!
//! Public (no auth):
//!   GET  /health    — detailed health check (T070)
//!   GET  /info      — API metadata
//!
//! Protected (X-API-Key header or ?api_key= param):
//!   GET  /api/v1/features                           — list features (T066)
//!   POST /api/v1/features                           — create feature (T066)
//!   GET  /api/v1/features/:slug                     — get feature (T066)
//!   PATCH /api/v1/features/:slug                    — update feature (T066)
//!   POST /api/v1/features/:slug/transition          — transition feature state (T066)
//!   GET  /api/v1/features/:slug/work-packages       — list WPs (T067)
//!   POST /api/v1/features/:slug/work-packages       — create WP (T067)
//!   GET  /api/v1/work-packages/:id                  — get WP (T067)
//!   PATCH /api/v1/work-packages/:id                 — update WP (T067)
//!   POST /api/v1/work-packages/:id/transition       — transition WP state (T067)
//!   GET  /api/v1/features/:slug/audit               — audit trail
//!   POST /api/v1/features/:slug/audit/verify        — verify audit chain
//!   GET  /api/v1/features/:slug/governance          — governance contract
//!   POST /api/v1/features/:slug/validate            — run governance validation
//!   GET  /api/v1/events                             — query events (T068)
//!   GET  /api/v1/events/:id                         — single event (T068)
//!   GET  /api/v1/stream                             — SSE real-time events (T069)
//!
//! Traceability: WP11-T065..T070

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use axum::Json;
use axum::routing::get;
use axum::{Router, middleware};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use agileplus_domain::credentials::CredentialStore;
use agileplus_domain::ports::{observability::ObservabilityPort, storage::StoragePort, vcs::VcsPort};

type BoxError = Box<dyn std::error::Error + Send + Sync>;

use crate::responses::{DetailedHealthResponse, HealthResponse};
use crate::routes::{audit, events, features, governance, stream, work_packages};
use crate::state::AppState;

/// Build the axum [`Router`] with all routes, middleware, and shared state.
pub fn create_router<S, V, O>(state: AppState<S, V, O>) -> Router
where
    S: StoragePort + Send + Sync + Clone + 'static,
    V: VcsPort + Send + Sync + Clone + 'static,
    O: ObservabilityPort + Send + Sync + Clone + 'static,
{
    let creds: Arc<dyn CredentialStore> = state.credentials.clone();

    // Public routes — no auth middleware.
    let public = Router::new()
        .route("/health", get(health_handler::<S, V, O>))
        .route("/info", get(info_handler))
        .with_state(state.clone());

    // Protected routes — all require a valid API key.
    let protected = Router::new()
        // Feature CRUD + transitions
        .nest("/api/v1/features", features::routes::<S, V, O>())
        // Work-package CRUD + transitions
        .nest("/api/v1/work-packages", work_packages::routes::<S, V, O>())
        // Work-package routes nested under features
        .nest("/api/v1/features", work_packages::feature_wp_routes::<S, V, O>())
        // Governance and audit nested under features
        .nest("/api/v1/features", governance::routes::<S, V, O>())
        .nest("/api/v1/features", audit::routes::<S, V, O>())
        // Event query endpoints
        .nest("/api/v1/events", events::routes::<S, V, O>())
        // SSE streaming
        .route("/api/v1/stream", get(stream::stream_events::<S, V, O>))
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

/// `GET /health` — aggregated health check, no auth required (T070).
async fn health_handler<S, V, O>(
    axum::extract::State(app): axum::extract::State<AppState<S, V, O>>,
) -> Json<DetailedHealthResponse>
where
    S: StoragePort + Send + Sync + Clone + 'static,
    V: VcsPort + Send + Sync + Clone + 'static,
    O: ObservabilityPort + Send + Sync + Clone + 'static,
{
    use std::collections::HashMap;

    // Probe storage with a lightweight call.
    let mut services: HashMap<String, crate::responses::ServiceHealth> = HashMap::new();

    let t0 = Instant::now();
    let sqlite_health = match app.storage.list_all_features().await {
        Ok(_) => crate::responses::ServiceHealth::healthy(t0.elapsed().as_millis() as u64),
        Err(e) => crate::responses::ServiceHealth::unavailable(e.to_string()),
    };
    services.insert("sqlite".to_string(), sqlite_health);

    // For services not yet wired (NATS, Dragonfly, Neo4j, MinIO), report
    // them as degraded with an explanatory note rather than unavailable.
    for name in &["nats", "dragonfly", "neo4j", "minio"] {
        services.insert(
            name.to_string(),
            crate::responses::ServiceHealth::degraded("not configured in this deployment"),
        );
    }

    let overall = DetailedHealthResponse::compute_status(&services).to_string();

    Json(DetailedHealthResponse {
        status: overall,
        timestamp: chrono::Utc::now().to_rfc3339(),
        services,
        api: crate::responses::ApiHealth {
            status: "healthy".to_string(),
            uptime_seconds: 0, // uptime tracking requires a startup timestamp in AppState
        },
    })
}

async fn info_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "name": "agileplus-api",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// Start the HTTP API server, binding to `addr`.
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
