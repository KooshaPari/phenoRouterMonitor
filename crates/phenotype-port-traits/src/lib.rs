<<<<<<< HEAD
//! # Phenotype Port Traits
//!
//! Canonical async trait definitions for hexagonal architecture ports.
//! This crate consolidates the duplicated port traits scattered across Phenotype crates.
//!
//! ## Port Categories
//!
//! - **Inbound Ports**: Use cases, command handlers, query handlers, event handlers
//! - **Outbound Ports**: Repositories, cache, event bus, secrets, external services
//!
//! ## Usage
//!
//! ```rust
//! use phenotype_port_traits::outbound::{Repository, CachePort};
//! use async_trait::async_trait;
//!
//! #[async_trait]
//! impl Repository<Entity, EntityId> for SqliteRepository {
//!     async fn save(&self, entity: &Entity) -> Result<(), RepositoryError> { ... }
//!     async fn find_by_id(&self, id: &EntityId) -> Result<Option<Entity>, RepositoryError> { ... }
//! }
//! ```

pub mod inbound;
pub mod outbound;

// Re-export common types
pub use async_trait::async_trait;
=======
// phenotype-port-traits
>>>>>>> origin/main
