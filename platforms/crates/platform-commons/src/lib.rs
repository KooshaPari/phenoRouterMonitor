//! Cross-platform shared utilities for the Phenotype ecosystem

pub mod error;
pub mod types;
pub mod utils;

pub use error::{PlatformError, Result};
pub use types::*;
pub use utils::*;
