//! AgilePlus end-to-end integration test infrastructure.
//!
//! Traceability: WP19-T107, T108, T109, T110, T111
//!
//! # Usage
//!
//! Run unit-safe tests (no external services required):
//! ```text
//! cargo test -p agileplus-integration-tests
//! ```
//!
//! Run full integration tests (requires process-compose + all services):
//! ```text
//! cargo test -p agileplus-integration-tests --features integration -- --include-ignored
//! ```

pub mod common;
