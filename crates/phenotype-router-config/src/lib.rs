//! # Phenotype Router Config
//!
//! TOML-based router configuration with file watching and hot reload detection.
//!
//! ## Features
//!
//! - **TOML Configuration**: Define routes and server settings in TOML format
//! - **File Watching**: Automatically detect configuration file changes
//! - **Hot Reload**: Support for configuration change callbacks
//! - **Validation**: Built-in configuration validation
//! - **Type-Safe**: Strongly-typed configuration structures
//! - **Async-First**: Fully async API with tokio integration
//!
//! ## Usage
//!
//! ```rust
//! use phenotype_router_config::ConfigLoader;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let config = ConfigLoader::load_from_file("config.toml").await?;
//! println!("Router: {}", config.name);
//! # Ok(())
//! # }
//! ```
//!
//! ## TOML Format
//!
//! ```toml
//! name = "my-router"
//!
//! [server]
//! host = "127.0.0.1"
//! port = 8080
//! timeout_secs = 30
//! max_connections = 1000
//!
//! [[routes]]
//! path = "/api/users"
//! method = "GET"
//! handler = "user_handler"
//! cors_enabled = true
//!
//! [middleware]
//! logging_enabled = true
//! auth_enabled = false
//! ```

pub mod config;
pub mod error;
pub mod loader;
pub mod watcher;

// Re-exports
pub use config::{CompressionConfig, MiddlewareConfig, RouteConfig, RouterConfig, ServerConfig};
pub use error::{Result, RouterConfigError};
pub use loader::ConfigLoader;
pub use watcher::{ConfigChangeCallback, ConfigChangeEvent, ConfigWatcher};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_exports() {
        // Verify public API is accessible
        let _: std::any::TypeId = std::any::TypeId::of::<RouterConfig>();
        let _: std::any::TypeId = std::any::TypeId::of::<ConfigLoader>();
        let _: std::any::TypeId = std::any::TypeId::of::<RouterConfigError>();
    }
}
