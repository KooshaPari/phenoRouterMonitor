//! Casbin adapter for Phenotype policy engine
//! Simplified implementation for casbin v2 API

use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use casbin::{CoreApi, Enforcer, Model};
use phenotype_policy_engine::adapter::{Adapter, Filter, LoadError};

/// Casbin adapter implementation
#[derive(Clone)]
pub struct CasbinAdapter {
    enforcer: Arc<RwLock<Enforcer>>,
}

impl CasbinAdapter {
    /// Create a new CasbinAdapter from model and policy text
    pub async fn new(model_text: &str, policy_text: &str) -> Result<Self, String> {
        // Create model from text
        let model = Model::from_str(model_text)
            .map_err(|e| format!("Failed to create model: {}", e))?;
        
        // Create enforcer
        let mut enforcer = Enforcer::new(model, casbin::MemoryAdapter::default())
            .await
            .map_err(|e| format!("Failed to create enforcer: {}", e))?;
        
        // Add policies
        for line in policy_text.lines() {
            if !line.is_empty() && !line.trim().starts_with('#') {
                let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                if !parts.is_empty() {
                    let _ = enforcer.add_policy(&parts);
                }
            }
        }
        
        Ok(Self {
            enforcer: Arc::new(RwLock::new(enforcer)),
        })
    }
}

#[async_trait]
impl Adapter for CasbinAdapter {
    async fn enforce(&self, sub: String, obj: String, act: String) -> Result<bool, LoadError> {
        let enforcer = self.enforcer.read().unwrap();
        enforcer
            .enforce(&[sub, obj, act])
            .await
            .map_err(|e| LoadError(e.to_string()))
    }

    async fn load_policies(&self) -> Result<(), LoadError> {
        // Policies are loaded in new()
        Ok(())
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    const MODEL: &str = r#"
[request_definition]
r = sub, obj, act

[policy_definition]
p = sub, obj, act

[policy_effect]
e = some(where (p.eft == allow))

[matchers]
m = r.sub == p.sub && r.obj == p.obj && r.act == p.act
"#;

    const POLICY: &str = r#"
p, alice, data1, read
p, bob, data2, write
p, data1_admin, data1, write
"#;

    #[tokio::test]
    async fn test_basic_enforcement() {
        let adapter = CasbinAdapter::new(MODEL, POLICY).await;
        assert!(adapter.is_ok());
        
        let adapter = adapter.unwrap();
        assert!(adapter.enforce("alice".into(), "data1".into(), "read".into()).await.unwrap());
        assert!(!adapter.enforce("bob".into(), "data1".into(), "read".into()).await.unwrap());
    }

    #[tokio::test]
    async fn test_multi_policy() {
        let adapter = CasbinAdapter::new(MODEL, POLICY).await.unwrap();
        
        // Test multiple subjects
        assert!(adapter.enforce("alice".into(), "data1".into(), "read".into()).await.unwrap());
        assert!(adapter.enforce("bob".into(), "data2".into(), "write".into()).await.unwrap());
        assert!(adapter.enforce("data1_admin".into(), "data1".into(), "write".into()).await.unwrap());
    }
}
