//! # phenotype-casbin-wrapper
//!
//! Casbin adapter providing policy enforcement for the Phenotype ecosystem.
//!
//! This crate wraps the `casbin` crate to provide a consistent interface
//! for policy evaluation with support for multiple access control models.
//!
//! ## Features
//!
//! - **RBAC**: Role-based access control with role hierarchies
//! - **ABAC**: Attribute-based access control with subject/object attributes
//! - **ACL**: Access control lists with explicit permission mapping
//! - **Policy Management**: Hot reloading, versioning, and batch updates
//!
//! ## Example
//!
//! ```ignore
//! use phenotype_casbin_wrapper::CasbinAdapter;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let adapter = CasbinAdapter::new(
//!         "examples/basic_model.conf",
//!         "examples/basic_policy.csv",
//!     ).await?;
//!
//!     // Check if request is allowed
//!     let request = vec!["alice", "data1", "read"];
//!     let allowed = adapter.enforce(&request).await?;
//!     assert!(allowed);
//!
//!     Ok(())
//! }
//! ```

pub mod adapter;
pub mod error;
pub mod models;

pub use adapter::CasbinAdapter;
pub use error::CasbinWrapperError;
