use rusqlite::Row;

use agileplus_domain::domain::{cycle::Cycle, feature::Feature, state_machine::FeatureState};

fn parse_datetime(
    value: &str,
    idx: usize,
) -> Result<chrono::DateTime<chrono::Utc>, rusqlite::Error> {
    value.parse::<chrono::DateTime<chrono::Utc>>().map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            idx,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            )),
        )
    })
}

pub(super) fn row_to_cycle(row: &Row<'_>) -> rusqlite::Result<Cycle> {
    let id: i64 = row.get(0)?;
    let name: String = row.get(1)?;
    let description: Option<String> = row.get(2)?;
    let state_str: String = row.get(3)?;
    let start_date_str: String = row.get(4)?;
    let end_date_str: String = row.get(5)?;
    let module_scope_id: Option<i64> = row.get(6)?;
    let created_at_str: String = row.get(7)?;
    let updated_at_str: String = row.get(8)?;

    let state = state_str
        .parse::<agileplus_domain::domain::cycle::CycleState>()
        .map_err(|e| {
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

    let created_at = parse_datetime(&created_at_str, 7)?;
    let updated_at = parse_datetime(&updated_at_str, 8)?;

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

pub(super) fn row_to_feature(row: &Row<'_>) -> rusqlite::Result<Feature> {
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

    let created_at = parse_datetime(&created_at_str, 6)?;
    let updated_at = parse_datetime(&updated_at_str, 7)?;

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
        created_at_commit: None,
        last_modified_commit: None,
        created_at,
        updated_at,
    })
}
