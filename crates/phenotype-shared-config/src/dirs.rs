//! Directory search utilities for configuration files.

use std::path::{Path, PathBuf};

/// Configuration directory types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigDir {
    /// System-wide configuration directory.
    System,
    /// User-specific configuration directory.
    User,
    /// Application-specific configuration directory.
    App,
    /// Current working directory.
    Cwd,
    /// Environment variable specified directory.
    Env,
}

impl ConfigDir {
    /// Get the directory path for this config directory type.
    pub fn path(&self, app_name: &str) -> Option<PathBuf> {
        match self {
            Self::System => {
                #[cfg(unix)]
                {
                    Some(PathBuf::from(format!("/etc/{}", app_name)))
                }
                #[cfg(windows)]
                {
                    std::env::var("PROGRAMDATA")
                        .ok()
                        .map(|p| PathBuf::from(format!("{}/{}", p, app_name)))
                }
                #[cfg(not(any(unix, windows)))]
                {
                    None
                }
            }
            Self::User => dirs::config_dir().map(|p| p.join(app_name)),
            Self::App => dirs::config_local_dir().map(|p| p.join(app_name)),
            Self::Cwd => Some(std::env::current_dir().unwrap_or_default()),
            Self::Env => std::env::var("CONFIG_DIR").ok().map(PathBuf::from),
        }
    }
}

/// Application directories helper.
#[derive(Debug, Clone)]
pub struct AppDirs {
    /// Application name.
    pub app_name: String,
}

impl AppDirs {
    /// Create a new AppDirs instance.
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
        }
    }

    /// Get the config directory.
    pub fn config_dir(&self) -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join(&self.app_name))
    }

    /// Get the data directory.
    pub fn data_dir(&self) -> Option<PathBuf> {
        dirs::data_dir().map(|p| p.join(&self.app_name))
    }

    /// Get the cache directory.
    pub fn cache_dir(&self) -> Option<PathBuf> {
        dirs::cache_dir().map(|p| p.join(&self.app_name))
    }

    /// Get the state directory.
    pub fn state_dir(&self) -> Option<PathBuf> {
        dirs::state_dir().map(|p| p.join(&self.app_name))
    }
}

/// Search for configuration files in standard locations.
pub fn search_config_dirs<P: AsRef<Path>>(
    app_name: &str,
    filename: &str,
    extra_dirs: &[P],
) -> Vec<(ConfigDir, PathBuf)> {
    let mut results = Vec::new();

    // Search standard directories
    for dir_type in [
        ConfigDir::Env,
        ConfigDir::User,
        ConfigDir::App,
        ConfigDir::Cwd,
        ConfigDir::System,
    ] {
        if let Some(base) = dir_type.path(app_name) {
            let path = base.join(filename);
            if path.exists() {
                results.push((dir_type, path));
            }
        }
    }

    // Search extra directories
    for extra in extra_dirs {
        let path = extra.as_ref().join(filename);
        if path.exists() {
            results.push((ConfigDir::Env, path));
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_dirs_creation() {
        let dirs = AppDirs::new("test-app");
        assert_eq!(dirs.app_name, "test-app");
    }

    #[test]
    fn test_config_dir_search() {
        let results: Vec<(ConfigDir, std::path::PathBuf)> = search_config_dirs::<&str>("nonexistent-app", ".config.toml", &[]);
        // Should return empty or partial matches depending on environment
        assert!(results.iter().all(|(d, p)| d != &ConfigDir::Env && p.exists()));
    }
}
