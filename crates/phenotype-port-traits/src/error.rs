//! Error types for phenotype-port-traits

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum Error {
    #[error("port error: {0}")]
    Port(String),
}
