//! # Phenotype String
//!
//! Canonical string utilities for Phenotype services.
//!
//! This crate consolidates the duplicated string manipulation patterns scattered across the codebase.

pub mod sanitize;
pub mod parse;
pub mod join;

pub use sanitize::Sanitize;
pub use parse::ParseExt;
pub use join::JoinExt;

/// Re-export for convenience.
pub use std::string::ToString;
