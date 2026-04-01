//! Casbin adapter for Phenotype policy engine.

use casbin::{CoreApi, DefaultModel, Enforcer, FileAdapter, MgmtApi};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::CasbinWrapperError;
use crate::models::ModelType;

pub struct CasbinAdapter {
    enforcer: Arc<RwLock<Enforcer>>,
    model_type: ModelType,
}

impl CasbinAdapter {
    pub async fn new(model_path: String, policy_path: String) -> Result<Self, CasbinWrapperError> {
        let model_path_static: &'static str = Box::leak(model_path.into_boxed_str());
        let policy_path_static: &'static str = Box::leak(policy_path.into_boxed_str());

        let model = DefaultModel::from_file(model_path_static)
            .await
            .map_err(|e| CasbinWrapperError::ModelError(e.to_string()))?;

        let adapter = FileAdapter::new(policy_path_static);

        let enforcer = Enforcer::new(model, adapter)
            .await
            .map_err(|e| CasbinWrapperError::InitError(e.to_string()))?;

        let model_type = if model_path_static.contains("rbac") {
            ModelType::Rbac
        } else if model_path_static.contains("abac") {
            ModelType::Abac
        } else if model_path_static.contains("acl") {
            ModelType::Acl
        } else {
            ModelType::Basic
        };

        Ok(Self {
            enforcer: Arc::new(RwLock::new(enforcer)),
            model_type,
        })
    }

    pub async fn enforce(
        &self,
        sub: &str,
        obj: &str,
        act: &str,
    ) -> Result<bool, CasbinWrapperError> {
        let enforcer = self.enforcer.read().await;
        let result = enforcer.enforce((sub, obj, act))?;
        Ok(result)
    }

    pub async fn enforce_named(
        &self,
        _policy_type: &str,
        sub: &str,
        obj: &str,
        act: &str,
    ) -> Result<bool, CasbinWrapperError> {
        self.enforce(sub, obj, act).await
    }

    pub async fn modify_policy(
        &self,
        policy_type: &str,
        rules: Vec<Vec<String>>,
    ) -> Result<(), CasbinWrapperError> {
        let mut enforcer = self.enforcer.write().await;
        let count = rules.len();

        for rule in &rules {
            enforcer
                .add_policy(rule.clone())
                .await
                .map_err(|e| CasbinWrapperError::PolicyError(e.to_string()))?;
        }

        tracing::info!("Modified policy {} with {} rules", policy_type, count);
        Ok(())
    }

    pub async fn remove_policy(
        &self,
        policy_type: &str,
        rules: Vec<Vec<String>>,
    ) -> Result<(), CasbinWrapperError> {
        let mut enforcer = self.enforcer.write().await;
        let count = rules.len();

        for rule in &rules {
            enforcer
                .remove_policy(rule.clone())
                .await
                .map_err(|e| CasbinWrapperError::PolicyError(e.to_string()))?;
        }

        tracing::info!("Removed {} rules from policy {}", count, policy_type);
        Ok(())
    }

    pub async fn clear_policy(&self) -> Result<(), CasbinWrapperError> {
        let mut enforcer = self.enforcer.write().await;
        enforcer
            .clear_policy()
            .await
            .map_err(|e| CasbinWrapperError::PolicyError(e.to_string()))?;
        tracing::info!("Cleared all policies");
        Ok(())
    }

    pub async fn reload_policy(&self) -> Result<(), CasbinWrapperError> {
        let mut enforcer = self.enforcer.write().await;
        enforcer
            .load_policy()
            .await
            .map_err(|e| CasbinWrapperError::PolicyError(e.to_string()))?;
        tracing::info!("Reloaded policies from disk");
        Ok(())
    }

    pub async fn batch_enforce(
        &self,
        requests: &[(&str, &str, &str)],
    ) -> Result<Vec<bool>, CasbinWrapperError> {
        let mut results = Vec::with_capacity(requests.len());

        for (sub, obj, act) in requests {
            let allowed = self.enforce(sub, obj, act).await?;
            results.push(allowed);
        }

        Ok(results)
    }

    pub fn model_type(&self) -> ModelType {
        self.model_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_files() -> (TempDir, String, String) {
        let dir = TempDir::new().unwrap();
        let model_path = dir.path().join("model.conf");
        let policy_path = dir.path().join("policy.csv");

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

        std::fs::write(&policy_path, "p, alice, data1, read\np, bob, data1, read\n").unwrap();

        (
            dir,
            model_path.to_string_lossy().to_string(),
            policy_path.to_string_lossy().to_string(),
        )
    }

    #[tokio::test]
    async fn test_adapter_creation() -> Result<(), CasbinWrapperError> {
        let (_dir, model_path, policy_path) = create_test_files();
        let adapter = CasbinAdapter::new(model_path, policy_path).await?;
        assert_eq!(adapter.model_type(), ModelType::Basic);
        Ok(())
    }

    #[tokio::test]
    async fn test_enforce() -> Result<(), CasbinWrapperError> {
        let (_dir, model_path, policy_path) = create_test_files();
        let adapter = CasbinAdapter::new(model_path, policy_path).await?;

        let allowed = adapter.enforce("alice", "data1", "read").await?;
        assert!(allowed);

        let allowed2 = adapter.enforce("bob", "data1", "read").await?;
        assert!(allowed2);

        let denied = adapter.enforce("charlie", "data1", "read").await?;
        assert!(!denied);

        Ok(())
    }

    #[tokio::test]
    async fn test_batch_enforce() -> Result<(), CasbinWrapperError> {
        let (_dir, model_path, policy_path) = create_test_files();
        let adapter = CasbinAdapter::new(model_path, policy_path).await?;

        let requests = vec![
            ("alice", "data1", "read"),
            ("bob", "data1", "read"),
            ("charlie", "data1", "read"),
        ];

        let results = adapter.batch_enforce(&requests).await?;
        assert_eq!(results.len(), 3);
        assert!(results[0]);
        assert!(results[1]);
        assert!(!results[2]);

        Ok(())
    }

    #[tokio::test]
    async fn test_modify_policy() -> Result<(), CasbinWrapperError> {
        let (_dir, model_path, policy_path) = create_test_files();
        let adapter = CasbinAdapter::new(model_path, policy_path).await?;

        let denied = adapter.enforce("charlie", "data1", "read").await?;
        assert!(!denied);

        let rules = vec![vec![
            "charlie".to_string(),
            "data1".to_string(),
            "read".to_string(),
        ]];
        adapter.modify_policy("p", rules).await?;

        let allowed = adapter.enforce("charlie", "data1", "read").await?;
        assert!(allowed);

        Ok(())
    }
}
