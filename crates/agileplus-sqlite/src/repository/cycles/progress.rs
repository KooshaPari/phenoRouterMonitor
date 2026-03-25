use rusqlite::{Connection, params};

use agileplus_domain::{domain::cycle::WpProgressSummary, error::DomainError};

use crate::repository::features::map_err;

/// Aggregate WP state counts across all features assigned to a cycle.
pub(super) fn compute_wp_progress(
    conn: &Connection,
    cycle_id: i64,
) -> Result<WpProgressSummary, DomainError> {
    // Count WPs per state for features in this cycle.
    let mut stmt = conn
        .prepare(
            "SELECT wp.state, COUNT(*) as cnt
             FROM work_packages wp
             INNER JOIN cycle_features cf ON wp.feature_id = cf.feature_id
             WHERE cf.cycle_id = ?1
             GROUP BY wp.state",
        )
        .map_err(map_err)?;

    let mut summary = WpProgressSummary::default();

    let rows = stmt
        .query_map(params![cycle_id], |row| {
            let state_str: String = row.get(0)?;
            let count: i64 = row.get(1)?;
            Ok((state_str, count))
        })
        .map_err(map_err)?;

    for row in rows {
        let (state_str, count) = row.map_err(map_err)?;
        let count = count as u32;
        summary.total += count;
        match state_str.as_str() {
            "planned" => summary.planned += count,
            "doing" => summary.in_progress += count,
            "review" => summary.in_progress += count,
            "done" => summary.done += count,
            "blocked" => summary.blocked += count,
            _ => {}
        }
    }

    Ok(summary)
}
