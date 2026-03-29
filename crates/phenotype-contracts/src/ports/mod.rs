//! # Ports Module
//!
//! Hexagonal architecture port interfaces.
//!
//! ## Port Types
//!
//! - **Inbound Ports**: Driving ports (use cases, commands, queries) that
//!   initiate actions. Defined in [`ports::inbound`].
//!
//! - **Outbound Ports**: Driven ports (repositories, cache, secrets, events)
//!   that are called by the domain. Defined in [`ports::outbound`].

pub mod inbound;
pub mod outbound;

pub use inbound::*;
pub use outbound::*;
