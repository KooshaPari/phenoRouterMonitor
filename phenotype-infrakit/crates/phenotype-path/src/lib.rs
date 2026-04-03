use std::collections::HashMap;
use std::path::{Path, PathBuf};
use which::{which, which_in};

/// Fast PATH resolution with skip directory support
pub struct PathResolver {
    skip_dirs: Vec<PathBuf>,
}

impl PathResolver {
    /// Create a new path resolver
    pub fn new() -> Self {
        Self {
            skip_dirs: Vec::new(),
        }
    }

    /// Create with directories to skip (e.g., shim directories)
    pub fn with_skip_dirs(skip_dirs: Vec<String>) -> Self {
        Self {
            skip_dirs: skip_dirs.iter().map(PathBuf::from).collect(),
        }
    }

    /// Resolve a binary name to its full path
    pub fn resolve(&self, name: &str) -> Option<String> {
        let safe_path = self.build_safe_path();

        match which(name) {
            Ok(path) => {
                let path_str = path.to_string_lossy().to_string();
                if self.is_in_skip_dirs(&path_str) {
                    None
                } else {
                    Some(path_str)
                }
            }
            Err(_) => {
                // Try with safe PATH if available
                if let Some(safe) = &safe_path {
                    match which_in(name, Some(safe.as_str()), Path::new(".")) {
                        Ok(path) => {
                            let path_str = path.to_string_lossy().to_string();
                            if self.is_in_skip_dirs(&path_str) {
                                None
                            } else {
                                Some(path_str)
                            }
                        }
                        Err(_) => None,
                    }
                } else {
                    None
                }
            }
        }
    }

    /// Resolve multiple binary names to their full paths
    pub fn resolve_many(&self, names: Vec<String>) -> HashMap<String, String> {
        names.into_iter().filter_map(|name| {
            self.resolve(&name).map(|path| (name, path))
        }).collect()
    }

    /// Check if path is in a skip directory
    fn is_in_skip_dirs(&self, path: &str) -> bool {
        self.skip_dirs.iter().any(|skip| path.starts_with(&skip.to_string_lossy().to_string()))
    }

    /// Build safe PATH excluding skip directories
    fn build_safe_path(&self) -> Option<String> {
        let path_var = std::env::var("PATH").ok()?;
        let filtered: Vec<_> = path_var
            .split(':')
            .filter(|p| !self.skip_dirs.iter().any(|skip| p.starts_with(&skip.to_string_lossy().to_string())))
            .collect();
        Some(filtered.join(":"))
    }
}

impl Default for PathResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Shim filtering utilities
pub struct ShimFilter;

impl ShimFilter {
    /// Get default skip directories for shims
    pub fn default_skip_dirs() -> Vec<String> {
        vec![
            ".asdf/shims".to_string(),
            ".mise/shims".to_string(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_sh() {
        let resolver = PathResolver::new();
        assert!(resolver.resolve("sh").is_some());
    }

    #[test]
    fn test_resolve_nonexistent() {
        let resolver = PathResolver::new();
        assert!(resolver.resolve("nonexistent12345").is_none());
    }

    #[test]
    fn test_with_skip_dirs() {
        let resolver = PathResolver::with_skip_dirs(vec!["/tmp".to_string()]);
        assert_eq!(resolver.skip_dirs.len(), 1);
    }

    #[test]
    fn test_default_skip_dirs() {
        let dirs = ShimFilter::default_skip_dirs();
        assert!(!dirs.is_empty());
    }
}
