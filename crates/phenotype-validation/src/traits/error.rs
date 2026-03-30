//! Enhanced validation error types with context.
//!
//! Provides rich error reporting for validation failures.

use std::collections::HashMap;

/// Severity level for validation failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Non-blocking warning; operation can proceed with caution.
    Warning,
    /// Operation cannot proceed; user action required.
    Error,
    /// Critical failure; system integrity compromised.
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

/// Validation context: tracks the path and metadata for a failing rule.
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// Rule name that failed (e.g., "required", "email_format").
    pub rule_name: String,
    /// Field path for context (e.g., "user.email" or "plans[0].name").
    pub field_path: Option<String>,
    /// Additional context metadata (e.g., "allowed_domains", "min_length").
    pub metadata: HashMap<String, String>,
}

impl ValidationContext {
    pub fn new(rule_name: impl Into<String>) -> Self {
        Self {
            rule_name: rule_name.into(),
            field_path: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.field_path = Some(path.into());
        self
    }

    pub fn with_meta(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Rich validation error type with context stack and suggestions.
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Primary error message or key (e.g., "invalid_email").
    pub message: String,
    /// Severity level for this error.
    pub severity: Severity,
    /// Stack of validation contexts (innermost first).
    pub context_stack: Vec<ValidationContext>,
    /// User-friendly suggestion for fixing the error.
    pub suggestion: Option<String>,
}

impl ValidationError {
    /// Create a new validation error with message/key.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            severity: Severity::Error,
            context_stack: vec![],
            suggestion: None,
        }
    }

    /// Set severity level.
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Add context to the stack.
    pub fn push_context(mut self, context: ValidationContext) -> Self {
        self.context_stack.push(context);
        self
    }

    /// Add a suggestion for fixing this error.
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Get human-readable error message with context.
    pub fn display_message(&self) -> String {
        let mut msg = self.message.clone();

        if let Some(field) = self.context_stack.first().and_then(|c| c.field_path.as_ref()) {
            msg = format!("{} (field: {})", msg, field);
        }

        if let Some(suggestion) = &self.suggestion {
            msg = format!("{}. {}", msg, suggestion);
        }

        msg
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_message())
    }
}

impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Warning < Severity::Error);
        assert!(Severity::Error < Severity::Critical);
    }

    #[test]
    fn test_validation_error_with_context() {
        let err = ValidationError::new("invalid_format")
            .with_severity(Severity::Error)
            .with_suggestion("Expected email format: user@example.com");

        assert_eq!(err.message, "invalid_format");
        assert!(err.suggestion.is_some());
    }

    #[test]
    fn test_validation_context_with_metadata() {
        let ctx = ValidationContext::new("pattern_mismatch")
            .with_path("user.email")
            .with_meta("pattern", r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$");

        assert_eq!(ctx.field_path, Some("user.email".to_string()));
        assert!(ctx.metadata.contains_key("pattern"));
    }

    #[test]
    fn test_error_display_with_suggestion() {
        let err = ValidationError::new("too_short")
            .with_suggestion("Minimum 6 characters required");

        let msg = err.display_message();
        assert!(msg.contains("too_short"));
        assert!(msg.contains("Minimum 6 characters"));
    }
}
