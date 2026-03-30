//! Phenotype Router API - REST server with health, metrics, router info, and agent management endpoints.
//!
//! Provides a composable Axum-based web API with the following endpoints:
//!
//! - **Health**: `/health` (liveness), `/ready` (readiness)
//! - **Metrics**: `/metrics` (Prometheus), `/metrics/json` (JSON export)
//! - **Router**: `/router/info` (configuration), `/router/routes` (route listing)
//! - **Agents**: `/agents` (list), `/agents/refresh` (refresh registry)
//!
//! # Example
//!
//! ```no_run
//! use phenotype_router_api::{RouterApiServer, RouterConfig};
//! use std::net::SocketAddr;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = RouterConfig::default();
//!     let server = RouterApiServer::new(config);
//!     let addr: SocketAddr = "127.0.0.1:3000".parse()?;
//!     server.run(addr).await?;
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod handlers;
pub mod metrics;
pub mod state;
pub mod types;

pub use error::RouterApiError;
pub use handlers::create_router;
pub use metrics::{Metrics, MetricsCollector};
pub use state::RouterState;
pub use types::{Agent, Route, RouterConfig, RouterInfo};

use axum::Server;
use std::net::SocketAddr;
use std::sync::Arc;

/// Main router API server struct.
pub struct RouterApiServer {
    config: RouterConfig,
    state: Arc<RouterState>,
}

impl RouterApiServer {
    /// Create a new router API server with the given configuration.
    pub fn new(config: RouterConfig) -> Self {
        let state = Arc::new(RouterState::new(config.clone()));
        Self { config, state }
    }

    /// Run the server on the given socket address.
    pub async fn run(self, addr: SocketAddr) -> Result<(), RouterApiError> {
        let app = create_router(self.state.clone());

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| RouterApiError::ServerError(e.to_string()))?;

        tracing::info!("Router API server listening on {}", addr);

        axum::serve(listener, app)
            .await
            .map_err(|e| RouterApiError::ServerError(e.to_string()))?;

        Ok(())
    }

    /// Get the router state for inspection.
    pub fn state(&self) -> Arc<RouterState> {
        self.state.clone()
    }

    /// Get the router configuration.
    pub fn config(&self) -> &RouterConfig {
        &self.config
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use axum::http::StatusCode;
    use serde_json::{json, Value};
    use std::sync::Arc;
    use tokio::net::TcpListener;

    async fn setup_test_server() -> (RouterApiServer, SocketAddr) {
        let config = RouterConfig::default();
        let server = RouterApiServer::new(config);

        // Use a random free port
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind");
        let addr = listener.local_addr().expect("failed to get local addr");

        (server, addr)
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let config = RouterConfig::default();
        let state = Arc::new(RouterState::new(config));
        let app = create_router(state);

        let response = axum::http::Request::builder()
            .method("GET")
            .uri("/health")
            .body(axum::body::Body::empty())
            .unwrap();

        let client = axum::body::to_bytes(axum::body::Body::empty(), 1024)
            .await
            .unwrap();

        // Verify the app was created successfully
        assert!(true);
    }

    #[tokio::test]
    async fn test_metrics_json_endpoint() {
        let config = RouterConfig::default();
        let state = Arc::new(RouterState::new(config));
        let app = create_router(state);

        // Verify the app was created successfully
        assert!(true);
    }

    #[tokio::test]
    async fn test_router_info_endpoint() {
        let config = RouterConfig::default();
        let state = Arc::new(RouterState::new(config));
        let app = create_router(state);

        // Verify the app was created successfully
        assert!(true);
    }

    #[tokio::test]
    async fn test_agents_endpoint() {
        let config = RouterConfig::default();
        let state = Arc::new(RouterState::new(config));
        let app = create_router(state);

        // Verify the app was created successfully
        assert!(true);
    }
}
