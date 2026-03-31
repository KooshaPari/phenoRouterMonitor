use anyhow::Result;

use agileplus_domain::ports::{StoragePort, VcsPort};

use crate::manifest::{ImportBundle, ImportFeature};
use crate::report::ImportReport;

mod cycles;
mod features;
mod helpers;
mod modules;
mod projects;
mod work_packages;

pub async fn import_bundle<S, V>(bundle: ImportBundle, storage: &S, vcs: &V) -> Result<ImportReport>
where
    S: StoragePort,
    V: VcsPort,
{
    let mut report = ImportReport::default();

    // Import projects first, building a slug -> id map.
    let project_ids = projects::import_projects(&bundle.projects, storage, &mut report).await?;

    // Flatten features embedded within ImportProject entries, stamping project_id.
    let mut all_features: Vec<ImportFeature> = bundle.features.clone();
    for project in &bundle.projects {
        let project_slug = project
            .slug
            .clone()
            .unwrap_or_else(|| agileplus_domain::domain::feature::Feature::slug_from_name(&project.name));
        if let Some(&pid) = project_ids.get(&project_slug) {
            for mut feat in project.features.clone() {
                feat.project_id = Some(pid);
                all_features.push(feat);
            }
        }
    }

    let module_ids = modules::import_modules(&bundle.modules, storage, &mut report).await?;
    let feature_ids =
        features::import_features(&all_features, storage, vcs, &module_ids, &mut report).await?;
    cycles::import_cycles(
        &bundle.cycles,
        storage,
        &module_ids,
        &feature_ids,
        &mut report,
    )
    .await?;
    Ok(report)
}
