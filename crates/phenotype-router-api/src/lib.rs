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
//! async fn main() {
//!     let config = RouterConfig::default();
//!     let server = RouterApiServer::new(config);
//!     let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
//!     let _ = server.run(addr).await;
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
    use std::sync::Arc;

    #[tokio::test]
    async fn test_router_server_creation() {
        let config = RouterConfig::default();
        let server = RouterApiServer::new(config);
        assert!(!server.config().id.is_empty());
    }

    #[tokio::test]
    async fn test_router_server_has_state() {
        let config = RouterConfig::default();
        let server = RouterApiServer::new(config);
        let state = server.state();
        assert_eq!(state.agents().len(), 0);
    }

    #[tokio::test]
    async fn test_router_creates_router_app() {
        let config = RouterConfig::default();
        let state = Arc::new(RouterState::new(config));
        let _app = create_router(state);
        // Router app created successfully
        assert!(true);
    }

    #[tokio::test]
    async fn test_server_uptime() {
        let config = RouterConfig::default();
        let server = RouterApiServer::new(config);
        let uptime = server.state().uptime_secs();
        assert_eq!(uptime, 0);
    }

    #[tokio::test]
    async fn test_server_with_agents() {
        let config = RouterConfig::default();
        let server = RouterApiServer::new(config);

        let agent = Agent::new("test-agent", "Test Agent");
        let _ = server.state().add_agent(agent);

        assert_eq!(server.state().agents().len(), 1);
    }

    #[tokio::test]
    async fn test_server_metrics_collection() {
        let config = RouterConfig::default();
        let server = RouterApiServer::new(config);

        server.state().record_request("/health", "GET", 200);
        server.state().record_request("/health", "GET", 200);
        server.state().record_error("test_error");

        let metrics = server.state().metrics();
        assert_eq!(metrics.total_requests, 2);
        assert_eq!(metrics.total_errors, 1);
    }

    #[tokio::test]
    async fn test_server_config_builder() {
        let config = RouterConfig::default()
            .with_id("test-router")
            .with_environment("test")
            .with_max_agents(50);

        let server = RouterApiServer::new(config);
        assert_eq!(server.config().id, "test-router");
        assert_eq!(server.config().environment, "test");
        assert_eq!(server.config().max_agents, 50);
    }
}
