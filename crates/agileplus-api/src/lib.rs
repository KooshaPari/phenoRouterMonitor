//! AgilePlus HTTP API — axum router, middleware, and route handlers.
//!
//! Traceability: WP15-T085, T086, T087, T090

pub mod error;
pub mod middleware;
pub mod responses;
pub mod router;
pub mod routes;
pub mod state;

pub use router::create_router;
pub use state::AppState;
