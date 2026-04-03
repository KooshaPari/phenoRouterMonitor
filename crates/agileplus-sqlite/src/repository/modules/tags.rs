use rusqlite::{Connection, params};

use agileplus_domain::domain::module::ModuleFeatureTag;
use agileplus_domain::error::DomainError;

use crate::repository::features::map_err;

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
