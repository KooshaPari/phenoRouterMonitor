use anyhow::{Context, Result};
use reqwest::Method;

use super::endpoints::ClientEndpoints;
use super::transport;
use super::{
    PlaneClient, PlaneCreateCycleRequest, PlaneCreateModuleRequest, PlaneCycleResponse, PlaneIssue,
    PlaneModuleResponse, PlaneWorkItem, PlaneWorkItemResponse,
};

mod cycles;
mod modules;
mod work_items;

impl PlaneClient {
    fn work_items_url(&self) -> String {
        ClientEndpoints::work_items_url(&self.base_url, &self.workspace_slug, &self.project_id)
    }

    fn modules_url(&self) -> String {
        ClientEndpoints::modules_url(&self.base_url, &self.workspace_slug, &self.project_id)
    }

    fn module_url(&self, module_id: &str) -> String {
        ClientEndpoints::module_url(
            &self.base_url,
            &self.workspace_slug,
            &self.project_id,
            module_id,
        )
    }

    fn module_work_items_url(&self, module_id: &str) -> String {
        ClientEndpoints::module_work_items_url(
            &self.base_url,
            &self.workspace_slug,
            &self.project_id,
            module_id,
        )
    }

    fn module_work_item_url(&self, module_id: &str, work_item_id: &str) -> String {
        ClientEndpoints::module_work_item_url(
            &self.base_url,
            &self.workspace_slug,
            &self.project_id,
            module_id,
            work_item_id,
        )
    }

    fn cycles_url(&self) -> String {
        ClientEndpoints::cycles_url(&self.base_url, &self.workspace_slug, &self.project_id)
    }

    fn cycle_url(&self, cycle_id: &str) -> String {
        ClientEndpoints::cycle_url(
            &self.base_url,
            &self.workspace_slug,
            &self.project_id,
            cycle_id,
        )
    }

    fn cycle_work_items_url(&self, cycle_id: &str) -> String {
        ClientEndpoints::cycle_work_items_url(
            &self.base_url,
            &self.workspace_slug,
            &self.project_id,
            cycle_id,
        )
    }

    fn cycle_work_item_url(&self, cycle_id: &str, work_item_id: &str) -> String {
        ClientEndpoints::cycle_work_item_url(
            &self.base_url,
            &self.workspace_slug,
            &self.project_id,
            cycle_id,
            work_item_id,
        )
    }

    fn work_item_url(&self, work_item_id: &str) -> String {
        ClientEndpoints::work_item_url(
            &self.base_url,
            &self.workspace_slug,
            &self.project_id,
            work_item_id,
        )
    }

    pub fn labels_url(&self) -> String {
        ClientEndpoints::labels_url(&self.base_url, &self.workspace_slug, &self.project_id)
    }

    /// Make a raw GET request and return response body as String.
    pub async fn get_raw(&self, url: &str) -> Result<String> {
        self.acquire_token().await?;
        let resp = self
            .execute_request_without_body(Method::GET, url)
            .await
            .context("Plane.so GET request failed")?;
        transport::read_text_response(resp, "reading Plane.so response body").await
    }

    /// Make a raw POST request with JSON body and return response body as String.
    pub async fn post_raw(&self, url: &str, json_body: &str) -> Result<String> {
        self.acquire_token().await?;
        let resp =
            transport::request_raw_body(&self.client, Method::POST, url, &self.api_key, json_body)
                .await
                .context("Plane.so POST request failed")?;
        transport::read_text_response(resp, "reading Plane.so response body").await
    }
}
