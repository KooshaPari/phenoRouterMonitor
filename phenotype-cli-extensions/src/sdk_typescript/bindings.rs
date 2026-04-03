//! TypeScript type bindings generated from Rust
//!
//! Uses ts-rs for automatic type generation.

/// Example type that will be exported to TypeScript
#[derive(Debug, Clone)]
pub struct CliConfig {
    pub name: String,
    pub version: String,
    pub enabled_features: Vec<String>,
}

/// Kitty graphics configuration
#[derive(Debug, Clone)]
pub struct GraphicsConfig {
    pub max_width: u32,
    pub max_height: u32,
    pub supported_formats: Vec<String>,
}
