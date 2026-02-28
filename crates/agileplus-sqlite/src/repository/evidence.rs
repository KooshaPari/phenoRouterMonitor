//! Evidence repository — CRUD for the `evidence` table.

use rusqlite::{Connection, Row, params};

use agileplus_domain::{
    domain::governance::{Evidence, EvidenceType},
    error::DomainError,
};

fn map_err(e: rusqlite::Error) -> DomainError {
    DomainError::Storage(e.to_string())
}

fn evidence_type_str(t: EvidenceType) -> &'static str {
    match t {
        EvidenceType::TestResult => "test_result",
        EvidenceType::CiOutput => "ci_output",
        EvidenceType::ReviewApproval => "review_approval",
        EvidenceType::SecurityScan => "security_scan",
        EvidenceType::LintResult => "lint_result",
        EvidenceType::ManualAttestation => "manual_attestation",
    }
}

fn evidence_type_from_str(s: &str) -> Result<EvidenceType, DomainError> {
    match s {
        "test_result" => Ok(EvidenceType::TestResult),
        "ci_output" => Ok(EvidenceType::CiOutput),
        "review_approval" => Ok(EvidenceType::ReviewApproval),
        "security_scan" => Ok(EvidenceType::SecurityScan),
        "lint_result" => Ok(EvidenceType::LintResult),
        "manual_attestation" => Ok(EvidenceType::ManualAttestation),
        _ => Err(DomainError::Storage(format!("invalid evidence type: {s}"))),
    }
}

fn row_to_evidence(row: &Row<'_>) -> rusqlite::Result<(i64, i64, String, String, String, Option<String>, String)> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
        row.get(5)?,
        row.get(6)?,
    ))
}

fn parse_evidence(row_data: (i64, i64, String, String, String, Option<String>, String)) -> Result<Evidence, DomainError> {
    let (id, wp_id, fr_id, evidence_type_s, artifact_path, metadata_s, created_at_s) = row_data;

    let evidence_type = evidence_type_from_str(&evidence_type_s)?;
    let metadata = metadata_s
        .map(|s| serde_json::from_str(&s))
        .transpose()
        .map_err(|e: serde_json::Error| DomainError::Storage(e.to_string()))?;
    let created_at = created_at_s
        .parse::<chrono::DateTime<chrono::Utc>>()
        .map_err(|e| DomainError::Storage(e.to_string()))?;

    Ok(Evidence {
        id,
        wp_id,
        fr_id,
        evidence_type,
        artifact_path,
        metadata,
        created_at,
    })
}

pub fn create_evidence(conn: &Connection, evidence: &Evidence) -> Result<i64, DomainError> {
    let metadata_s = evidence
        .metadata
        .as_ref()
        .map(|m| serde_json::to_string(m))
        .transpose()
        .map_err(|e| DomainError::Storage(e.to_string()))?;

    conn.execute(
        "INSERT INTO evidence (wp_id, fr_id, evidence_type, artifact_path, metadata, created_at)
         VALUES (?1,?2,?3,?4,?5,?6)",
        params![
            evidence.wp_id,
            evidence.fr_id,
            evidence_type_str(evidence.evidence_type),
            evidence.artifact_path,
            metadata_s,
            evidence.created_at.to_rfc3339(),
        ],
    )
    .map_err(map_err)?;

    Ok(conn.last_insert_rowid())
}

pub fn get_evidence_by_wp(conn: &Connection, wp_id: i64) -> Result<Vec<Evidence>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id,wp_id,fr_id,evidence_type,artifact_path,metadata,created_at
             FROM evidence WHERE wp_id = ?1",
        )
        .map_err(map_err)?;

    let rows = stmt
        .query_map(params![wp_id], row_to_evidence)
        .map_err(map_err)?;

    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(map_err)?
        .into_iter()
        .map(parse_evidence)
        .collect()
}

pub fn get_evidence_by_fr(conn: &Connection, fr_id: &str) -> Result<Vec<Evidence>, DomainError> {
    let mut stmt = conn
        .prepare(
            "SELECT id,wp_id,fr_id,evidence_type,artifact_path,metadata,created_at
             FROM evidence WHERE fr_id = ?1",
        )
        .map_err(map_err)?;

    let rows = stmt
        .query_map(params![fr_id], row_to_evidence)
        .map_err(map_err)?;

    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(map_err)?
        .into_iter()
        .map(parse_evidence)
        .collect()
}
