//! Configuration management with hot reload support
//!
//! Provides:
//! - Configuration loading from files
//! - Hot reload detection
//! - Configuration validation
//! - Watch-based file monitoring

pub mod error;
pub mod file_watcher;
pub mod manager;

pub use error::{ConfigError, Result};
pub use file_watcher::FileWatcher;
pub use manager::ConfigManager;

#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-ROUTER-017 (Configuration management)
    #[test]
    fn test_config_error_display() {
        let err = ConfigError::LoadError("file not found".to_string());
        assert!(err.to_string().contains("file not found"));
    }
}
