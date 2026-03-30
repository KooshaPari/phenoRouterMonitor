//! # Phenotype Time
//!
//! Canonical time and duration utilities for Phenotype services.
//!
//! This crate consolidates the duplicated Duration patterns scattered across the codebase.
//!
//! ## Common Duration Constants
//!
//! ```rust
//! use phenotype_time::{Duration, SECONDS, MINUTES, HOURS};
//!
//! let timeout = *MINUTES * 5;
//! ```

pub mod duration;
pub mod timestamp;
pub mod constants;

pub use duration::DurationExt;
pub use timestamp::{Timestamp, TimestampKind};
pub use constants::*;
