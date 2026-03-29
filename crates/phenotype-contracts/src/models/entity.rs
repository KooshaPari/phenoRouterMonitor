//! # Entity Module
//!
//! Base entity types with identity and equality semantics.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A base entity with identity-based equality.
///
/// Entities are domain objects with a unique identifier.
/// Two entities are equal if and only if their IDs match.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Unique entity identifier.
    pub id: String,
    /// Optional entity type name.
    #[serde(default)]
    pub entity_type: Option<String>,
}

impl Entity {
    /// Create a new entity with the given ID.
    pub fn new(id: String) -> Self {
        Self { id, entity_type: None }
    }

    /// Create a new entity with ID and type.
    pub fn with_type(id: String, entity_type: impl Into<String>) -> Self {
        Self {
            id,
            entity_type: Some(entity_type.into()),
        }
    }

    /// Get the entity's ID as a string slice.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Check if this entity matches the given ID.
    pub fn is(&self, id: &str) -> bool {
        self.id == id
    }
}

impl PartialEq for Entity {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Entity {}

impl std::hash::Hash for Entity {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.entity_type {
            Some(t) => write!(f, "{}:{}", t, self.id),
            None => write!(f, "Entity({})", self.id),
        }
    }
}

impl From<String> for Entity {
    fn from(id: String) -> Self {
        Self::new(id)
    }
}

impl From<&str> for Entity {
    fn from(id: &str) -> Self {
        Self::new(id.to_string())
    }
}

/// Extension trait for entities.
pub trait EntityExt {
    /// Return the full display name.
    fn display_name(&self) -> String;
}

impl EntityExt for Entity {
    fn display_name(&self) -> String {
        self.to_string()
    }
}
