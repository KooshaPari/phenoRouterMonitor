//! # AgilePlus Plugin Core
//!
//! Core plugin traits and registry for AgilePlus extensibility.
//! This crate defines the port interfaces that adapters must implement.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Plugin Core (this crate)                 │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
//! │  │ PluginTrait │  │  Registry   │  │   Config/Traits      │ │
//! │  └─────────────┘  └─────────────┘  └─────────────────────┘ │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!         ┌────────────────────┼────────────────────┐
//!         ▼                    ▼                    ▼
//! ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
//! │ Git Adapter  │    │SQLite Adapter│    │ LLM Adapter  │
//! │ (gitoxide)  │    │   (rusqlite) │    │   (ollama)   │
//! └──────────────┘    └──────────────┘    └──────────────┘
//! ```
//!
//! ## X-DDs Applied
//!
//! - **Hexagonal Architecture**: Ports define boundaries
//! - **Interface Segregation**: Small, focused trait methods
//! - **Dependency Inversion**: Core depends on abstractions
//! - **Open/Closed**: Open for extension, closed for modification
//! - **DRY**: Shared types defined once
//! - **KISS**: Simple, focused traits
//!
//! ## Usage
//!
//! ```rust,ignore
//! use agileplus_plugin_core::{PluginRegistry, VcsPlugin};
//!
//! let mut registry = PluginRegistry::new();
//! registry.register_vcs(Box::new(MyGitAdapter::new()));
//!
//! if let Some(vcs) = registry.vcs("git") {
//!     let path = vcs.create_worktree("my-feature", "WP01").await?;
//! }
//! ```
//!
//! ## Feature Flags
//!
//! - `runtime-tokio`: Enable async runtime support (default)

pub mod error;
pub mod registry;
pub mod traits;

pub use error::PluginError;
pub use registry::PluginRegistry;
pub use traits::{AdapterPlugin, StoragePlugin, VcsPlugin};
