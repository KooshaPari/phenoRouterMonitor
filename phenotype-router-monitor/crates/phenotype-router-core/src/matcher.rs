//! Path matching strategies for HTTP routes.

use crate::error::{RouterError, RouterResult};
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Strategy for matching request paths against route patterns.
pub trait MatcherStrategy: Send + Sync {
    /// Check if the given path matches this matcher's pattern.
    fn matches(&self, path: &str) -> bool;

    /// Get the strategy name.
    fn strategy_name(&self) -> &str;
}

/// Exact path matching (case-sensitive).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExactMatcher {
    pattern: String,
}

impl ExactMatcher {
    pub fn new(pattern: String) -> RouterResult<Self> {
        if pattern.is_empty() {
            return Err(RouterError::InvalidPattern("Pattern cannot be empty".to_string()));
        }
        Ok(Self { pattern })
    }

    pub fn pattern(&self) -> &str {
        &self.pattern
    }
}

impl MatcherStrategy for ExactMatcher {
    fn matches(&self, path: &str) -> bool {
        self.pattern == path
    }

    fn strategy_name(&self) -> &str {
        "exact"
    }
}

/// Wildcard path matching with `*` and `?` support.
/// - `*` matches any sequence of characters (including `/`)
/// - `?` matches any single character
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WildcardMatcher {
    pattern: String,
}

impl WildcardMatcher {
    pub fn new(pattern: String) -> RouterResult<Self> {
        if pattern.is_empty() {
            return Err(RouterError::InvalidPattern("Pattern cannot be empty".to_string()));
        }
        Ok(Self { pattern })
    }

    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Convert wildcard pattern to regex.
    fn wildcard_to_regex(pattern: &str) -> String {
        let escaped = regex::escape(pattern)
            .replace(r"\*", ".*")
            .replace(r"\?", ".");
        format!("^{}$", escaped)
    }
}

impl MatcherStrategy for WildcardMatcher {
    fn matches(&self, path: &str) -> bool {
        let regex_pattern = Self::wildcard_to_regex(&self.pattern);
        if let Ok(re) = Regex::new(&regex_pattern) {
            re.is_match(path)
        } else {
            false
        }
    }

    fn strategy_name(&self) -> &str {
        "wildcard"
    }
}

/// Regular expression path matching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegexMatcher {
    pattern: String,
    #[serde(skip)]
    compiled: Option<Regex>,
}

impl RegexMatcher {
    pub fn new(pattern: String) -> RouterResult<Self> {
        if pattern.is_empty() {
            return Err(RouterError::InvalidPattern("Pattern cannot be empty".to_string()));
        }

        let compiled = Regex::new(&pattern).map_err(|e| {
            RouterError::RegexError(format!("Failed to compile regex: {}", e))
        })?;

        Ok(Self {
            pattern,
            compiled: Some(compiled),
        })
    }

    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    fn get_compiled(&mut self) -> RouterResult<&Regex> {
        if self.compiled.is_none() {
            self.compiled = Some(Regex::new(&self.pattern).map_err(|e| {
                RouterError::RegexError(format!("Failed to compile regex: {}", e))
            })?);
        }
        Ok(self.compiled.as_ref().unwrap())
    }
}

impl MatcherStrategy for RegexMatcher {
    fn matches(&self, path: &str) -> bool {
        if let Some(ref re) = self.compiled {
            re.is_match(path)
        } else {
            false
        }
    }

