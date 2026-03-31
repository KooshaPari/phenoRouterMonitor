//! Test fixtures module — re-export from agileplus-fixtures.
//!
//! This module re-exports all fixture functionality from the shared agileplus-fixtures
//! crate. The actual implementations have been extracted to enable reuse across multiple
//! test suites.
//!
//! Traceability: WP19-T107

pub use agileplus_fixtures::{
    TestFixtures, seed_test_data,
    feature_create_payload, transition_payload, plane_webhook_payload,
    FeatureBuilder, WorkPackageBuilder,
};
