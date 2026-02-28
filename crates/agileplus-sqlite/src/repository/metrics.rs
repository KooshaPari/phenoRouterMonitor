//! Metrics repository — CRUD for the `metrics` table.

use rusqlite::{Connection, params};

use agileplus_domain::{
    domain::metric::Metric,
    error::DomainError,
};

fn map_err(e: rusqlite::Error) -> DomainError {
    DomainError::Storage(e.to_string())
}

pub fn record_metric(conn: &Connection, metric: &Metric) -> Result<i64, DomainError> {
    let metadata_s = metric
        .metadata
        .as_ref()
        .map(|m| serde_json::to_string(m))
        .transpose()
        .map_err(|e| DomainError::Storage(e.to_string()))?;

    conn.execute(
        "INSERT INTO metrics (feature_id, command, duration_ms, agent_runs, review_cycles, metadata, timestamp)
         VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![
            metric.feature_id,
            metric.command,
            metric.duration_ms,
            metric.agent_runs,
            metric.review_cycles,
            metadata_s,
            metric.timestamp.to_rfc3339(),
        ],
    )
    .map_err(map_err)?;

    Ok(conn.last_insert_rowid())
}

pub fn get_metrics_by_feature(
    conn: &Connection,
    feature_id: i64,
) -> Result<Vec<Metric>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id,feature_id,command,duration_ms,agent_runs,review_cycles,metadata,timestamp
             FROM metrics WHERE feature_id = ?1 ORDER BY timestamp",
        )
        .map_err(map_err)?;

    let rows = stmt
        .query_map(params![feature_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, Option<i64>>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i32>(4)?,
                row.get::<_, i32>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, String>(7)?,
            ))
        })
        .map_err(map_err)?;

    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(map_err)?
        .into_iter()
        .map(
            |(id, feature_id, command, duration_ms, agent_runs, review_cycles, metadata_s, timestamp_s)| {
                let metadata = metadata_s
                    .map(|s| serde_json::from_str(&s))
                    .transpose()
                    .map_err(|e: serde_json::Error| DomainError::Storage(e.to_string()))?;
                let timestamp = timestamp_s
                    .parse::<chrono::DateTime<chrono::Utc>>()
                    .map_err(|e| DomainError::Storage(e.to_string()))?;
                Ok(Metric {
                    id,
                    feature_id,
                    command,
                    duration_ms,
                    agent_runs,
                    review_cycles,
                    metadata,
                    timestamp,
                })
            },
        )
        .collect()
}
