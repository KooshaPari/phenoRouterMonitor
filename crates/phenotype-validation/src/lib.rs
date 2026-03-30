pub mod validators;

pub use validators::{is_valid_email, is_valid_phone, is_valid_url, is_valid_uuid};

use regex::Regex;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
#[error("{field}: {message} (code: {code})")]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub code: String,
}

impl ValidationError {
    pub fn new(
        field: impl Into<String>,
        message: impl Into<String>,
        code: impl Into<String>,
    ) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            code: code.into(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ValidationErrors {
    errors: Vec<ValidationError>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    pub fn add_if_err(&mut self, result: Result<(), ValidationError>) {
        if let Err(e) = result {
            self.errors.push(e);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }

    pub fn merge(&mut self, other: ValidationErrors) {
        self.errors.extend(other.errors);
    }

    pub fn into_result(self) -> Result<(), Self> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(self)
        }
    }

    pub fn errors(&self) -> &[ValidationError] {
        &self.errors
    }
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, err) in self.errors.iter().enumerate() {
            if i > 0 {
                write!(f, "; ")?;
            }
            write!(f, "{}", err)?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationErrors {}

pub trait Validatable {
    fn validate(&self) -> Result<(), ValidationErrors>;
}

/// A custom validator trait for applying domain-specific validation rules.
///
/// Validators receive a field value and return a Result indicating success or a ValidationError.
/// This trait allows for composable, reusable validation logic.
///
/// # Example
/// ```
/// use phenotype_validation::Validator;
///
/// struct MinLengthValidator { min: usize }
///
/// impl Validator for MinLengthValidator {
///     fn validate(&self, value: &str, field: &str) -> Result<(), phenotype_validation::ValidationError> {
///         if value.len() >= self.min {
///             Ok(())
///         } else {
///             Err(phenotype_validation::ValidationError::new(field, "too short", self.code()))
///         }
///     }
///     fn code(&self) -> &str { "min_length_custom" }
/// }
/// ```
pub trait Validator {
    /// Execute the validation.
    ///
    /// Returns Ok(()) if validation passes, or ValidationError if it fails.
    fn validate(&self, value: &str, field: &str) -> Result<(), ValidationError>;

    /// Validate and return a detailed error message.
    fn validate_with_message(
        &self,
        value: &str,
        field: &str,
        error_message: &str,
    ) -> Result<(), ValidationError> {
        self.validate(value, field)
            .map_err(|_| ValidationError::new(field, error_message, self.code()))
    }

    /// Return the validation code/name for this validator.
    fn code(&self) -> &str {
        "custom"
    }
}

pub fn required(value: &str, field: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        Err(ValidationError::new(field, "is required", "required"))
    } else {
        Ok(())
    }
}

pub fn min_length(value: &str, min: usize, field: &str) -> Result<(), ValidationError> {
    if value.len() < min {
        Err(ValidationError::new(
            field,
            format!("must be at least {} characters", min),
            "min_length",
        ))
    } else {
        Ok(())
    }
}

pub fn max_length(value: &str, max: usize, field: &str) -> Result<(), ValidationError> {
    if value.len() > max {
        Err(ValidationError::new(
            field,
            format!("must be at most {} characters", max),
            "max_length",
        ))
    } else {
        Ok(())
    }
}

pub fn pattern(value: &str, regex: &Regex, field: &str) -> Result<(), ValidationError> {
    if !regex.is_match(value) {
        Err(ValidationError::new(
            field,
            "does not match required pattern",
            "pattern",
        ))
    } else {
        Ok(())
    }
}

pub fn range<T: PartialOrd + fmt::Display>(
    value: T,
    min: T,
    max: T,
    field: &str,
) -> Result<(), ValidationError> {
    if value < min || value > max {
        Err(ValidationError::new(
            field,
            format!("must be between {} and {}", min, max),
            "range",
        ))
    } else {
        Ok(())
    }
}

pub fn email(value: &str, field: &str) -> Result<(), ValidationError> {
    if !is_valid_email(value) {
        Err(ValidationError::new(field, "is not a valid email", "email"))
    } else {
        Ok(())
    }
}

pub fn url(value: &str, field: &str) -> Result<(), ValidationError> {
    if !is_valid_url(value) {
        Err(ValidationError::new(field, "is not a valid URL", "url"))
    } else {
        Ok(())
    }
}

pub fn phone(value: &str, field: &str) -> Result<(), ValidationError> {
    if !is_valid_phone(value) {
        Err(ValidationError::new(
            field,
            "is not a valid phone number",
            "phone",
        ))
    } else {
        Ok(())
    }
}

pub fn uuid(value: &str, field: &str) -> Result<(), ValidationError> {
    if !is_valid_uuid(value) {
        Err(ValidationError::new(field, "is not a valid UUID", "uuid"))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_valid() {
        assert!(required("hello", "name").is_ok());
    }

    #[test]
    fn test_required_empty() {
        assert!(required("", "name").is_err());
        assert!(required("   ", "name").is_err());
    }

    #[test]
    fn test_min_length_valid() {
        assert!(min_length("hello", 3, "name").is_ok());
    }

    #[test]
    fn test_min_length_too_short() {
        assert!(min_length("hi", 3, "name").is_err());
    }

    #[test]
    fn test_max_length_valid() {
        assert!(max_length("hi", 10, "name").is_ok());
    }

    #[test]
    fn test_max_length_too_long() {
        assert!(max_length("hello world", 5, "name").is_err());
    }

    #[test]
    fn test_pattern_valid() {
        let re = Regex::new(r"^\d{3}-\d{4}$").unwrap();
        assert!(pattern("123-4567", &re, "phone").is_ok());
    }

    #[test]
    fn test_pattern_invalid() {
        let re = Regex::new(r"^\d{3}-\d{4}$").unwrap();
        assert!(pattern("abc", &re, "phone").is_err());
    }

    #[test]
    fn test_range_valid() {
        assert!(range(5, 1, 10, "age").is_ok());
    }

    #[test]
    fn test_range_out_of_bounds() {
        assert!(range(15, 1, 10, "age").is_err());
        assert!(range(0, 1, 10, "age").is_err());
    }

    #[test]
    fn test_email_valid() {
        assert!(email("user@example.com", "email").is_ok());
    }

    #[test]
    fn test_email_invalid() {
        assert!(email("not-an-email", "email").is_err());
    }

    #[test]
    fn test_validation_errors_collection() {
        let mut errors = ValidationErrors::new();
        assert!(errors.is_empty());
        errors.add_if_err(required("", "name"));
        errors.add_if_err(min_length("x", 5, "password"));
        assert_eq!(errors.len(), 2);
        assert!(errors.into_result().is_err());
    }

    #[test]
    fn test_validation_errors_merge() {
        let mut e1 = ValidationErrors::new();
        e1.add(ValidationError::new("a", "msg", "code"));
        let mut e2 = ValidationErrors::new();
        e2.add(ValidationError::new("b", "msg", "code"));
        e1.merge(e2);
        assert_eq!(e1.len(), 2);
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
        let valid = User {
            name: "Alice".into(),
            email_addr: "alice@test.com".into(),
        };
        assert!(valid.validate().is_ok());
        let invalid = User {
            name: "".into(),
            email_addr: "bad".into(),
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_url_valid() {
        assert!(url("https://example.com", "website").is_ok());
    }

    #[test]
    fn test_url_invalid() {
        assert!(url("not-a-url", "website").is_err());
    }

    #[test]
    fn test_phone_valid() {
        assert!(phone("123-456-7890", "phone").is_ok());
    }

    #[test]
    fn test_phone_valid_international() {
        assert!(phone("+1 (555) 123-4567", "phone").is_ok());
    }

    #[test]
    fn test_phone_invalid() {
        assert!(phone("123", "phone").is_err());
    }

    #[test]
    fn test_uuid_valid() {
        assert!(uuid("550e8400-e29b-41d4-a716-446655440000", "id").is_ok());
    }

    #[test]
    fn test_uuid_invalid() {
        assert!(uuid("not-a-uuid", "id").is_err());
    }

    #[test]
    fn test_validator_trait_implementation() {
        struct MinLengthValidator {
            min: usize,
        }

        impl Validator for MinLengthValidator {
            fn validate(&self, value: &str, field: &str) -> Result<(), ValidationError> {
                if value.len() >= self.min {
                    Ok(())
                } else {
                    Err(ValidationError::new(
                        field,
                        format!("must be at least {} characters", self.min),
                        self.code(),
                    ))
                }
            }

            fn code(&self) -> &str {
                "min_length_custom"
            }
        }

        let validator = MinLengthValidator { min: 5 };
        assert!(validator.validate("hello", "name").is_ok());
        assert!(validator.validate("hi", "name").is_err());
    }

    #[test]
    fn test_complex_validation_workflow() {
        struct FormData {
            name: String,
            email: String,
            phone: String,
            website: String,
        }

        impl Validatable for FormData {
            fn validate(&self) -> Result<(), ValidationErrors> {
                let mut errors = ValidationErrors::new();
                errors.add_if_err(required(&self.name, "name"));
                errors.add_if_err(min_length(&self.name, 2, "name"));
                errors.add_if_err(email(&self.email, "email"));
                errors.add_if_err(phone(&self.phone, "phone"));
                errors.add_if_err(url(&self.website, "website"));
                errors.into_result()
            }
        }

        let valid_form = FormData {
            name: "John Doe".into(),
            email: "john@example.com".into(),
            phone: "(555) 123-4567".into(),
            website: "https://example.com".into(),
        };
        assert!(valid_form.validate().is_ok());

        let invalid_form = FormData {
            name: "J".into(),
            email: "invalid-email".into(),
            phone: "123".into(),
            website: "not-a-url".into(),
        };
        let result = invalid_form.validate();
        assert!(result.is_err());
        if let Err(errors) = result {
            assert!(errors.len() >= 4);
        }
    }

    #[test]
    fn test_email_validation_edge_cases() {
        assert!(email("user+tag@subdomain.example.co.uk", "email").is_ok());
        assert!(email("first.last@example123.com", "email").is_ok());
        assert!(email("user_name@example.com", "email").is_ok());
    }

    #[test]
    fn test_phone_validation_various_formats() {
        assert!(phone("1234567890", "phone").is_ok());
        assert!(phone("123-456-7890", "phone").is_ok());
        assert!(phone("(123) 456-7890", "phone").is_ok());
        assert!(phone("+1 (555) 123-4567", "phone").is_ok());
    }

    #[test]
    fn test_validation_error_codes() {
        let email_err = email("bad", "email").unwrap_err();
        assert_eq!(email_err.code, "email");

        let phone_err = phone("1", "phone").unwrap_err();
        assert_eq!(phone_err.code, "phone");

        let url_err = url("not-url", "website").unwrap_err();
        assert_eq!(url_err.code, "url");
    }

    #[test]
    fn test_validation_errors_display() {
        let mut errors = ValidationErrors::new();
        errors.add(ValidationError::new("email", "invalid", "email"));
        errors.add(ValidationError::new("phone", "too short", "phone"));
        let display_string = format!("{}", errors);
        assert!(display_string.contains("email"));
        assert!(display_string.contains("phone"));
    }

    #[test]
    fn test_validator_trait_with_message() {
        struct CustomValidator;

        impl Validator for CustomValidator {
            fn validate(&self, value: &str, _field: &str) -> Result<(), ValidationError> {
                if value.len() > 10 {
                    Ok(())
                } else {
                    Err(ValidationError::new("field", "error", self.code()))
                }
            }

            fn code(&self) -> &str {
                "custom_length"
            }
        }

        let validator = CustomValidator;
        let result = validator.validate_with_message("short", "field", "field is too short");
        assert!(result.is_err());
        if let Err(err) = result {
            assert_eq!(err.message, "field is too short");
            assert_eq!(err.code, "custom_length");
        }
    }
}
