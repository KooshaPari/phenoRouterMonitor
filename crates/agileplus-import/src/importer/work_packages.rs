use std::collections::HashMap;

use anyhow::{Context, Result, anyhow};
use chrono::Utc;

use agileplus_domain::domain::work_package::{DependencyType, WorkPackage, WpDependency, WpState};
use agileplus_domain::ports::StoragePort;

use crate::manifest::ImportFeature;
use crate::report::ImportReport;

pub(super) async fn import_work_packages<S: StoragePort>(
    spec: &ImportFeature,
    feature_id: i64,
    storage: &S,
    report: &mut ImportReport,
) -> Result<()> {
    let existing = storage
        .list_wps_by_feature(feature_id)
        .await
        .context("listing existing work packages")?;
    let mut existing_by_sequence: HashMap<i32, i64> = HashMap::new();
    let mut existing_by_title: HashMap<String, i64> = HashMap::new();
    for wp in existing {
        existing_by_sequence.insert(wp.sequence, wp.id);
        existing_by_title.insert(wp.title, wp.id);
    }

    let mut sequence_to_id = HashMap::new();

    for (index, wp_spec) in spec.work_packages.iter().enumerate() {
        let sequence = wp_spec.sequence.unwrap_or((index + 1) as i32);
        let id = if let Some(id) = existing_by_sequence.get(&sequence).copied() {
            if wp_spec.state != WpState::Planned {
                storage
                    .update_wp_state(id, wp_spec.state)
                    .await
                    .context("updating existing work package state")?;
                report.work_packages_updated += 1;
            }
            id
        } else if let Some(id) = existing_by_title.get(&wp_spec.title).copied() {
            if wp_spec.state != WpState::Planned {
                storage
                    .update_wp_state(id, wp_spec.state)
                    .await
                    .context("updating existing work package state")?;
                report.work_packages_updated += 1;
            }
            id
        } else {
            let wp = WorkPackage {
                id: 0,
                feature_id,
                title: wp_spec.title.clone(),
                state: WpState::Planned,
                sequence,
                file_scope: wp_spec.file_scope.clone(),
                acceptance_criteria: wp_spec.acceptance_criteria.clone().unwrap_or_default(),
                agent_id: wp_spec.agent_id.clone(),
                pr_url: wp_spec.pr_url.clone(),
                pr_state: wp_spec.pr_state,
                worktree_path: wp_spec.worktree_path.clone(),
                plane_sub_issue_id: wp_spec.plane_sub_issue_id.clone(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                base_commit: None,
                head_commit: None,
            };
            let id = storage
                .create_work_package(&wp)
                .await
                .context("creating work package")?;
            report.work_packages_created += 1;
            id
        };

        sequence_to_id.insert(sequence, id);
    }

    for wp_spec in &spec.work_packages {
        let sequence = wp_spec.sequence.unwrap_or(0);
        let Some(&wp_id) = sequence_to_id.get(&sequence) else {
            continue;
        };
        for depends_on_sequence in &wp_spec.depends_on_sequences {
            let depends_on = sequence_to_id
                .get(depends_on_sequence)
                .copied()
                .ok_or_else(|| {
                    anyhow!(
                        "work package '{}' depends on unknown sequence {}",
                        wp_spec.title,
                        depends_on_sequence
                    )
                })?;
            storage
                .add_wp_dependency(&WpDependency {
                    wp_id,
                    depends_on,
                    dep_type: DependencyType::Explicit,
                })
                .await
                .context("adding work package dependency")?;
        }
    }

    Ok(())
}
