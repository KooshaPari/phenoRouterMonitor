//! # Entity
//!
//! Entities are objects with identity that persists over time.
//!
//! ## Characteristics
//!
//! - **Identity**: Has a unique ID
//! - **Mutable**: Can change state over time
//! - **Equality by ID**: Two entities are equal if their IDs are equal
//!
//! ## Examples
//!
//! - `User`: Identified by user ID
//! - `Order`: Identified by order ID
//! - `Product`: Identified by product ID
//!
//! ## Usage
//!
//! ```rust,ignore
//! use uuid::Uuid;
//! use phenotype_contracts::models::Entity;
//!
//! #[derive(Debug, Clone)]
//! struct UserId(String);
//!
//! impl UserId {
//!     pub fn new() -> Self {
//!         Self(Uuid::new_v4().to_string())
//!     }
//! }
//!
//! #[derive(Debug, Clone)]
//! struct User {
//!     id: UserId,
//!     email: String,
//!     name: String,
//! }
//!
//! impl Entity for User {
//!     type Id = UserId;
//!
//!     fn id(&self) -> &Self::Id {
//!         &self.id
//!     }
//! }
//! ```

use std::fmt::Debug;
use std::cmp::PartialEq;

/// Trait for entities.
///
/// Entities have identity that persists over time.
pub trait Entity: Debug + Clone + Send + Sync {
    /// The type of ID this entity uses
    type Id: Clone + Send + Sync + PartialEq + Debug;

    /// Get the entity's ID
    fn id(&self) -> &Self::Id;

    /// Check if this entity is the same as another (by ID)
    fn is_same(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

/// Extension trait for entity operations
pub trait EntityExt: Entity {
    /// Get a reference to the ID
    fn id_ref(&self) -> Self::Id;
}

impl<T: Entity> EntityExt for T {
    fn id_ref(&self) -> Self::Id {
        self.id().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test implementation of Entity for testing
    #[derive(Debug, Clone, PartialEq)]
    struct TestId(String);

    impl TestId {
        fn new(id: &str) -> Self {
            Self(id.to_string())
        }
    }

    #[derive(Debug, Clone)]
    struct TestEntity {
        id: TestId,
        name: String,
    }

    impl Entity for TestEntity {
        type Id = TestId;

        fn id(&self) -> &Self::Id {
            &self.id
        }
    }

    #[test]
    fn test_entity_id_access() {
        let entity = TestEntity {
            id: TestId::new("test-123"),
            name: "Test".to_string(),
        };

        assert_eq!(entity.id().0, "test-123");
    }

    #[test]
    fn test_entity_is_same() {
        let entity1 = TestEntity {
            id: TestId::new("same-id"),
            name: "Entity 1".to_string(),
        };
        let entity2 = TestEntity {
            id: TestId::new("same-id"),
            name: "Entity 2".to_string(),
        };
        let entity3 = TestEntity {
            id: TestId::new("different-id"),
            name: "Entity 1".to_string(),
        };

        assert!(entity1.is_same(&entity2));
        assert!(!entity1.is_same(&entity3));
    }

    #[test]
    fn test_entity_ext_id_ref() {
        let entity = TestEntity {
            id: TestId::new("ref-test"),
            name: "Test".to_string(),
        };

        let id: TestId = entity.id_ref();
        assert_eq!(id.0, "ref-test");
    }

    #[test]
    fn test_entity_id_clone() {
        let entity = TestEntity {
            id: TestId::new("clone-test"),
            name: "Test".to_string(),
        };

        let cloned_id = entity.id().clone();
        assert_eq!(cloned_id.0, "clone-test");
    }
}
