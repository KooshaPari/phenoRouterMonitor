//! Test fixtures and utilities for Phenotype crates.
//!
//! Provides reusable test infrastructure including mock servers, temporary directories,
//! and canned response handlers.

use std::path::PathBuf;
use tempfile::TempDir;

/// Temporary directory fixture that auto-cleans on drop.
pub struct TempDirFixture {
    _temp: TempDir,
    path: PathBuf,
}

impl TempDirFixture {
    /// Creates a new temporary directory.
    pub fn new() -> std::io::Result<Self> {
        let temp = TempDir::new()?;
        let path = temp.path().to_path_buf();
        Ok(Self { _temp: temp, path })
    }

    /// Returns the path to the temporary directory.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Creates a file in the temporary directory.
    pub fn create_file(&self, name: &str, contents: &str) -> std::io::Result<PathBuf> {
        let path = self.path.join(name);
        std::fs::write(&path, contents)?;
        Ok(path)
    }
}

impl Default for TempDirFixture {
    fn default() -> Self {
        Self::new().expect("failed to create temp directory")
    }
}

/// Mock HTTP server for testing.
pub struct MockServer {
    _temp: TempDir,
}

impl MockServer {
    /// Creates a new mock server.
    pub fn new() -> std::io::Result<Self> {
        let temp = TempDir::new()?;
        Ok(Self { _temp: temp })
    }

    /// Returns the server URL.
    pub fn url(&self) -> String {
        "http://127.0.0.1:0".to_string()
    }
}

impl Default for MockServer {
    fn default() -> Self {
        Self::new().expect("failed to create mock server")
    }
}
