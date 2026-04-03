//! Analytics framework

pub mod error;
pub mod event;
pub mod client;
pub mod traits;

pub use error::{AnalyticsError, Result};
pub use event::AnalyticsEvent;
pub use client::AnalyticsClient;
pub use traits::Trackable;
