//! Repository operations for backlog queue items.

use rusqlite::{Connection, Row, params, params_from_iter, types::Value};

use agileplus_domain::{
    domain::backlog::{
        BacklogFilters, BacklogItem, BacklogPriority, BacklogSort, BacklogStatus, Intent,
    },
    error::DomainError,
};

fn map_err(e: rusqlite::Error) -> DomainError {
    DomainError::Storage(e.to_string())
}

fn intent_str(intent: Intent) -> &'static str {
    match intent {
        Intent::Bug => "bug",
        Intent::Feature => "feature",
        Intent::Idea => "idea",
        Intent::Task => "task",
    }
}

fn intent_from_str(s: &str) -> Result<Intent, DomainError> {
    s.parse::<Intent>()
        .map_err(|e| DomainError::Storage(format!("invalid backlog intent: {e}")))
}

fn priority_str(priority: BacklogPriority) -> &'static str {
    match priority {
        BacklogPriority::Critical => "critical",
        BacklogPriority::High => "high",
        BacklogPriority::Medium => "medium",
        BacklogPriority::Low => "low",
    }
}

fn priority_from_str(s: &str) -> Result<BacklogPriority, DomainError> {
    s.parse::<BacklogPriority>()
        .map_err(|e| DomainError::Storage(format!("invalid backlog priority: {e}")))
}

fn status_str(status: BacklogStatus) -> &'static str {
    match status {
        BacklogStatus::New => "new",
        BacklogStatus::Triaged => "triaged",
        BacklogStatus::InProgress => "in_progress",
        BacklogStatus::Done => "done",
        BacklogStatus::Dismissed => "dismissed",
    }
}

fn status_from_str(s: &str) -> Result<BacklogStatus, DomainError> {
    s.parse::<BacklogStatus>()
        .map_err(|e| DomainError::Storage(format!("invalid backlog status: {e}")))
}

fn row_to_backlog(row: &Row<'_>) -> rusqlite::Result<BacklogItem> {
    let id: i64 = row.get(0)?;
    let title: String = row.get(1)?;
    let description: String = row.get(2)?;
    let intent_s: String = row.get(3)?;
    let priority_s: String = row.get(4)?;
    let status_s: String = row.get(5)?;
    let source: String = row.get(6)?;
    let feature_slug: Option<String> = row.get(7)?;
    let tags_json: String = row.get(8)?;
    let created_at_s: String = row.get(9)?;
    let updated_at_s: String = row.get(10)?;

    let intent = intent_from_str(&intent_s).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            3,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::other(e.to_string())),
        )
    })?;
    let priority = priority_from_str(&priority_s).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            4,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::other(e.to_string())),
        )
    })?;
    let status = status_from_str(&status_s).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            5,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::other(e.to_string())),
        )
    })?;

    let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
    let created_at = created_at_s
        .parse::<chrono::DateTime<chrono::Utc>>()
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                9,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::other(e.to_string())),
            )
        })?;
    let updated_at = updated_at_s
        .parse::<chrono::DateTime<chrono::Utc>>()
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                10,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::other(e.to_string())),
            )
        })?;

    Ok(BacklogItem {
        id: Some(id),
        title,
        description,
        intent,
        priority,
        status,
        source,
        feature_slug,
        tags,
        created_at,
        updated_at,
    })
}

pub fn create_backlog_item(conn: &Connection, item: &BacklogItem) -> Result<i64, DomainError> {
    let tags_json =
        serde_json::to_string(&item.tags).map_err(|e| DomainError::Storage(e.to_string()))?;
    conn.execute(
        "INSERT INTO backlog_items
         (title, description, intent, priority, status, source, feature_slug, tags_json, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            item.title,
            item.description,
            intent_str(item.intent),
            priority_str(item.priority),
            status_str(item.status),
            item.source,
            item.feature_slug,
            tags_json,
            item.created_at.to_rfc3339(),
            item.updated_at.to_rfc3339(),
        ],
    )
    .map_err(map_err)?;
    Ok(conn.last_insert_rowid())
}

