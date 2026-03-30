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
    let email_re = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    if !email_re.is_match(value) {
        Err(ValidationError::new(field, "is not a valid email", "email"))
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
}
