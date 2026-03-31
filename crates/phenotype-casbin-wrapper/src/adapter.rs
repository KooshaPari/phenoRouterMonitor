//! Casbin adapter implementation.

use casbin::{CoreApi, Enforcer, MgmtApi};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::CasbinWrapperError;
use crate::models::ModelType;

pub type CasbinEnforcer = Arc<RwLock<Enforcer>>;

#[async_trait::async_trait]
pub trait CasbinAdapterExt: Send + Sync {
    async fn new(model_path: String, policy_path: String) -> Result<Self, CasbinWrapperError>
    where
        Self: Sized;

    async fn enforce(&self, request: &[&str]) -> Result<bool, CasbinWrapperError>;

    async fn enforce_named(&self, policy_type: &str, request: &[&str]) -> Result<bool, CasbinWrapperError>;

    async fn modify_policy(&self, policy_type: &str, rules: Vec<Vec<String>>) -> Result<(), CasbinWrapperError>;

    async fn remove_policy(&self, policy_type: &str, rules: Vec<Vec<String>>) -> Result<(), CasbinWrapperError>;

    async fn clear_policy(&self) -> Result<(), CasbinWrapperError>;

    async fn reload_policy(&self) -> Result<(), CasbinWrapperError>;

    async fn batch_enforce(&self, requests: &[Vec<&str>]) -> Result<Vec<bool>, CasbinWrapperError>;

    fn model_type(&self) -> ModelType;
}

pub struct CasbinAdapter {
    enforcer: CasbinEnforcer,
    model_type: ModelType,
}

impl CasbinAdapter {
    pub async fn new_with_enforcer(enforcer: Enforcer, model_type: ModelType) -> Result<Self, CasbinWrapperError> {
        Ok(Self {
            enforcer: Arc::new(RwLock::new(enforcer)),
            model_type,
        })
    }

    pub fn enforcer(&self) -> CasbinEnforcer {
        self.enforcer.clone()
    }
}

#[async_trait::async_trait]
impl CasbinAdapterExt for CasbinAdapter {
    async fn new(model_path: String, policy_path: String) -> Result<Self, CasbinWrapperError> {
        let enforcer = Enforcer::new(model_path.as_str(), policy_path.as_str())
            .await
            .map_err(|e| CasbinWrapperError::InitError(e.to_string()))?;

        let model_type = if model_path.contains("rbac") {
            ModelType::Rbac
        } else if model_path.contains("abac") {
            ModelType::Abac
        } else if model_path.contains("acl") {
            ModelType::Acl
        } else {
            ModelType::Basic
        };

        Ok(Self {
            enforcer: Arc::new(RwLock::new(enforcer)),
            model_type,
        })
    }

    async fn enforce(&self, request: &[&str]) -> Result<bool, CasbinWrapperError> {
        let enforcer = self.enforcer.read().await;
        let result = match request.len() {
            1 => enforcer.enforce((request[0],)),
            2 => enforcer.enforce((request[0], request[1])),
            3 => enforcer.enforce((request[0], request[1], request[2])),
            4 => enforcer.enforce((request[0], request[1], request[2], request[3])),
            _ => return Err(CasbinWrapperError::EnforcementFailed(
                "Unsupported request arity".to_string(),
            )),
        };
        result.map_err(|e| CasbinWrapperError::EnforcementFailed(e.to_string()))
    }

    async fn enforce_named(&self, _policy_type: &str, request: &[&str]) -> Result<bool, CasbinWrapperError> {
        self.enforce(request).await
    }

    async fn modify_policy(&self, policy_type: &str, rules: Vec<Vec<String>>) -> Result<(), CasbinWrapperError> {
        let mut enforcer = self.enforcer.write().await;
        let rule_count = rules.len();

        for rule in &rules {
            enforcer
                .add_policy(rule.clone())
                .await
                .map_err(|e| CasbinWrapperError::PolicyError(e.to_string()))?;
        }

        tracing::info!("Modified policy {} with {} rules", policy_type, rule_count);
        Ok(())
    }

    async fn remove_policy(&self, policy_type: &str, rules: Vec<Vec<String>>) -> Result<(), CasbinWrapperError> {
        let mut enforcer = self.enforcer.write().await;
        let rule_count = rules.len();

        for rule in &rules {
            enforcer
                .remove_policy(rule.clone())
                .await
                .map_err(|e| CasbinWrapperError::PolicyError(e.to_string()))?;
        }

        tracing::info!("Removed {} rules from policy {}", rule_count, policy_type);
        Ok(())
    }

    async fn clear_policy(&self) -> Result<(), CasbinWrapperError> {
        let mut enforcer = self.enforcer.write().await;
        enforcer
            .clear_policy()
            .await
            .map_err(|e| CasbinWrapperError::PolicyError(e.to_string()))?;
        tracing::info!("Cleared all policies");
        Ok(())
    }

    async fn reload_policy(&self) -> Result<(), CasbinWrapperError> {
        let mut enforcer = self.enforcer.write().await;
        enforcer
            .load_policy()
            .await
            .map_err(|e| CasbinWrapperError::PolicyError(e.to_string()))?;
        tracing::info!("Reloaded policies from disk");
        Ok(())
    }

