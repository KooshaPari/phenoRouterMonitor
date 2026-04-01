//! Directory resolution utilities following XDG Base Directory Specification.
//!
//! Provides cross-platform directory resolution for configuration files.

use crate::error::{ConfigError, Result};
use std::path::{Path, PathBuf};

/// Directory resolution following XDG Base Directory Specification.
#[derive(Debug, Clone)]
pub struct ConfigDirs {
    app_name: String,
}

impl ConfigDirs {
    /// Create a new ConfigDirs resolver for an application.
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
        }
    }

    /// Get the config home directory.
    ///
    /// Unix: `$XDG_CONFIG_HOME` or `~/.config`
    /// Windows: `%APPDATA%`
    /// macOS: `~/Library/Application Support`
    pub fn config_home(&self) -> Result<PathBuf> {
        dirs::config_dir()
            .ok_or_else(|| ConfigError::directory("could not determine config home"))
    }

    /// Get the system config directory.
    ///
    /// Unix: `/etc` (or `$XDG_CONFIG_DIRS[0]`)
    /// Windows: `%PROGRAMDATA%`
    /// macOS: `/Library/Application Support`
    pub fn config_system(&self) -> Result<PathBuf> {
        dirs::config_dir()
            .map(|p| {
                if cfg!(target_os = "windows") {
                    p
                } else {
                    PathBuf::from("/etc")
                }
            })
            .ok_or_else(|| ConfigError::directory("could not determine system config dir"))
    }

    /// Get the cache directory.
    ///
    /// Unix: `$XDG_CACHE_HOME` or `~/.cache`
    /// Windows: `%LOCALAPPDATA%`
    /// macOS: `~/Library/Caches`
    pub fn cache_home(&self) -> Result<PathBuf> {
        dirs::cache_dir()
            .ok_or_else(|| ConfigError::directory("could not determine cache home"))
    }

    /// Get the data directory.
    ///
    /// Unix: `$XDG_DATA_HOME` or `~/.local/share`
    /// Windows: `%APPDATA%`
    /// macOS: `~/Library/Application Support`
    pub fn data_home(&self) -> Result<PathBuf> {
        dirs::data_dir()
            .ok_or_else(|| ConfigError::directory("could not determine data home"))
    }

    /// Get the config directory for this application.
    pub fn app_config_dir(&self) -> Result<PathBuf> {
        Ok(self.config_home()?.join(&self.app_name))
    }

    /// Get the config file path for this application.
    pub fn config_file(&self, filename: &str) -> Result<PathBuf> {
        Ok(self.app_config_dir()?.join(filename))
    }

    /// Get the cache directory for this application.
    pub fn app_cache_dir(&self) -> Result<PathBuf> {
        Ok(self.cache_home()?.join(&self.app_name))
    }

    /// Get the data directory for this application.
    pub fn app_data_dir(&self) -> Result<PathBuf> {
        Ok(self.data_home()?.join(&self.app_name))
    }

    /// Get all search paths for configuration files, in priority order.
    ///
    /// Returns paths from lowest to highest priority:
    /// 1. System config directory
    /// 2. User config directory (XDG_CONFIG_HOME)
    /// 3. Application-specific config directory
    pub fn search_paths(&self, filename: &str) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();

        // System config (lowest priority)
        if let Ok(system_dir) = self.config_system() {
            let system_file = system_dir.join(&self.app_name).join(filename);
            if system_file.exists() {
                paths.push(system_file);
            }
        }

        // User config home
        if let Ok(config_home) = self.config_home() {
            let user_file = config_home.join(&self.app_name).join(filename);
            if user_file.exists() {
                paths.push(user_file);
            }
        }

        // Application-specific config (highest priority)
        if let Ok(app_config) = self.app_config_dir() {
            let app_file = app_config.join(filename);
            if app_file.exists() {
                paths.push(app_file);
            }
        }

        Ok(paths)
    }

    /// Get the first existing config file for this application.
    pub fn find_config_file(&self, filename: &str) -> Result<Option<PathBuf>> {
        let paths = self.search_paths(filename)?;
        Ok(paths.into_iter().next())
    }

    /// Ensure a directory exists, creating it if necessary.
    pub fn ensure_dir(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            std::fs::create_dir_all(path)
                .map_err(|e| ConfigError::io(format!("failed to create directory: {}", e)))?;
        }
        Ok(())
    }

    /// Ensure the application config directory exists.
    pub fn ensure_app_config_dir(&self) -> Result<PathBuf> {
        let dir = self.app_config_dir()?;
        self.ensure_dir(&dir)?;
        Ok(dir)
    }

    /// Get the home directory.
    pub fn home_dir() -> Result<PathBuf> {
        dirs::home_dir()
            .ok_or_else(|| ConfigError::directory("could not determine home directory"))
    }

    /// Get the project directory (current working directory or git root).
    pub fn project_dir() -> Result<PathBuf> {
        std::env::current_dir()
            .map_err(|e| ConfigError::io(format!("failed to get current dir: {}", e)))
    }
}

impl Default for ConfigDirs {
    fn default() -> Self {
        Self::new("app")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let dirs = ConfigDirs::new("myapp");
        assert_eq!(dirs.app_name, "myapp");
    }

    #[test]
    fn test_config_home() {
        let dirs = ConfigDirs::new("test");
        assert!(dirs.config_home().is_ok());
    }

    #[test]
    fn test_cache_home() {
        let dirs = ConfigDirs::new("test");
        assert!(dirs.cache_home().is_ok());
    }

    #[test]
    fn test_app_config_dir() {
        let dirs = ConfigDirs::new("myapp");
        let config_dir = dirs.app_config_dir().unwrap();
        assert!(config_dir.to_string_lossy().contains("myapp"));
    }

    #[test]
    fn test_search_paths() {
        let dirs = ConfigDirs::new("nonexistent-app");
        // Non-existent app should return empty paths
        let paths = dirs.search_paths("config.toml").unwrap();
        assert!(paths.is_empty() || paths.len() >= 0);
    }
}
