//! # Inbound Ports
//!
//! Driving ports that represent the actions the system can perform.
//! These are implemented by the domain/application layer.
//!
//! ## CQRS Pattern
//!
//! Inbound ports follow the CQRS (Command Query Responsibility Segregation) pattern:
//!
//! - **Commands**: Operations that modify state (Create, Update, Delete)
//! - **Queries**: Operations that read state without modifying it
//!
//! ## Generic Interfaces
//!
//! - [`UseCase`] - Generic use case with input/output
//! - [`CommandHandler<C>`] - Handles commands, returns `Result<(), Error>`
//! - [`QueryHandler<Q, R>`] - Handles queries, returns `Result<R, Error>`
//!
//! ## Examples
//!
//! ```rust,ignore
//! use phenotype_contracts::ports::inbound::{CommandHandler, QueryHandler};
//!
//! // Command handler
//! struct SetCacheCommand {
//!     key: String,
//!     value: String,
//!     ttl: Duration,
//! }
//!
//! impl CommandHandler<SetCacheCommand> for CacheService {
//!     async fn handle(&self, cmd: SetCacheCommand) -> Result<(), Error> {
//!         // implementation
//!     }
//! }
//!
//! // Query handler
//! struct GetCacheQuery {
//!     key: String,
//! }
//!
//! impl QueryHandler<GetCacheQuery, String> for CacheService {
//!     async fn handle(&self, query: GetCacheQuery) -> Result<String, Error> {
//!         // implementation
//!     }
//! }
//! ```

use async_trait::async_trait;

/// Generic use case interface.
///
/// # Type Parameters
///
/// - `I`: Input type (parameters for the use case)
/// - `O`: Output type (result of the use case)
///
/// # Examples
///
/// ```rust,ignore
/// use phenotype_contracts::ports::inbound::UseCase;
///
/// struct CalculateDiscount {
///     customer_id: String,
///     order_total: f64,
/// }
///
/// struct Discount(f64);
///
/// impl UseCase<CalculateDiscount, Discount> for PricingService {
///     async fn execute(&self, input: CalculateDiscount) -> Result<Discount, Error> {
///         // business logic
///     }
/// }
/// ```
#[async_trait]
pub trait UseCase<I, O>: Send + Sync {
    /// Execute the use case with the given input
    async fn execute(&self, input: I) -> Result<O, Error>;
}

/// Error type for port operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("entity not found: {0}")]
    NotFound(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("timeout: {0}")]
    Timeout(String),
}

/// Command handler trait for CQRS pattern.
///
/// Commands are operations that modify state.
/// Returns `Result<(), Error>` (no output data).
///
/// # Type Parameters
///
/// - `C`: Command type
///
/// # Examples
///
/// ```rust,ignore
/// use async_trait::async_trait;
/// use phenotype_contracts::ports::inbound::{CommandHandler, Error};
///
/// struct CreateOrderCommand {
///     customer_id: String,
///     items: Vec<OrderItem>,
/// }
///
/// #[async_trait]
/// impl CommandHandler<CreateOrderCommand> for OrderService {
///     async fn handle(&self, cmd: CreateOrderCommand) -> Result<(), Error> {
///         // validate and create order
///     }
/// }
/// ```
#[async_trait]
pub trait CommandHandler<C>: Send + Sync {
    /// Handle the command
    async fn handle(&self, command: C) -> Result<(), Error>;
}

/// Query handler trait for CQRS pattern.
///
/// Queries are operations that read state without modifying it.
/// Returns `Result<R, Error>` where `R` is the query result.
///
/// # Type Parameters
///
/// - `Q`: Query type
/// - `R`: Result type
///
/// # Examples
///
/// ```rust,ignore
/// use async_trait::async_trait;
/// use phenotype_contracts::ports::inbound::{QueryHandler, Error};
///
/// struct GetOrderQuery {
///     order_id: String,
/// }
///
/// struct OrderDetails { ... }
///
/// #[async_trait]
/// impl QueryHandler<GetOrderQuery, OrderDetails> for OrderService {
///     async fn handle(&self, query: GetOrderQuery) -> Result<OrderDetails, Error> {
///         // fetch and return order details
///     }
/// }
/// ```
#[async_trait]
pub trait QueryHandler<Q, R>: Send + Sync {
    /// Handle the query
    async fn handle(&self, query: Q) -> Result<R, Error>;
}

/// Event handler trait for domain events.
///
/// Handles events produced by the domain.
/// Used for event-driven architectures.
///
/// # Type Parameters
///
/// - `E`: Event type
///
/// # Examples
///
/// ```rust,ignore
/// use async_trait::async_trait;
/// use phenotype_contracts::ports::inbound::{EventHandler, Error};
///
/// struct OrderCreatedEvent {
///     order_id: String,
///     customer_id: String,
///     total: f64,
/// }
///
/// #[async_trait]
/// impl EventHandler<OrderCreatedEvent> for OrderEventHandler {
///     async fn handle(&self, event: OrderCreatedEvent) -> Result<(), Error> {
///         // send notification, update analytics, etc.
///     }
/// }
/// ```
#[async_trait]
pub trait EventHandler<E>: Send + Sync {
    /// Handle the event
    async fn handle(&self, event: E) -> Result<(), Error>;
}
