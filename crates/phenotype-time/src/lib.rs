//! phenotype-time

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Invalid(String),
}

pub type Result<T> = std::result::Result<T, Error>;

pub mod duration;
pub mod timestamp;

pub use duration::constants as duration_constants;
pub use duration::DurationExt;
pub use timestamp::constants as time_constants;
pub use timestamp::Timestamp;
