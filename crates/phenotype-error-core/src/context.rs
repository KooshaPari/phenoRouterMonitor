//! Error context propagation
//!
//! Provides structured error context with spans and metadata.

use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Span for error tracing
#[derive(Debug, Clone)]
pub struct Span {
    /// Span name (usually function or operation name)
    pub name: String,
    /// Target/layer
    pub target: String,
    /// Timestamp when span started
    pub start: DateTime<Utc>,
    /// Optional location
    pub location: Option<Location>,
}

#[derive(Debug, Clone)]
pub struct Location {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

impl Span {
    /// Create a new span
    pub fn new(target: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            target: target.into(),
            start: Utc::now(),
            location: None,
        }
    }
    
    /// Create a span with location
    pub fn with_location(mut self, file: impl Into<String>, line: u32, column: u32) -> Self {
        self.location = Some(Location {
            file: file.into(),
            line,
            column,
        });
        self
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{} at {}", self.target, self.name, self.start)?;
        if let Some(loc) = &self.location {
            write!(f, " ({}:{}:{})", loc.file, loc.line, loc.column)?;
        }
        Ok(())
    }
}

/// Error context with tracing information
#[derive(Debug, Clone)]
pub struct Context<E> {
    /// The underlying error
    pub error: E,
    /// Stack trace spans
    pub spans: Vec<Span>,
    /// Error timestamps
    pub timestamps: Vec<DateTime<Utc>>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl<E> Context<E> {
    /// Create a new error context
    pub fn new(error: E) -> Self {
        Self {
            error,
            spans: Vec::new(),
            timestamps: vec![Utc::now()],
            metadata: HashMap::new(),
        }
    }
    
    /// Add a span to the context
    pub fn with_span<S: Into<String>>(mut self, target: S, name: S) -> Self {
        self.spans.push(Span::new(target, name));
        self
    }
    
    /// Add metadata to the context
    pub fn with_metadata<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Add a timestamp
    pub fn with_timestamp(mut self, ts: DateTime<Utc>) -> Self {
        self.timestamps.push(ts);
        self
    }
    
    /// Map the inner error
    pub fn map<F, R>(self, f: F) -> Context<R>
    where
        F: FnOnce(E) -> R,
    {
        Context {
            error: f(self.error),
            spans: self.spans,
            timestamps: self.timestamps,
            metadata: self.metadata,
        }
    }
}

impl<E: std::fmt::Display> std::fmt::Display for Context<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)?;
        for span in &self.spans {
            write!(f, "\n  at {}", span)?;
        }
        if !self.metadata.is_empty() {
            write!(f, "\n  metadata: {:?}", self.metadata)?;
        }
        Ok(())
    }
}

impl<E: std::fmt::Debug> std::fmt::Debug for Context<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("error", &self.error)
            .field("spans", &self.spans)
            .field("timestamps", &self.timestamps)
            .field("metadata", &self.metadata)
            .finish()
    }
}

impl<E: std::error::Error> std::error::Error for Context<E> {}

impl<E> From<E> for Context<E> {
    fn from(error: E) -> Self {
        Self::new(error)
    }
}

/// Extension trait for adding context to errors
pub trait ContextExt<E> {
    /// Add a span to the error
    fn in_span<T: Into<String>>(self, target: T, name: T) -> Context<E>;
    
    /// Add metadata to the error
    fn with<C: Into<String>, V: Into<String>>(self, key: C, value: V) -> Context<E>;
}

impl<E> ContextExt<E> for E {
    fn in_span<T: Into<String>>(self, target: T, name: T) -> Context<E> {
        Context::new(self).with_span(target, name)
    }
    
    fn with<C: Into<String>, V: Into<String>>(self, key: C, value: V) -> Context<E> {
        Context::new(self).with_metadata(key, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_span_creation() {
        let span = Span::new("auth", "verify_user");
        assert_eq!(span.target, "auth");
        assert_eq!(span.name, "verify_user");
    }
    
    #[test]
    fn test_context_creation() {
        let error = "test error";
        let ctx = Context::new(error)
            .with_span("auth", "verify")
            .with_metadata("request_id", "123");
        
        assert_eq!(ctx.spans.len(), 1);
        assert_eq!(ctx.metadata.get("request_id"), Some(&"123".to_string()));
    }
}
