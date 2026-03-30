//! Phenotype Validation Rule System

pub mod errors;
pub mod presets;
pub mod rules;
pub mod validator;

pub use errors::{ValidationError, ValidationErrors};
pub use presets::*;
pub use rules::ValidationRule;
pub use validator::{ValidateFields, Validator, ValidatorBuilder};

use regex::Regex;

pub fn required(value: &str, field: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        Err(ValidationError::new(field, "is required", "required"))
    } else {
        Ok(())
    }
}

pub fn min_length(value: &str, min: usize, field: &str) -> Result<(), ValidationError> {
    if value.len() < min {
        Err(ValidationError::new(field, format!("must be at least {} characters", min), "min_length"))
    } else { Ok(()) }
}

pub fn max_length(value: &str, max: usize, field: &str) -> Result<(), ValidationError> {
    if value.len() > max {
        Err(ValidationError::new(field, format!("must be at most {} characters", max), "max_length"))
    } else { Ok(()) }
}

pub fn pattern(value: &str, regex: &Regex, field: &str) -> Result<(), ValidationError> {
    if !regex.is_match(value) {
        Err(ValidationError::new(field, "does not match required pattern", "pattern"))
    } else { Ok(()) }
}

pub fn range<T: PartialOrd + std::fmt::Display>(value: T, min: T, max: T, field: &str) -> Result<(), ValidationError> {
    if value < min || value > max {
        Err(ValidationError::new(field, format!("must be between {} and {}", min, max), "range"))
    } else { Ok(()) }
}

pub fn email(value: &str, field: &str) -> Result<(), ValidationError> {
    let email_re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    if !email_re.is_match(value) {
        Err(ValidationError::new(field, "is not a valid email", "email"))
    } else { Ok(()) }
}

pub trait Validatable {
    fn validate(&self) -> Result<(), ValidationErrors>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_function() {
        assert!(required("hello", "name").is_ok());
        assert!(required("", "name").is_err());
    }

    #[test]
    fn test_email_function() {
        assert!(email("user@example.com", "email").is_ok());
        assert!(email("not-an-email", "email").is_err());
    }

    #[test]
    fn test_validatable_trait() {
        struct User {
            name: String,
            email_addr: String,
        }

        impl Validatable for User {
            fn validate(&self) -> Result<(), ValidationErrors> {
                let mut errors = ValidationErrors::new();
                errors.add_if_err(required(&self.name, "name"));
                errors.add_if_err(email(&self.email_addr, "email"));
                errors.into_result()
            }
        }

        let valid = User { name: "Alice".into(), email_addr: "alice@test.com".into() };
        assert!(valid.validate().is_ok());

        let invalid = User { name: "".into(), email_addr: "bad".into() };
        assert!(invalid.validate().is_err());
    }
}
