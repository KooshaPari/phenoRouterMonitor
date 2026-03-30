//! # phenotype-monitor-contracts
//!
//! Shared traits and types for the phenotype-router-monitor ecosystem.
//! Defines the contract boundaries between router, metrics, and metering domains.

pub mod error;
pub mod router;
pub mod metrics;
pub mod meter;

pub use error::MonitorError;
pub use router::*;
pub use metrics::*;
pub use meter::*;
