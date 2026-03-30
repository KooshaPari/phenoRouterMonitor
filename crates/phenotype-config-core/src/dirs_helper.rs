//! Platform-aware configuration directory resolution using XDG standards.

use crate::error::{ConfigError, Result};
use std::path::PathBuf;

/// Helper for resolving standard configuration directories.
///
/// Uses XDG Base Directory Specification on Unix-like systems,
/// and standard Windows directories on Windows.
pub struct ConfigDirs;

impl ConfigDirs {
    /// Get the user's home directory configuration path.
    ///
    /// On Unix: `~/.config`
    /// On macOS: `~/Library/Application Support`
    /// On Windows: `%APPDATA%`
    pub fn config_home() -> Result<PathBuf> {
        dirs::config_dir().ok_or_else(|| {
            ConfigError::environment("Could not determine config directory".to_string())
        })
    }

    /// Get the system-wide configuration path.
    ///
    /// On Unix: `/etc`
    /// On Windows: `%ProgramData%`
    pub fn config_system() -> Result<PathBuf> {
        #[cfg(unix)]
        {
            Ok(PathBuf::from("/etc"))
        }
        #[cfg(windows)]
        {
            // On Windows, use ProgramData or ALLUSERSPROFILE
            std::env::var("ProgramData")
                .map(PathBuf::from)
                .or_else(|_| std::env::var("ALLUSERSPROFILE").map(PathBuf::from))
                .map_err(|_| {
                    ConfigError::environment(
                        "Could not determine system config directory on Windows".to_string(),
                    )
                })
        }
    }

    /// Get the user's cache directory path.
    ///
    /// On Unix: `~/.cache`
    /// On macOS: `~/Library/Caches`
    /// On Windows: `%LOCALAPPDATA%\cache`
    pub fn cache_home() -> Result<PathBuf> {
        dirs::cache_dir().ok_or_else(|| {
            ConfigError::environment("Could not determine cache directory".to_string())
        })
    }

    /// Build a config path for an application.
    ///
    /// # Arguments
    /// * `app_name` - Name of the application (e.g., "myapp")
    /// * `filename` - Configuration filename (e.g., "config.toml")
    ///
    /// Returns the user's config directory joined with app_name and filename.
    pub fn config_file(app_name: &str, filename: &str) -> Result<PathBuf> {
        let mut path = Self::config_home()?;
        path.push(app_name);
        path.push(filename);
        Ok(path)
    }

    /// Build system-wide config path for an application.
    ///
    /// # Arguments
    /// * `app_name` - Name of the application
    /// * `filename` - Configuration filename
    ///
    /// Returns the system config directory joined with app_name and filename.
    pub fn system_config_file(app_name: &str, filename: &str) -> Result<PathBuf> {
        let mut path = Self::config_system()?;
        path.push(app_name);
        path.push(filename);
        Ok(path)
    }

    /// Build a cache file path for an application.
    pub fn cache_file(app_name: &str, filename: &str) -> Result<PathBuf> {
        let mut path = Self::cache_home()?;
        path.push(app_name);
        path.push(filename);
        Ok(path)
    }

    /// Search for a config file in standard locations.
    ///
    /// Searches in order:
    /// 1. User config directory (~/.config/app_name/)
    /// 2. System config directory (/etc/app_name/ or %ProgramData%/app_name/)
    ///
    /// Returns the first existing file path, or None if not found.
    pub fn find_config_file(app_name: &str, filename: &str) -> Result<Option<PathBuf>> {
        // Try user config first
        let user_path = Self::config_file(app_name, filename)?;
        if user_path.exists() {
            return Ok(Some(user_path));
        }

        // Try system config
        let sys_path = Self::system_config_file(app_name, filename)?;
        if sys_path.exists() {
            return Ok(Some(sys_path));
        }

        Ok(None)
    }

    /// Get multiple potential config file locations (in search order).
    ///
    /// Useful for debugging or showing users where the config could be placed.
    pub fn search_paths(app_name: &str, filename: &str) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();

        if let Ok(user_path) = Self::config_file(app_name, filename) {
            paths.push(user_path);
        }

        if let Ok(sys_path) = Self::system_config_file(app_name, filename) {
            paths.push(sys_path);
        }

        Ok(paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_home_exists() {
        assert!(ConfigDirs::config_home().is_ok());
    }

    #[test]
    fn test_config_system_exists() {
        assert!(ConfigDirs::config_system().is_ok());
    }

    #[test]
    fn test_cache_home_exists() {
        assert!(ConfigDirs::cache_home().is_ok());
    }

    #[test]
    fn test_config_file_path() {
        let path = ConfigDirs::config_file("test_app", "config.toml").unwrap();
        assert!(path.to_string_lossy().contains("test_app"));
        assert!(path.to_string_lossy().contains("config.toml"));
    }

    #[test]
    fn test_search_paths() {
        let paths = ConfigDirs::search_paths("test_app", "config.toml").unwrap();
        assert!(!paths.is_empty());
        for path in paths {
            assert!(path.to_string_lossy().contains("test_app"));
            assert!(path.to_string_lossy().contains("config.toml"));
        }
    }
}
