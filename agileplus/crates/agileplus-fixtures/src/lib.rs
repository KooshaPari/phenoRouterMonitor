//! Shared test fixtures and data builders for AgilePlus.
//!
//! This crate provides deterministic, reusable test fixtures for use across
//! integration tests, unit tests, and dashboard seeding. All fixtures are
//! built without database I/O and can be composed using builder patterns.
//!
//! # Features
//!
//! - `TestFixtures`: Pre-built feature and work package test data
//! - Builder patterns for creating features and work packages
//! - Dogfood seed data for dashboard initialization
//! - API payload builders for testing HTTP handlers
//!
//! # Traceability
//!
//! Phase 1 LOC Reduction: Fixtures extraction from seed.rs and fixtures.rs

pub mod builders;
pub mod dogfood;
pub mod payloads;
pub mod test_fixtures;

pub use test_fixtures::{TestFixtures, seed_test_data};
pub use payloads::{feature_create_payload, transition_payload, plane_webhook_payload};
pub use dogfood::seed_dogfood_features;
pub use builders::{FeatureBuilder, WorkPackageBuilder};
