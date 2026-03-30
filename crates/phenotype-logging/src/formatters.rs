//! Log formatters for JSON and human-readable output.
//!
//! This module provides utilities for formatting structured logs in various formats,
//! with support for including contextual metadata.

use crate::context::RequestContext;
use std::fmt;

/// A structured log entry that can be formatted in multiple ways.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Log level (trace, debug, info, warn, error).
    pub level: String,

    /// Log message.
    pub message: String,

    /// Target module or span.
    pub target: Option<String>,

    /// Key-value pairs for structured logging.
    pub fields: Vec<(String, String)>,

    /// Optional request context.
    pub context: Option<RequestContext>,
}

impl LogEntry {
    /// Create a new log entry.
    pub fn new(level: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            level: level.into(),
            message: message.into(),
            target: None,
            fields: Vec::new(),
            context: None,
        }
    }

    /// Set the target module.
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    /// Add a structured field.
    pub fn with_field(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields.push((key.into(), value.into()));
        self
    }

    /// Add multiple structured fields.
    pub fn with_fields(mut self, fields: Vec<(String, String)>) -> Self {
        self.fields.extend(fields);
        self
    }

    /// Set the request context.
    pub fn with_context(mut self, context: RequestContext) -> Self {
        self.context = Some(context);
        self
    }

    /// Format as a JSON string.
    ///
    /// Includes all fields and context information in a flat JSON object.
    pub fn format_json(&self) -> String {
        let mut obj = serde_json::json!({
            "level": self.level,
            "message": self.message,
        });

        if let Some(ref target) = self.target {
            obj["target"] = serde_json::json!(target);
        }

        // Add structured fields
        for (k, v) in &self.fields {
            obj[k] = serde_json::json!(v);
        }

        // Add context fields
        if let Some(ref ctx) = self.context {
            obj["correlation_id"] = serde_json::json!(ctx.correlation_id.to_string());
            obj["source"] = serde_json::json!(ctx.source.as_ref());
            obj["target_op"] = serde_json::json!(ctx.target.as_ref());

            if let Some(uid) = &ctx.user_id {
                obj["user_id"] = serde_json::json!(uid.as_ref());
            }

            if let Some(sid) = &ctx.session_id {
                obj["session_id"] = serde_json::json!(sid.as_ref());
            }

            if let Some(pid) = &ctx.parent_span_id {
                obj["parent_span_id"] = serde_json::json!(pid.as_ref());
            }
        }

        obj.to_string()
    }

    /// Format as a human-readable single line.
    ///
    /// Example: `[INFO] my_module: message (correlation_id=xyz)`
    pub fn format_compact(&self) -> String {
        let mut result = format!("[{}] ", self.level.to_uppercase());

        if let Some(ref target) = self.target {
            result.push_str(&format!("{}: ", target));
        }

        result.push_str(&self.message);

        if !self.fields.is_empty() {
            result.push_str(" (");
            for (i, (k, v)) in self.fields.iter().enumerate() {
                if i > 0 {
                    result.push_str(", ");
                }
                result.push_str(&format!("{}={}", k, v));
            }
            result.push(')');
        }

        if let Some(ref ctx) = self.context {
            result.push_str(&format!(" [correlation_id={}]", ctx.correlation_id));
            if let Some(uid) = &ctx.user_id {
                result.push_str(&format!(" [user_id={}]", uid));
            }
        }

        result
    }

    /// Format as a pretty multi-line human-readable format.
    pub fn format_pretty(&self) -> String {
        let mut result = format!("{}  {}\n", self.level.to_uppercase(), self.message);

        if let Some(ref target) = self.target {
            result.push_str(&format!("  target: {}\n", target));
        }

        for (k, v) in &self.fields {
            result.push_str(&format!("  {}: {}\n", k, v));
        }

        if let Some(ref ctx) = self.context {
            result.push_str(&format!("  correlation_id: {}\n", ctx.correlation_id));
            result.push_str(&format!("  source: {}\n", ctx.source));
            result.push_str(&format!("  target_op: {}\n", ctx.target));

            if let Some(uid) = &ctx.user_id {
                result.push_str(&format!("  user_id: {}\n", uid));
            }

            if let Some(sid) = &ctx.session_id {
                result.push_str(&format!("  session_id: {}\n", sid));
            }

            if let Some(pid) = &ctx.parent_span_id {
                result.push_str(&format!("  parent_span_id: {}\n", pid));
            }
        }

        result
    }
}

impl fmt::Display for LogEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_compact())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_entry_new() {
        let entry = LogEntry::new("info", "test message");
        assert_eq!(entry.level, "info");
        assert_eq!(entry.message, "test message");
        assert!(entry.target.is_none());
        assert!(entry.fields.is_empty());
        assert!(entry.context.is_none());
    }

    #[test]
    fn log_entry_with_target() {
        let entry = LogEntry::new("info", "msg").with_target("my_module");
        assert_eq!(entry.target.as_ref().map(|s| s.as_str()), Some("my_module"));
    }

    #[test]
    fn log_entry_with_field() {
        let entry = LogEntry::new("info", "msg")
            .with_field("key1", "val1")
            .with_field("key2", "val2");

        assert_eq!(entry.fields.len(), 2);
        assert_eq!(entry.fields[0], ("key1".to_string(), "val1".to_string()));
    }

    #[test]
    fn log_entry_format_json() {
        let entry = LogEntry::new("info", "test")
            .with_target("my_module")
            .with_field("key", "value");

        let json = entry.format_json();
        assert!(json.contains("\"level\":\"info\""));
        assert!(json.contains("\"message\":\"test\""));
        assert!(json.contains("\"target\":\"my_module\""));
        assert!(json.contains("\"key\":\"value\""));
    }

    #[test]
    fn log_entry_format_compact() {
        let entry = LogEntry::new("debug", "processing");
        let compact = entry.format_compact();
        assert!(compact.contains("[DEBUG]"));
        assert!(compact.contains("processing"));
    }

    #[test]
    fn log_entry_format_compact_with_fields() {
        let entry = LogEntry::new("warn", "alert")
            .with_field("severity", "high")
            .with_field("code", "ERR42");

        let compact = entry.format_compact();
        assert!(compact.contains("[WARN]"));
        assert!(compact.contains("severity=high"));
        assert!(compact.contains("code=ERR42"));
    }

    #[test]
    fn log_entry_format_pretty() {
        let entry = LogEntry::new("error", "failure")
            .with_target("handler")
            .with_field("attempt", "3");

        let pretty = entry.format_pretty();
        assert!(pretty.contains("ERROR"));
        assert!(pretty.contains("failure"));
        assert!(pretty.contains("target: handler"));
        assert!(pretty.contains("attempt: 3"));
    }

    #[test]
    fn log_entry_with_context() {
        let ctx = RequestContext::new("svc", "op").with_user_id("user123");
        let entry = LogEntry::new("info", "action").with_context(ctx.clone());

        let json = entry.format_json();
        assert!(json.contains("correlation_id"));
        assert!(json.contains("\"user_id\":\"user123\""));

        let compact = entry.format_compact();
        assert!(compact.contains("correlation_id="));
        assert!(compact.contains("user_id=user123"));

        let pretty = entry.format_pretty();
        assert!(pretty.contains("correlation_id:"));
        assert!(pretty.contains("user_id: user123"));
    }

    #[test]
    fn log_entry_display() {
        let entry = LogEntry::new("info", "message");
        let display = entry.to_string();
        assert!(display.contains("[INFO]"));
        assert!(display.contains("message"));
    }
}
