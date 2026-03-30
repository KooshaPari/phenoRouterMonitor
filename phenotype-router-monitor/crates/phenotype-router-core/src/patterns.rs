//! Path pattern matching for routing rules
//!
//! Supports:
//! - Exact matching: `/exact/path`
//! - Regex patterns: `^/api/.*`
//! - Wildcard patterns: `/api/*`

use crate::error::{Result, RouterError};
use regex::Regex;

/// Path pattern for route matching
#[derive(Debug, Clone)]
pub enum PathPattern {
    /// Exact string match
    Exact(String),

    /// Regex pattern match
    Regex(Regex),

    /// Wildcard pattern (converted to regex)
    Wildcard(String),
}

impl PathPattern {
    /// Create an exact match pattern
    pub fn exact(path: &str) -> Self {
        PathPattern::Exact(path.to_string())
    }

    /// Create a regex pattern
    pub fn regex(pattern: &str) -> Result<Self> {
        let re = Regex::new(pattern).map_err(|e| RouterError::InvalidPattern {
            pattern: pattern.to_string(),
            reason: e.to_string(),
        })?;
        Ok(PathPattern::Regex(re))
    }

    /// Create a wildcard pattern
    pub fn wildcard(pattern: &str) -> Self {
        PathPattern::Wildcard(pattern.to_string())
    }

    /// Test if path matches this pattern
    pub fn matches(&self, path: &str) -> bool {
        match self {
            PathPattern::Exact(exact) => path == exact,
            PathPattern::Regex(re) => re.is_match(path),
            PathPattern::Wildcard(pattern) => {
                // Convert wildcard to regex: /api/* -> /api/.*
                let re_pattern = pattern.replace("*", ".*").replace("?", ".");
                if let Ok(re) = Regex::new(&re_pattern) {
                    re.is_match(path)
                } else {
                    false
                }
            }
        }
    }

    /// Extract captured groups if pattern is regex
    pub fn captures<'a>(&self, path: &'a str) -> Option<regex::Captures<'a>> {
        match self {
            PathPattern::Regex(re) => re.captures(path),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-ROUTER-003 (Path pattern matching)
    #[test]
    fn test_exact_match() {
        let pattern = PathPattern::exact("/api/users");
        assert!(pattern.matches("/api/users"));
        assert!(!pattern.matches("/api/users/1"));
    }

    // Traces to: FR-ROUTER-003
    #[test]
    fn test_regex_match() {
        let pattern = PathPattern::regex("^/api/.*").unwrap();
        assert!(pattern.matches("/api/users"));
        assert!(pattern.matches("/api/users/123"));
        assert!(!pattern.matches("/other/path"));
    }

    // Traces to: FR-ROUTER-003
    #[test]
    fn test_wildcard_match() {
        let pattern = PathPattern::wildcard("/api/*");
        assert!(pattern.matches("/api/users"));
        assert!(pattern.matches("/api/users/123"));
        assert!(!pattern.matches("/other/path"));
    }

    // Traces to: FR-ROUTER-003
    #[test]
    fn test_invalid_regex() {
        let result = PathPattern::regex("[invalid(");
        assert!(result.is_err());
    }

    // Traces to: FR-ROUTER-003
    #[test]
    fn test_regex_captures() {
        let pattern = PathPattern::regex("^/api/(.*)/(.*)").unwrap();
        let caps = pattern.captures("/api/users/123");
        assert!(caps.is_some());
        if let Some(c) = caps {
            assert_eq!(c.get(1).unwrap().as_str(), "users");
            assert_eq!(c.get(2).unwrap().as_str(), "123");
        }
    }
}
