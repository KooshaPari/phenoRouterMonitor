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
//! ```rust
//! use phenotype_casbin_wrapper::{CasbinAdapter, CasbinAdapterExt};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let adapter = CasbinAdapter::new(
//!         "r", "p", "examples/rbac_with_dummy.toml",
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

pub mod error;
pub mod adapter;
pub mod models;

pub use adapter::CasbinAdapter;
pub use error::CasbinWrapperError;

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_enforcement() -> Result<(), CasbinWrapperError> {
        let adapter = CasbinAdapter::new(
            "r", "p", "examples/basic_model.conf",
            "examples/basic_policy.csv",
        )
        .await?;

        let request = vec!["alice", "data1", "read"];
        let allowed = adapter.enforce(&request).await?;
        assert!(allowed, "alice should be allowed to read data1");

        let request2 = vec!["bob", "data1", "write"];
        let denied = adapter.enforce(&request2).await?;
        assert!(!denied, "bob should not be allowed to write data1");

        Ok(())
    }

    #[tokio::test]
    async fn test_rbac_enforcement() -> Result<(), CasbinWrapperError> {
        let adapter = CasbinAdapter::new(
            "r", "p", "examples/rbac_model.conf",
            "examples/rbac_policy.csv",
        )
        .await?;

        // Alice has admin role which should allow anything under data1
        let request = vec!["alice", "data1", "read"];
        let allowed = adapter.enforce(&request).await?;
        assert!(allowed, "alice (admin) should be allowed to read data1");

        // Bob is a user with data1_reader role
        let request2 = vec!["bob", "data1", "read"];
        let allowed2 = adapter.enforce(&request2).await?;
        assert!(allowed2, "bob (user) should be allowed to read data1");

        // Bob should not have write access
        let request3 = vec!["bob", "data1", "write"];
        let denied = adapter.enforce(&request3).await?;
        assert!(!denied, "bob (user) should not be allowed to write data1");

        Ok(())
    }

    #[tokio::test]
    async fn test_enforce_named() -> Result<(), CasbinWrapperError> {
        let adapter = CasbinAdapter::new(
            "r", "p", "examples/rbac_model.conf",
            "examples/rbac_policy.csv",
        )
        .await?;

        // Using named enforcement with different policy type
        let request = vec!["alice", "data2", "read"];
        let allowed = adapter.enforce_named("p", &request).await?;
        assert!(!allowed, "alice should not have access to data2 by default");

        Ok(())
    }

    #[tokio::test]
    async fn test_modify_policy() -> Result<(), CasbinWrapperError> {
        let adapter = CasbinAdapter::new(
            "r", "p", "examples/rbac_model.conf",
            "examples/rbac_policy.csv",
        )
        .await?;

        // Initially bob should not have access to data1/write
        let request = vec!["bob", "data1", "write"];
        let denied = adapter.enforce(&request).await?;
        assert!(!denied);

        // Add policy: bob can write data1
        let rules = vec![vec!["bob", "data1", "write"]];
        adapter.modify_policy("p", rules.clone()).await?;

        // Now bob should have access
        let allowed = adapter.enforce(&request).await?;
        assert!(allowed, "bob should now be allowed to write data1 after policy update");

        // Remove policy
        adapter.remove_policy("p", rules).await?;

        // Bob should be denied again
        let denied2 = adapter.enforce(&request).await?;
        assert!(!denied2, "bob should be denied after policy removal");

        Ok(())
    }

    #[tokio::test]
    async fn test_batch_enforcement() -> Result<(), CasbinWrapperError> {
        let adapter = CasbinAdapter::new(
            "r", "p", "examples/rbac_model.conf",
            "examples/rbac_policy.csv",
        )
        .await?;

        let requests = vec![
            vec!["alice", "data1", "read"],
            vec!["bob", "data1", "read"],
            vec!["bob", "data1", "write"],
        ];

        let results = adapter.batch_enforce(&requests).await?;
        assert_eq!(results.len(), 3);
        assert!(results[0], "alice should be allowed");
        assert!(results[1], "bob should be allowed to read");
        assert!(!results[2], "bob should not be allowed to write");

        Ok(())
    }

    #[tokio::test]
    async fn test_clear_policy() -> Result<(), CasbinWrapperError> {
        let adapter = CasbinAdapter::new(
            "r", "p", "examples/rbac_model.conf",
            "examples/rbac_policy.csv",
        )
        .await?;

        // Initially alice should have access
        let request = vec!["alice", "data1", "read"];
        let allowed = adapter.enforce(&request).await?;
        assert!(allowed);

        // Clear all policies
        adapter.clear_policy().await?;

        // Alice should no longer have access
        let denied = adapter.enforce(&request).await?;
        assert!(!denied, "alice should be denied after policy clear");

        Ok(())
    }

    #[tokio::test]
    async fn test_policy_reload() -> Result<(), CasbinWrapperError> {
        let adapter = CasbinAdapter::new(
            "r", "p", "examples/rbac_model.conf",
            "examples/rbac_policy.csv",
        )
        .await?;

        // Modify policy in memory
        let rules = vec![vec!["charlie", "data1", "read"]];
        adapter.modify_policy("p", rules).await?;

        // Charlie should have access via modified policy
        let request = vec!["charlie", "data1", "read"];
        let allowed = adapter.enforce(&request).await?;
        assert!(allowed);

        // Reload from file should restore original
        adapter.reload_policy().await?;

        // Charlie should no longer have access
        let denied = adapter.enforce(&request).await?;
        assert!(!denied, "charlie should be denied after reload");

        Ok(())
    }
}
