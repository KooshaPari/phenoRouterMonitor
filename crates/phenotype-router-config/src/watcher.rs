//! File watcher for configuration hot reload detection

use crate::config::RouterConfig;
use crate::error::Result;
use crate::loader::ConfigLoader;
use async_trait::async_trait;
use notify::{Watcher, RecursiveMode};
use notify::recommended_watcher;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Callback for configuration changes
#[async_trait]
pub trait ConfigChangeCallback: Send + Sync {
    /// Called when configuration changes are detected
    async fn on_config_change(&self, old: &RouterConfig, new: &RouterConfig) -> Result<()>;
}

/// Configuration change event
#[derive(Debug, Clone)]
pub struct ConfigChangeEvent {
    /// Old configuration
    pub old: RouterConfig,
    /// New configuration
    pub new: RouterConfig,
}

/// Configuration file watcher
pub struct ConfigWatcher {
    config_path: PathBuf,
    current_config: Arc<tokio::sync::Mutex<RouterConfig>>,
    callbacks: Arc<tokio::sync::Mutex<Vec<Arc<dyn ConfigChangeCallback>>>>,
}

impl ConfigWatcher {
    /// Create a new configuration watcher
    pub fn new(config_path: PathBuf, initial_config: RouterConfig) -> Self {
        Self {
            config_path,
            current_config: Arc::new(tokio::sync::Mutex::new(initial_config)),
            callbacks: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }

    /// Register a callback for configuration changes
    pub async fn register_callback(&self, callback: Arc<dyn ConfigChangeCallback>) {
        let mut callbacks = self.callbacks.lock().await;
        callbacks.push(callback);
        debug!("Registered configuration change callback");
    }

    /// Start watching the configuration file
    pub async fn start_watching(&self) -> Result<()> {
        let config_path = self.config_path.clone();
        let current_config = Arc::clone(&self.current_config);
        let callbacks = Arc::clone(&self.callbacks);

        let handle = std::thread::spawn(move || {
            let (tx, rx) = std::sync::mpsc::channel();

            let mut watcher = match recommended_watcher(move |res| {
                let _ = tx.send(res);
            }) {
                Ok(w) => w,
                Err(e) => {
                    warn!("Failed to create watcher: {}", e);
                    return;
                }
            };

            if let Err(e) = watcher.watch(&config_path, RecursiveMode::NonRecursive) {
                warn!("Failed to watch config file: {}", e);
                return;
            }

            info!("Started watching configuration file: {:?}", config_path);

            for _event in rx.iter() {
                debug!("Configuration file change detected");

                // Attempt to reload the configuration
                match std::fs::read_to_string(&config_path) {
                    Ok(content) => {
                        match toml::from_str::<RouterConfig>(&content) {
                            Ok(new_config) => {
                                if new_config.validate().is_ok() {
                                    let rt = tokio::runtime::Handle::current();
                                    let config_clone = current_config.clone();
                                    let callbacks_clone = callbacks.clone();

                                    rt.block_on(async {
                                        let mut current = config_clone.lock().await;
                                        let old_config = current.clone();

                                        // Call all registered callbacks
                                        let cbs = callbacks_clone.lock().await;
                                        for callback in cbs.iter() {
                                            match callback.on_config_change(&old_config, &new_config).await {
                                                Ok(()) => {
                                                    debug!("Callback executed successfully");
                                                }
                                                Err(e) => {
                                                    warn!("Callback execution failed: {:?}", e);
                                                }
                                            }
                                        }

                                        *current = new_config;
                                        info!("Configuration reloaded successfully");
                                    });
                                } else {
                                    warn!("New configuration failed validation");
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse configuration: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to read configuration file: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    /// Get the current configuration
    pub async fn get_current_config(&self) -> RouterConfig {
        self.current_config.lock().await.clone()
    }

    /// Update configuration manually (for testing)
    pub async fn update_config(&self, new_config: RouterConfig) {
        let mut current = self.current_config.lock().await;
        *current = new_config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestCallback {
        call_count: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl ConfigChangeCallback for TestCallback {
        async fn on_config_change(&self, _old: &RouterConfig, _new: &RouterConfig) -> Result<()> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    fn create_test_config() -> RouterConfig {
        use crate::config::{RouteConfig, ServerConfig};
        use std::collections::HashMap;

        RouterConfig {
            name: "test-router".to_string(),
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                timeout_secs: 30,
                max_connections: 1000,
            },
            routes: vec![RouteConfig {
                path: "/api/users".to_string(),
                method: "GET".to_string(),
                handler: "user_handler".to_string(),
                cors_enabled: false,
                timeout_secs: None,
                metadata: HashMap::new(),
            }],
            middleware: Default::default(),
            extra: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_watcher_creation() {
        let config = create_test_config();
        let watcher = ConfigWatcher::new(PathBuf::from("/tmp/test.toml"), config.clone());
        let current = watcher.get_current_config().await;
        assert_eq!(current.name, config.name);
    }

    #[tokio::test]
    async fn test_register_callback() {
        let config = create_test_config();
        let watcher = ConfigWatcher::new(PathBuf::from("/tmp/test.toml"), config);
        let callback = Arc::new(TestCallback {
            call_count: Arc::new(AtomicUsize::new(0)),
        });
        watcher.register_callback(callback).await;
        let callbacks = watcher.callbacks.lock().await;
        assert_eq!(callbacks.len(), 1);
    }

    #[tokio::test]
    async fn test_update_config_manual() {
        let config = create_test_config();
        let watcher = ConfigWatcher::new(PathBuf::from("/tmp/test.toml"), config);

        let mut new_config = create_test_config();
        new_config.name = "updated-router".to_string();

        watcher.update_config(new_config).await;
        let current = watcher.get_current_config().await;
        assert_eq!(current.name, "updated-router");
    }

    #[tokio::test]
    async fn test_multiple_callbacks() {
        let config = create_test_config();
        let watcher = ConfigWatcher::new(PathBuf::from("/tmp/test.toml"), config);

        let cb1 = Arc::new(TestCallback {
            call_count: Arc::new(AtomicUsize::new(0)),
        });
        let cb2 = Arc::new(TestCallback {
            call_count: Arc::new(AtomicUsize::new(0)),
        });

        watcher.register_callback(cb1).await;
        watcher.register_callback(cb2).await;

        let callbacks = watcher.callbacks.lock().await;
        assert_eq!(callbacks.len(), 2);
    }
}
