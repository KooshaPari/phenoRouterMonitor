//! # Aggregate Root Module
//!
//! Aggregate root entities with domain event support.
//!
//! ## Usage
//!
//! ```rust
//! use phenotype_contracts::models::{AggregateRoot, DomainEvent};
//!
//! #[derive(Debug, Clone)]
//! struct OrderAggregate {
//!     aggregate_id: String,
//!     version: i64,
//!     total: f64,
//!     status: String,
//! }
//!
//! impl AggregateRoot for OrderAggregate {
//!     type Event = DomainEvent;
//!
//!     fn aggregate_id(&self) -> &str { &self.aggregate_id }
//!     fn version(&self) -> i64 { self.version }
//!     fn domain_type() -> &'static str { "Order" }
//!
//!     fn apply_event(&mut self, event: &DomainEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//!         match event.event_type.as_str() {
//!             "OrderCreated" => { self.status = "created".into(); }
//!             "OrderPaid" => { self.status = "paid".into(); }
//!             "OrderShipped" => { self.status = "shipped".into(); }
//!             _ => {}
//!         }
//!         Ok(())
//!     }
//! }
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// A domain event that can be stored and replayed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    /// Unique event identifier (UUID).
    pub event_id: String,
    /// Aggregate this event belongs to.
    pub aggregate_id: String,
    /// Event type name (e.g., "OrderCreated", "PaymentFailed").
    pub event_type: String,
    /// Serialized event payload (JSON).
    pub payload: serde_json::Value,
    /// Optional event metadata.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    /// Version/sequence number within aggregate.
    pub version: i64,
    /// When the event occurred (UTC).
    pub timestamp: DateTime<Utc>,
}

impl DomainEvent {
    /// Create a new domain event.
    pub fn new(
        aggregate_id: String,
        event_type: String,
        payload: serde_json::Value,
        version: i64,
    ) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            aggregate_id,
            event_type,
            payload,
            metadata: HashMap::new(),
            version,
            timestamp: Utc::now(),
        }
    }

    /// Add a metadata key-value pair.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

impl fmt::Display for DomainEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DomainEvent({} [{}] v{})",
            self.event_type, self.aggregate_id, self.version
        )
    }
}

/// Trait for aggregate root entities.
///
/// An aggregate root is the primary entity that enforces invariants
/// and is the entry point for domain operations.
pub trait AggregateRoot: Send + Sync {
    /// Return the aggregate's unique identifier.
    fn aggregate_id(&self) -> &str;

    /// Current version (for optimistic concurrency).
    fn version(&self) -> i64;

    /// Domain type name for this aggregate.
    fn domain_type() -> &'static str
    where
        Self: Sized;

    /// Apply a domain event to mutate state.
    fn apply_event(&mut self, event: &DomainEvent) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Marker for aggregate state type.
    type Event: Send + Sync;
}

/// Extension methods for aggregate roots.
pub trait AggregateExt {
    /// Replay a sequence of events to rebuild aggregate state.
    fn replay(&mut self, events: &[DomainEvent]) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Check if aggregate is in a terminal state.
    fn is_terminal(&self) -> bool;
}

impl<T: AggregateRoot> AggregateExt for T {
    fn replay(&mut self, events: &[DomainEvent]) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for event in events {
            self.apply_event(event)?;
        }
        Ok(())
    }

    fn is_terminal(&self) -> bool {
        false // Override in concrete implementations
    }
}
