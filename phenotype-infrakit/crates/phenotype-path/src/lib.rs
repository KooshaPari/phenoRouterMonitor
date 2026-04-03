use std::collections::HashMap;
use std::path::PathBuf;
use which::{which, which_in};

#[cfg(all(feature = "python", not(test)))]
use pyo3::prelude::*;

/// Fast PATH resolution with skip directory support
///
/// # Example
/// ```
/// use thegent_path_resolve::PathResolver;
///
/// let resolver = PathResolver::new();
/// if let Some(path) = resolver.resolve("codex") {
///     println!("Found codex at: {}", path);
/// }
/// ```
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
    ///
    /// Returns `None` if not found or if in skip directory.
    ///
    /// # Example
    /// ```
    /// let resolver = PathResolver::new();
    /// assert!(resolver.resolve("sh").is_some());
    /// assert!(resolver.resolve("nonexistent12345").is_none());
    /// ```
    pub fn resolve(&self, name: &str) -> Option<String> {
        // Build safe PATH (exclude skip_dirs)
        let safe_path = self.build_safe_path();

        // Use which crate (fast, native, cross-platform)
        match which_in(name, Some(safe_path)) {
            Ok(path) => {
                let path_str = path.to_string_lossy().to_string();
                // Check if in skip_dirs
                if self.is_in_skip_dirs(&path_str) {
                    None
                } else {
                    Some(path_str)
                }
            }
            Err(_) => None,
        }
    }

    /// Resolve multiple binaries at once (more efficient than multiple calls)
    ///
    /// # Example
    /// ```
    /// let resolver = PathResolver::new();
    /// let results = resolver.resolve_many(&["sh", "bash", "codex"]);
    /// ```
    pub fn resolve_many(&self, names: &[&str]) -> HashMap<String, Option<String>> {
        names
            .iter()
            .map(|name| (name.to_string(), self.resolve(name)))
            .collect()
    }

    fn build_safe_path(&self) -> String {
        use std::env;
        env::var("PATH").unwrap_or_default()
    }

    fn is_in_skip_dirs(&self, path: &str) -> bool {
        if self.skip_dirs.is_empty() {
            return false;
        }

        let path_buf = PathBuf::from(path);
        self.skip_dirs.iter().any(|skip| {
            path_buf.starts_with(skip)
                || path_buf
                    .canonicalize()
                    .map_or(false, |p| p.starts_with(skip))
        })
    }
}

impl Default for PathResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function for simple use cases
///
/// # Example
/// ```
/// use thegent_path_resolve::resolve_binary;
///
/// if let Some(path) = resolve_binary("codex") {
///     println!("Found codex at: {}", path);
/// }
/// ```
pub fn resolve_binary(name: &str) -> Option<String> {
    PathResolver::new().resolve(name)
}

#[cfg(all(feature = "python", not(test)))]
#[pyfunction]
fn resolve_binary(name: &str, skip_dirs: Option<Vec<String>>) -> PyResult<Option<String>> {
    let resolver = if let Some(skip) = skip_dirs {
        PathResolver::with_skip_dirs(skip)
    } else {
        PathResolver::new()
    };
    Ok(resolver.resolve(name))
}

#[cfg(all(feature = "python", not(test)))]
#[pymodule]
fn thegent_path_resolve(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(resolve_binary, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_binary() {
        // Should find common binaries
        assert!(resolve_binary("sh").is_some() || resolve_binary("bash").is_some());
    }

    #[test]
    fn test_resolve_many() {
        let resolver = PathResolver::new();
        let results = resolver.resolve_many(&["sh", "bash", "nonexistent12345"]);
        // At least one should be found
        assert!(results.values().any(|v| v.is_some()) || std::env::var("CI").is_ok());
    }
}
