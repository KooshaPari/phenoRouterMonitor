//! NATS-style event bus for AgilePlus.
//!
//! Provides a trait-based event bus abstraction with typed domain events,
//! hierarchical subject routing, and request/reply semantics. The default
//! implementation is an in-memory bus suitable for testing; a NATS-backed
//! implementation can be plugged in when the real broker is available.
//!
//! Traceability: WP06 — Event Bus

pub mod bus;
pub mod config;
pub mod envelope;
pub mod handler;
pub mod health;
pub mod subject;

pub use bus::{EventBus, EventBusError, EventBusStore, InMemoryBus};
pub use config::NatsConfig;
pub use envelope::Envelope;
pub use handler::Handler;
pub use health::BusHealth;
pub use subject::Subject;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Event bus error: {0}")]
    Bus(#[from] EventBusError),
    #[error("Config error: {0}")]
    Config(String),
}
