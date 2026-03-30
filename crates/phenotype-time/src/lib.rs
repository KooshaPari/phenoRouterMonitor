//! Time utilities for Phenotype.

use chrono::{DateTime, Duration, Utc};

/// Get current time in UTC.
pub fn now_utc() -> DateTime<Utc> {
    Utc::now()
}

/// Format DateTime as ISO 8601 string.
pub fn format_iso8601(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

/// Format DateTime as a human-readable relative time string.
///
/// Examples: "2 hours ago", "in 3 days", "just now"
pub fn format_human_relative(dt: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(*dt);

    if duration.num_seconds().abs() < 60 {
        if duration.num_seconds() < 0 {
            "in a moment".to_string()
        } else {
            "just now".to_string()
        }
    } else if duration.num_minutes().abs() < 60 {
        let mins = duration.num_minutes().abs();
        if duration.num_minutes() < 0 {
            format!("in {} minute{}", mins, if mins == 1 { "" } else { "s" })
        } else {
            format!("{} minute{} ago", mins, if mins == 1 { "" } else { "s" })
        }
    } else if duration.num_hours().abs() < 24 {
        let hours = duration.num_hours().abs();
        if duration.num_hours() < 0 {
            format!("in {} hour{}", hours, if hours == 1 { "" } else { "s" })
        } else {
            format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
        }
    } else if duration.num_days().abs() < 30 {
        let days = duration.num_days().abs();
        if duration.num_days() < 0 {
            format!("in {} day{}", days, if days == 1 { "" } else { "s" })
        } else {
            format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
        }
    } else if duration.num_days().abs() < 365 {
        let months = duration.num_days().abs() / 30;
        if duration.num_days() < 0 {
            format!(
                "in {} month{}",
                months,
                if months == 1 { "" } else { "s" }
            )
        } else {
            format!(
                "{} month{} ago",
                months,
                if months == 1 { "" } else { "s" }
            )
        }
    } else {
        let years = duration.num_days().abs() / 365;
        if duration.num_days() < 0 {
            format!("in {} year{}", years, if years == 1 { "" } else { "s" })
        } else {
            format!("{} year{} ago", years, if years == 1 { "" } else { "s" })
        }
    }
}

/// Parse a duration string into a chrono::Duration.
///
/// Supports formats like: "5m", "2h", "1d", "30s"
pub fn parse_duration(s: &str) -> Result<Duration, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("empty duration string".to_string());
    }

    let (num_part, unit_part) = s
        .char_indices()
        .find(|(_, c)| !c.is_numeric())
        .map(|(i, _)| (&s[..i], &s[i..]))
        .unwrap_or((s, ""));

    let num: i64 = num_part
        .parse()
        .map_err(|_| format!("invalid number in duration: {}", num_part))?;

    match unit_part.trim() {
        "s" | "sec" | "second" | "seconds" => Ok(Duration::seconds(num)),
        "m" | "min" | "minute" | "minutes" => Ok(Duration::minutes(num)),
        "h" | "hr" | "hour" | "hours" => Ok(Duration::hours(num)),
        "d" | "day" | "days" => Ok(Duration::days(num)),
        "w" | "week" | "weeks" => Ok(Duration::weeks(num)),
        "" => Err("no unit specified (try '5m', '2h', '1d')".to_string()),
        _ => Err(format!("unknown duration unit: {}", unit_part)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_now_utc() {
        let now = now_utc();
        assert!(now.timestamp() > 0);
    }

    #[test]
    fn test_format_iso8601() {
        let dt = DateTime::parse_from_rfc3339("2023-01-15T10:30:45Z")
            .expect("valid date")
            .with_timezone(&Utc);
        let formatted = format_iso8601(&dt);
        assert!(formatted.contains("2023-01-15"));
        assert!(formatted.contains("10:30:45"));
    }

    #[test]
    fn test_format_human_relative_just_now() {
        let now = now_utc();
        let result = format_human_relative(&now);
        assert_eq!(result, "just now");
    }

    #[test]
    fn test_format_human_relative_minutes_ago() {
        let past = now_utc() - Duration::minutes(5);
        let result = format_human_relative(&past);
        assert!(result.contains("minute"));
        assert!(result.contains("ago"));
    }

    #[test]
    fn test_format_human_relative_hours_ago() {
        let past = now_utc() - Duration::hours(3);
        let result = format_human_relative(&past);
        assert!(result.contains("hour"));
        assert!(result.contains("ago"));
    }

    #[test]
    fn test_parse_duration_seconds() {
        let dur = parse_duration("30s").expect("valid");
        assert_eq!(dur.num_seconds(), 30);
    }

    #[test]
    fn test_parse_duration_minutes() {
        let dur = parse_duration("5m").expect("valid");
        assert_eq!(dur.num_minutes(), 5);
    }

    #[test]
    fn test_parse_duration_hours() {
        let dur = parse_duration("2h").expect("valid");
        assert_eq!(dur.num_hours(), 2);
    }

    #[test]
    fn test_parse_duration_days() {
        let dur = parse_duration("1d").expect("valid");
        assert_eq!(dur.num_days(), 1);
    }

    #[test]
    fn test_parse_duration_weeks() {
        let dur = parse_duration("2w").expect("valid");
        assert_eq!(dur.num_weeks(), 2);
    }

    #[test]
    fn test_parse_duration_with_spacing() {
        let dur = parse_duration("  5m  ").expect("valid");
        assert_eq!(dur.num_minutes(), 5);
    }

    #[test]
    fn test_parse_duration_invalid_unit() {
        let result = parse_duration("5x");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_duration_no_unit() {
        let result = parse_duration("5");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_duration_empty() {
        let result = parse_duration("");
        assert!(result.is_err());
    }
}
