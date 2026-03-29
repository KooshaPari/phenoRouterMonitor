//! # Outbound Ports
//!
//! Driven ports that represent dependencies the domain needs.
//! These are implemented by adapters (infrastructure layer).
//!
//! ## Port Types
//!
//! - [`Repository`] - Data persistence operations
//! - [`CachePort`] - Caching operations
//! - [`SecretPort`] - Secrets management
//! - [`EventPublisher`] - Event publishing
//! - [`EventSubscriber`] - Event consuming
//!
//! ## Dependency Inversion Principle
//!
//! The domain defines these interfaces, and adapters implement them.
//! This allows the domain to be independent of infrastructure concerns.
//!
//! ```text
//! ┌──────────────┐       ┌──────────────┐       ┌──────────────┐
//! │    Domain    │       │    Ports     │       │   Adapters   │
//! │              │──────▶│ (Interfaces) │──────▶│(Implementations│
//! │  Business    │       │              │       │              │
//! │  Logic       │       │ - Repository │       │ - Postgres   │
//! │              │       │ - CachePort  │       │ - Redis      │
//! │              │       │ - SecretPort │       │ - Vault      │
//! └──────────────┘       └──────────────┘       └──────────────┘
//! ```
//!
//! ## Examples
//!
//! ```rust,ignore
//! use async_trait::async_trait;
//! use phenotype_contracts::ports::outbound::{Repository, Error};
//!
//! // Domain defines the interface
//! #[async_trait]
//! impl Repository<User, UserId> for UserRepository {
//!     async fn find_by_id(&self, id: UserId) -> Result<Option<User>, Error> {
//!         // adapter implementation
//!     }
//!
//!     async fn save(&self, user: User) -> Result<(), Error> {
//!         // adapter implementation
//!     }
//! }
//! ```

pub mod cache;
pub mod repository;
pub mod secret;
pub mod event;

pub use cache::CachePort;
pub use repository::Repository;
pub use secret::SecretPort;
pub use event::{EventPublisher, EventSubscriber};

use thiserror::Error;

/// Error type for outbound port operations
#[derive(Debug, Error)]
pub enum Error {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("already exists: {0}")]
    AlreadyExists(String),

    #[error("connection error: {0}")]
    Connection(String),

    #[error("timeout: {0}")]
    Timeout(String),

    #[error("validation: {0}")]
    Validation(String),

    #[error("internal: {0}")]
    Internal(String),
}
