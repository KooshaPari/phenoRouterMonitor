/// Test suite for ErrorType derive macro
use phenotype_macros::ErrorType;
use std::error::Error;

#[derive(ErrorType, Debug)]
pub struct ValidationError {
    message: String,
    field: String,
}

#[test]
fn test_error_type_display() {
    let error = ValidationError {
        message: "Email is invalid".to_string(),
        field: "email".to_string(),
    };

    let display_string = format!("{}", error);
    assert_eq!(display_string, "Email is invalid");
}

#[test]
fn test_error_type_error_trait() {
    let error = ValidationError {
        message: "Invalid input".to_string(),
        field: "name".to_string(),
    };

    // Verify it implements Error trait
    let _: &dyn Error = &error;
}

#[test]
fn test_error_type_debug() {
    let error = ValidationError {
        message: "Parse error".to_string(),
        field: "config".to_string(),
    };

    let debug_string = format!("{:?}", error);
    // Debug should delegate to Display
    assert_eq!(debug_string, "Parse error");
}

#[derive(ErrorType)]
pub struct NotFoundError {
    resource: String,
}

#[test]
fn test_error_type_not_found() {
    let error = NotFoundError {
        resource: "user:123".to_string(),
    };

    // Should have some default message since no message field
    let _: String = format!("{:?}", error);
}

#[derive(ErrorType)]
pub enum ApiError {
    NotFound,
    Unauthorized,
    InternalServerError(String),
    BadRequest { message: String, code: u32 },
}

#[test]
fn test_error_enum_unit_variant() {
    let error = ApiError::NotFound;
    let _: &dyn Error = &error;
}

#[test]
fn test_error_enum_tuple_variant() {
    let error = ApiError::InternalServerError("Database connection failed".to_string());
    let display = format!("{}", error);
    assert!(display.contains("InternalServerError"));
}

#[test]
fn test_error_enum_struct_variant() {
    let error = ApiError::BadRequest {
        message: "Invalid JSON".to_string(),
        code: 400,
    };
    let display = format!("{}", error);
    assert!(display.contains("BadRequest"));
}

#[derive(ErrorType, Debug)]
pub struct DatabaseError {
    message: String,
    code: i32,
}

#[test]
fn test_error_multiple_fields() {
    let error = DatabaseError {
        message: "Connection timeout".to_string(),
        code: 1001,
    };

    assert_eq!(format!("{}", error), "Connection timeout");
    // Verify Error trait impl
    let _: Box<dyn Error> = Box::new(error);
}

#[derive(ErrorType)]
pub struct SimpleError;

#[test]
fn test_error_unit_struct() {
    let error = SimpleError;
    let display = format!("{}", error);
    assert!(display.contains("SimpleError"));
    let _: &dyn Error = &error;
}
