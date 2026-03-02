//! Event repository — CRUD for the `events` table.

use rusqlite::{Connection, Row, params};

use agileplus_domain::domain::event::Event;
use agileplus_domain::error::DomainError;

fn map_err(e: rusqlite::Error) -> DomainError {
    DomainError::Storage(e.to_string())
}

fn row_to_event(row: &Row<'_>) -> rusqlite::Result<Event> {
    let id: i64 = row.get(0)?;
    let entity_type: String = row.get(1)?;
    let entity_id: i64 = row.get(2)?;
    let event_type: String = row.get(3)?;
    let payload_str: String = row.get(4)?;
    let actor: String = row.get(5)?;
    let timestamp_str: String = row.get(6)?;
    let prev_hash_bytes: Vec<u8> = row.get(7)?;
    let hash_bytes: Vec<u8> = row.get(8)?;
    let sequence: i64 = row.get(9)?;

    let timestamp = timestamp_str
        .parse::<chrono::DateTime<chrono::Utc>>()
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                6,
                rusqlite::types::Type::Text,
                Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())),
            )
        })?;

    let payload: serde_json::Value = serde_json::from_str(&payload_str).unwrap_or_default();

    let mut prev_hash = [0u8; 32];
    if prev_hash_bytes.len() == 32 {
        prev_hash.copy_from_slice(&prev_hash_bytes);
    }
    let mut hash = [0u8; 32];
    if hash_bytes.len() == 32 {
        hash.copy_from_slice(&hash_bytes);
    }

    Ok(Event {
        id,
        entity_type,
        entity_id,
        event_type,
        payload,
        actor,
        timestamp,
        prev_hash,
        hash,
        sequence,
    })
}

const SELECT_COLS: &str =
    "id, entity_type, entity_id, event_type, payload, actor, timestamp, prev_hash, hash, sequence";

pub fn append_event(conn: &Connection, event: &Event) -> Result<i64, DomainError> {
    let payload_json = serde_json::to_string(&event.payload)
        .map_err(|e| DomainError::Storage(format!("serialize payload: {e}")))?;

    conn.execute(
        &format!(
            "INSERT INTO events ({SELECT_COLS}) VALUES (NULL, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
        ),
        params![
            event.entity_type,
            event.entity_id,
            event.event_type,
            payload_json,
            event.actor,
            event.timestamp.to_rfc3339(),
            &event.prev_hash[..],
            &event.hash[..],
            event.sequence,
        ],
    )
    .map_err(map_err)?;

    Ok(conn.last_insert_rowid())
}

pub fn get_events(
    conn: &Connection,
    entity_type: &str,
    entity_id: i64,
) -> Result<Vec<Event>, DomainError> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {SELECT_COLS} FROM events WHERE entity_type = ?1 AND entity_id = ?2 ORDER BY sequence ASC"
        ))
        .map_err(map_err)?;

    let rows = stmt
        .query_map(params![entity_type, entity_id], row_to_event)
        .map_err(map_err)?;

    rows.collect::<Result<Vec<_>, _>>().map_err(map_err)
}

pub fn get_events_since(
    conn: &Connection,
    entity_type: &str,
    entity_id: i64,
    sequence: i64,
) -> Result<Vec<Event>, DomainError> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {SELECT_COLS} FROM events WHERE entity_type = ?1 AND entity_id = ?2 AND sequence > ?3 ORDER BY sequence ASC"
        ))
        .map_err(map_err)?;

    let rows = stmt
        .query_map(params![entity_type, entity_id, sequence], row_to_event)
        .map_err(map_err)?;

    rows.collect::<Result<Vec<_>, _>>().map_err(map_err)
}

pub fn get_events_by_range(
    conn: &Connection,
    entity_type: &str,
    entity_id: i64,
    from: &str,
    to: &str,
) -> Result<Vec<Event>, DomainError> {
    let mut stmt = conn
        .prepare(&format!(
            "SELECT {SELECT_COLS} FROM events WHERE entity_type = ?1 AND entity_id = ?2 AND timestamp >= ?3 AND timestamp <= ?4 ORDER BY sequence ASC"
        ))
        .map_err(map_err)?;

    let rows = stmt
        .query_map(params![entity_type, entity_id, from, to], row_to_event)
        .map_err(map_err)?;

    rows.collect::<Result<Vec<_>, _>>().map_err(map_err)
}

pub fn get_latest_sequence(
    conn: &Connection,
    entity_type: &str,
    entity_id: i64,
) -> Result<i64, DomainError> {
    let result: Option<i64> = conn
        .query_row(
            "SELECT MAX(sequence) FROM events WHERE entity_type = ?1 AND entity_id = ?2",
            params![entity_type, entity_id],
            |row| row.get(0),
        )
        .map_err(map_err)?;

    Ok(result.unwrap_or(0))
}
