use super::*;
use agileplus_domain::domain::cycle::{Cycle, CycleState};
use chrono::NaiveDate;

/// Traces to: FR-C01
#[test]
fn create_args_round_trip() {
    let args = CreateArgs {
        name: "Q1-2026".to_string(),
        start: "2026-01-01".to_string(),
        end: "2026-03-31".to_string(),
        description: Some("First quarter".to_string()),
        module: None,
    };
    assert_eq!(args.name, "Q1-2026");
    let start = NaiveDate::parse_from_str(&args.start, "%Y-%m-%d").unwrap();
    let end = NaiveDate::parse_from_str(&args.end, "%Y-%m-%d").unwrap();
    assert!(end > start);
}

/// Traces to: FR-C01
#[test]
fn create_args_invalid_date_format() {
    let bad = "2026/01/01";
    let result = NaiveDate::parse_from_str(bad, "%Y-%m-%d");
    assert!(result.is_err(), "bad date format should not parse");
}

/// Traces to: FR-C02
#[test]
fn transition_args_state_parsing() {
    let valid = ["Draft", "Active", "Review", "Shipped", "Archived"];
    for s in valid {
        assert!(
            s.parse::<CycleState>().is_ok(),
            "state '{}' should parse",
            s
        );
    }
    let bad = "unknown".parse::<CycleState>();
    assert!(bad.is_err());
}

/// Traces to: FR-C02
#[test]
fn prior_state_label_coverage() {
    let cycle = Cycle::new(
        "c",
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
        None,
    )
    .unwrap();
    let label = prior_state_label(CycleState::Active, &cycle);
    assert!(!label.is_empty());
}

/// Traces to: FR-C03
#[test]
fn add_args_fields() {
    let args = AddArgs {
        cycle: "Q1".to_string(),
        feature: "feat-auth".to_string(),
    };
    assert_eq!(args.cycle, "Q1");
    assert_eq!(args.feature, "feat-auth");
}

/// Traces to: FR-C03
#[test]
fn remove_args_fields() {
    let args = RemoveArgs {
        cycle: "Q1".to_string(),
        feature: "feat-auth".to_string(),
    };
    assert_eq!(args.cycle, "Q1");
    assert_eq!(args.feature, "feat-auth");
}

/// Traces to: FR-C04
#[test]
fn list_args_no_state() {
    let args = ListArgs { state: None };
    assert!(args.state.is_none());
}

/// Traces to: FR-C04
#[test]
fn list_args_with_state() {
    let args = ListArgs {
        state: Some("Active".to_string()),
    };
    let state = args.state.unwrap().parse::<CycleState>().unwrap();
    assert_eq!(state, CycleState::Active);
}

/// Traces to: FR-C05
#[test]
fn show_args_name() {
    let args = ShowArgs {
        name: "Q1-2026".to_string(),
    };
    assert_eq!(args.name, "Q1-2026");
}
