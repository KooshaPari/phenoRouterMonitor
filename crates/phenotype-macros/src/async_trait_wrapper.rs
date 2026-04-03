//! Re-exports from async-trait with Phenotype-specific configuration.

// Re-export async-trait for use across the ecosystem
pub use async_trait::async_trait;

/// Type alias for async trait bounds used in Phenotype crates.
pub type AsyncFn<T> = Box<dyn std::future::Future<Output = T> + Send>;
