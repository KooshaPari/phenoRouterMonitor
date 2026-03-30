//! File system watcher for configuration hot reload
//!
//! Monitors configuration files and detects changes for automatic reload

use crate::error::{ConfigError, Result};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tokio::fs;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};

/// File change event
#[derive(Debug, Clone)]
pub struct FileChangeEvent {
    /// Path to the changed file
    pub path: PathBuf,
    /// Time of the change
    pub changed_at: SystemTime,
}

/// File watcher for configuration changes
pub struct FileWatcher {
    path: PathBuf,
    poll_interval: Duration,
}

impl FileWatcher {
    /// Create a new file watcher
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            poll_interval: Duration::from_millis(500),
        }
    }

    /// Set the poll interval
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    /// Start watching for changes and send events to channel
    pub async fn watch(self) -> Result<mpsc::Receiver<FileChangeEvent>> {
        let (tx, rx) = mpsc::channel(100);
        let path = self.path.clone();

        if !path.exists() {
            return Err(ConfigError::LoadError(format!(
                "File does not exist: {}",
                path.display()
            )));
        }

        let path = self.path;
        let interval_duration = self.poll_interval;

        tokio::spawn(async move {
            let mut last_modified: Option<SystemTime> = None;
            let mut ticker = interval(interval_duration);

            loop {
                ticker.tick().await;

                match fs::metadata(&path).await {
                    Ok(metadata) => {
                        if let Ok(modified) = metadata.modified() {
                            if let Some(last) = last_modified {
                                if modified > last {
                                    let _ = tx
                                        .send(FileChangeEvent {
                                            path: path.clone(),
                                            changed_at: modified,
                                        })
                                        .await;
                                }
                            }
                            last_modified = Some(modified);
                        }
                    }
                    Err(_) => {
                        // File was deleted or inaccessible
                        break;
                    }
                }
            }
        });

        Ok(rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-ROUTER-019 (File watching)
    #[test]
    fn test_file_watcher_creation() {
        let watcher = FileWatcher::new("./test.toml");
        assert_eq!(watcher.path, PathBuf::from("./test.toml"));
        assert_eq!(watcher.poll_interval, Duration::from_millis(500));
    }

    // Traces to: FR-ROUTER-019
    #[test]
    fn test_file_watcher_with_interval() {
        let watcher = FileWatcher::new("./test.toml")
            .with_interval(Duration::from_secs(1));
        assert_eq!(watcher.poll_interval, Duration::from_secs(1));
    }

    // Traces to: FR-ROUTER-019
    #[tokio::test]
    async fn test_file_watcher_missing_file() {
        let watcher = FileWatcher::new("./nonexistent.toml");
        let result = watcher.watch().await;
        assert!(result.is_err());
    }
}
