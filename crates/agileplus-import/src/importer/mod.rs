use anyhow::Result;

use agileplus_domain::ports::{StoragePort, VcsPort};

use crate::manifest::ImportBundle;
use crate::report::ImportReport;

mod cycles;
mod features;
mod helpers;
mod modules;
mod work_packages;

pub async fn import_bundle<S, V>(bundle: ImportBundle, storage: &S, vcs: &V) -> Result<ImportReport>
where
    S: StoragePort,
    V: VcsPort,
{
    let mut report = ImportReport::default();
    let module_ids = modules::import_modules(&bundle.modules, storage, &mut report).await?;
    let feature_ids =
        features::import_features(&bundle.features, storage, vcs, &module_ids, &mut report).await?;
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
