//! # Phenotype Contracts
//!
//! Hexagonal architecture ports and contracts for phenotype-infrakit.

pub mod models;
pub mod ports;

/// Result type alias for contract operations.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
