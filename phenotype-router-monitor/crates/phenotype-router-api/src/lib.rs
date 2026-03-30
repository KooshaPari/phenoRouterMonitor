//! REST API server for router monitoring
//!
//! Provides HTTP endpoints for:
//! - Metrics collection (/metrics)
//! - Agent management (/agents)
//! - Configuration reload (/config)
//! - Health checks (/health)

pub mod error;
pub mod handlers;
pub mod server;
pub mod state;

pub use error::{ApiError, Result};
pub use server::ApiServer;
pub use state::AppState;

#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-ROUTER-012 (API server)
    #[test]
    fn test_api_error_display() {
        let err = ApiError::NotFound("route".to_string());
        assert!(err.to_string().contains("route"));
    }
}
