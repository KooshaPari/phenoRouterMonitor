//! Common domain models for hexagonal architecture.

use std::fmt;

/// Trait for entities that have a unique identity.
pub trait Entity: Send + Sync {
    type Id: Clone + fmt::Debug + Send + Sync + 'static;
    fn id(&self) -> &Self::Id;
}

/// Trait for immutable value objects.
pub trait ValueObject: Clone + Eq + fmt::Debug + Send + Sync {
    fn validate(&self) -> Result<(), String>;
}

/// Trait for aggregate roots in domain-driven design.
pub trait AggregateRoot: Entity {
    type Event: DomainEvent;
    fn uncommitted_events(&self) -> &[Self::Event];
    fn clear_events(&mut self);
    fn apply_event(&mut self, event: Self::Event);
}

/// Trait for domain events in event sourcing.
pub trait DomainEvent: Clone + fmt::Debug + Send + Sync + 'static {
    type AggregateId: Clone + fmt::Debug + Send + Sync + 'static;
    fn aggregate_id(&self) -> &Self::AggregateId;
    fn version(&self) -> u64;
    fn timestamp(&self) -> &str;
    fn event_type(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct TestValueObject {
        value: String,
    }

    impl ValueObject for TestValueObject {
        fn validate(&self) -> Result<(), String> {
            if self.value.is_empty() {
                Err("value must not be empty".to_string())
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn test_value_object_validation() {
        let valid = TestValueObject {
            value: "test".to_string(),
        };
        assert!(valid.validate().is_ok());

        let invalid = TestValueObject {
            value: String::new(),
        };
        assert!(invalid.validate().is_err());
    }
}
