//! Domain model base traits: Entity, AggregateRoot, ValueObject, DomainEvent.
//!
//! These are the foundational building blocks for domain-driven design.
//! Previously defined in `phenotype_contracts::models`.

use std::fmt::Debug;

// ---------------------------------------------------------------------------
// Entity
// ---------------------------------------------------------------------------

/// An object with persistent identity.
///
/// Two entities are equal when their IDs match, regardless of attribute values.
///
/// # Examples
///
/// ```rust
/// use phenotype_ports_canonical::Entity;
///
/// #[derive(Debug, Clone, PartialEq)]
/// struct UserId(String);
///
/// #[derive(Debug, Clone)]
/// struct User { id: UserId, name: String }
///
/// impl Entity for User {
///     type Id = UserId;
///     fn id(&self) -> &Self::Id { &self.id }
/// }
/// ```
pub trait Entity: Debug + Clone + Send + Sync {
    /// The identifier type for this entity.
    type Id: Clone + Send + Sync + PartialEq + Debug;

    /// Returns a reference to the entity's identifier.
    fn id(&self) -> &Self::Id;

    /// Returns `true` when two entities share the same identity.
    fn is_same(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

// ---------------------------------------------------------------------------
// ValueObject
// ---------------------------------------------------------------------------

/// An immutable object defined entirely by its attributes.
///
/// Value objects have no identity; equality is structural.
///
/// # Examples
///
/// ```rust
/// use phenotype_ports_canonical::ValueObject;
///
/// #[derive(Debug, Clone, PartialEq, Eq)]
/// struct Email(String);
///
/// impl ValueObject for Email {
///     fn validate(&self) -> Result<(), String> {
///         if self.0.contains('@') { Ok(()) }
///         else { Err("missing @".into()) }
///     }
/// }
/// ```
pub trait ValueObject: Debug + Clone + PartialEq + Eq + Send + Sync {
    /// Validate invariants of this value object.
    fn validate(&self) -> Result<(), String>;
}

// ---------------------------------------------------------------------------
// DomainEvent
// ---------------------------------------------------------------------------

/// A domain event that records something that happened in the aggregate.
///
/// Two flavours existed in `phenotype-contracts`:
///
/// 1. `models::aggregate::DomainEvent` (marker, no serde bounds)
/// 2. `ports::outbound::event::DomainEvent` (requires Serialize + Deserialize)
///
/// This canonical version uses the minimal marker form.  Serialization is the
/// responsibility of the adapter (EventPublisher) rather than the event itself.
pub trait DomainEvent: Send + Sync {
    /// Event type discriminator (used for routing / deserialization).
    fn event_type(&self) -> &str;

    /// Schema version for forward/backward compatibility.
    fn event_version(&self) -> u32;

    /// The aggregate this event belongs to.
    fn aggregate_id(&self) -> &str;
}

// ---------------------------------------------------------------------------
// AggregateRoot
// ---------------------------------------------------------------------------

/// The root entity of an aggregate that enforces invariants and emits events.
///
/// External code may only hold references to the aggregate root, never to
/// internal entities.
pub trait AggregateRoot: Entity {
    /// Optimistic-concurrency version counter.
    fn version(&self) -> u64;

    /// Drain pending domain events (consumed by the repository / event store).
    fn take_events(&mut self) -> Vec<Box<dyn DomainEvent>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    struct TestId(String);

    #[derive(Debug, Clone)]
    struct TestEntity {
        id: TestId,
    }

    impl Entity for TestEntity {
        type Id = TestId;
        fn id(&self) -> &Self::Id {
            &self.id
        }
    }

    #[test]
    fn entity_is_same() {
        let a = TestEntity { id: TestId("1".into()) };
        let b = TestEntity { id: TestId("1".into()) };
        let c = TestEntity { id: TestId("2".into()) };
        assert!(a.is_same(&b));
        assert!(!a.is_same(&c));
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Email(String);

    impl ValueObject for Email {
        fn validate(&self) -> Result<(), String> {
            if self.0.contains('@') {
                Ok(())
            } else {
                Err("invalid".into())
            }
        }
    }

    #[test]
    fn value_object_validate() {
        assert!(Email("a@b.c".into()).validate().is_ok());
        assert!(Email("bad".into()).validate().is_err());
    }

    struct TestEvent;
    impl DomainEvent for TestEvent {
        fn event_type(&self) -> &str { "TestEvent" }
        fn event_version(&self) -> u32 { 1 }
        fn aggregate_id(&self) -> &str { "agg-1" }
    }

    #[test]
    fn domain_event_accessors() {
        let e = TestEvent;
        assert_eq!(e.event_type(), "TestEvent");
        assert_eq!(e.event_version(), 1);
        assert_eq!(e.aggregate_id(), "agg-1");
    }
}
