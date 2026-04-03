//! Ports layer - Primary/driving and secondary/driven interfaces
//!
//! Following hexagonal architecture, these define the interfaces
//! that the application uses to interact with external systems.

pub mod driven;
pub mod driving;

pub use driven::{StoragePort, LoaderPort, SandboxPort, EventPort};
pub use driving::{ApiPort, CliPort};
