//! Tracing setup for Phenotype.

/// Initialize tracing with JSON formatting.
pub fn init() {
    tracing_subscriber::fmt()
        .with_target(true)
        .json()
        .init();
}
