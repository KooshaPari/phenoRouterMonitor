//! Project import logic.
//!
//! Traceability: FR-IMPORT-PROJECT

use std::collections::HashMap;

use anyhow::{Context, Result};

use agileplus_domain::domain::{feature::Feature, project::Project};
use agileplus_domain::ports::StoragePort;

use crate::manifest::ImportProject;
use crate::report::ImportReport;

/// Import all projects from the bundle. Returns a map of project slug -> project ID.
pub(super) async fn import_projects<S: StoragePort>(
    projects: &[ImportProject],
    storage: &S,
    report: &mut ImportReport,
) -> Result<HashMap<String, i64>> {
    let mut project_ids = HashMap::new();

    for project in projects {
        let project_slug = project
            .slug
            .clone()
            .unwrap_or_else(|| Feature::slug_from_name(&project.name));

        if let Some(existing) = storage
            .get_project_by_slug(&project_slug)
            .await
            .context("Failed to check existing project")?
        {
            report.projects_updated += 1;
            project_ids.insert(project_slug, existing.id);
        } else {
            let new_project = Project::new(
                &project_slug,
                &project.name,
                project.description.as_deref().unwrap_or_default(),
            );
            let id = storage
                .create_project(&new_project)
                .await
                .context("Failed to create project")?;
            report.projects_created += 1;
            project_ids.insert(project_slug, id);
        }
    }

    Ok(project_ids)
}
