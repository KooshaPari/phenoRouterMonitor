use anyhow::{Context, Result};
use reqwest::Method;

use super::{PlaneClient, PlaneCreateModuleRequest, PlaneModuleResponse, transport};

impl PlaneClient {
    /// Create a Module in Plane.so. Returns Plane's module UUID.
    pub async fn create_module(
        &self,
        req: &PlaneCreateModuleRequest,
    ) -> Result<PlaneModuleResponse> {
        self.acquire_token().await?;
        let resp = self
            .execute_request_json(Method::POST, &self.modules_url(), req)
            .await
            .context("Plane.so create module request failed")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Plane.so module create failed: HTTP {status}: {body}");
        }
        resp.json()
            .await
            .context("parsing Plane.so module response")
    }

    /// Update a Module in Plane.so (PATCH).
    pub async fn update_module(
        &self,
        plane_module_id: &str,
        req: &PlaneCreateModuleRequest,
    ) -> Result<()> {
        self.acquire_token().await?;
        let resp = self
            .execute_request_json(Method::PATCH, &self.module_url(plane_module_id), req)
            .await
            .context("Plane.so update module request failed")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Plane.so module update failed: HTTP {status}: {body}");
        }
        Ok(())
    }

    /// Delete a Module in Plane.so.
    pub async fn delete_module(&self, plane_module_id: &str) -> Result<()> {
        self.acquire_token().await?;
        let resp = self
            .execute_request_without_body(Method::DELETE, &self.module_url(plane_module_id))
            .await
            .context("Plane.so delete module request failed")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Plane.so module delete failed: HTTP {status}: {body}");
        }
        Ok(())
    }

    /// Add a Plane work item to a Plane module.
    pub async fn add_work_item_to_module(
        &self,
        plane_module_id: &str,
        plane_work_item_id: &str,
    ) -> Result<()> {
        self.acquire_token().await?;
        let body = serde_json::json!({ "issues": [plane_work_item_id] });
        let resp = transport::request_json_value(
            &self.client,
            Method::POST,
            &self.module_work_items_url(plane_module_id),
            &self.api_key,
            &body,
        )
        .await
        .context("Plane.so add work item to module request failed")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Plane.so add work item to module failed: HTTP {status}: {body}");
        }
        Ok(())
    }

    /// Remove a Plane work item from a Plane module.
    pub async fn delete_work_item_from_module(
        &self,
        plane_module_id: &str,
        plane_work_item_id: &str,
    ) -> Result<()> {
        self.acquire_token().await?;
        let resp = self
            .execute_request_without_body(
                Method::DELETE,
                &self.module_work_item_url(plane_module_id, plane_work_item_id),
            )
            .await
            .context("Plane.so delete work item from module request failed")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Plane.so delete work item from module failed: HTTP {status}: {body}");
        }
        Ok(())
    }
}
