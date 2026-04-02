//! Cross-platform shared utilities for the Phenotype ecosystem.
//!
//! This crate provides utilities intended for use across multiple platforms
//! (phenotype-infrakit, thegent, future platforms).
//!
//! ## Design Principles
//!
//! - **Zero internal dependencies**: Only use std and widely-adopted crates
//! - **No platform-specific code**: Pure Rust implementations preferred
//! - **Minimal footprint**: Small, focused utilities only
//!
//! ## Crate Organization
//!
//! - `error.rs` - Cross-platform error types
//! - `types.rs` - Shared type definitions
//! - `utils.rs` - General utilities

mod error;
mod types;
mod utils;

pub use error::{Error, Result};
pub use types::*;
pub use utils::*;
