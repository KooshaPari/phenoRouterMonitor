//! Phenotype Errors

use std::fmt;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    #[allow(dead_code)]
    NotFound(String),
    #[allow(dead_code)]
    Validation(String),
    #[allow(dead_code)]
    Conflict(String),
    #[allow(dead_code)]
    Unauthorized(String),
    #[allow(dead_code)]
    Forbidden(String),
    #[allow(dead_code)]
    Io(String),
    #[allow(dead_code)]
    Serialization(String),
    #[allow(dead_code)]
    Internal(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(s) => write!(f, "not found: {}", s),
            Self::Validation(s) => write!(f, "validation failed: {}", s),
            Self::Conflict(s) => write!(f, "conflict: {}", s),
            Self::Unauthorized(s) => write!(f, "unauthorized: {}", s),
            Self::Forbidden(s) => write!(f, "forbidden: {}", s),
            Self::Io(s) => write!(f, "IO error: {}", s),
            Self::Serialization(s) => write!(f, "serialization error: {}", s),
            Self::Internal(s) => write!(f, "internal error: {}", s),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialization(e.to_string())
    }
}

impl Error {
    #[allow(dead_code)]
    pub fn not_found<S: Into<String>>(msg: S) -> Self { Self::NotFound(msg.into()) }
    #[allow(dead_code)]
    pub fn validation<S: Into<String>>(msg: S) -> Self { Self::Validation(msg.into()) }
    #[allow(dead_code)]
    pub fn conflict<S: Into<String>>(msg: S) -> Self { Self::Conflict(msg.into()) }
    #[allow(dead_code)]
    pub fn internal<S: Into<String>>(msg: S) -> Self { Self::Internal(msg.into()) }
}
