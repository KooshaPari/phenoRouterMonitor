//! Re-export seed functionality from agileplus-fixtures.
//!
//! This module re-exports seed data functions from the agileplus-fixtures crate.
//! The actual implementation has been extracted to agileplus-fixtures for reuse
//! across integration tests.
//!
//! Traceability: WP12 (Phase 1 LOC Reduction)

pub use agileplus_fixtures::seed_dogfood_features;
