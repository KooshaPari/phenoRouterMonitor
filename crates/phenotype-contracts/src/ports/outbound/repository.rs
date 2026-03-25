//! # Repository Port
//!
//! Outbound port for data persistence.
//!
//! ## Generic Interface
//!
//! ```rust,ignore
//! use async_trait::async_trait;
//! use phenotype_contracts::ports::outbound::Repository;
//!
//! #[async_trait]
//! impl Repository<User, UserId> for PostgresUserRepository {
//!     async fn find_by_id(&self, id: UserId) -> Result<Option<User>, Error>;
//!     async fn save(&self, user: User) -> Result<(), Error>;
//!     async fn delete(&self, id: UserId) -> Result<(), Error>;
//!     async fn find_all(&self) -> Result<Vec<User>, Error>;
//! }
//! ```
//!
//! ## Design Principles
//!
//! - **Information Expert**: Repository knows how to persist entities
//! - **Low Coupling**: Domain doesn't know about persistence details
//! - **Pure Fabrication**: Repository is a created service, not domain entity

use async_trait::async_trait;

use super::Error;

/// Repository port interface.
///
/// Generic repository for data persistence.
/// 
/// # Type Parameters
///
/// - `E`: Entity type (the domain object)
/// - `I`: Identifier type (the entity's ID)
///
/// # Examples
///
/// ```rust,ignore
/// use async_trait::async_trait;
/// use uuid::Uuid;
/// use phenotype_contracts::ports::outbound::Repository;
///
/// #[derive(Clone)]
/// struct UserId(String);
///
/// #[derive(Clone)]
/// struct User {
///     id: UserId,
///     email: String,
///     name: String,
/// }
///
/// struct PostgresUserRepository { pool: PgPool }
///
/// #[async_trait]
/// impl Repository<User, UserId> for PostgresUserRepository {
///     async fn find_by_id(&self, id: UserId) -> Result<Option<User>, Error> {
///         // query database
///     }
///     
///     async fn save(&self, user: User) -> Result<(), Error> {
///         // insert or update
///     }
///     
///     async fn delete(&self, id: UserId) -> Result<(), Error> {
///         // delete row
///     }
/// }
/// ```
#[async_trait]
pub trait Repository<E, I>: Send + Sync
where
    E: Send + Sync,
    I: Send + Sync,
{
    /// Find entity by ID
    async fn find_by_id(&self, id: I) -> Result<Option<E>, Error>;

    /// Save entity (insert or update)
    async fn save(&self, entity: E) -> Result<(), Error>;

    /// Delete entity by ID
    async fn delete(&self, id: I) -> Result<(), Error>;

    /// Find all entities
    async fn find_all(&self) -> Result<Vec<E>, Error>;

    /// Find entities matching a filter
    async fn find_by(&self, filter: &str) -> Result<Vec<E>, Error> {
        let _ = filter;
        Ok(vec![])
    }
}

/// Unit of Work pattern for transactional operations.
///
/// Coordinates multiple repositories in a single transaction.
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    /// Start the unit of work (begin transaction)
    async fn begin(&self) -> Result<(), Error>;

    /// Commit all changes
    async fn commit(&self) -> Result<(), Error>;

    /// Rollback all changes
    async fn rollback(&self) -> Result<(), Error>;
}
