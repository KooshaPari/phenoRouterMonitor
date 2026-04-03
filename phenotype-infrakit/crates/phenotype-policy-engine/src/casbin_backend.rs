//! Casbin backend adapter for policy engine.

use async_trait::async_trait;
use phenotype_casbin_wrapper::{CasbinAdapter, CasbinWrapperError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CasbinBackendError {
    #[error("Casbin error: {0}")]
    Casbin(#[from] CasbinWrapperError),
    #[error("Context conversion error: {0}")]
    ContextConversion(String),
    #[error("Backend not available")]
    NotAvailable,
}

#[derive(Debug, Clone)]
pub struct CasbinRequest {
    pub sub: String,
    pub obj: String,
    pub act: String,
}

impl CasbinRequest {
    pub fn new(sub: impl Into<String>, obj: impl Into<String>, act: impl Into<String>) -> Self {
        Self {
            sub: sub.into(),
            obj: obj.into(),
            act: act.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CasbinResponse {
    pub allowed: bool,
}

#[async_trait]
pub trait PolicyBackend: Send + Sync {
    async fn enforce(&self, request: CasbinRequest) -> Result<CasbinResponse, CasbinBackendError>;
    async fn batch_enforce(
        &self,
        requests: &[CasbinRequest],
    ) -> Result<Vec<CasbinResponse>, CasbinBackendError>;
    async fn modify_policy(
        &self,
        policy_type: &str,
        rules: Vec<Vec<String>>,
    ) -> Result<(), CasbinBackendError>;
    async fn remove_policy(
        &self,
        policy_type: &str,
        rules: Vec<Vec<String>>,
    ) -> Result<(), CasbinBackendError>;
    async fn reload_policy(&self) -> Result<(), CasbinBackendError>;
}

pub struct CasbinBackend {
    adapter: CasbinAdapter,
}

impl CasbinBackend {
    pub async fn new(model_path: String, policy_path: String) -> Result<Self, CasbinBackendError> {
        let adapter = CasbinAdapter::new(model_path, policy_path).await?;
        Ok(Self { adapter })
    }

    pub async fn from_adapter(adapter: CasbinAdapter) -> Self {
        Self { adapter }
    }

    pub fn adapter(&self) -> &CasbinAdapter {
        &self.adapter
    }
}

#[async_trait]
impl PolicyBackend for CasbinBackend {
    async fn enforce(&self, request: CasbinRequest) -> Result<CasbinResponse, CasbinBackendError> {
        let allowed = self
            .adapter
            .enforce(&request.sub, &request.obj, &request.act)
            .await?;
        Ok(CasbinResponse { allowed })
    }

    async fn batch_enforce(
        &self,
        requests: &[CasbinRequest],
    ) -> Result<Vec<CasbinResponse>, CasbinBackendError> {
        let casbin_requests: Vec<(&str, &str, &str)> = requests
            .iter()
            .map(|r| (r.sub.as_str(), r.obj.as_str(), r.act.as_str()))
            .collect();

        let results = self.adapter.batch_enforce(&casbin_requests).await?;
        Ok(results
            .into_iter()
            .map(|allowed| CasbinResponse { allowed })
            .collect())
    }

    async fn modify_policy(
        &self,
        policy_type: &str,
        rules: Vec<Vec<String>>,
    ) -> Result<(), CasbinBackendError> {
        self.adapter.modify_policy(policy_type, rules).await?;
        Ok(())
    }

    async fn remove_policy(
        &self,
        policy_type: &str,
        rules: Vec<Vec<String>>,
    ) -> Result<(), CasbinBackendError> {
        self.adapter.remove_policy(policy_type, rules).await?;
        Ok(())
    }

    async fn reload_policy(&self) -> Result<(), CasbinBackendError> {
        self.adapter.reload_policy().await?;
        Ok(())
    }
}

pub fn evaluation_context_to_casbin_request(
    ctx: &crate::EvaluationContext,
    sub_key: &str,
    obj_key: &str,
    act_key: &str,
) -> Result<CasbinRequest, CasbinBackendError> {
    let sub = ctx.get_string(sub_key).ok_or_else(|| {
        CasbinBackendError::ContextConversion(format!("Missing subject: {}", sub_key))
    })?;
    let obj = ctx.get_string(obj_key).ok_or_else(|| {
        CasbinBackendError::ContextConversion(format!("Missing object: {}", obj_key))
    })?;
    let act = ctx.get_string(act_key).ok_or_else(|| {
        CasbinBackendError::ContextConversion(format!("Missing action: {}", act_key))
    })?;

    Ok(CasbinRequest::new(sub, obj, act))
}

pub fn evaluation_context_to_casbin_requests(
    ctx: &crate::EvaluationContext,
    sub_key: &str,
    obj_key: &str,
    act_key: &str,
) -> Result<Vec<CasbinRequest>, CasbinBackendError> {
    let sub_values = extract_string_array(ctx, sub_key)?;
    let obj_values = extract_string_array(ctx, obj_key)?;
    let act_values = extract_string_array(ctx, act_key)?;

    if sub_values.len() != obj_values.len() || sub_values.len() != act_values.len() {
        return Err(CasbinBackendError::ContextConversion(
            "Subject, object, and action arrays must have the same length".to_string(),
        ));
    }

    Ok(sub_values
        .into_iter()
        .zip(obj_values)
        .zip(act_values)
        .map(|((sub, obj), act)| CasbinRequest::new(sub, obj, act))
        .collect())
}

fn extract_string_array(
    ctx: &crate::EvaluationContext,
    key: &str,
) -> Result<Vec<String>, CasbinBackendError> {
    match ctx.get(key) {
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .map(|v| v.as_str().map(String::from).ok_or(()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| {
                CasbinBackendError::ContextConversion(format!(
                    "Non-string value in array for key: {}",
                    key
                ))
            }),
        Some(serde_json::Value::String(s)) => Ok(vec![s.clone()]),
        None => Ok(vec![]),
        _ => Err(CasbinBackendError::ContextConversion(format!(
            "Expected array or string for key: {}",
            key
        ))),
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
    async fn test_backend_enforce_allowed() -> Result<(), CasbinBackendError> {
        let (_dir, model_path, policy_path) = create_test_files();
        let backend = CasbinBackend::new(model_path, policy_path).await?;

        let request = CasbinRequest::new("alice", "data1", "read");
        let response = backend.enforce(request).await?;

        assert!(response.allowed);
        Ok(())
    }

    #[tokio::test]
    async fn test_backend_enforce_denied() -> Result<(), CasbinBackendError> {
        let (_dir, model_path, policy_path) = create_test_files();
        let backend = CasbinBackend::new(model_path, policy_path).await?;

        let request = CasbinRequest::new("charlie", "data1", "read");
        let response = backend.enforce(request).await?;

        assert!(!response.allowed);
        Ok(())
    }

    #[tokio::test]
    async fn test_backend_batch_enforce() -> Result<(), CasbinBackendError> {
        let (_dir, model_path, policy_path) = create_test_files();
        let backend = CasbinBackend::new(model_path, policy_path).await?;

        let requests = vec![
            CasbinRequest::new("alice", "data1", "read"),
            CasbinRequest::new("bob", "data1", "read"),
            CasbinRequest::new("charlie", "data1", "read"),
        ];

        let responses = backend.batch_enforce(&requests).await?;

        assert_eq!(responses.len(), 3);
        assert!(responses[0].allowed);
        assert!(responses[1].allowed);
        assert!(!responses[2].allowed);
        Ok(())
    }

    #[tokio::test]
    async fn test_backend_modify_policy() -> Result<(), CasbinBackendError> {
        let (_dir, model_path, policy_path) = create_test_files();
        let backend = CasbinBackend::new(model_path, policy_path).await?;

        let denied = backend
            .enforce(CasbinRequest::new("charlie", "data1", "read"))
            .await?;
        assert!(!denied.allowed);

        let rules = vec![vec![
            "charlie".to_string(),
            "data1".to_string(),
            "read".to_string(),
        ]];
        backend.modify_policy("p", rules).await?;

        let allowed = backend
            .enforce(CasbinRequest::new("charlie", "data1", "read"))
            .await?;
        assert!(allowed.allowed);
        Ok(())
    }

    #[tokio::test]
    async fn test_backend_reload_policy() -> Result<(), CasbinBackendError> {
        let (_dir, model_path, policy_path) = create_test_files();
        let backend = CasbinBackend::new(model_path, policy_path).await?;

        backend.reload_policy().await?;
        Ok(())
    }

    #[test]
    fn test_casbin_request_creation() {
        let request = CasbinRequest::new("user", "resource", "action");
        assert_eq!(request.sub, "user");
        assert_eq!(request.obj, "resource");
        assert_eq!(request.act, "action");
    }

    #[test]
    fn test_context_to_request_single_values() {
        let mut ctx = crate::EvaluationContext::new();
        ctx.set_string("subject", "alice");
        ctx.set_string("object", "data1");
        ctx.set_string("action", "read");

        let request =
            evaluation_context_to_casbin_request(&ctx, "subject", "object", "action").unwrap();

        assert_eq!(request.sub, "alice");
        assert_eq!(request.obj, "data1");
        assert_eq!(request.act, "read");
    }

    #[test]
    fn test_context_to_request_missing_key() {
        let ctx = crate::EvaluationContext::new();

        let result = evaluation_context_to_casbin_request(&ctx, "sub", "obj", "act");
        assert!(result.is_err());
    }

    #[test]
    fn test_context_to_requests_arrays() {
        let mut ctx = crate::EvaluationContext::new();
        ctx.set("subjects", serde_json::json!(["alice", "bob"]));
        ctx.set("objects", serde_json::json!(["data1", "data1"]));
        ctx.set("actions", serde_json::json!(["read", "read"]));

        let requests =
            evaluation_context_to_casbin_requests(&ctx, "subjects", "objects", "actions").unwrap();

        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].sub, "alice");
        assert_eq!(requests[1].sub, "bob");
    }
}
