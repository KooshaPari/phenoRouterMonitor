//! Model types for Casbin policies.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ModelType {
    Basic,
    Abac,
    Rbac,
    RbacWithResourceRoles,
    RbacWithDomains,
    Acl,
    AclWithResource,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyEffect {
    AllowOverride,
    DenyOverride,
    AllowAndDeny,
    Priority,
    SubjectPriority,
    NoType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub model_type: ModelType,
    pub request_definition: RequestDef,
    pub policy_definition: PolicyDef,
    pub policy_effect: PolicyEffect,
    pub matchers: Vec<Matcher>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestDef {
    pub request_type: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDef {
    pub policy_type: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Matcher {
    pub matcher: String,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            model_type: ModelType::Basic,
            request_definition: RequestDef {
                request_type: vec!["r".to_string(), "p".to_string()],
            },
            policy_definition: PolicyDef {
                policy_type: vec!["r".to_string(), "p".to_string()],
            },
            policy_effect: PolicyEffect::AllowOverride,
            matchers: vec![Matcher {
                matcher: "m".to_string(),
            }],
        }
    }
}
