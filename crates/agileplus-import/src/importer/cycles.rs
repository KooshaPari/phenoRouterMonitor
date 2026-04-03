use std::collections::HashMap;

use anyhow::{Context, Result, anyhow};

use agileplus_domain::domain::cycle::{Cycle, CycleFeature, CycleState};
use agileplus_domain::ports::StoragePort;

use crate::manifest::ImportCycle;
use crate::report::ImportReport;

pub(super) async fn import_cycles<S: StoragePort>(
    cycles: &[ImportCycle],
    storage: &S,
    module_ids: &HashMap<String, i64>,
    feature_ids: &HashMap<String, i64>,
    report: &mut ImportReport,
) -> Result<()> {
    let existing_cycles = storage
        .list_all_cycles()
        .await
        .context("listing existing cycles")?;

    for spec in cycles {
        let module_scope_id = if let Some(slug) = spec.module_scope_slug.as_ref() {
            if let Some(id) = module_ids.get(slug).copied() {
                Some(id)
            } else {
                Some(
                    storage
                        .get_module_by_slug(slug)
                        .await
                        .context("resolving cycle module scope")?
                        .ok_or_else(|| {
                            anyhow!("cycle '{}' references unknown module '{}'", spec.name, slug)
                        })?
                        .id,
                )
            }
        } else {
            None
        };

        let id = if let Some(existing) =
            existing_cycles.iter().find(|cycle| cycle.name == spec.name)
        {
            existing.id
        } else {
            let mut cycle = Cycle::new(&spec.name, spec.start_date, spec.end_date, module_scope_id)
                .context("creating cycle")?;
            cycle.description = spec.description.clone();

            let id = storage
                .create_cycle(&cycle)
                .await
                .context("persisting cycle")?;
            report.cycles_created += 1;
            id
        };

        if spec.state != CycleState::Draft {
            storage
                .update_cycle_state(id, spec.state)
                .await
                .context("updating cycle state")?;
            report.cycles_updated += 1;
        }

        for feature_slug in &spec.feature_slugs {
            let feature_id = if let Some(id) = feature_ids.get(feature_slug).copied() {
                id
            } else {
                storage
                    .get_feature_by_slug(feature_slug)
                    .await
                    .context("resolving cycle feature")?
                    .ok_or_else(|| {
                        anyhow!(
                            "cycle '{}' references unknown feature '{}'",
                            spec.name,
                            feature_slug
                        )
                    })?
                    .id
            };
            storage
                .add_feature_to_cycle(&CycleFeature::new(id, feature_id))
                .await
                .context("adding feature to cycle")?;
            report.cycle_links_created += 1;
        }
    }

    Ok(())
}
