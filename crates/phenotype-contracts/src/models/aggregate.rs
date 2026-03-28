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

#[cfg(test)]
mod tests {
    use super::*;

    // Test aggregate ID
    #[derive(Debug, Clone, PartialEq)]
    struct TestAggregateId(String);

    impl TestAggregateId {
        fn new(id: &str) -> Self {
            Self(id.to_string())
        }
    }

    // Test domain event
    #[derive(Debug, Clone)]
    struct TestDomainEvent {
        aggregate_id: String,
        _event_data: String,
    }

    impl TestDomainEvent {
        fn new(aggregate_id: &str, data: &str) -> Self {
            Self {
                aggregate_id: aggregate_id.to_string(),
                _event_data: data.to_string(),
            }
        }
    }

    impl DomainEvent for TestDomainEvent {
        fn event_type(&self) -> &'static str {
            "TestDomainEvent"
        }

        fn event_version(&self) -> u32 {
            1
        }

        fn aggregate_id(&self) -> &str {
            &self.aggregate_id
        }
    }

    // Test aggregate
    #[derive(Debug, Clone)]
    struct TestAggregate {
        id: TestAggregateId,
        version: u64,
        _name: String,
    }
    // Note: pending_events removed for Clone derive - would need Box<dyn DomainEvent + Clone> in production

    impl TestAggregate {
        fn new(id: &str, name: &str) -> Self {
            Self {
                id: TestAggregateId::new(id),
                version: 0,
                _name: name.to_string(),
            }
        }
    }

    impl Entity for TestAggregate {
        type Id = TestAggregateId;

        fn id(&self) -> &Self::Id {
            &self.id
        }
    }

    impl AggregateRoot for TestAggregate {
        fn version(&self) -> u64 {
            self.version
        }

        fn take_events(&mut self) -> Vec<Box<dyn DomainEvent>> {
            Vec::new() // Empty implementation for test
        }
    }

    #[test]
    fn test_aggregate_id_access() {
        // Test aggregate creation
        let aggregate = TestAggregate::new("agg-123", "TestAggregate");
        assert_eq!(aggregate.id().0, "agg-123");
        assert_eq!(aggregate.version(), 0);
    }

    #[test]
    fn test_aggregate_is_new() {
        let aggregate = TestAggregate::new("agg-123", "Test");
        assert!(aggregate.is_new());
    }

    #[test]
    fn test_aggregate_id_string() {
        let aggregate = TestAggregate::new("agg-123", "Test");
        assert_eq!(aggregate.id_string(), "TestAggregateId(\"agg-123\")");
    }

    #[test]
    fn test_domain_event() {
        let event = TestDomainEvent::new("agg-123", "test data");

        assert_eq!(event.event_type(), "TestDomainEvent");
        assert_eq!(event.event_version(), 1);
        assert_eq!(event.aggregate_id(), "agg-123");
    }
}
