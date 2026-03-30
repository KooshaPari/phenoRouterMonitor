//! # Phenotype Errors
//!
//! Unified error types for the Phenotype ecosystem.

pub use phenotype_error_core::{Error, ErrorContext, ErrorExt, ErrorKind, ErrorKindInner, Result};

pub type ErrorAlias = Error;
