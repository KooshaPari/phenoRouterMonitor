//! Casbin adapter implementation.

use casbin::{CoreApi, Enforcer, MgmtApi};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::CasbinWrapperError;
use crate::models::ModelType;

pub type CasbinEnforcer = Arc<RwLock<Enforcer>>;

/// Trait for extending Casbin adapter functionality.
pub trait CasbinAdapterExt {
    async fn new(
        request_type: &str,
        policy_type: &str,
        model_path: &str,
        policy_path: &str,
    ) -> Result<Self, CasbinWrapperError>
    where
        Self: Sized;

    async fn enforce(
        &self,
        request: &[&str],
    ) -> Result<bool, CasbinWrapperError>;

    async fn enforce_named(
        &self,
        policy_type: &str,
        request: &[&str],
    ) -> Result<bool, CasbinWrapperError>;

    async fn modify_policy(
        &self,
        policy_type: &str,
        rules: Vec<Vec<String>>,
    ) -> Result<(), CasbinWrapperError>;

    async fn remove_policy(
        &self,
        policy_type: &str,
        rules: Vec<Vec<String>>,
    ) -> Result<(), CasbinWrapperError>;

    async fn clear_policy(&self) -> Result<(), CasbinWrapperError>;

    async fn reload_policy(&self) -> Result<(), CasbinWrapperError>;

    async fn batch_enforce(
        &self,
        requests: &[Vec<&str>],
    ) -> Result<Vec<bool>, CasbinWrapperError>;

    fn model_type(&self) -> ModelType;
}

pub struct CasbinAdapter {
    enforcer: CasbinEnforcer,
    model_type: ModelType,
}

impl CasbinAdapter {
    pub async fn new_with_enforcer(
        enforcer: Enforcer,
        model_type: ModelType,
    ) -> Result<Self, CasbinWrapperError> {
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
    async fn new(
        request_type: &str,
        policy_type: &str,
        model_path: &str,
        policy_path: &str,
    ) -> Result<Self, CasbinWrapperError> {
        let mut builder = casbin::Enforcer::new(model_path, policy_path)
            .await
            .map_err(|e| CasbinWrapperError::InitError(e.to_string()))?;

        builder
            .add_request_def(1, request_type)
            .map_err(|e| CasbinWrapperError::ModelError(e.to_string()))?;

        builder
            .add_policy_def(1, policy_type)
            .map_err(|e| CasbinWrapperError::ModelError(e.to_string()))?;

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
            enforcer: Arc::new(RwLock::new(builder)),
            model_type,
        })
    }

    async fn enforce(
        &self,
        request: &[&str],
    ) -> Result<bool, CasbinWrapperError> {
        let enforcer = self.enforcer.read().await;
        let result = enforcer.enforce(request).await?;
        Ok(result)
    }

    async fn enforce_named(
        &self,
        _policy_type: &str,
        request: &[&str],
    ) -> Result<bool, CasbinWrapperError> {
        self.enforce(request).await
    }

    async fn modify_policy(
        &self,
        policy_type: &str,
        rules: Vec<Vec<String>>,
    ) -> Result<(), CasbinWrapperError> {
        let mut enforcer = self.enforcer.write().await;

        for rule in rules {
            enforcer
                .add_policy(rule)
                .map_err(|e| CasbinWrapperError::PolicyError(e.to_string()))?;
        }

        tracing::info!("Modified policy {} with {} rules", policy_type, rules.len());
        Ok(())
    }

    async fn remove_policy(
        &self,
        policy_type: &str,
        rules: Vec<Vec<String>>,
    ) -> Result<(), CasbinWrapperError> {
        let mut enforcer = self.enforcer.write().await;

        for rule in rules {
            enforcer
                .remove_policy(rule)
                .map_err(|e| CasbinWrapperError::PolicyError(e.to_string()))?;
        }

        tracing::info!("Removed {} rules from policy {}", rules.len(), policy_type);
        Ok(())
    }

    async fn clear_policy(&self) -> Result<(), CasbinWrapperError> {
        let mut enforcer = self.enforcer.write().await;
        enforcer
            .clear_policy()
            .map_err(|e| CasbinWrapperError::PolicyError(e.to_string()))?;
        tracing::info!("Cleared all policies");
        Ok(())
    }

    async fn reload_policy(&self) -> Result<(), CasbinWrapperError> {
        let mut enforcer = self.enforcer.write().await;
        enforcer
            .reload_policy()
            .map_err(|e| CasbinWrapperError::PolicyError(e.to_string()))?;
        tracing::info!("Reloaded policies from disk");
        Ok(())
    }

    async fn batch_enforce(
        &self,
        requests: &[Vec<&str>],
    ) -> Result<Vec<bool>, CasbinWrapperError> {
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

    async fn create_test_enforcer() -> Result<CasbinAdapter, CasbinWrapperError> {
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

        std::fs::write(
            &policy_path,
            "p, alice, data1, read\np, bob, data1, read\n",
        )
        .unwrap();

        CasbinAdapterExt::new("r", "p", &model_path.to_str().unwrap(), &policy_path.to_str().unwrap())
            .await
    }

    #[tokio::test]
    async fn test_adapter_creation() -> Result<(), CasbinWrapperError> {
        let adapter = create_test_enforcer().await?;
        assert_eq!(adapter.model_type(), ModelType::Basic);
        Ok(())
    }

    #[tokio::test]
    async fn test_enforce() -> Result<(), CasbinWrapperError> {
        let adapter = create_test_enforcer().await?;

        let request = vec!["alice", "data1", "read"];
        let allowed = adapter.enforce(&request).await?;
        assert!(allowed);

        let request2 = vec!["bob", "data1", "read"];
        let allowed2 = adapter.enforce(&request2).await?;
        assert!(allowed2);

        let request3 = vec!["charlie", "data1", "read"];
        let denied = adapter.enforce(&request3).await?;
        assert!(!denied);

        Ok(())
    }

    #[tokio::test]
    async fn test_batch_enforce() -> Result<(), CasbinWrapperError> {
        let adapter = create_test_enforcer().await?;

        let requests = vec![
            vec!["alice", "data1", "read"],
            vec!["bob", "data1", "read"],
            vec!["charlie", "data1", "read"],
        ];

        let results = adapter.batch_enforce(&requests).await?;
        assert_eq!(results.len(), 3);
        assert!(results[0]);
        assert!(results[1]);
        assert!(!results[2]);

        Ok(())
    }
}
