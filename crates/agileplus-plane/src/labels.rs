//! T050: Label Sync — bidirectional label CRUD via Plane.so API.
//!
//! Traceability: WP08-T050

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::client::PlaneClient;

/// A label in Plane.so.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaneLabel {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub color: Option<String>,
}

/// Request payload for creating a label.
#[derive(Debug, Clone, Serialize)]
pub struct CreateLabelRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

/// Response envelope for label list.
#[derive(Debug, Clone, Deserialize)]
struct LabelListResponse {
    results: Vec<PlaneLabel>,
}

/// Label sync adapter.
#[derive(Debug)]
pub struct LabelSync {
    client: PlaneClient,
}

impl LabelSync {
    pub fn new(client: PlaneClient) -> Self {
        Self { client }
    }

    /// Fetch all labels from Plane.so for the configured project.
    pub async fn fetch_remote_labels(&self) -> Result<Vec<PlaneLabel>> {
        let url = self.client.labels_url();
        let resp = self
            .client
            .get_raw(&url)
            .await
            .context("fetching Plane.so labels")?;

        // Plane returns either a list directly or wrapped in `results`.
        // Try list first, then wrapped.
        if let Ok(list) = serde_json::from_str::<Vec<PlaneLabel>>(&resp) {
            return Ok(list);
        }
        let wrapped: LabelListResponse =
            serde_json::from_str(&resp).context("parsing label list response")?;
        Ok(wrapped.results)
    }

    /// Create a label in Plane.so.
    pub async fn create_remote_label(&self, name: &str, color: Option<&str>) -> Result<PlaneLabel> {
        let url = self.client.labels_url();
        let body = CreateLabelRequest {
            name: name.to_string(),
            color: color.map(|s| s.to_string()),
        };
        let json_body = serde_json::to_string(&body)?;
        let resp = self
            .client
            .post_raw(&url, &json_body)
            .await
            .context("creating Plane.so label")?;
        let label: PlaneLabel = serde_json::from_str(&resp).context("parsing created label")?;
        Ok(label)
    }

    /// Sync local labels to remote: create any that don't exist in Plane.so.
    ///
    /// Returns a mapping of label name → Plane label ID for all labels
    /// (both pre-existing and newly created).
    pub async fn sync_labels_to_remote(
        &self,
        local_labels: &[String],
    ) -> Result<std::collections::HashMap<String, String>> {
        let remote = self.fetch_remote_labels().await?;
        let mut name_to_id: std::collections::HashMap<String, String> = remote
            .into_iter()
            .map(|l| (l.name.clone(), l.id))
            .collect();

        for label in local_labels {
            if !name_to_id.contains_key(label.as_str()) {
                let created = self.create_remote_label(label, None).await?;
                tracing::info!(label_name = label, plane_label_id = created.id, "created remote label");
                name_to_id.insert(label.clone(), created.id);
            }
        }

        Ok(name_to_id)
    }

    /// Sync remote labels to local: return list of label names from Plane.so.
    pub async fn sync_labels_from_remote(&self) -> Result<Vec<String>> {
        let labels = self.fetch_remote_labels().await?;
        Ok(labels.into_iter().map(|l| l.name).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plane_label_deserialize() {
        let json = "{\"id\":\"abc\",\"name\":\"bug\",\"color\":\"#ff0000\"}";
        let label: PlaneLabel = serde_json::from_str(json).unwrap();
        assert_eq!(label.id, "abc");
        assert_eq!(label.name, "bug");
        assert_eq!(label.color.unwrap(), "#ff0000");
    }

    #[test]
    fn create_label_request_no_color() {
        let req = CreateLabelRequest {
            name: "enhancement".into(),
            color: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("enhancement"));
        assert!(!json.contains("color"));
    }
}
