//! Core routing engine for phenotype-router-monitor
//!
//! This crate provides:
//! - Path-based routing with regex and wildcard patterns
//! - Round-robin load balancing across backend pools
//! - Request forwarding with full header/query preservation
//! - TOML configuration schema and validation

pub mod backend;
pub mod error;
pub mod loader;
pub mod patterns;
pub mod router;

pub use backend::{BackendAddress, BackendPool, LoadBalancingStrategy};
pub use error::{RouterError, Result};
pub use loader::ConfigLoader;
pub use patterns::PathPattern;
pub use router::Router;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        let router = Router::new();
        assert_eq!(router.routes_count(), 0);
    }
}
