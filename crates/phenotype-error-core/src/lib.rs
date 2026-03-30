//! # phenotype-error-core (Backward Compatibility Re-export)
//!
//! This crate has been consolidated into `phenotype-errors` which provides a
//! unified error hierarchy for the entire Phenotype ecosystem.
//!
//! All error types and result type aliases are re-exported from `phenotype-errors`
//! for backward compatibility with code that previously imported from this crate.
//!
//! ## Migration
//!
//! For new code, import directly from `phenotype-errors`:
//! ```rust,ignore
//! use phenotype_errors::{PhenotypeError, Result};
//! ```

// Re-export all types from phenotype-errors for backward compatibility
pub use phenotype_errors::{PhenotypeError, Result};
