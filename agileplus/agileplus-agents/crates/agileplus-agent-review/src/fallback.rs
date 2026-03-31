//! Manual review fallback: CLI-based approval flow for when Coderabbit is
//! unavailable or times out.

use agileplus_agent_dispatch::DomainError;
use chrono::{DateTime, Utc};
use std::io::{self, BufRead, IsTerminal, Write};
use std::time::Duration;
use tracing::{info, warn};

// ─── Public types ─────────────────────────────────────────────────────────────

/// Outcome of a manual review interaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManualReviewResult {
    Approved {
        reviewer: String,
    },
    Rejected {
        reviewer: String,
        reason: String,
    },
    ChangesRequested {
        reviewer: String,
        comments: Vec<String>,
    },
}

// ─── Timeout heuristic ────────────────────────────────────────────────────────

/// Returns `true` if Coderabbit has not responded within `timeout`.
///
/// - If `last_check` is `None` (never checked), returns `true` immediately.
/// - Otherwise returns `true` once `timeout` has elapsed since `last_check`.
pub fn should_fallback(last_check: Option<DateTime<Utc>>, timeout: Duration) -> bool {
    match last_check {
        None => true,
        Some(ts) => {
            let elapsed = Utc::now()
                .signed_duration_since(ts)
                .to_std()
                .unwrap_or(Duration::ZERO);
            elapsed >= timeout
        }
    }
}

// ─── Interactive prompt ───────────────────────────────────────────────────────

/// Prompt a human reviewer via stdin/stdout.
///
/// Prints the PR URL and WP context, then asks for approval, rejection, or
/// change requests.  Multi-line reason/comments are collected until an empty
/// line is entered.
///
/// # Non-interactive mode
///
/// If stdin is not a TTY, returns `Err(DomainError::Other("non-interactive"))`.
/// This prevents the function from hanging in CI or background-agent contexts.
pub fn prompt_manual_review(
    pr_url: &str,
    wp_title: &str,
) -> Result<ManualReviewResult, DomainError> {
    if !io::stdin().is_terminal() {
        warn!("stdin is not a TTY — manual review fallback unavailable");
        return Err(DomainError::Other(
            "non-interactive: stdin is not a TTY, manual review fallback skipped".to_owned(),
        ));
    }

    let stdout = io::stdout();
    let mut out = stdout.lock();

    writeln!(out, "\n========== MANUAL REVIEW REQUIRED ==========").ok();
    writeln!(out, "Work package : {wp_title}").ok();
    writeln!(out, "PR URL       : {pr_url}").ok();
    writeln!(out, "Coderabbit did not respond in time. Human review needed.").ok();
    writeln!(out, "Options: [a] approve   [r] reject   [c] request changes").ok();
    write!(out, "Your choice: ").ok();
    out.flush().ok();

    let choice = read_line()?;
    let choice = choice.trim().to_lowercase();

    match choice.as_str() {
        "a" | "approve" => {
            let reviewer = prompt_string(&mut out, "Reviewer name: ")?;
            info!(reviewer, "manual reviewer approved PR");
            Ok(ManualReviewResult::Approved { reviewer })
        }
        "r" | "reject" => {
            let reviewer = prompt_string(&mut out, "Reviewer name: ")?;
            let reason = prompt_multiline(&mut out, "Reason (empty line to finish):")?;
            warn!(reviewer, "manual reviewer rejected PR");
            Ok(ManualReviewResult::Rejected {
                reviewer,
                reason: reason.join("\n"),
            })
        }
        "c" | "changes" => {
            let reviewer = prompt_string(&mut out, "Reviewer name: ")?;
            let lines =
                prompt_multiline(&mut out, "Comments (one per line, empty line to finish):")?;
            warn!(reviewer, count = lines.len(), "manual reviewer requested changes");
            Ok(ManualReviewResult::ChangesRequested {
                reviewer,
                comments: lines,
            })
        }
        _ => Err(DomainError::Other(format!(
            "unrecognised review choice: {choice}"
        ))),
    }
}

// ─── stdin helpers ────────────────────────────────────────────────────────────

fn read_line() -> Result<String, DomainError> {
    let stdin = io::stdin();
    let mut line = String::new();
    stdin
        .lock()
        .read_line(&mut line)
        .map_err(|e| DomainError::Io(e))?;
    Ok(line)
}

fn prompt_string(out: &mut impl Write, prompt: &str) -> Result<String, DomainError> {
    write!(out, "{prompt}").ok();
    out.flush().ok();
    let raw = read_line()?;
    Ok(raw.trim().to_owned())
}

fn prompt_multiline(out: &mut impl Write, header: &str) -> Result<Vec<String>, DomainError> {
    writeln!(out, "{header}").ok();
    out.flush().ok();

    let stdin = io::stdin();
    let mut lines = Vec::new();
    for raw in stdin.lock().lines() {
        let line = raw.map_err(DomainError::Io)?;
        if line.trim().is_empty() {
            break;
        }
        lines.push(line);
    }
    Ok(lines)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::time::Duration;

    #[test]
    fn should_fallback_none() {
        assert!(should_fallback(None, Duration::from_secs(300)));
    }

    #[test]
    fn should_fallback_recent() {
        let recent = Utc::now();
        assert!(!should_fallback(Some(recent), Duration::from_secs(300)));
    }

    #[test]
    fn should_fallback_expired() {
        // Simulate a timestamp 10 minutes in the past.
        let old = Utc::now() - chrono::Duration::minutes(10);
        assert!(should_fallback(Some(old), Duration::from_secs(300)));
    }

    #[test]
    fn non_interactive_returns_error() {
        // In test mode stdin is not a TTY, so prompt_manual_review should Err.
        let result = prompt_manual_review("https://github.com/x/y/pull/1", "WP09 test");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("non-interactive"));
    }
}
