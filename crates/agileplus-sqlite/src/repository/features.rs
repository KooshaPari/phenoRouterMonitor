//! Feature repository — CRUD operations for the `features` table.

use rusqlite::{Connection, Row, params};

use agileplus_domain::{
    domain::{feature::Feature, state_machine::FeatureState},
    error::DomainError,
};

pub(crate) fn map_err(e: rusqlite::Error) -> DomainError {
    DomainError::Storage(e.to_string())
}

fn state_str(s: FeatureState) -> &'static str {
    match s {
        FeatureState::Created => "created",
        FeatureState::Specified => "specified",
        FeatureState::Researched => "researched",
        FeatureState::Planned => "planned",
        FeatureState::Implementing => "implementing",
        FeatureState::Validated => "validated",
        FeatureState::Shipped => "shipped",
        FeatureState::Retrospected => "retrospected",
    }
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

    let state = state_str
        .parse::<FeatureState>()
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e))))?;

    let mut spec_hash = [0u8; 32];
    if spec_hash_bytes.len() == 32 {
        spec_hash.copy_from_slice(&spec_hash_bytes);
    }

    let created_at = created_at_str
        .parse::<chrono::DateTime<chrono::Utc>>()
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(6, rusqlite::types::Type::Text, Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))))?;

    let updated_at = updated_at_str
        .parse::<chrono::DateTime<chrono::Utc>>()
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(7, rusqlite::types::Type::Text, Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))))?;

    Ok(Feature {
        id,
        slug,
        friendly_name,
        state,
        spec_hash,
        target_branch,
        created_at,
        updated_at,
    })
}

pub fn create_feature(conn: &Connection, feature: &Feature) -> Result<i64, DomainError> {
    conn.execute(
        "INSERT INTO features (slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            feature.slug,
            feature.friendly_name,
            state_str(feature.state),
            feature.spec_hash.as_slice(),
            feature.target_branch,
            feature.created_at.to_rfc3339(),
            feature.updated_at.to_rfc3339(),
        ],
    )
    .map_err(map_err)?;
    Ok(conn.last_insert_rowid())
}

pub fn get_feature_by_slug(conn: &Connection, slug: &str) -> Result<Option<Feature>, DomainError> {
    conn.query_row(
        "SELECT id, slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at
         FROM features WHERE slug = ?1",
        params![slug],
        row_to_feature,
    )
    .optional()
    .map_err(map_err)
}

pub fn get_feature_by_id(conn: &Connection, id: i64) -> Result<Option<Feature>, DomainError> {
    conn.query_row(
        "SELECT id, slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at
         FROM features WHERE id = ?1",
        params![id],
        row_to_feature,
    )
    .optional()
    .map_err(map_err)
}

pub fn update_feature_state(
    conn: &Connection,
    id: i64,
    state: FeatureState,
) -> Result<(), DomainError> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE features SET state = ?1, updated_at = ?2 WHERE id = ?3",
        params![state_str(state), now, id],
    )
    .map_err(map_err)?;
    Ok(())
}

pub fn list_features_by_state(
    conn: &Connection,
    state: FeatureState,
) -> Result<Vec<Feature>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at
             FROM features WHERE state = ?1 ORDER BY created_at",
        )
        .map_err(map_err)?;

    let rows = stmt
        .query_map(params![state_str(state)], row_to_feature)
        .map_err(map_err)?;

    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(map_err)
}

pub fn list_all_features(conn: &Connection) -> Result<Vec<Feature>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at
             FROM features ORDER BY created_at DESC",
        )
        .map_err(map_err)?;

    let rows = stmt.query_map([], row_to_feature).map_err(map_err)?;

    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(map_err)
}

/// Extension trait to add `.optional()` on rusqlite query results.
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
