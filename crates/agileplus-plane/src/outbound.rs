//! T048: Outbound Sync — push features and work packages to Plane.so.
//!
//! Traceability: WP08-T048

use anyhow::{Context, Result};
use agileplus_domain::domain::feature::Feature;

use crate::{
    client::{PlaneClient, PlaneIssue},
    state_mapper::PlaneStateMapper,
};

/// Outbound sync adapter for pushing AgilePlus entities to Plane.so.
#[derive(Debug)]
pub struct OutboundSync {
    client: PlaneClient,
    mapper: PlaneStateMapper,
}

impl OutboundSync {
    pub fn new(client: PlaneClient, mapper: PlaneStateMapper) -> Self {
        Self { client, mapper }
    }

    /// Push a Feature to Plane.so.
    ///
    /// - Creates a new issue if `feature.plane_issue_id` is None.
    /// - Updates the existing issue via PATCH if it exists.
    ///
    /// Returns the Plane.so issue ID.
    pub async fn push_feature(&self, feature: &Feature) -> Result<String> {
        let (_group, state_id) = self.mapper.to_plane(feature.state);
        let state_opt = if state_id.is_empty() {
            None
        } else {
            Some(state_id)
        };

        let issue = PlaneIssue {
            id: None,
            name: feature.friendly_name.clone(),
            description_html: None, // description not on Feature; extend if needed
            state: state_opt,
            priority: Some(2),
            parent: None,
            labels: feature.labels.clone(),
        };

        let issue_id = if let Some(ref existing_id) = feature.plane_issue_id {
            let resp = self
                .client
                .update_issue(existing_id, &issue)
                .await
                .with_context(|| format!("updating Plane issue {existing_id}"))?;
            tracing::info!(
                feature_slug = feature.slug,
                plane_issue_id = resp.id,
                "updated Plane.so issue"
            );
            resp.id
        } else {
            let resp = self
                .client
                .create_issue(&issue)
                .await
                .with_context(|| format!("creating Plane issue for feature {}", feature.slug))?;
            tracing::info!(
                feature_slug = feature.slug,
                plane_issue_id = resp.id,
                "created Plane.so issue"
            );
            resp.id
        };

        Ok(issue_id)
    }

    /// Push a work package as a sub-issue under a parent Plane.so issue.
    ///
    /// `parent_plane_id` is the Plane.so issue ID of the parent feature.
    /// `wp_plane_id` is the existing Plane sub-issue ID, if any.
    /// Returns the Plane.so sub-issue ID.
    pub async fn push_work_package(
        &self,
        wp_id: &str,
        title: &str,
        description: Option<&str>,
        labels: &[String],
        parent_plane_id: &str,
        wp_plane_id: Option<&str>,
    ) -> Result<String> {
        let desc_html = description.map(|d| format!("<p>{d}</p>"));
        let issue = PlaneIssue {
            id: None,
            name: format!("[{wp_id}] {title}"),
            description_html: desc_html,
            state: None,
            priority: Some(3),
            parent: Some(parent_plane_id.to_string()),
            labels: labels.to_vec(),
        };

        let issue_id = if let Some(existing_id) = wp_plane_id {
            let resp = self
                .client
                .update_issue(existing_id, &issue)
                .await
                .with_context(|| format!("updating Plane sub-issue {existing_id}"))?;
            tracing::info!(wp_id, plane_issue_id = resp.id, "updated Plane.so sub-issue");
            resp.id
        } else {
            let resp = self
                .client
                .create_issue(&issue)
                .await
                .with_context(|| format!("creating Plane sub-issue for WP {wp_id}"))?;
            tracing::info!(
                wp_id,
                plane_issue_id = resp.id,
                "created Plane.so sub-issue"
            );
            resp.id
        };

        Ok(issue_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_mapper::PlaneStateMapper;

    #[test]
    fn outbound_sync_constructs() {
        let client = PlaneClient::new(
            "http://localhost".into(),
            "key".into(),
            "slug".into(),
            "project".into(),
        );
        let mapper = PlaneStateMapper::new();
        let _sync = OutboundSync::new(client, mapper);
    }
}
