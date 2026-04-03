use std::str::FromStr;

use anyhow::Result;

use agileplus_domain::domain::backlog::{BacklogPriority, BacklogSort, BacklogStatus, Intent};

pub(crate) fn parse_intent(value: Option<String>) -> Result<Intent> {
    let value = value.unwrap_or_else(|| "task".to_string());
    Intent::from_str(&value).map_err(|e| anyhow::anyhow!(e))
}

pub(crate) fn parse_intent_opt(value: Option<String>) -> Result<Option<Intent>> {
    value
        .map(|v| Intent::from_str(&v).map_err(|e| anyhow::anyhow!(e)))
        .transpose()
}

pub(crate) fn parse_priority(value: String) -> Result<BacklogPriority> {
    BacklogPriority::from_str(&value).map_err(|e| anyhow::anyhow!(e))
}

pub(crate) fn parse_priority_opt(value: Option<String>) -> Result<Option<BacklogPriority>> {
    value
        .map(|v| BacklogPriority::from_str(&v).map_err(|e| anyhow::anyhow!(e)))
        .transpose()
}

pub(crate) fn parse_status_opt(value: Option<String>) -> Result<Option<BacklogStatus>> {
    value
        .map(|v| BacklogStatus::from_str(&v).map_err(|e| anyhow::anyhow!(e)))
        .transpose()
}

pub(crate) fn parse_sort(value: &str) -> Result<BacklogSort> {
    BacklogSort::from_str(value).map_err(|e| anyhow::anyhow!(e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_intent_valid() {
        assert_eq!(parse_intent(Some("bug".into())).unwrap(), Intent::Bug);
        assert_eq!(
            parse_intent(Some("FEATURE".into())).unwrap(),
            Intent::Feature
        );
    }

    #[test]
    fn parse_intent_invalid() {
        assert!(parse_intent(Some("xxx".into())).is_err());
    }
}
