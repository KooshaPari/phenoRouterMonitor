//! Error context propagation

use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Span for error tracing
#[derive(Debug, Clone)]
pub struct Span {
    pub name: String,
    pub target: String,
    pub start: DateTime<Utc>,
    pub location: Option<Location>,
}

#[derive(Debug, Clone)]
pub struct Location {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

impl Span {
    pub fn new(target: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            target: target.into(),
            start: Utc::now(),
            location: None,
        }
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{} at {}", self.target, self.name, self.start)
    }
}

/// Error context with tracing information
#[derive(Debug, Clone)]
pub struct Context<E> {
    pub error: E,
    pub spans: Vec<Span>,
    pub timestamps: Vec<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
}

impl<E> Context<E> {
    pub fn new(error: E) -> Self {
        Self {
            error,
            spans: Vec::new(),
            timestamps: vec![Utc::now()],
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_span(mut self, target: impl Into<String>, name: impl Into<String>) -> Self {
        self.spans.push(Span::new(target, name));
        self
    }
    
    pub fn with_metadata<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

impl<E: std::fmt::Display> std::fmt::Display for Context<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)?;
        for span in &self.spans {
            write!(f, "\n  at {}", span)?;
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

impl<E> From<E> for Context<E> {
    fn from(error: E) -> Self {
        Self::new(error)
    }
}

/// Extension trait for adding context to errors
pub trait ContextExt<E> {
    fn in_span(self, target: impl Into<String>, name: impl Into<String>) -> Context<E>;
    fn with(self, key: impl Into<String>, value: impl Into<String>) -> Context<E>;
}

impl<E> ContextExt<E> for E {
    fn in_span(self, target: impl Into<String>, name: impl Into<String>) -> Context<E> {
        Context::new(self).with_span(target, name)
    }
    
    fn with(self, key: impl Into<String>, value: impl Into<String>) -> Context<E> {
        Context::new(self).with_metadata(key, value)
    }
}
