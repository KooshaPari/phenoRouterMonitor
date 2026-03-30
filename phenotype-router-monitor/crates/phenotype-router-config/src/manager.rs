//! Configuration manager with hot reload
//!
//! Manages application configuration with support for dynamic reloading

use crate::error::{ConfigError, Result};
use crate::file_watcher::FileWatcher;
use parking_lot::RwLock;
use std::path::Path;
use std::sync::Arc;

/// Configuration manager
pub struct ConfigManager<T> {
    config: Arc<RwLock<T>>,
    path: String,
    on_change: Option<Arc<dyn Fn(&T) + Send + Sync>>,
}

impl<T> ConfigManager<T>
where
    T: serde::de::DeserializeOwned + Send + Sync + 'static,
{
    /// Create a new configuration manager from a file
    pub async fn from_file(path: impl AsRef<Path>, loader: impl Fn(&str) -> Result<T>) -> Result<Self> {
        let path_str = path.as_ref().display().to_string();
        let content = std::fs::read_to_string(&path_str)
            .map_err(|e| ConfigError::LoadError(format!("Failed to read {}: {}", path_str, e)))?;

        let config = loader(&content)?;

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            path: path_str,
            on_change: None,
        })
    }

    /// Set a callback to be called when configuration changes
    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        self.on_change = Some(Arc::new(callback));
        self
    }

    /// Get current configuration
    pub fn get<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        f(&*self.config.read())
    }

    /// Reload configuration from file
    pub async fn reload(&self, loader: impl Fn(&str) -> Result<T>) -> Result<()> {
        let content =
            std::fs::read_to_string(&self.path)
                .map_err(|e| ConfigError::LoadError(format!(
                    "Failed to read {}: {}",
                    self.path, e
                )))?;

        let new_config = loader(&content)?;
        {
            let mut config = self.config.write();
            *config = new_config;
        }

        if let Some(callback) = &self.on_change {
            callback(&*self.config.read());
        }

        Ok(())
    }

    /// Start watching for configuration changes
    pub async fn watch<F>(&self, loader: F) -> Result<()>
    where
        F: Fn(&str) -> Result<T> + Send + Sync + 'static,
    {
        let watcher = FileWatcher::new(&self.path);
        let mut rx = watcher.watch().await?;

        let config = self.config.clone();
        let on_change = self.on_change.clone();
        let path = self.path.clone();

        tokio::spawn(async move {
            while let Some(_event) = rx.recv().await {
                match std::fs::read_to_string(&path) {
                    Ok(content) => match loader(&content) {
                        Ok(new_config) => {
                            let mut cfg = config.write();
                            *cfg = new_config;
                            drop(cfg); // Release lock

                            if let Some(callback) = &on_change {
                                callback(&*config.read());
                            }
                        }
                        Err(_e) => {
                            // Log error but continue watching
                        }
                    },
                    Err(_e) => {
                        // Log error but continue watching
                    }
                }
            }
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestConfig {
        name: String,
        timeout: u64,
    }

    // Traces to: FR-ROUTER-020 (Configuration management)
    #[tokio::test]
    async fn test_config_manager_creation() {
        let toml_content = r#"
name = "test"
timeout = 30
"#;

        // Create a temporary file
        let temp_path = "/tmp/test_config.toml";
        std::fs::write(temp_path, toml_content).unwrap();

        let manager = ConfigManager::from_file(temp_path, |content| {
            toml::from_str(content)
                .map_err(|e| ConfigError::TomlError(e))
        })
        .await;

        assert!(manager.is_ok());
        let mgr = manager.unwrap();

        mgr.get(|config: &TestConfig| {
            assert_eq!(config.name, "test");
            assert_eq!(config.timeout, 30);
        });

        std::fs::remove_file(temp_path).ok();
    }

    // Traces to: FR-ROUTER-020
    #[tokio::test]
    async fn test_config_reload() {
        let toml_content1 = r#"
name = "test1"
timeout = 30
"#;

        let toml_content2 = r#"
name = "test2"
timeout = 60
"#;

        // Create a temporary file
        let temp_path = "/tmp/test_config_reload.toml";
        std::fs::write(temp_path, toml_content1).unwrap();

        let manager = ConfigManager::from_file(temp_path, |content| {
            toml::from_str(content)
                .map_err(|e| ConfigError::TomlError(e))
        })
        .await
        .unwrap();

        // Update file
        std::fs::write(temp_path, toml_content2).unwrap();

        // Reload
        let result = manager.reload(|content| {
            toml::from_str(content)
                .map_err(|e| ConfigError::TomlError(e))
        }).await;

        assert!(result.is_ok());

        manager.get(|config: &TestConfig| {
            assert_eq!(config.name, "test2");
            assert_eq!(config.timeout, 60);
        });

        std::fs::remove_file(temp_path).ok();
    }

    // Traces to: FR-ROUTER-020
    #[tokio::test]
    async fn test_config_manager_on_change() {
        let toml_content = r#"
name = "test"
timeout = 30
"#;

        let temp_path = "/tmp/test_config_change.toml";
        std::fs::write(temp_path, toml_content).unwrap();

        let change_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let change_count_clone = change_count.clone();

        let manager = ConfigManager::from_file(temp_path, |content| {
            toml::from_str(content)
                .map_err(|e| ConfigError::TomlError(e))
        })
        .await
        .unwrap()
        .on_change(move |_config: &TestConfig| {
            change_count_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        });

        // Trigger change
        std::fs::write(temp_path, r#"
name = "test2"
timeout = 60
"#).unwrap();

        let result = manager.reload(|content| {
            toml::from_str(content)
                .map_err(|e| ConfigError::TomlError(e))
        }).await;

        assert!(result.is_ok());
        assert_eq!(change_count.load(std::sync::atomic::Ordering::Relaxed), 1);

        std::fs::remove_file(temp_path).ok();
    }
}
