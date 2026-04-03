use anyhow::Context;
use reqwest::Method;

use super::{PlaneClient, PlaneIssue, PlaneWorkItem, PlaneWorkItemResponse, transport};

impl PlaneClient {
    /// Create a work item in Plane.so.
    pub async fn create_work_item(
        &self,
        work_item: &PlaneWorkItem,
    ) -> anyhow::Result<PlaneWorkItemResponse> {
        self.acquire_token().await?;
        let resp = self
            .execute_request_json(Method::POST, &self.work_items_url(), work_item)
            .await
            .context("Plane.so create work item request failed")?;
        transport::read_json_response(resp, "parsing Plane.so response").await
    }

    /// Update an existing work item.
    pub async fn update_work_item(
        &self,
        work_item_id: &str,
        work_item: &PlaneWorkItem,
    ) -> anyhow::Result<PlaneWorkItemResponse> {
        self.acquire_token().await?;
        let resp = self
            .execute_request_json(Method::PATCH, &self.work_item_url(work_item_id), work_item)
            .await
            .context("Plane.so update work item request failed")?;
        transport::read_json_response(resp, "parsing Plane.so response").await
    }

    /// Get a work item by ID.
    pub async fn get_work_item(&self, work_item_id: &str) -> anyhow::Result<PlaneWorkItemResponse> {
        self.acquire_token().await?;
        let resp = self
            .execute_request_without_body(Method::GET, &self.work_item_url(work_item_id))
            .await
            .context("Plane.so get work item request failed")?;
        transport::read_json_response(resp, "parsing Plane.so response").await
    }

    // --- Back-compat aliases (outbound / sync use "issue" naming) ---

    /// Alias for [`Self::create_work_item`].
    pub async fn create_issue(&self, issue: &PlaneIssue) -> anyhow::Result<PlaneWorkItemResponse> {
        self.create_work_item(issue).await
    }

    /// Alias for [`Self::update_work_item`].
    pub async fn update_issue(
        &self,
        issue_id: &str,
        issue: &PlaneIssue,
    ) -> anyhow::Result<PlaneWorkItemResponse> {
        self.update_work_item(issue_id, issue).await
    }

    /// Alias for [`Self::get_work_item`].
    pub async fn get_issue(&self, issue_id: &str) -> anyhow::Result<PlaneWorkItemResponse> {
        self.get_work_item(issue_id).await
    }

    /// Alias for [`Self::add_work_item_to_module`].
    pub async fn add_issue_to_module(
        &self,
        plane_module_id: &str,
        plane_issue_id: &str,
    ) -> anyhow::Result<()> {
        self.add_work_item_to_module(plane_module_id, plane_issue_id)
            .await
    }

    /// Alias for [`Self::add_work_item_to_cycle`].
    pub async fn add_issue_to_cycle(
        &self,
        plane_cycle_id: &str,
        plane_issue_id: &str,
    ) -> anyhow::Result<()> {
        self.add_work_item_to_cycle(plane_cycle_id, plane_issue_id)
            .await
    }
}
