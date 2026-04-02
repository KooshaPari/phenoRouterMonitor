//! Configuration loader

pub mod error;
pub mod loader;

pub use error::{ConfigError, Result};
pub use loader::ConfigLoader;
