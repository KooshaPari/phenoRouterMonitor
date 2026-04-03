use std::collections::HashMap;

use anyhow::{Context, Result, anyhow};

use agileplus_domain::domain::module::Module;
use agileplus_domain::ports::StoragePort;

use crate::manifest::ImportModule;
use crate::report::ImportReport;

pub(super) async fn import_modules<S: StoragePort>(
    modules: &[ImportModule],
    storage: &S,
    report: &mut ImportReport,
) -> Result<HashMap<String, i64>> {
    let mut pending: Vec<ImportModule> = modules.to_vec();
    let mut module_ids = HashMap::new();

    while !pending.is_empty() {
        let mut next_round = Vec::new();
        let mut progressed = false;

        for spec in pending {
            let slug = spec.slug();
            let parent_id = resolve_parent_module_id(&spec, storage, &module_ids).await?;

            if spec.parent_slug.is_some() && parent_id.is_none() {
                next_round.push(spec);
                continue;
            }

            if spec.parent_slug.as_deref() == Some(slug.as_str()) {
                return Err(anyhow!("module '{slug}' cannot be its own parent"));
            }

            let mut module = Module::new(&spec.friendly_name, parent_id);
            module.slug = slug.clone();
            module.description = spec.description.clone();

            let id = if let Some(existing) = storage
                .get_module_by_slug(&slug)
                .await
                .context("checking for existing module")?
            {
                storage
                    .update_module(
                        existing.id,
                        &module.friendly_name,
                        module.description.as_deref(),
                    )
                    .await
                    .context("updating existing module")?;
                report.modules_updated += 1;
                existing.id
            } else {
                let id = storage
                    .create_module(&module)
                    .await
                    .context("creating module")?;
                report.modules_created += 1;
                id
            };

            module_ids.insert(slug, id);
            progressed = true;
        }

        if !progressed {
            let unresolved: Vec<String> = next_round
                .iter()
                .map(|m| {
                    format!(
                        "{} (parent={})",
                        m.slug(),
                        m.parent_slug.clone().unwrap_or_else(|| "-".into())
                    )
                })
                .collect();
            return Err(anyhow!(
                "could not resolve module parents for: {}",
                unresolved.join(", ")
            ));
        }

        pending = next_round;
    }

    Ok(module_ids)
}

pub(super) async fn resolve_parent_module_id<S: StoragePort>(
    spec: &ImportModule,
    storage: &S,
    module_ids: &HashMap<String, i64>,
) -> Result<Option<i64>> {
    match spec.parent_slug.as_ref() {
        None => Ok(None),
        Some(parent_slug) => {
            if let Some(id) = module_ids.get(parent_slug).copied() {
                Ok(Some(id))
            } else {
                Ok(storage
                    .get_module_by_slug(parent_slug)
                    .await
                    .context("checking parent module")?
                    .map(|m| m.id))
            }
        }
    }
}
