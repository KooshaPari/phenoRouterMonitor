//! # phenotype-router
//!
//! Pareto-efficient routing engine with audit trails and hysteresis control.
//! Extracted from thegent-router for use across the Phenotype ecosystem.
//!
//! ## Key Features
//! - Pareto-efficient task routing (cost, latency, reliability tradeoffs)
//! - Immutable SHA-256 hash-chain audit trails
//! - Hysteresis-aware decision making (prevents oscillation)
//! - Task execution tracking and coordination
//! - Risk assessment and failover logic
//! - Python FFI bindings (optional, pyo3)

pub mod router;
pub mod audit;
pub mod hysteresis;
pub mod executor;
pub mod risk;
pub mod orchestrator;

#[cfg(feature = "python-ffi")]
pub mod python;

pub use router::Router;
pub use audit::AuditChain;
pub use hysteresis::HysteresisState;
pub use executor::Executor;
