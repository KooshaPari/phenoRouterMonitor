//! Design-by-contract assertions and invariants for Rust.

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum ContractError {
    #[error("Precondition violated: {message}")]
    Precondition { message: String, location: Location },
    #[error("Postcondition violated: {message}")]
    Postcondition { message: String, location: Location, return_value: String },
    #[error("Invariant violated: {message}")]
    Invariant { message: String, location: Location },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location { pub file: String, pub line: u32, pub column: u32 }

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

pub trait Contract: Sized {
    fn check_invariant(&self) -> bool;
    fn invariant_message(&self) -> String { "Invariant violated".to_string() }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invariant<T: Contract> { value: T }

impl<T: Contract> Invariant<T> {
    pub fn new(value: T) -> Result<Self, ContractError> {
        if value.check_invariant() { Ok(Self { value }) } else {
            Err(ContractError::Invariant {
                message: value.invariant_message(),
                location: Location { file: file!().to_string(), line: line!(), column: column!() },
            })
        }
    }
    pub unsafe fn new_unchecked(value: T) -> Self { Self { value } }
    pub fn into_inner(self) -> T { self.value }
    pub fn get_ref(&self) -> &T { &self.value }
    pub fn check(&self) -> Result<(), ContractError> {
        if self.value.check_invariant() { Ok(()) } else {
            Err(ContractError::Invariant {
                message: self.value.invariant_message(),
                location: Location { file: file!().to_string(), line: line!(), column: column!() },
            })
        }
    }
}

impl<T: Contract> std::ops::Deref for Invariant<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target { &self.value }
}

#[derive(Debug, Clone)]
pub struct Precondition;

impl Precondition {
    pub fn new(condition: bool, message: &str) -> Result<(), ContractError> {
        if condition { Ok(()) } else {
            Err(ContractError::Precondition {
                message: message.to_string(),
                location: Location { file: file!().to_string(), line: line!(), column: column!() },
            })
        }
    }
}

#[macro_export] macro_rules! requires {
    ($condition:expr, $message:expr) => {{
        $crate::Precondition::new($condition, $message)?
    }};
}

#[macro_export] macro_rules! ensures {
    ($condition:expr, $message:expr, $return_value:expr) => {{
        if !$condition {
            return Err($crate::ContractError::Postcondition {
                message: $message.to_string(),
                location: $crate::Location { file: file!().to_string(), line: line!(), column: column!() },
                return_value: format!("{:?}", $return_value),
            });
        }
    }};
}

pub trait ResultContract<T, E> {
    fn check_postcondition<F>(self, f: F) -> Self where F: FnOnce(&T) -> bool;
}

impl<T, E> ResultContract<T, E> for Result<T, E> {
    fn check_postcondition<F>(self, f: F) -> Self where F: FnOnce(&T) -> bool {
        if let Ok(ref value) = self { assert!(f(value), "Postcondition violated"); }
        self
    }
}

#[derive(Debug, Clone)]
pub struct ContractBuilder<T> { value: T, errors: Vec<ContractError> }

impl<T> ContractBuilder<T> {
    pub fn new(value: T) -> Self { Self { value, errors: Vec::new() } }
    pub fn requires(mut self, condition: bool, message: &str) -> Self {
        if !condition { self.errors.push(ContractError::Precondition {
            message: message.to_string(),
            location: Location { file: file!().to_string(), line: line!(), column: column!() },
        }); }
        self
    }
    pub fn ensures(mut self, condition: bool, message: &str) -> Self {
        if !condition { self.errors.push(ContractError::Postcondition {
            message: message.to_string(),
            location: Location { file: file!().to_string(), line: line!(), column: column!() },
            return_value: format!("{:?}", &self.value),
        }); }
        self
    }
    pub fn build(self) -> Result<T, Vec<ContractError>> {
        if self.errors.is_empty() { Ok(self.value) } else { Err(self.errors) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)] struct PositiveInt(i32);
    impl PositiveInt {
        pub fn new(value: i32) -> Result<Self, ContractError> { 
            Precondition::new(value > 0, "Value must be positive")?; 
            Ok(Self(value)) 
        }
        pub fn get(&self) -> i32 { self.0 }
    }
    impl Contract for PositiveInt {
        fn check_invariant(&self) -> bool { self.0 > 0 }
        fn invariant_message(&self) -> String { format!("PositiveInt invariant violated: value {} must be > 0", self.0) }
    }

    #[test]
    fn test_positive_int_new_valid() { assert_eq!(PositiveInt::new(42).unwrap().get(), 42); }
    #[test]
    fn test_positive_int_new_invalid() { assert!(PositiveInt::new(-1).is_err()); }
    #[test]
    fn test_invariant_wrapper_valid() { assert_eq!(Invariant::new(PositiveInt::new(10).unwrap()).unwrap().get(), 10); }
    #[test]
    fn test_invariant_wrapper_invalid() { assert!(Invariant::new(PositiveInt::new(-5).unwrap()).is_err()); }
    #[test]
    fn test_requires_macro_valid() { 
        fn divide(a: i32, b: i32) -> Result<i32, ContractError> { 
            requires!(b != 0, "Division by zero"); 
            Ok(a / b) 
        } 
        assert_eq!(divide(10, 2).unwrap(), 5); 
    }
    #[test]
    fn test_requires_macro_invalid() { 
        fn divide(a: i32, b: i32) -> Result<i32, ContractError> { 
            requires!(b != 0, "Division by zero"); 
            Ok(a / b) 
        } 
        assert!(divide(10, 0).is_err()); 
    }
    #[test]
    fn test_ensures_macro_valid() { 
        fn absolute_value(x: i32) -> Result<i32, ContractError> { 
            let result = if x < 0 { -x } else { x }; 
            ensures!(result >= 0, "Must be non-negative", result); 
            Ok(result) 
        } 
        assert_eq!(absolute_value(-5).unwrap(), 5); 
    }
    #[test]
    fn test_contract_builder_success() { 
        assert_eq!(ContractBuilder::new(42).requires(true, "ok").ensures(true, "ok").build().unwrap(), 42); 
    }
    #[test]
    fn test_contract_builder_failure() { 
        assert!(ContractBuilder::new(41).requires(false, "fail").build().is_err()); 
    }
    #[test]
    fn test_location_display() { 
        assert_eq!(format!("{}", Location { file: "test.rs".to_string(), line: 1, column: 1 }), "test.rs:1:1"); 
    }
    #[test]
    fn test_result_contract() { 
        let r: Result<i32, ()> = Ok(42); 
        assert!(r.check_postcondition(|v| *v > 0).is_ok()); 
    }
    #[test]
    fn test_invariant_into_inner() { 
        assert_eq!(Invariant::new(PositiveInt::new(42).unwrap()).unwrap().into_inner().get(), 42); 
    }
    #[test]
    fn test_invariant_check() { 
        assert!(Invariant::new(PositiveInt::new(10).unwrap()).unwrap().check().is_ok()); 
    }
    #[test]
    fn test_invariant_deref() { 
        let wrapper = Invariant::new(PositiveInt::new(20).unwrap()).unwrap(); 
        let value: &PositiveInt = &wrapper; 
        assert_eq!(value.get(), 20); 
    }
}
