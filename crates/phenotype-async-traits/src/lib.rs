//! phenotype-async-traits: Common async trait utilities for Phenotype.
//!
//! This crate centralizes `async_trait` usage so that all other crates can
//! share a stable import path and avoid repetitive `async_trait` dependency
//! declarations.

pub use async_trait::async_trait;

#[cfg(test)]
mod tests {
    use super::*;

    #[async_trait]
    trait TestAsyncTrait {
        async fn hello(&self) -> String;
    }

    struct Hello;

    #[async_trait]
    impl TestAsyncTrait for Hello {
        async fn hello(&self) -> String {
            "hello".to_string()
        }
    }

    #[tokio::test]
    async fn async_trait_reexport_works() {
        let h = Hello;
        assert_eq!(h.hello().await, "hello");
    }
}
