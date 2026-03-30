//! Error types for phenotype-health

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum Error {
    #[error("health error: {0}")]
    Health(String),
}
