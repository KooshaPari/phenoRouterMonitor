//! Work package repository — CRUD operations for `work_packages` and `wp_dependencies`.

use rusqlite::{Connection, Row, params};

use agileplus_domain::{
    domain::work_package::{DependencyType, PrState, WpDependency, WpState, WorkPackage},
    error::DomainError,
};

fn map_err(e: rusqlite::Error) -> DomainError {
    DomainError::Storage(e.to_string())
}

fn wp_state_str(s: WpState) -> &'static str {
    match s {
        WpState::Planned => "planned",
        WpState::Doing => "doing",
        WpState::Review => "review",
        WpState::Done => "done",
        WpState::Blocked => "blocked",
    }
}

fn wp_state_from_str(s: &str) -> Result<WpState, DomainError> {
    match s {
        "planned" => Ok(WpState::Planned),
        "doing" => Ok(WpState::Doing),
        "review" => Ok(WpState::Review),
        "done" => Ok(WpState::Done),
        "blocked" => Ok(WpState::Blocked),
        _ => Err(DomainError::Storage(format!("invalid wp state: {s}"))),
    }
}

fn pr_state_str(s: PrState) -> &'static str {
    match s {
        PrState::Open => "open",
        PrState::Review => "review",
        PrState::ChangesRequested => "changes_requested",
        PrState::Approved => "approved",
        PrState::Merged => "merged",
    }
}

fn dep_type_str(d: DependencyType) -> &'static str {
    match d {
        DependencyType::Explicit => "explicit",
        DependencyType::FileOverlap => "file_overlap",
        DependencyType::Data => "data",
    }
}

fn dep_type_from_str(s: &str) -> Result<DependencyType, DomainError> {
    match s {
        "explicit" => Ok(DependencyType::Explicit),
        "file_overlap" => Ok(DependencyType::FileOverlap),
        "data" => Ok(DependencyType::Data),
        _ => Err(DomainError::Storage(format!("invalid dep type: {s}"))),
    }
}

fn row_to_wp(row: &Row<'_>) -> rusqlite::Result<WorkPackage> {
    let id: i64 = row.get(0)?;
    let feature_id: i64 = row.get(1)?;
    let title: String = row.get(2)?;
    let state_s: String = row.get(3)?;
    let sequence: i32 = row.get(4)?;
    let file_scope_json: String = row.get(5)?;
    let acceptance_criteria: String = row.get(6)?;
    let agent_id: Option<String> = row.get(7)?;
    let pr_url: Option<String> = row.get(8)?;
    let pr_state_s: Option<String> = row.get(9)?;
    let worktree_path: Option<String> = row.get(10)?;
    let created_at_s: String = row.get(11)?;
    let updated_at_s: String = row.get(12)?;

    let state = wp_state_from_str(&state_s)
        .map_err(|_| rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "bad wp state"))))?;

    // We use serde_json to parse, but can't use ? directly in rusqlite's Result<T>
    let file_scope: Vec<String> =
        serde_json::from_str(&file_scope_json).unwrap_or_default();

    let pr_state = pr_state_s.as_deref().map(|s| match s {
        "open" => PrState::Open,
        "review" => PrState::Review,
        "changes_requested" => PrState::ChangesRequested,
        "approved" => PrState::Approved,
        "merged" => PrState::Merged,
        _ => PrState::Open, // fallback
    });

    let created_at = created_at_s
        .parse::<chrono::DateTime<chrono::Utc>>()
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(11, rusqlite::types::Type::Text, Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))))?;

    let updated_at = updated_at_s
        .parse::<chrono::DateTime<chrono::Utc>>()
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(12, rusqlite::types::Type::Text, Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))))?;

    Ok(WorkPackage {
        id,
        feature_id,
        title,
        state,
        sequence,
        file_scope,
        acceptance_criteria,
        agent_id,
        pr_url,
        pr_state,
        worktree_path,
        plane_sub_issue_id: None,
        created_at,
        updated_at,
    })
}

