//! Trait definitions for the validation framework.
//!
//! Exports the core trait hierarchy:
//! - `ValidationRule`: Individual validation rules (composable)
//! - `FieldValidator`: Multi-rule field validation
//! - `CommandValidator`: Command-level validation orchestration

pub mod rule;
pub mod field_validator;
pub mod command_validator;
pub mod error;

pub use rule::ValidationRule;
pub use field_validator::FieldValidator;
pub use command_validator::CommandValidator;
pub use error::{ValidationError, Severity, ValidationContext};
