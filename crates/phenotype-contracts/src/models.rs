//! Domain model marker traits for hexagonal architecture.
//!
//! Traces to: FR-PHENO-002

use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use std::hash::Hash;

/// Marker trait for entity identifiers.
///
/// Identifiers must be cloneable, hashable, displayable, serializable,
/// and thread-safe.
pub trait EntityId:
    Debug + Clone + PartialEq + Eq + Hash + Send + Sync + Serialize + DeserializeOwned + 'static
{
}

// Blanket implementation: any type satisfying the bounds is an `EntityId`.
impl<T> EntityId for T where
    T: Debug + Clone + PartialEq + Eq + Hash + Send + Sync + Serialize + DeserializeOwned + 'static
{
}

/// A domain entity with a stable identity.
pub trait DomainEntity: Debug + Clone + Send + Sync + 'static {
    /// The type of this entity's identifier.
    type Id: EntityId;

    /// Returns the entity's unique identifier.
    fn id(&self) -> &Self::Id;
}

/// A value object -- equality is based on all fields, not identity.
///
/// Value objects are immutable and compared structurally.
pub trait ValueObject: Debug + Clone + PartialEq + Eq + Send + Sync + 'static {}

/// A domain event representing something that happened in the domain.
pub trait DomainEvent: Debug + Clone + Send + Sync + Serialize + DeserializeOwned + 'static {
    /// A human-readable name for this event type (e.g. `"OrderPlaced"`).
    fn event_type(&self) -> &str;
}

/// An aggregate root -- the consistency boundary for a cluster of entities.
///
/// Aggregate roots track uncommitted domain events that occurred during
/// the current unit of work.
pub trait AggregateRoot: DomainEntity {
    /// The domain event type emitted by this aggregate.
    type Event: DomainEvent;

    /// Returns a slice of uncommitted domain events.
    fn pending_events(&self) -> &[Self::Event];

    /// Clears (drains) uncommitted events after they have been persisted.
    fn clear_events(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct UserCreated {
        user_id: String,
    }

    impl DomainEvent for UserCreated {
        fn event_type(&self) -> &str {
            "UserCreated"
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Email(String);
    impl ValueObject for Email {}

    #[derive(Debug, Clone)]
    struct User {
        id: String,
        events: Vec<UserCreated>,
    }

    impl DomainEntity for User {
        type Id = String;
        fn id(&self) -> &String {
            &self.id
        }
    }

    impl AggregateRoot for User {
        type Event = UserCreated;

        fn pending_events(&self) -> &[UserCreated] {
            &self.events
        }

        fn clear_events(&mut self) {
            self.events.clear();
        }
    }

    // Traces to: FR-PHENO-002
    #[test]
    fn entity_id_and_identity() {
        let user = User {
            id: "u-1".into(),

            events: vec![],
        };
        assert_eq!(user.id(), "u-1");
    }

    // Traces to: FR-PHENO-002
    #[test]
    fn value_object_equality() {
        let a = Email("a@b.com".into());
        let b = Email("a@b.com".into());
        assert_eq!(a, b);
    }

    // Traces to: FR-PHENO-002
    #[test]
    fn aggregate_root_events() {
        let mut user = User {
            id: "u-2".into(),

            events: vec![UserCreated {
                user_id: "u-2".into(),
            }],
        };
        assert_eq!(user.pending_events().len(), 1);
        assert_eq!(user.pending_events()[0].event_type(), "UserCreated");
        user.clear_events();
        assert!(user.pending_events().is_empty());
    }
}
