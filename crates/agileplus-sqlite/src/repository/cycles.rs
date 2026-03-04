//! Cycle repository -- CRUD operations for the `cycles` table and `cycle_features`.
//!
//! Traces to: FR-C01, FR-C02, FR-C03, FR-C04, FR-C05, FR-C07

use rusqlite::{Connection, Row, params};

use agileplus_domain::{
    domain::{
        cycle::{Cycle, CycleFeature, CycleState, CycleWithFeatures, WpProgressSummary},
        feature::Feature,
        state_machine::FeatureState,
        work_package::WpState,
    },
    error::DomainError,
};

use crate::repository::features::map_err;

// ---------------------------------------------------------------------------
// Row mappers
// ---------------------------------------------------------------------------

fn row_to_cycle(row: &Row<'_>) -> rusqlite::Result<Cycle> {
    let id: i64 = row.get(0)?;
    let name: String = row.get(1)?;
    let description: Option<String> = row.get(2)?;
    let state_str: String = row.get(3)?;
    let start_date_str: String = row.get(4)?;
    let end_date_str: String = row.get(5)?;
    let module_scope_id: Option<i64> = row.get(6)?;
    let created_at_str: String = row.get(7)?;
    let updated_at_str: String = row.get(8)?;

    let state = state_str.parse::<CycleState>().map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            3,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            )),
        )
    })?;

    let start_date =
        chrono::NaiveDate::parse_from_str(&start_date_str, "%Y-%m-%d").map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                4,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string(),
                )),
            )
        })?;

    let end_date = chrono::NaiveDate::parse_from_str(&end_date_str, "%Y-%m-%d").map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            5,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            )),
        )
    })?;

    let created_at = created_at_str
        .parse::<chrono::DateTime<chrono::Utc>>()
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                7,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string(),
                )),
            )
        })?;

    let updated_at = updated_at_str
        .parse::<chrono::DateTime<chrono::Utc>>()
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                8,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string(),
                )),
            )
        })?;

    Ok(Cycle {
        id,
        name,
        description,
        state,
        start_date,
        end_date,
        module_scope_id,
        created_at,
        updated_at,
    })
}

fn row_to_feature(row: &Row<'_>) -> rusqlite::Result<Feature> {
    let id: i64 = row.get(0)?;
    let slug: String = row.get(1)?;
    let friendly_name: String = row.get(2)?;
    let state_str: String = row.get(3)?;
    let spec_hash_bytes: Vec<u8> = row.get(4)?;
    let target_branch: String = row.get(5)?;
    let created_at_str: String = row.get(6)?;
    let updated_at_str: String = row.get(7)?;

    let state = state_str.parse::<FeatureState>().map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            3,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
        )
    })?;

    let mut spec_hash = [0u8; 32];
    if spec_hash_bytes.len() == 32 {
        spec_hash.copy_from_slice(&spec_hash_bytes);
    }

    let created_at = created_at_str
        .parse::<chrono::DateTime<chrono::Utc>>()
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                6,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string(),
                )),
            )
        })?;

    let updated_at = updated_at_str
        .parse::<chrono::DateTime<chrono::Utc>>()
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                7,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string(),
                )),
            )
        })?;

    Ok(Feature {
        id,
        slug,
        friendly_name,
        state,
        spec_hash,
        target_branch,
        plane_issue_id: None,
        plane_state_id: None,
        labels: Vec::new(),
        module_id: None,
        created_at,
        updated_at,
    })
}

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

pub fn list_cycles_by_module(
    conn: &Connection,
    module_id: i64,
) -> Result<Vec<Cycle>, DomainError> {
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

/// Aggregate WP state counts across all features assigned to a cycle.
fn compute_wp_progress(conn: &Connection, cycle_id: i64) -> Result<WpProgressSummary, DomainError> {
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
        // Parse the WpState text stored in the DB.
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

// ---------------------------------------------------------------------------
// Cycle-feature join ops
// ---------------------------------------------------------------------------

/// Add a feature to a cycle. Enforces module_scope_id validation if set.
/// Idempotent (INSERT OR IGNORE).
pub fn add_feature_to_cycle(
    conn: &Connection,
    entry: &CycleFeature,
) -> Result<(), DomainError> {
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

// ---------------------------------------------------------------------------
// Helper: wp_state as string (mirrors work_packages.rs)
// ---------------------------------------------------------------------------

#[allow(dead_code)]
fn wp_state_str(s: WpState) -> &'static str {
    match s {
        WpState::Planned => "planned",
        WpState::Doing => "doing",
        WpState::Review => "review",
        WpState::Done => "done",
        WpState::Blocked => "blocked",
    }
}

// ---------------------------------------------------------------------------
// Optional trait
// ---------------------------------------------------------------------------

trait OptionalExt<T> {
    fn optional(self) -> rusqlite::Result<Option<T>>;
}

impl<T> OptionalExt<T> for rusqlite::Result<T> {
    fn optional(self) -> rusqlite::Result<Option<T>> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
