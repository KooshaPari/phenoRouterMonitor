use anyhow::{Context, Result};
use reqwest::Method;

use super::{PlaneClient, PlaneCreateCycleRequest, PlaneCycleResponse, transport};

impl PlaneClient {
    /// Create a Cycle in Plane.so. Returns Plane's cycle UUID.
    pub async fn create_cycle(&self, req: &PlaneCreateCycleRequest) -> Result<PlaneCycleResponse> {
        self.acquire_token().await?;
        let resp = self
            .execute_request_json(Method::POST, &self.cycles_url(), req)
            .await
            .context("Plane.so create cycle request failed")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Plane.so cycle create failed: HTTP {status}: {body}");
        }
        resp.json().await.context("parsing Plane.so cycle response")
    }

    /// Update a Cycle in Plane.so (PATCH).
    pub async fn update_cycle(
        &self,
        plane_cycle_id: &str,
        req: &PlaneCreateCycleRequest,
    ) -> Result<()> {
        self.acquire_token().await?;
        let resp = self
            .execute_request_json(Method::PATCH, &self.cycle_url(plane_cycle_id), req)
            .await
            .context("Plane.so update cycle request failed")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Plane.so cycle update failed: HTTP {status}: {body}");
        }
        Ok(())
    }

    /// Delete a Cycle in Plane.so.
    pub async fn delete_cycle(&self, plane_cycle_id: &str) -> Result<()> {
        self.acquire_token().await?;
        let resp = self
            .execute_request_without_body(Method::DELETE, &self.cycle_url(plane_cycle_id))
            .await
            .context("Plane.so delete cycle request failed")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Plane.so cycle delete failed: HTTP {status}: {body}");
        }
        Ok(())
    }

    /// Add a Plane work item to a Plane cycle.
    pub async fn add_work_item_to_cycle(
        &self,
        plane_cycle_id: &str,
        plane_work_item_id: &str,
    ) -> Result<()> {
        self.acquire_token().await?;
        let body = serde_json::json!({ "issues": [plane_work_item_id] });
        let resp = transport::request_json_value(
            &self.client,
            Method::POST,
            &self.cycle_work_items_url(plane_cycle_id),
            &self.api_key,
            &body,
        )
        .await
        .context("Plane.so add work item to cycle request failed")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Plane.so add work item to cycle failed: HTTP {status}: {body}");
        }
        Ok(())
    }

    /// Remove a Plane work item from a Plane cycle.
    pub async fn delete_work_item_from_cycle(
        &self,
        plane_cycle_id: &str,
        plane_work_item_id: &str,
    ) -> Result<()> {
        self.acquire_token().await?;
        let resp = self
            .execute_request_without_body(
                Method::DELETE,
                &self.cycle_work_item_url(plane_cycle_id, plane_work_item_id),
            )
            .await
            .context("Plane.so delete work item from cycle request failed")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Plane.so delete work item from cycle failed: HTTP {status}: {body}");
        }
        Ok(())
    }
}
