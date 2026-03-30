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
    pub fn new(field: impl Into<String>, message: impl Into<String>, code: impl Into<String>) -> Self {
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
    pub fn new() -> Self { Self::default() }
    pub fn add(&mut self, error: ValidationError) { self.errors.push(error); }
    pub fn add_if_err(&mut self, result: Result<(), ValidationError>) {
        if let Err(e) = result { self.errors.push(e); }
    }
    pub fn is_empty(&self) -> bool { self.errors.is_empty() }
    pub fn len(&self) -> usize { self.errors.len() }
    pub fn merge(&mut self, other: ValidationErrors) { self.errors.extend(other.errors); }
    pub fn into_result(self) -> Result<(), Self> {
        if self.is_empty() { Ok(()) } else { Err(self) }
    }
    pub fn errors(&self) -> &[ValidationError] { &self.errors }
}

impl fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, err) in self.errors.iter().enumerate() {
            if i > 0 { write!(f, "; ")?; }
            write!(f, "{}", err)?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationErrors {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_creation() {
        let err = ValidationError::new("email", "invalid format", "email_invalid");
        assert_eq!(err.field, "email");
        assert_eq!(err.message, "invalid format");
        assert_eq!(err.code, "email_invalid");
    }

    #[test]
    fn test_validation_errors_add() {
        let mut errors = ValidationErrors::new();
        assert!(errors.is_empty());
        errors.add(ValidationError::new("name", "required", "required"));
        assert_eq!(errors.len(), 1);
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
    fn test_validation_errors_into_result() {
        let errors = ValidationErrors::new();
        assert!(errors.into_result().is_ok());
        let mut errors = ValidationErrors::new();
        errors.add(ValidationError::new("name", "required", "required"));
        assert!(errors.into_result().is_err());
    }
}
