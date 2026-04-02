//! Validation framework for phenotype

pub mod error;
pub mod types;
pub mod ports;
pub mod schema;
pub mod validator;
pub mod context;
pub mod rules;

pub use error::{ValidationError, Result};
pub use types::{ValidationResult, ValidationContext, Severity};
pub use ports::ValidatorPort;
pub use schema::JsonSchemaAdapter;
pub use validator::Validator;
pub use context::ValidationContext;
