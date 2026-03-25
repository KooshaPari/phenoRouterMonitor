//! Cycle repository -- CRUD operations for the `cycles` table and `cycle_features`.
//!
//! Traces to: FR-C01, FR-C02, FR-C03, FR-C04, FR-C05, FR-C07

use rusqlite::{Connection, OptionalExtension, params};

use agileplus_domain::{
    domain::cycle::{Cycle, CycleFeature, CycleState, CycleWithFeatures},
    error::DomainError,
};

use crate::repository::features::map_err;

mod mappers;
mod progress;

use mappers::{row_to_cycle, row_to_feature};
use progress::compute_wp_progress;

// ---------------------------------------------------------------------------
// Cycle CRUD
// ---------------------------------------------------------------------------

pub fn create_cycle(conn: &Connection, cycle: &Cycle) -> Result<i64, DomainError> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO cycles (name, description, state, start_date, end_date, module_scope_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            cycle.name,
            cycle.description,
            cycle.state.to_string(),
            cycle.start_date.format("%Y-%m-%d").to_string(),
            cycle.end_date.format("%Y-%m-%d").to_string(),
            cycle.module_scope_id,
            now,
            now,
        ],
    )
    .map_err(map_err)?;
    Ok(conn.last_insert_rowid())
}

pub fn get_cycle(conn: &Connection, id: i64) -> Result<Option<Cycle>, DomainError> {
    conn.query_row(
        "SELECT id, name, description, state, start_date, end_date, module_scope_id,
                created_at, updated_at
         FROM cycles WHERE id = ?1",
        params![id],
        row_to_cycle,
    )
    .optional()
    .map_err(map_err)
}

pub fn update_cycle_state(
    conn: &Connection,
    id: i64,
    state: CycleState,
) -> Result<(), DomainError> {
    let now = chrono::Utc::now().to_rfc3339();
    let rows = conn
        .execute(
            "UPDATE cycles SET state = ?1, updated_at = ?2 WHERE id = ?3",
            params![state.to_string(), now, id],
        )
        .map_err(map_err)?;
    if rows == 0 {
        return Err(DomainError::CycleNotFound(id.to_string()));
    }
    Ok(())
}

pub fn list_cycles_by_state(
    conn: &Connection,
    state: CycleState,
) -> Result<Vec<Cycle>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, state, start_date, end_date, module_scope_id,
                    created_at, updated_at
             FROM cycles WHERE state = ?1 ORDER BY start_date",
        )
        .map_err(map_err)?;
    let rows = stmt
        .query_map(params![state.to_string()], row_to_cycle)
        .map_err(map_err)?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(map_err)
}

pub fn list_cycles_by_module(conn: &Connection, module_id: i64) -> Result<Vec<Cycle>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, state, start_date, end_date, module_scope_id,
                    created_at, updated_at
             FROM cycles WHERE module_scope_id = ?1 ORDER BY start_date",
        )
        .map_err(map_err)?;
    let rows = stmt
        .query_map(params![module_id], row_to_cycle)
        .map_err(map_err)?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(map_err)
}

/// List every cycle regardless of state.
pub fn list_all_cycles(conn: &Connection) -> Result<Vec<Cycle>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, state, start_date, end_date, module_scope_id,
                    created_at, updated_at
             FROM cycles ORDER BY start_date",
        )
        .map_err(map_err)?;
    let rows = stmt.query_map([], row_to_cycle).map_err(map_err)?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(map_err)
}

/// Load a cycle with its assigned features and compute a `WpProgressSummary`.
pub fn get_cycle_with_features(
    conn: &Connection,
    id: i64,
) -> Result<Option<CycleWithFeatures>, DomainError> {
    let cycle = match get_cycle(conn, id)? {
        Some(c) => c,
        None => return Ok(None),
    };

    // Load assigned features.
    let mut stmt = conn
        .prepare(
            "SELECT f.id, f.slug, f.friendly_name, f.state, f.spec_hash, f.target_branch,
                    f.created_at, f.updated_at
             FROM features f
             INNER JOIN cycle_features cf ON f.id = cf.feature_id
             WHERE cf.cycle_id = ?1
             ORDER BY f.created_at",
        )
        .map_err(map_err)?;
    let features = stmt
        .query_map(params![id], row_to_feature)
        .map_err(map_err)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(map_err)?;

    // Compute WP progress summary for all features in this cycle.
    let wp_progress = compute_wp_progress(conn, id)?;

    Ok(Some(CycleWithFeatures {
        cycle,
        features,
        wp_progress,
    }))
}

// ---------------------------------------------------------------------------
// Cycle-feature join ops
// ---------------------------------------------------------------------------

/// Add a feature to a cycle. Enforces module_scope_id validation if set.
/// Idempotent (INSERT OR IGNORE).
pub fn add_feature_to_cycle(conn: &Connection, entry: &CycleFeature) -> Result<(), DomainError> {
    // Check if the cycle has a module scope restriction.
    let module_scope_id: Option<i64> = conn
        .query_row(
            "SELECT module_scope_id FROM cycles WHERE id = ?1",
            params![entry.cycle_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(map_err)?
        .ok_or_else(|| DomainError::CycleNotFound(entry.cycle_id.to_string()))?;

    if let Some(scope_module_id) = module_scope_id {
        // Feature must be owned by or tagged to the scope module.
        let in_scope: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM (
                    SELECT id FROM features WHERE id = ?1 AND module_id = ?2
                    UNION
                    SELECT feature_id FROM module_feature_tags WHERE feature_id = ?1 AND module_id = ?2
                )",
                params![entry.feature_id, scope_module_id],
                |row| row.get(0),
            )
            .map_err(map_err)?;

        if in_scope == 0 {
            // Load slugs for a good error message.
            let feature_slug: String = conn
                .query_row(
                    "SELECT slug FROM features WHERE id = ?1",
                    params![entry.feature_id],
                    |row| row.get(0),
                )
                .map_err(map_err)?;
            let module_slug: String = conn
                .query_row(
                    "SELECT slug FROM modules WHERE id = ?1",
                    params![scope_module_id],
                    |row| row.get(0),
                )
                .map_err(map_err)?;
            return Err(DomainError::FeatureNotInModuleScope {
                feature_slug,
                module_slug,
            });
        }
    }

    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT OR IGNORE INTO cycle_features (cycle_id, feature_id, added_at)
         VALUES (?1, ?2, ?3)",
        params![entry.cycle_id, entry.feature_id, now],
    )
    .map_err(map_err)?;
    Ok(())
}

pub fn remove_feature_from_cycle(
    conn: &Connection,
    cycle_id: i64,
    feature_id: i64,
) -> Result<(), DomainError> {
    conn.execute(
        "DELETE FROM cycle_features WHERE cycle_id = ?1 AND feature_id = ?2",
        params![cycle_id, feature_id],
    )
    .map_err(map_err)?;
    Ok(())
}
