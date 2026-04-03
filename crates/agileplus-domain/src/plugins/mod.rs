//! Plugin integration layer for AgilePlus.
//!
//! This module provides integration between the domain ports and the plugin system,
//! allowing runtime plugin discovery and management.
//!
//! # X-DDs Applied
//! - Hexagonal Architecture: Adapters for plugin system
//! - Open/Closed: Extensible via plugins
//! - Dependency Inversion: Domain depends on abstractions
//!
//! Traceability: WP11-T070

#[cfg(feature = "plugins")]
pub mod registry;

#[cfg(feature = "plugins")]
pub use registry::PluginRegistry;
