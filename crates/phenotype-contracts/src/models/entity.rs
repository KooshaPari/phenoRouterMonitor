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