pub fn get_backlog_item(conn: &Connection, id: i64) -> Result<Option<BacklogItem>, DomainError> {
    conn.query_row(
        "SELECT id, title, description, intent, priority, status, source, feature_slug, tags_json, created_at, updated_at
         FROM backlog_items WHERE id = ?1",
        params![id],
        row_to_backlog,
    )
    .optional()
    .map_err(map_err)
}

pub fn list_backlog_items(
    conn: &Connection,
    filters: &BacklogFilters,
) -> Result<Vec<BacklogItem>, DomainError> {
    let mut sql = String::from(
        "SELECT id, title, description, intent, priority, status, source, feature_slug, tags_json, created_at, updated_at
         FROM backlog_items WHERE 1=1",
    );
    let mut values: Vec<Value> = Vec::new();

    if let Some(intent) = filters.intent {
        sql.push_str(" AND intent = ?");
        values.push(Value::Text(intent_str(intent).to_string()));
    }
    if let Some(status) = filters.status {
        sql.push_str(" AND status = ?");
        values.push(Value::Text(status_str(status).to_string()));
    }
    if let Some(priority) = filters.priority {
        sql.push_str(" AND priority = ?");
        values.push(Value::Text(priority_str(priority).to_string()));
    }
    if let Some(ref feature_slug) = filters.feature_slug {
        sql.push_str(" AND feature_slug = ?");
        values.push(Value::Text(feature_slug.clone()));
    }
    if let Some(ref source) = filters.source {
        sql.push_str(" AND source = ?");
        values.push(Value::Text(source.clone()));
    }

    match filters.sort {
        BacklogSort::Age => sql.push_str(" ORDER BY created_at ASC"),
        BacklogSort::Impact | BacklogSort::Priority => {
            sql.push_str(
                " ORDER BY CASE priority
                    WHEN 'critical' THEN 0
                    WHEN 'high' THEN 1
                    WHEN 'medium' THEN 2
                    ELSE 3
                 END ASC, created_at ASC",
            );
        }
    }

    if let Some(limit) = filters.limit {
        sql.push_str(" LIMIT ?");
        values.push(Value::Integer(limit as i64));
    }

    let mut stmt = conn.prepare(&sql).map_err(map_err)?;
    let rows = stmt
        .query_map(params_from_iter(values), row_to_backlog)
        .map_err(map_err)?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(map_err)
}

pub fn update_backlog_status(
    conn: &Connection,
    id: i64,
    status: BacklogStatus,
) -> Result<(), DomainError> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE backlog_items SET status = ?1, updated_at = ?2 WHERE id = ?3",
        params![status_str(status), now, id],
    )
    .map_err(map_err)?;
    Ok(())
}

pub fn update_backlog_priority(
    conn: &Connection,
    id: i64,
    priority: BacklogPriority,
) -> Result<(), DomainError> {
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE backlog_items SET priority = ?1, updated_at = ?2 WHERE id = ?3",
        params![priority_str(priority), now, id],
    )
    .map_err(map_err)?;
    Ok(())
}

pub fn pop_next_backlog_item(conn: &Connection) -> Result<Option<BacklogItem>, DomainError> {
    let next = conn
        .query_row(
            "SELECT id, title, description, intent, priority, status, source, feature_slug, tags_json, created_at, updated_at
             FROM backlog_items
             WHERE status = 'new'
             ORDER BY CASE priority
                 WHEN 'critical' THEN 0
                 WHEN 'high' THEN 1
                 WHEN 'medium' THEN 2
                 ELSE 3
             END ASC, created_at ASC
             LIMIT 1",
            [],
            row_to_backlog,
        )
        .optional()
        .map_err(map_err)?;

    if let Some(mut item) = next {
        let id = item
            .id
            .ok_or_else(|| DomainError::Storage("backlog item missing id".to_string()))?;
        update_backlog_status(conn, id, BacklogStatus::Triaged)?;
        item.status = BacklogStatus::Triaged;
        item.updated_at = chrono::Utc::now();
        Ok(Some(item))
    } else {
        Ok(None)
    }
}

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
