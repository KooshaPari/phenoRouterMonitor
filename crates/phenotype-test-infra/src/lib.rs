//! # Phenotype Test Infrastructure
//!
//! Common testing utilities for Phenotype crates.

use std::path::PathBuf;

/// Test helper for creating temp directories.
pub fn temp_dir() -> PathBuf {
    std::env::temp_dir().join(format!("phenotype-test-{}", uuid::Uuid::new_v4()))
}

/// Test helper for creating temp files.
pub fn temp_file(name: &str) -> PathBuf {
    temp_dir().join(name)
}
