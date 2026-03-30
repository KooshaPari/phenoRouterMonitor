//! Nexus - Service Registry and Discovery
//!
//! A service registry and discovery library with hash-consign based state management.
//!
//! # Example
//!
//! ```ignore
//! use nexus::{Registry, Service, Endpoint};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let registry = Registry::new();
//! registry.register(Service::new("api", Endpoint::new("localhost:8080"))).await?;
//! let services = registry.discover("api").await?;
//! # Ok(())
//! # }
//! ```

pub mod registry;
pub mod service;
pub mod discovery;
pub mod error;
pub mod health;

pub use registry::Registry;
pub use service::{Service, Endpoint};
pub use discovery::Discovery;
pub use error::NexusError;
pub use health::{HealthMonitor, HealthStatus, HealthCheckConfig, ServiceHealth};
