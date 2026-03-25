//! # Aggregate Root
//!
//! Aggregate root is the main entity in an aggregate that enforces invariants.
//!
//! ## Characteristics
//!
//! - **Entry point**: All access to the aggregate goes through the root
//! - **Invariant enforcement**: The root ensures all invariants are maintained
//! - **Event sourcing**: Can emit domain events
//!
//! ## Examples
//!
//! - `Order`: Contains OrderItems, enforces order rules
//! - `User`: Contains Profile, enforces user rules
//!
//! ## Usage
//!
//! ```rust,ignore
//! use uuid::Uuid;
//! use phenotype_contracts::models::{AggregateRoot, DomainEvent};
//!
//! struct OrderId(String);
//! struct OrderCreatedEvent { order_id: String }
//!
//! struct Order {
//!     id: OrderId,
//!     items: Vec<OrderItem>,
//!     status: OrderStatus,
//! }
//!
//! impl AggregateRoot for Order {
//!     type Id = OrderId;
//!
//!     fn id(&self) -> &Self::Id {
//!         &self.id
//!     }
//!
//!     fn version(&self) -> u64 {
//!         self.version
//!     }
//! }
//! ```

use super::Entity;
use super::entity::EntityExt;

/// Trait for aggregate roots.
///
/// Aggregate roots are the main entity in an aggregate that:
///
/// 1. Is the only entity that external code can reference
/// 2. Enforces all invariants for the aggregate
/// 3. Can emit domain events
pub trait AggregateRoot: Entity {
    /// Get the aggregate version (for optimistic concurrency)
    fn version(&self) -> u64;

    /// Take pending events (used by event sourcing)
    fn take_events(&mut self) -> Vec<Box<dyn DomainEvent>>;
}

/// Marker trait for domain events.
///
/// Types implementing this trait can be used with event sourcing.
pub trait DomainEvent: Send + Sync {
    /// Event type name
    fn event_type(&self) -> &'static str;

    /// Event version for schema evolution
    fn event_version(&self) -> u32;

    /// Aggregate ID this event belongs to
    fn aggregate_id(&self) -> &str;
}

/// Extension trait for aggregate operations
pub trait AggregateExt: AggregateRoot {
    /// Get aggregate ID as string
    fn id_string(&self) -> String;

    /// Check if aggregate is new (version 0)
    fn is_new(&self) -> bool {
        self.version() == 0
    }
}

impl<T: AggregateRoot> AggregateExt for T {
    fn id_string(&self) -> String {
        format!("{:?}", self.id_ref())
    }
}
