use std::collections::HashMap;

use anyhow::{Context, Result, anyhow};

use agileplus_domain::domain::{
    feature::Feature, module::ModuleFeatureTag, state_machine::FeatureState,
};
use agileplus_domain::ports::{StoragePort, VcsPort};

use crate::importer::helpers::{build_import_audit_entry, feature_meta_json, sha256_bytes};
use crate::importer::work_packages::import_work_packages;
use crate::manifest::ImportFeature;
use crate::report::ImportReport;

pub(super) async fn import_features<S, V>(
    features: &[ImportFeature],
    storage: &S,
    vcs: &V,
    module_ids: &HashMap<String, i64>,
    report: &mut ImportReport,
) -> Result<HashMap<String, i64>>
where
    S: StoragePort,
    V: VcsPort,
{
    let mut feature_ids = HashMap::new();

    for spec in features {
        let slug = spec
            .slug
            .clone()
            .unwrap_or_else(|| Feature::slug_from_name(&spec.friendly_name));
        let spec_hash = sha256_bytes(&spec.spec_content);
        let target_branch = spec.target_branch.as_deref();
        let mut feature = Feature::new(&slug, &spec.friendly_name, spec_hash, target_branch);
        feature.labels = spec.labels.clone();
        feature.module_id = if let Some(module_slug) = spec.module_slug.as_ref() {
            if let Some(id) = module_ids.get(module_slug).copied() {
                Some(id)
            } else {
                Some(
                    storage
                        .get_module_by_slug(module_slug)
                        .await
                        .context("resolving feature module")?
                        .ok_or_else(|| {
                            anyhow!(
                                "feature '{}' references unknown module '{}'",
                                spec.friendly_name,
                                module_slug
                            )
                        })?
                        .id,
                )
            }
        } else {
            None
        };
        feature.project_id = spec.project_id;
        feature.plane_issue_id = spec.plane_issue_id.clone();
        feature.plane_state_id = spec.plane_state_id.clone();

        let existing = storage
            .get_feature_by_slug(&slug)
            .await
            .context("checking for existing feature")?;

        let feature_id = if let Some(existing) = existing {
            report.features_updated += 1;
            existing.id
        } else {
            let id = storage
                .create_feature(&feature)
                .await
                .context("creating feature")?;
            report.features_created += 1;
            id
        };
        feature_ids.insert(slug.clone(), feature_id);

        if let Some(module_id) = feature.module_id {
            storage
                .tag_feature_to_module(&ModuleFeatureTag::new(module_id, feature_id))
                .await
                .context("tagging feature to module")?;
            report.module_links_created += 1;
        }

        if spec.state != FeatureState::Created {
            storage
                .update_feature_state(feature_id, spec.state)
                .await
                .context("updating feature state")?;
        }

        let audit = build_import_audit_entry(feature_id, &spec.state);
        storage
            .append_audit_entry(&audit)
            .await
            .context("appending import audit")?;
        report.audits_written += 1;

        vcs.write_artifact(&slug, "spec.md", &spec.spec_content)
            .await
            .context("writing spec artifact")?;
        vcs.write_artifact(&slug, "meta.json", &feature_meta_json(&feature, spec.state))
            .await
            .context("writing meta artifact")?;
        report.artifacts_written += 2;

        import_work_packages(spec, feature_id, storage, report).await?;
    }

    Ok(feature_ids)
}
