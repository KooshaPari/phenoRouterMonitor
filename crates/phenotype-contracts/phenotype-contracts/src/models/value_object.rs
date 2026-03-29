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

#[cfg(test)]
mod tests {
    use super::*;

    // Test value object
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestEmail(String);

    impl_value_object!(TestEmail, |s: &TestEmail| {
        if s.0.contains('@') {
            Ok(())
        } else {
            Err("Invalid email format".into())
        }
    });

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct TestMoney {
        amount: i64,
        currency: String,
    }

    impl_value_object!(TestMoney, |m: &TestMoney| {
        if m.amount < 0 {
            Err("Amount cannot be negative".into())
        } else {
            Ok(())
        }
    });

    #[test]
    fn test_value_object_validate_valid() {
        let email = TestEmail("test@example.com".to_string());
        assert!(email.validate().is_ok());
    }

    #[test]
    fn test_value_object_validate_invalid() {
        let email = TestEmail("invalid-email".to_string());
        assert!(email.validate().is_err());
        assert_eq!(email.validate().unwrap_err(), "Invalid email format");
    }

    #[test]
    fn test_value_object_equality() {
        let email1 = TestEmail("test@example.com".to_string());
        let email2 = TestEmail("test@example.com".to_string());
        let email3 = TestEmail("other@example.com".to_string());

        assert_eq!(email1, email2);
        assert_ne!(email1, email3);
    }

    #[test]
    fn test_value_object_immutability() {
        let email = TestEmail("test@example.com".to_string());
        let email_clone = email.clone();

        assert_eq!(email, email_clone);
    }

    #[test]
    fn test_value_object_complex() {
        let money = TestMoney {
            amount: 100,
            currency: "USD".to_string(),
        };
        assert!(money.validate().is_ok());
    }

    #[test]
    fn test_value_object_negative_amount() {
        let money = TestMoney {
            amount: -50,
            currency: "USD".to_string(),
        };
        let result = money.validate();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Amount cannot be negative");
    }
}
