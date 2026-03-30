//! System introspection tool for environment and platform information.

use serde::{Deserialize, Serialize};
use std::env;

/// System information snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub cpu_count: usize,
    pub available_memory_bytes: Option<u64>,
    pub user: Option<String>,
    pub current_dir: Option<String>,
    pub home_dir: Option<String>,
}

/// System introspector for environment queries.
#[derive(Debug, Clone)]
pub struct SystemIntrospector;

impl SystemIntrospector {
    /// Create a new system introspector.
    pub fn new() -> Self {
        Self
    }

    /// Get current system information.
    pub fn get_system_info() -> SystemInfo {
        SystemInfo {
            os: env::consts::OS.to_string(),
            arch: env::consts::ARCH.to_string(),
            cpu_count: num_cpus::get(),
            available_memory_bytes: sys_info::mem_info().ok().map(|m| m.avail * 1024),
            user: env::var("USER").ok(),
            current_dir: env::current_dir()
                .ok()
                .and_then(|p| p.to_str().map(String::from)),
            home_dir: dirs::home_dir().and_then(|p| p.to_str().map(String::from)),
        }
    }

    /// Get an environment variable.
    pub fn get_env(&self, key: &str) -> Option<String> {
        env::var(key).ok()
    }

    /// Get all environment variables.
    pub fn get_all_env(&self) -> Vec<(String, String)> {
        env::vars().collect()
    }

    /// Get current working directory.
    pub fn get_cwd(&self) -> Result<String, String> {
        env::current_dir()
            .ok()
            .and_then(|p| p.to_str().map(String::from))
            .ok_or_else(|| "Could not determine current directory".to_string())
    }
}

impl Default for SystemIntrospector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_system_info() {
        let info = SystemIntrospector::get_system_info();
        assert!(!info.os.is_empty());
        assert!(!info.arch.is_empty());
        assert!(info.cpu_count > 0);
    }

    #[test]
    fn get_env_var() {
        env::set_var("TEST_VAR_PHENOTYPE", "test_value");
        let introspector = SystemIntrospector::new();
        assert_eq!(
            introspector.get_env("TEST_VAR_PHENOTYPE"),
            Some("test_value".to_string())
        );
        assert!(introspector.get_env("NONEXISTENT_VAR_XYZ").is_none());
    }

    #[test]
    fn get_cwd() {
        let introspector = SystemIntrospector::new();
        let cwd = introspector.get_cwd().unwrap();
        assert!(!cwd.is_empty());
    }

    #[test]
    fn get_all_env() {
        let introspector = SystemIntrospector::new();
        let vars = introspector.get_all_env();
        assert!(!vars.is_empty());
    }
}
