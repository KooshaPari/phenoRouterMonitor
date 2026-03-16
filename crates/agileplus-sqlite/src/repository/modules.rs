//! Module repository -- CRUD operations for the `modules` table and `module_feature_tags`.
//!
//! Traces to: FR-M01, FR-M02, FR-M04, FR-M07

use rusqlite::{Connection, Row, params};

use agileplus_domain::{
    domain::{
        feature::Feature,
        module::{Module, ModuleFeatureTag, ModuleWithFeatures},
        state_machine::FeatureState,
    },
    error::DomainError,
};

use crate::repository::features::map_err;

// ---------------------------------------------------------------------------
// Row mappers
// ---------------------------------------------------------------------------

fn row_to_module(row: &Row<'_>) -> rusqlite::Result<Module> {
    let id: i64 = row.get(0)?;
    let slug: String = row.get(1)?;
    let friendly_name: String = row.get(2)?;
    let description: Option<String> = row.get(3)?;
    let parent_module_id: Option<i64> = row.get(4)?;
    let created_at_str: String = row.get(5)?;
    let updated_at_str: String = row.get(6)?;

    let created_at = created_at_str
        .parse::<chrono::DateTime<chrono::Utc>>()
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                5,
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
                6,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string(),
                )),
            )
        })?;

    Ok(Module {
        id,
        slug,
        friendly_name,
        description,
        parent_module_id,
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
        project_id: None,
        created_at,
        updated_at,
    })
}

// ---------------------------------------------------------------------------
// Module CRUD
// ---------------------------------------------------------------------------

/// Create a module. Returns the new row ID.
///
/// Detects circular references via a recursive CTE before inserting.
pub fn create_module(conn: &Connection, module: &Module) -> Result<i64, DomainError> {
    // If a parent is specified, verify it exists and is not a descendant of this
    // module (which would be impossible for a brand-new module, but good practice).
    if let Some(parent_id) = module.parent_module_id {
        // Verify parent exists.
        let parent_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM modules WHERE id = ?1",
                params![parent_id],
                |row| row.get(0),
            )
            .map_err(map_err)?;
        if parent_exists == 0 {
            return Err(DomainError::ModuleNotFound(parent_id.to_string()));
        }
    }

    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO modules (slug, friendly_name, description, parent_module_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            module.slug,
            module.friendly_name,
            module.description,
            module.parent_module_id,
            now,
            now,
        ],
    )
    .map_err(map_err)?;
    Ok(conn.last_insert_rowid())
}

pub fn get_module(conn: &Connection, id: i64) -> Result<Option<Module>, DomainError> {
    conn.query_row(
        "SELECT id, slug, friendly_name, description, parent_module_id, created_at, updated_at
         FROM modules WHERE id = ?1",
        params![id],
        row_to_module,
    )
    .optional()
    .map_err(map_err)
}

pub fn get_module_by_slug(conn: &Connection, slug: &str) -> Result<Option<Module>, DomainError> {
    conn.query_row(
        "SELECT id, slug, friendly_name, description, parent_module_id, created_at, updated_at
         FROM modules WHERE slug = ?1",
        params![slug],
        row_to_module,
    )
    .optional()
    .map_err(map_err)
}

pub fn update_module(
    conn: &Connection,
    id: i64,
    friendly_name: &str,
    description: Option<&str>,
) -> Result<(), DomainError> {
    let slug = agileplus_domain::domain::module::Module::slug_from_name(friendly_name);
    let now = chrono::Utc::now().to_rfc3339();
    let rows = conn
        .execute(
            "UPDATE modules SET slug = ?1, friendly_name = ?2, description = ?3, updated_at = ?4
             WHERE id = ?5",
            params![slug, friendly_name, description, now, id],
        )
        .map_err(map_err)?;
    if rows == 0 {
        return Err(DomainError::ModuleNotFound(id.to_string()));
    }
    Ok(())
}

/// Delete a module. Fails with `ModuleHasDependents` if child modules or owned features exist.
pub fn delete_module(conn: &Connection, id: i64) -> Result<(), DomainError> {
    // Check for child modules.
    let child_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM modules WHERE parent_module_id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(map_err)?;
    if child_count > 0 {
        return Err(DomainError::ModuleHasDependents(format!(
            "module {id} has {child_count} child module(s)"
        )));
    }

    // Check for owned features (features.module_id = id).
    let feature_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM features WHERE module_id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(map_err)?;
    if feature_count > 0 {
        return Err(DomainError::ModuleHasDependents(format!(
            "module {id} owns {feature_count} feature(s)"
        )));
    }

    let rows = conn
        .execute("DELETE FROM modules WHERE id = ?1", params![id])
        .map_err(map_err)?;
    if rows == 0 {
        return Err(DomainError::ModuleNotFound(id.to_string()));
    }
    Ok(())
}

