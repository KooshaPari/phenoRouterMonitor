//! # Inbound Ports (Driving Ports)
//!
//! Inbound ports define the interfaces for driving the application.
//! These are typically use cases, command handlers, and query handlers.

/// A marker trait for inbound (driving) ports.
pub trait InboundPort: Send + Sync {}

/// A marker trait for command handlers.
pub trait CommandHandler: Send + Sync {}

/// A marker trait for query handlers.
pub trait QueryHandler: Send + Sync {}

/// A marker trait for use cases.
pub trait UseCase: Send + Sync {}
