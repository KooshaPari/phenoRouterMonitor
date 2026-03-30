//! Error types for Nexus

use thiserror::Error;

/// Errors that can occur in Nexus operations
#[derive(Error, Debug)]
pub enum NexusError {
    /// Service not found in registry
    #[error("Service not found: {0}")]
    NotFound(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Registration failed
    #[error("Registration failed: {0}")]
    RegistrationFailed(String),

    /// Discovery failed
    #[error("Discovery failed: {0}")]
    DiscoveryFailed(String),
}