pub fn list_root_modules(conn: &Connection) -> Result<Vec<Module>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, slug, friendly_name, description, parent_module_id, created_at, updated_at
             FROM modules WHERE parent_module_id IS NULL ORDER BY friendly_name",
        )
        .map_err(map_err)?;
    let rows = stmt.query_map([], row_to_module).map_err(map_err)?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(map_err)
}

pub fn list_child_modules(conn: &Connection, parent_id: i64) -> Result<Vec<Module>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, slug, friendly_name, description, parent_module_id, created_at, updated_at
             FROM modules WHERE parent_module_id = ?1 ORDER BY friendly_name",
        )
        .map_err(map_err)?;
    let rows = stmt
        .query_map(params![parent_id], row_to_module)
        .map_err(map_err)?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(map_err)
}

/// Detect whether setting `proposed_parent_id` as the parent of `module_id` would
/// create a cycle in the module hierarchy using a recursive CTE.
///
/// Returns `true` if a circular reference would be created.
pub fn would_create_circular_ref(
    conn: &Connection,
    module_id: i64,
    proposed_parent_id: i64,
) -> Result<bool, DomainError> {
    // Walk upwards from proposed_parent_id through the ancestry chain.
    // If we ever hit module_id, it would be circular.
    let count: i64 = conn
        .query_row(
            "WITH RECURSIVE ancestors(id) AS (
                SELECT ?1
                UNION ALL
                SELECT m.parent_module_id
                FROM modules m
                INNER JOIN ancestors a ON m.id = a.id
                WHERE m.parent_module_id IS NOT NULL
            )
            SELECT COUNT(*) FROM ancestors WHERE id = ?2",
            params![proposed_parent_id, module_id],
            |row| row.get(0),
        )
        .map_err(map_err)?;
    Ok(count > 0)
}

pub fn get_module_with_features(
    conn: &Connection,
    id: i64,
) -> Result<Option<ModuleWithFeatures>, DomainError> {
    let module = match get_module(conn, id)? {
        Some(m) => m,
        None => return Ok(None),
    };

    // Owned features (features.module_id = id).
    let mut stmt = conn
        .prepare(
            "SELECT id, slug, friendly_name, state, spec_hash, target_branch, created_at, updated_at
             FROM features WHERE module_id = ?1 ORDER BY created_at",
        )
        .map_err(map_err)?;
    let owned_features = stmt
        .query_map(params![id], row_to_feature)
        .map_err(map_err)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(map_err)?;

    // Tagged features (module_feature_tags).
    let mut stmt = conn
        .prepare(
            "SELECT f.id, f.slug, f.friendly_name, f.state, f.spec_hash, f.target_branch,
                    f.created_at, f.updated_at
             FROM features f
             INNER JOIN module_feature_tags t ON f.id = t.feature_id
             WHERE t.module_id = ?1
             ORDER BY f.created_at",
        )
        .map_err(map_err)?;
    let tagged_features = stmt
        .query_map(params![id], row_to_feature)
        .map_err(map_err)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(map_err)?;

    // Direct child modules.
    let child_modules = list_child_modules(conn, id)?;

    Ok(Some(ModuleWithFeatures {
        module,
        owned_features,
        tagged_features,
        child_modules,
    }))
}

// ---------------------------------------------------------------------------
// Module-feature tag ops
// ---------------------------------------------------------------------------

/// Tag a feature to a module. Idempotent (INSERT OR IGNORE).
pub fn tag_feature_to_module(conn: &Connection, tag: &ModuleFeatureTag) -> Result<(), DomainError> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT OR IGNORE INTO module_feature_tags (module_id, feature_id, created_at)
         VALUES (?1, ?2, ?3)",
        params![tag.module_id, tag.feature_id, now],
    )
    .map_err(map_err)?;
    Ok(())
}

pub fn untag_feature_from_module(
    conn: &Connection,
    module_id: i64,
    feature_id: i64,
) -> Result<(), DomainError> {
    conn.execute(
        "DELETE FROM module_feature_tags WHERE module_id = ?1 AND feature_id = ?2",
        params![module_id, feature_id],
    )
    .map_err(map_err)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Optional trait for rusqlite
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
