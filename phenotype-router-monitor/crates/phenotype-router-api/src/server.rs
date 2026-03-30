//! API server initialization and routing

use crate::error::Result;
use crate::handlers;
use crate::state::AppState;
use axum::{
    routing::{get, post},
    Router as AxumRouter,
};
use phenotype_router_core::Router;
use phenotype_router_metrics::MetricsCollector;
use std::net::SocketAddr;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tracing::info;

/// REST API server
pub struct ApiServer {
    app: AxumRouter,
    addr: SocketAddr,
}

impl ApiServer {
    /// Create a new API server
    pub fn new(router: Router, metrics: MetricsCollector) -> Result<Self> {
        let state = AppState::new(router, metrics);
        let addr = state.router.socket_addr().parse::<SocketAddr>()?;

        let app = AxumRouter::new()
            // Health and readiness checks
            .route("/health", get(handlers::get_health))
            .route("/ready", get(handlers::get_ready))
            // Metrics endpoints
            .route("/metrics", get(handlers::get_metrics))
            .route("/metrics/json", get(handlers::get_metrics_json))
            // Router information
            .route("/router/info", get(handlers::get_router_info))
            .route("/router/routes", get(handlers::get_router_routes))
            // Agent management
            .route("/agents", get(handlers::get_agents))
            .route("/agents/refresh", post(handlers::post_agents_refresh))
            // Layers
            .layer(CompressionLayer::new())
            .layer(CorsLayer::permissive())
            .layer(
                tower::ServiceBuilder::new()
                    .layer(
                        tower_http::trace::TraceLayer::new_for_http()
                            .make_span_with(tower_http::trace::DefaultMakeSpan::new().include_headers(true)),
                    ),
            )
            .with_state(state);

        Ok(Self { app, addr })
    }

    /// Get the socket address the server will listen on
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Start the server
    pub async fn run(self) -> Result<()> {
        let listener = tokio::net::TcpListener::bind(self.addr).await?;
        info!("API server listening on {}", self.addr);

        axum::serve(listener, self.app)
            .await
            .map_err(|e| crate::error::ApiError::InternalError(e.to_string()))?;

        Ok(())
    }

    /// Start the server with graceful shutdown on signal
    pub async fn run_with_shutdown(self) -> Result<()> {
        let listener = tokio::net::TcpListener::bind(self.addr).await?;
        info!("API server listening on {} (with graceful shutdown)", self.addr);

        let (tx, rx) = tokio::sync::oneshot::channel();

        // Spawn signal handler
        tokio::spawn(async move {
            let _ = tokio::signal::ctrl_c().await;
            let _ = tx.send(());
        });

        axum::serve(listener, self.app)
            .with_graceful_shutdown(async {
                let _ = rx.await;
            })
            .await
            .map_err(|e| crate::error::ApiError::InternalError(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-ROUTER-016 (API server)
    #[test]
    fn test_api_server_creation() {
        let toml = r#"
listen_addr = "127.0.0.1"
listen_port = 3030

[[routes]]
service = "api"
path_pattern = "^/api/.*"
backends = ["http://localhost:3000"]
timeout_ms = 30000
strategy = "roundrobin"
"#;
        let router = Router::from_string(toml).unwrap();
        let metrics = MetricsCollector::new();
        let server = ApiServer::new(router, metrics);

        assert!(server.is_ok());
        let s = server.unwrap();
        assert_eq!(s.addr.port(), 3030);
    }

    // Traces to: FR-ROUTER-016
    #[test]
    fn test_api_server_addr() {
        let toml = r#"
listen_addr = "0.0.0.0"
listen_port = 8080

[[routes]]
service = "api"
path_pattern = "^/api/.*"
backends = ["http://localhost:3000"]
timeout_ms = 30000
strategy = "roundrobin"
"#;
        let router = Router::from_string(toml).unwrap();
        let metrics = MetricsCollector::new();
        let server = ApiServer::new(router, metrics).unwrap();

        assert_eq!(server.addr.port(), 8080);
    }
}
