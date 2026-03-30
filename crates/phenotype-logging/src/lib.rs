//! # Phenotype Logging
//!
//! Canonical logging and tracing setup for Phenotype services.
//!
//! This crate was created because ZERO logging existed in the codebase - only 1 println!
//!
//! ## Features
//!
//! - Structured logging with tracing
//! - OpenTelemetry integration
//! - Log level configuration via environment
//! - JSON and human-readable output formats
//!
//! ## Usage
//!
//! ```rust
//! use phenotype_logging::{init, init_with_otel};
//!
//! // Basic initialization
//! init();
//!
//! // With OpenTelemetry
//! init_with_otel("service-name", "otlp-endpoint")?;
//! ```

pub mod config;
pub mod subscriber;
pub mod otel;

pub use config::{LogConfig, LogLevel, OutputFormat};
pub use subscriber::init;
pub use otel::init_with_otel;
