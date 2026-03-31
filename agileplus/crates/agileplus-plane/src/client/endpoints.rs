//! Endpoint path builders for Plane.so API URLs.

/// Internal helpers for formatting Plane.so endpoint URLs.
pub(super) struct ClientEndpoints;

impl ClientEndpoints {
    pub(super) fn work_items_url(base_url: &str, workspace_slug: &str, project_id: &str) -> String {
        format!(
            "{}/api/v1/workspaces/{}/projects/{}/work-items/",
            base_url, workspace_slug, project_id
        )
    }

    pub(super) fn modules_url(base_url: &str, workspace_slug: &str, project_id: &str) -> String {
        format!(
            "{}/api/v1/workspaces/{}/projects/{}/modules/",
            base_url, workspace_slug, project_id
        )
    }

    pub(super) fn module_url(
        base_url: &str,
        workspace_slug: &str,
        project_id: &str,
        module_id: &str,
    ) -> String {
        format!(
            "{}/api/v1/workspaces/{}/projects/{}/modules/{}/",
            base_url, workspace_slug, project_id, module_id
        )
    }

    pub(super) fn module_work_items_url(
        base_url: &str,
        workspace_slug: &str,
        project_id: &str,
        module_id: &str,
    ) -> String {
        format!(
            "{}/api/v1/workspaces/{}/projects/{}/modules/{}/module-issues/",
            base_url, workspace_slug, project_id, module_id
        )
    }

    pub(super) fn module_work_item_url(
        base_url: &str,
        workspace_slug: &str,
        project_id: &str,
        module_id: &str,
        work_item_id: &str,
    ) -> String {
        format!(
            "{}/api/v1/workspaces/{}/projects/{}/modules/{}/module-issues/{}/",
            base_url, workspace_slug, project_id, module_id, work_item_id
        )
    }

    pub(super) fn cycles_url(base_url: &str, workspace_slug: &str, project_id: &str) -> String {
        format!(
            "{}/api/v1/workspaces/{}/projects/{}/cycles/",
            base_url, workspace_slug, project_id
        )
    }

    pub(super) fn cycle_url(
        base_url: &str,
        workspace_slug: &str,
        project_id: &str,
        cycle_id: &str,
    ) -> String {
        format!(
            "{}/api/v1/workspaces/{}/projects/{}/cycles/{}/",
            base_url, workspace_slug, project_id, cycle_id
        )
    }

    pub(super) fn cycle_work_items_url(
        base_url: &str,
        workspace_slug: &str,
        project_id: &str,
        cycle_id: &str,
    ) -> String {
        format!(
            "{}/api/v1/workspaces/{}/projects/{}/cycles/{}/cycle-issues/",
            base_url, workspace_slug, project_id, cycle_id
        )
    }

    pub(super) fn cycle_work_item_url(
        base_url: &str,
        workspace_slug: &str,
        project_id: &str,
        cycle_id: &str,
        work_item_id: &str,
    ) -> String {
        format!(
            "{}/api/v1/workspaces/{}/projects/{}/cycles/{}/cycle-issues/{}/",
            base_url, workspace_slug, project_id, cycle_id, work_item_id
        )
    }

    pub(super) fn work_item_url(
        base_url: &str,
        workspace_slug: &str,
        project_id: &str,
        work_item_id: &str,
    ) -> String {
        format!(
            "{}/api/v1/workspaces/{}/projects/{}/work-items/{}/",
            base_url, workspace_slug, project_id, work_item_id
        )
    }

    pub(super) fn labels_url(base_url: &str, workspace_slug: &str, project_id: &str) -> String {
        format!(
            "{}/api/v1/workspaces/{}/projects/{}/labels/",
            base_url, workspace_slug, project_id
        )
    }
}