pub fn create_work_package(conn: &Connection, wp: &WorkPackage) -> Result<i64, DomainError> {
    let file_scope_json =
        serde_json::to_string(&wp.file_scope).map_err(|e| DomainError::Storage(e.to_string()))?;
    let pr_state_s = wp.pr_state.map(pr_state_str);

    conn.execute(
        "INSERT INTO work_packages
         (feature_id, title, state, sequence, file_scope, acceptance_criteria,
          agent_id, pr_url, pr_state, worktree_path, created_at, updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        params![
            wp.feature_id,
            wp.title,
            wp_state_str(wp.state),
            wp.sequence,
            file_scope_json,
            wp.acceptance_criteria,
            wp.agent_id,
            wp.pr_url,
            pr_state_s,
            wp.worktree_path,
            wp.created_at.to_rfc3339(),
            wp.updated_at.to_rfc3339(),
        ],
    )
    .map_err(map_err)?;
    Ok(conn.last_insert_rowid())
}

pub fn get_work_package(conn: &Connection, id: i64) -> Result<Option<WorkPackage>, DomainError> {
    conn.query_row(
        "SELECT id,feature_id,title,state,sequence,file_scope,acceptance_criteria,
                agent_id,pr_url,pr_state,worktree_path,created_at,updated_at
         FROM work_packages WHERE id = ?1",
        params![id],
        row_to_wp,
    )
    .optional()
    .map_err(map_err)
}

pub fn update_wp_state(conn: &Connection, id: i64, state: WpState) -> Result<(), DomainError> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE work_packages SET state = ?1, updated_at = ?2 WHERE id = ?3",
        params![wp_state_str(state), now, id],
    )
    .map_err(map_err)?;
    Ok(())
}

pub fn list_wps_by_feature(
    conn: &Connection,
    feature_id: i64,
) -> Result<Vec<WorkPackage>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id,feature_id,title,state,sequence,file_scope,acceptance_criteria,
                    agent_id,pr_url,pr_state,worktree_path,created_at,updated_at
             FROM work_packages WHERE feature_id = ?1 ORDER BY sequence",
        )
        .map_err(map_err)?;

    let rows = stmt.query_map(params![feature_id], row_to_wp).map_err(map_err)?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(map_err)
}

pub fn add_wp_dependency(conn: &Connection, dep: &WpDependency) -> Result<(), DomainError> {
    conn.execute(
        "INSERT OR IGNORE INTO wp_dependencies (wp_id, depends_on, dep_type)
         VALUES (?1, ?2, ?3)",
        params![dep.wp_id, dep.depends_on, dep_type_str(dep.dep_type)],
    )
    .map_err(map_err)?;
    Ok(())
}

pub fn get_wp_dependencies(
    conn: &Connection,
    wp_id: i64,
) -> Result<Vec<WpDependency>, DomainError> {
    let mut stmt = conn
        .prepare("SELECT wp_id, depends_on, dep_type FROM wp_dependencies WHERE wp_id = ?1")
        .map_err(map_err)?;

    let rows = stmt
        .query_map(params![wp_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(map_err)?;

    let mut deps = Vec::new();
    for row in rows {
        let (wp_id, depends_on, dep_type_s) = row.map_err(map_err)?;
        let dep_type = dep_type_from_str(&dep_type_s)?;
        deps.push(WpDependency { wp_id, depends_on, dep_type });
    }
    Ok(deps)
}

pub fn get_ready_wps(conn: &Connection, feature_id: i64) -> Result<Vec<WorkPackage>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT wp.id,wp.feature_id,wp.title,wp.state,wp.sequence,wp.file_scope,
                    wp.acceptance_criteria,wp.agent_id,wp.pr_url,wp.pr_state,
                    wp.worktree_path,wp.created_at,wp.updated_at
             FROM work_packages wp
             WHERE wp.feature_id = ?1 AND wp.state = 'planned'
               AND NOT EXISTS (
                   SELECT 1 FROM wp_dependencies d
                   JOIN work_packages dep ON dep.id = d.depends_on
                   WHERE d.wp_id = wp.id AND dep.state != 'done'
               )
             ORDER BY wp.sequence",
        )
        .map_err(map_err)?;

    let rows = stmt.query_map(params![feature_id], row_to_wp).map_err(map_err)?;
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

