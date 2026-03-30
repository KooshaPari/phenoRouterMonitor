//! AgilePlus HTTP API — axum router, middleware, and route handlers.
//!
//! Traceability: WP11-T064..T070

pub mod api_key;
pub mod error;
pub mod middleware;
pub mod responses;
pub mod router;
pub mod routes;
pub mod state;

pub use router::create_router;
pub use state::AppState;
