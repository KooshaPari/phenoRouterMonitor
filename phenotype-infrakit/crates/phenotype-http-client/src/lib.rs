//! HTTP client abstractions and adapters
//!
//! This crate provides a generic HTTP client interface and adapters for different HTTP implementations.

pub mod error;
pub mod types;
pub mod adapters;

pub use error::{HttpClientError, Result};
pub use types::{HttpRequest, HttpResponse, Method};
