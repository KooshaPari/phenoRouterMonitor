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
//! use phenotype_casbin_wrapper::{CasbinAdapter, CasbinAdapterExt};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let adapter = CasbinAdapter::new(
//!         "examples/basic_model.conf".to_string(),
//!         "examples/basic_policy.csv".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_basic_model(dir: &std::path::Path) -> std::path::PathBuf {
        let model_path = dir.join("model.conf");
        std::fs::write(
            &model_path,
            r#"
[request_definition]
r = sub, obj, act

[policy_definition]
p = sub, obj, act

[policy_effect]
e = some(where (p.eft == allow))

[matchers]
m = r.sub == p.sub && r.obj == p.obj && r.act == p.act
"#,
        )
        .unwrap();
        model_path
    }

    fn create_basic_policy(dir: &std::path::Path) -> std::path::PathBuf {
        let policy_path = dir.join("policy.csv");
        std::fs::write(&policy_path, "p, alice, data1, read\np, bob, data1, read\n")
            .unwrap();
        policy_path
    }

    fn create_rbac_model(dir: &std::path::Path) -> std::path::PathBuf {
        let model_path = dir.join("rbac_model.conf");
        std::fs::write(
            &model_path,
            r#"
[request_definition]
r = sub, obj, act

[policy_definition]
p = sub, obj, act

[role_definition]
g = _, _

[policy_effect]
e = some(where (p.eft == allow))

[matchers]
m = r.sub == p.sub && r.obj == p.obj && r.act == p.act
"#,
        )
        .unwrap();
        model_path
    }

    fn create_rbac_policy(dir: &std::path::Path) -> std::path::PathBuf {
        let policy_path = dir.join("rbac_policy.csv");
        std::fs::write(
            &policy_path,
            "p, alice, data1, read\np, bob, data1, read\ng, bob, user\ng, alice, admin\n",
        )
        .unwrap();
        policy_path
    }

    #[tokio::test]
    async fn test_basic_enforcement() -> Result<(), CasbinWrapperError> {
        let dir = TempDir::new().unwrap();
        let model_path = create_basic_model(dir.path());
        let policy_path = create_basic_policy(dir.path());

        let adapter = CasbinAdapterExt::new(
            model_path.to_string_lossy().to_string(),
            policy_path.to_string_lossy().to_string(),
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
        let dir = TempDir::new().unwrap();
        let model_path = create_rbac_model(dir.path());
        let policy_path = create_rbac_policy(dir.path());

        let adapter = CasbinAdapterExt::new(
            model_path.to_string_lossy().to_string(),
            policy_path.to_string_lossy().to_string(),
        )
        .await?;

        let request = vec!["alice", "data1", "read"];
        let allowed = adapter.enforce(&request).await?;
        assert!(allowed, "alice (admin) should be allowed to read data1");

        let request2 = vec!["bob", "data1", "read"];
        let allowed2 = adapter.enforce(&request2).await?;
        assert!(allowed2, "bob (user) should be allowed to read data1");

        let request3 = vec!["bob", "data1", "write"];
        let denied = adapter.enforce(&request3).await?;
        assert!(!denied, "bob (user) should not be allowed to write data1");

        Ok(())
    }

    #[tokio::test]
    async fn test_batch_enforcement() -> Result<(), CasbinWrapperError> {
        let dir = TempDir::new().unwrap();
        let model_path = create_basic_model(dir.path());
        let policy_path = create_basic_policy(dir.path());

        let adapter = CasbinAdapterExt::new(
            model_path.to_string_lossy().to_string(),
            policy_path.to_string_lossy().to_string(),
        )
        .await?;

        let requests = vec![
            vec!["alice", "data1", "read"],
            vec!["bob", "data1", "read"],
            vec!["charlie", "data1", "read"],
        ];

        let results = adapter.batch_enforce(&requests).await?;
        assert_eq!(results.len(), 3);
        assert!(results[0], "alice should be allowed");
        assert!(results[1], "bob should be allowed to read");
        assert!(!results[2], "charlie should be denied");

        Ok(())
    }

    #[tokio::test]
    async fn test_modify_policy() -> Result<(), CasbinWrapperError> {
        let dir = TempDir::new().unwrap();
        let model_path = create_basic_model(dir.path());
        let policy_path = create_basic_policy(dir.path());

        let adapter = CasbinAdapterExt::new(
            model_path.to_string_lossy().to_string(),
            policy_path.to_string_lossy().to_string(),
        )
        .await?;

        let request = vec!["charlie", "data1", "read"];
        let initially_denied = adapter.enforce(&request).await?;
        assert!(!initially_denied);

        let rules = vec![vec!["charlie".to_string(), "data1".to_string(), "read".to_string()]];
        adapter.modify_policy("p", rules.clone()).await?;

        let allowed = adapter.enforce(&request).await?;
        assert!(allowed, "charlie should now be allowed after policy update");

        adapter.remove_policy("p", rules).await?;

        let denied = adapter.enforce(&request).await?;
        assert!(!denied, "charlie should be denied after policy removal");

        Ok(())
    }

    #[tokio::test]
    async fn test_clear_policy() -> Result<(), CasbinWrapperError> {
        let dir = TempDir::new().unwrap();
        let model_path = create_basic_model(dir.path());
        let policy_path = create_basic_policy(dir.path());

        let adapter = CasbinAdapterExt::new(
            model_path.to_string_lossy().to_string(),
            policy_path.to_string_lossy().to_string(),
        )
        .await?;

        let request = vec!["alice", "data1", "read"];
        let initially_allowed = adapter.enforce(&request).await?;
        assert!(initially_allowed);

        adapter.clear_policy().await?;

        let denied = adapter.enforce(&request).await?;
        assert!(!denied, "alice should be denied after policy clear");

        Ok(())
    }

    #[tokio::test]
    async fn test_policy_reload() -> Result<(), CasbinWrapperError> {
        let dir = TempDir::new().unwrap();
        let model_path = create_basic_model(dir.path());
        let policy_path = create_basic_policy(dir.path());

        let adapter = CasbinAdapterExt::new(
            model_path.to_string_lossy().to_string(),
            policy_path.to_string_lossy().to_string(),
        )
        .await?;

        let rules = vec![vec!["charlie".to_string(), "data1".to_string(), "read".to_string()]];
        adapter.modify_policy("p", rules).await?;

        let request = vec!["charlie", "data1", "read"];
        let allowed = adapter.enforce(&request).await?;
        assert!(allowed);

        adapter.reload_policy().await?;

        let denied = adapter.enforce(&request).await?;
        assert!(!denied, "charlie should be denied after reload");

        Ok(())
    }
}
