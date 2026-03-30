//! Cycle domain entity, CycleState lifecycle, and related types.
//!
//! Traces to: FR-C01, FR-C02, FR-C03, FR-C04, FR-C05, FR-C07

mod entity;
mod feature;
mod progress;
mod state;
mod with_features;

pub use entity::Cycle;
pub use feature::CycleFeature;
pub use progress::WpProgressSummary;
pub use state::CycleState;
pub use with_features::CycleWithFeatures;

#[cfg(test)]
mod tests;
