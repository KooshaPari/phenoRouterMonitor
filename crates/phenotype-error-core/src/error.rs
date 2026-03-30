//! Error type definitions
//!
//! Centralized error types for Phenotype ecosystem.

use std::fmt;

/// Source errors from standard library
pub mod source {
    pub use std::io::Error as Io;
    pub use std::fmt::Error as Format;
}

/// Domain errors for business logic
pub mod domain {
    use super::*;
    
    /// Entity not found
    #[derive(Debug)]
    pub struct NotFound {
        pub entity: &'static str,
        pub id: String,
    }
    
    impl fmt::Display for NotFound {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "not found: {}:{}", self.entity, self.id)
        }
    }
    
    impl std::error::Error for NotFound {}
    
    /// Validation failed
    #[derive(Debug)]
    pub struct Validation {
        pub field: String,
        pub message: String,
    }
    
    impl fmt::Display for Validation {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "validation failed for '{}': {}", self.field, self.message)
        }
    }
    
    impl std::error::Error for Validation {}
}

/// Application errors
pub mod application {
    /// Configuration error
    #[derive(Debug)]
    pub struct Config {
        pub message: String,
    }
    
    impl std::fmt::Display for Config {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "configuration error: {}", self.message)
        }
    }
    
    impl std::error::Error for Config {}
    
    /// Authentication required
    #[derive(Debug)]
    pub struct Unauthorized {
        pub reason: String,
    }
    
    impl std::fmt::Display for Unauthorized {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "unauthorized: {}", self.reason)
        }
    }
    
    impl std::error::Error for Unauthorized {}
}

/// Infrastructure errors
pub mod infrastructure {
    /// Database error
    #[derive(Debug)]
    pub struct Database {
        pub message: String,
    }
    
    impl std::fmt::Display for Database {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "database error: {}", self.message)
        }
    }
    
    impl std::error::Error for Database {}
    
    /// Cache error
    #[derive(Debug)]
    pub struct Cache {
        pub message: String,
    }
    
    impl std::fmt::Display for Cache {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "cache error: {}", self.message)
        }
    }
    
    impl std::error::Error for Cache {}
}