    async fn batch_enforce(&self, requests: &[Vec<&str>]) -> Result<Vec<bool>, CasbinWrapperError> {
        let mut results = Vec::with_capacity(requests.len());

        for request in requests {
            let allowed = self.enforce(request).await?;
            results.push(allowed);
        }

        Ok(results)
    }

    fn model_type(&self) -> ModelType {
        self.model_type.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_basic_model_file(dir: &std::path::Path) -> std::path::PathBuf {
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

    fn create_rbac_model_file(dir: &std::path::Path) -> std::path::PathBuf {
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

    fn create_basic_policy_file(dir: &std::path::Path) -> std::path::PathBuf {
        let policy_path = dir.join("policy.csv");
        std::fs::write(&policy_path, "p, alice, data1, read\np, bob, data1, read\n")
            .unwrap();
        policy_path
    }

    fn create_rbac_policy_file(dir: &std::path::Path) -> std::path::PathBuf {
        let policy_path = dir.join("rbac_policy.csv");
        std::fs::write(
            &policy_path,
            "p, alice, data1, read\np, alice, data1, write\np, bob, data1, read\np, bob, data2, read\ng, bob, user\ng, alice, admin\n",
        )
        .unwrap();
        policy_path
    }

    #[tokio::test]
    async fn test_basic_enforcement() -> Result<(), CasbinWrapperError> {
        let dir = TempDir::new().unwrap();
        let model_path = create_basic_model_file(dir.path());
        let policy_path = create_basic_policy_file(dir.path());

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
        let model_path = create_rbac_model_file(dir.path());
        let policy_path = create_rbac_policy_file(dir.path());

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
        let model_path = create_basic_model_file(dir.path());
        let policy_path = create_basic_policy_file(dir.path());

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
        let model_path = create_basic_model_file(dir.path());
        let policy_path = create_basic_policy_file(dir.path());

        let adapter = CasbinAdapterExt::new(
            model_path.to_string_lossy().to_string(),
            policy_path.to_string_lossy().to_string(),
        )
        .await?;

        let request = vec!["charlie", "data1", "read"];
        let initially_denied = adapter.enforce(&request).await?;
        assert!(!initially_denied, "charlie should initially be denied");

        let rules = vec![vec!["charlie".to_string(), "data1".to_string(), "read".to_string()]];
        adapter.modify_policy("p", rules.clone()).await?;

        let allowed = adapter.enforce(&request).await?;
        assert!(allowed, "charlie should be allowed after policy update");

        adapter.remove_policy("p", rules).await?;

        let denied = adapter.enforce(&request).await?;
        assert!(!denied, "charlie should be denied after policy removal");

        Ok(())
    }

    #[tokio::test]
    async fn test_clear_policy() -> Result<(), CasbinWrapperError> {
        let dir = TempDir::new().unwrap();
        let model_path = create_basic_model_file(dir.path());
        let policy_path = create_basic_policy_file(dir.path());

        let adapter = CasbinAdapterExt::new(
            model_path.to_string_lossy().to_string(),
            policy_path.to_string_lossy().to_string(),
        )
        .await?;

        let request = vec!["alice", "data1", "read"];
        let initially_allowed = adapter.enforce(&request).await?;
        assert!(initially_allowed, "alice should initially be allowed");

        adapter.clear_policy().await?;

        let denied = adapter.enforce(&request).await?;
        assert!(!denied, "alice should be denied after policy clear");

        Ok(())
    }

    #[tokio::test]
    async fn test_policy_reload() -> Result<(), CasbinWrapperError> {
        let dir = TempDir::new().unwrap();
        let model_path = create_basic_model_file(dir.path());
        let policy_path = create_basic_policy_file(dir.path());

        let adapter = CasbinAdapterExt::new(
            model_path.to_string_lossy().to_string(),
            policy_path.to_string_lossy().to_string(),
        )
        .await?;

        let rules = vec![vec!["charlie".to_string(), "data1".to_string(), "read".to_string()]];
        adapter.modify_policy("p", rules).await?;

        let request = vec!["charlie", "data1", "read"];
        let allowed = adapter.enforce(&request).await?;
        assert!(allowed, "charlie should be allowed after policy modification");

        adapter.reload_policy().await?;

        let denied = adapter.enforce(&request).await?;
        assert!(!denied, "charlie should be denied after reload");

        Ok(())
    }

    #[tokio::test]
    async fn test_model_type_detection() -> Result<(), CasbinWrapperError> {
        let dir = TempDir::new().unwrap();
        let model_path = create_basic_model_file(dir.path());
        let policy_path = create_basic_policy_file(dir.path());

        let adapter = CasbinAdapterExt::new(
            model_path.to_string_lossy().to_string(),
            policy_path.to_string_lossy().to_string(),
        )
        .await?;
        assert_eq!(adapter.model_type(), ModelType::Basic);

        Ok(())
    }
}
