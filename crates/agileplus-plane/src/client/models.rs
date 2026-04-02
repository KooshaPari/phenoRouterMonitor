use serde::{Deserialize, Serialize};

/// Work item representation in Plane.so.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaneWorkItem {
    pub id: Option<String>,
    pub name: String,
    pub description_html: Option<String>,
    pub state: Option<String>,
    pub priority: Option<i32>,
    pub parent: Option<String>,
    pub labels: Vec<String>,
}

/// Legacy name for [`PlaneWorkItem`] (features sync and outbound code).
pub type PlaneIssue = PlaneWorkItem;

/// Response from Plane.so API for work item creation/update.
#[derive(Debug, Clone, Deserialize)]
pub struct PlaneWorkItemResponse {
    pub id: String,
    pub name: String,
    pub description_html: Option<String>,
    pub state: Option<String>,
    pub updated_at: Option<String>,
}

/// Request body for creating/updating a Plane module.
#[derive(Debug, Clone, Serialize)]
pub struct PlaneCreateModuleRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Response from Plane.so module API.
#[derive(Debug, Clone, Deserialize)]
pub struct PlaneModuleResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

/// Request body for creating/updating a Plane cycle.
#[derive(Debug, Clone, Serialize)]
pub struct PlaneCreateCycleRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub start_date: String,
    pub end_date: String,
}

/// Response from Plane.so cycle API.
#[derive(Debug, Clone, Deserialize)]
pub struct PlaneCycleResponse {
    pub id: String,
    pub name: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}
