//! Error types for phenotype-test-infra

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Serialize, Deserialize, Error)]
pub enum Error {
    #[error("test error: {0}")]
    Test(String),
}
