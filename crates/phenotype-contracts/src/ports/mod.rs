//! Port interfaces for hexagonal architecture.

// Inbound (driving) ports
pub mod inbound;

// Outbound (driven) ports
pub mod outbound;

pub use inbound::*;
pub use outbound::*;