    fn strategy_name(&self) -> &str {
        "regex"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_matcher_exact_match() {
        let matcher = ExactMatcher::new("/api/v1/users".to_string()).unwrap();
        assert!(matcher.matches("/api/v1/users"));
    }

    #[test]
    fn test_exact_matcher_no_match() {
        let matcher = ExactMatcher::new("/api/v1/users".to_string()).unwrap();
        assert!(!matcher.matches("/api/v1/posts"));
    }

    #[test]
    fn test_exact_matcher_case_sensitive() {
        let matcher = ExactMatcher::new("/api/v1/Users".to_string()).unwrap();
        assert!(!matcher.matches("/api/v1/users"));
    }

    #[test]
    fn test_exact_matcher_empty_pattern() {
        assert!(ExactMatcher::new("".to_string()).is_err());
    }

    #[test]
    fn test_wildcard_matcher_asterisk() {
        let matcher = WildcardMatcher::new("/api/*/users".to_string()).unwrap();
        assert!(matcher.matches("/api/v1/users"));
        assert!(matcher.matches("/api/v2/users"));
        assert!(matcher.matches("/api/auth/users"));
    }

    #[test]
    fn test_wildcard_matcher_question_mark() {
        let matcher = WildcardMatcher::new("/api/v?/users".to_string()).unwrap();
        assert!(matcher.matches("/api/v1/users"));
        assert!(matcher.matches("/api/v2/users"));
        assert!(!matcher.matches("/api/v10/users"));
    }

    #[test]
    fn test_wildcard_matcher_combined() {
        let matcher = WildcardMatcher::new("/api/v*/user?".to_string()).unwrap();
        assert!(matcher.matches("/api/v1/users"));
        assert!(matcher.matches("/api/v2/usera"));
        assert!(!matcher.matches("/api/v1/user"));
    }

    #[test]
    fn test_wildcard_matcher_asterisk_slash() {
        let matcher = WildcardMatcher::new("/api/*".to_string()).unwrap();
        assert!(matcher.matches("/api/v1/users"));
        assert!(matcher.matches("/api/posts"));
    }

    #[test]
    fn test_wildcard_matcher_empty_pattern() {
        assert!(WildcardMatcher::new("".to_string()).is_err());
    }

    #[test]
    fn test_regex_matcher_simple() {
        let matcher = RegexMatcher::new("^/api/v[0-9]+/.*".to_string()).unwrap();
        assert!(matcher.matches("/api/v1/users"));
        assert!(matcher.matches("/api/v2/posts"));
        assert!(!matcher.matches("/api/users"));
    }

    #[test]
    fn test_regex_matcher_named_groups() {
        let matcher = RegexMatcher::new("^/users/(?P<id>[0-9]+)$".to_string()).unwrap();
        assert!(matcher.matches("/users/123"));
        assert!(matcher.matches("/users/999"));
        assert!(!matcher.matches("/users/abc"));
    }

    #[test]
    fn test_regex_matcher_alternation() {
        let matcher = RegexMatcher::new("^/(users|posts|comments)$".to_string()).unwrap();
        assert!(matcher.matches("/users"));
        assert!(matcher.matches("/posts"));
        assert!(matcher.matches("/comments"));
        assert!(!matcher.matches("/likes"));
    }

    #[test]
    fn test_regex_matcher_invalid_pattern() {
        let result = RegexMatcher::new("[invalid".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_regex_matcher_empty_pattern() {
        assert!(RegexMatcher::new("".to_string()).is_err());
    }

    #[test]
    fn test_matcher_strategy_names() {
        let exact = ExactMatcher::new("/api".to_string()).unwrap();
        let wildcard = WildcardMatcher::new("/api/*".to_string()).unwrap();
        let regex = RegexMatcher::new("^/api/.*".to_string()).unwrap();

        assert_eq!(exact.strategy_name(), "exact");
        assert_eq!(wildcard.strategy_name(), "wildcard");
        assert_eq!(regex.strategy_name(), "regex");
    }

    #[test]
    fn test_wildcard_special_chars() {
        let matcher = WildcardMatcher::new("/api/v1.?/users".to_string()).unwrap();
        assert!(matcher.matches("/api/v1./users"));
        assert!(matcher.matches("/api/v1a/users"));
    }

    #[test]
    fn test_exact_matcher_strategy_trait() {
        let matcher: Box<dyn MatcherStrategy> =
            Box::new(ExactMatcher::new("/api/v1".to_string()).unwrap());
        assert!(matcher.matches("/api/v1"));
        assert!(!matcher.matches("/api/v2"));
    }

    #[test]
    fn test_wildcard_matcher_strategy_trait() {
        let matcher: Box<dyn MatcherStrategy> =
            Box::new(WildcardMatcher::new("/api/*".to_string()).unwrap());
        assert!(matcher.matches("/api/v1"));
        assert!(matcher.matches("/api/v1/nested"));
    }

    #[test]
    fn test_regex_matcher_strategy_trait() {
        let matcher: Box<dyn MatcherStrategy> =
            Box::new(RegexMatcher::new("^/api/.*".to_string()).unwrap());
        assert!(matcher.matches("/api/v1"));
        assert!(matcher.matches("/api/v1/users"));
    }

    #[test]
    fn test_complex_wildcard_pattern() {
        let matcher = WildcardMatcher::new("/service/*/resource/*/action".to_string()).unwrap();
        assert!(matcher.matches("/service/api/resource/123/action"));
        assert!(matcher.matches("/service/db/resource/456/action"));
        assert!(!matcher.matches("/service/api/resource/123"));
    }

    #[test]
    fn test_regex_case_insensitive() {
        let matcher = RegexMatcher::new("(?i)^/api/users$".to_string()).unwrap();
        assert!(matcher.matches("/api/users"));
        assert!(matcher.matches("/API/USERS"));
        assert!(matcher.matches("/Api/Users"));
    }
}
