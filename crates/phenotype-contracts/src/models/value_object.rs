//! # Value Object
//!
//! Value objects are immutable objects defined by their attributes.
//!
//! ## Characteristics
//!
//! - **Immutable**: Once created, cannot be modified
//! - **Value-based equality**: Two value objects are equal if their attributes are equal
//! - **No identity**: They don't have an ID
//!
//! ## Examples
//!
//! - `Email`: Defined by its address
//! - `Money`: Defined by amount and currency
//! - `Address`: Defined by street, city, etc.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use phenotype_contracts::models::ValueObject;
//!
//! #[derive(Debug, Clone, PartialEq, Eq)]
//! struct Email(String);
//!
//! impl ValueObject for Email {
//!     fn validate(&self) -> Result<(), String> {
//!         if self.0.contains('@') {
//!             Ok(())
//!         } else {
//!             Err("Invalid email".to_string())
//!         }
//!     }
//! }
//! ```

use std::fmt::Debug;

/// Trait for value objects.
///
/// Value objects are immutable and compared by their attributes.
pub trait ValueObject: Debug + Clone + PartialEq + Eq + Send + Sync {
    /// Validate the value object
    fn validate(&self) -> Result<(), String>;
}

/// Macro to implement ValueObject for simple types
///
/// # Example
///
/// ```rust,ignore
/// use phenotype_contracts::models::value_object::impl_value_object;
///
/// struct Email(String);
///
/// impl_value_object!(Email, |s: &Email| {
///     if s.0.contains('@') { Ok(()) } else { Err("invalid".into()) }
/// });
/// ```
#[macro_export]
macro_rules! impl_value_object {
    ($type:ty, $validate:expr) => {
        impl $crate::models::ValueObject for $type {
            fn validate(&self) -> Result<(), String> {
                $validate(self)
            }
        }
    };
}
