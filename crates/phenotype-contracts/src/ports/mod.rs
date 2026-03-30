//! # Ports Module
//!
//! Ports define the interfaces (contracts) for the hexagonal architecture.
//!
//! ## Port Types
//!
//! ### Inbound Ports (Driving)
//!
//! These are interfaces that the **domain** exposes for external actors to call.
//! They represent the actions the system can perform.
//!
//! - [`UseCase`] - Generic use case interface
//! - [`CommandHandler`] - Handles commands (CQRS)
//! - [`QueryHandler`] - Handles queries (CQRS)
//!
//! ### Outbound Ports (Driven)
//!
//! These are interfaces that the **domain** uses to interact with external systems.
//! They represent dependencies the domain needs to fulfill its responsibilities.
//!
//! - [`Repository`] - Data persistence
//! - [`CachePort`] - Caching operations
//! - [`SecretPort`] - Secrets management
//! - [`EventPublisher`] - Event publishing
//! - [`EventSubscriber`] - Event consuming

pub mod inbound;
pub mod outbound;
