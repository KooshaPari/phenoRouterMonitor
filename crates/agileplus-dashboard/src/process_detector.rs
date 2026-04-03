//! Process detection for real agent activity monitoring.
//!
//! Detects Claude and other AI tool processes running on the system
//! and extracts agent metadata from process information.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::System;

/// Agent process information extracted from system processes.
#[derive(Debug, Clone)]
pub struct DetectedAgent {
    pub name: String,
    pub pid: u32,
    pub process_name: String,
    pub worktree: Option<String>,
    pub started_at: Option<String>,
    pub status: String,
    pub current_task: String,
}

/// Detect all running agent processes on the system.
pub fn detect_agents() -> Vec<DetectedAgent> {
    let mut system = System::new_all();
    system.refresh_all();

    let agent_patterns = [
        "claude", "gemini", "codex", "cursor", "windsurf", "aider", "cline",
    ];

    let mut agents = Vec::new();

    for (pid, process) in system.processes() {
        let process_name_lower = process.name().to_string_lossy().to_lowercase();

        // Check if process matches any agent pattern
        for pattern in &agent_patterns {
            if process_name_lower.contains(pattern) {
                let agent = extract_agent_info(pid.as_u32(), process);
                agents.push(agent);
                break;
            }
        }
    }

    // Sort by process ID for consistency
    agents.sort_by_key(|a| a.pid);
    agents
}

/// Extract agent information from a process.
fn extract_agent_info(pid: u32, process: &sysinfo::Process) -> DetectedAgent {
    let process_name = process.name().to_string_lossy().to_string();

    // Convert OsStr command line to String for easier parsing
    let cmd_line = process
        .cmd()
        .iter()
        .map(|s| s.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(" ");

    // Try to extract worktree from command line arguments
    let worktree = extract_worktree_from_cmdline(&cmd_line);

    // Determine status based on CPU usage and memory
    let status = if process.cpu_usage() > 1.0 {
        "running".to_string()
    } else {
        "idle".to_string()
    };

    // Try to extract task context from the command line
    let current_task = extract_task_context(&cmd_line);

    // Format the agent name
    let display_name = format_agent_name(&process_name, &worktree);

    // Get start time if available
    let started_at = get_process_start_time(process);

    DetectedAgent {
        name: display_name,
        pid,
        process_name,
        worktree,
        started_at,
        status,
        current_task,
    }
}

/// Extract the worktree/working directory from command line.
fn extract_worktree_from_cmdline(cmdline: &str) -> Option<String> {
    // Look for common patterns like --cwd or -C followed by a path
    let parts: Vec<&str> = cmdline.split_whitespace().collect();
    for i in 0..parts.len() {
        if (parts[i] == "--cwd" || parts[i] == "-C") && i + 1 < parts.len() {
            return Some(parts[i + 1].to_string());
        }
    }
    None
}

/// Extract task context from command line arguments.
fn extract_task_context(cmdline: &str) -> String {
    // Look for task identifiers like WP13, FR-XXX, etc. in command line
    if let Some(pos) = cmdline.find("WP") {
        let rest = &cmdline[pos..];
        // Find the end of the alphanumeric sequence (or end of string)
        let end = rest.find(|c: char| !c.is_alphanumeric())
            .unwrap_or(rest.len());
        if end > 0 {
            return format!("Task: {}", &rest[..end]);
        }
    }

    // Look for feature references
    if let Some(pos) = cmdline.find("feature") {
        if let Some(next_space) = cmdline[pos..].find(' ') {
            let context = &cmdline[pos..pos + next_space];
            return context.to_string();
        }
    }

    String::new()
}

/// Format the agent name based on process and context.
fn format_agent_name(process_name: &str, worktree: &Option<String>) -> String {
    let base_name = if process_name.contains("claude") {
        "claude"
    } else if process_name.contains("gemini") {
        "gemini"
    } else if process_name.contains("codex") {
        "codex"
    } else if process_name.contains("cursor") {
        "cursor"
    } else if process_name.contains("windsurf") {
        "windsurf"
    } else if process_name.contains("aider") {
        "aider"
    } else {
        process_name
    };

    if let Some(wt) = worktree {
        // Extract just the project name from the worktree path
        if let Some(last_segment) = wt.split('/').next_back() {
            return format!("{}-{}", base_name, last_segment);
        }
    }

    base_name.to_string()
}

/// Get the start time of a process as a human-readable elapsed string.
///
/// sysinfo exposes `start_time()` as seconds since UNIX epoch on supported
/// platforms. We compare against wall clock to produce "Xm ago" strings.
fn get_process_start_time(process: &sysinfo::Process) -> Option<String> {
    let start_secs = process.start_time();
    if start_secs == 0 {
        return None;
    }
    let now_secs = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    let elapsed = now_secs.saturating_sub(start_secs);
    let formatted = format_elapsed(elapsed);
    Some(formatted)
}

/// Format an elapsed duration in seconds as a human-readable string.
fn format_elapsed(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

/// Read agent state files from .agileplus/agents/*.json
pub fn read_agent_state_files(base_path: &str) -> Vec<DetectedAgent> {
    let agents_dir = PathBuf::from(base_path).join(".agileplus").join("agents");
    let mut agents = Vec::new();

    if !agents_dir.exists() {
        return agents;
    }

    if let Ok(entries) = std::fs::read_dir(&agents_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        // Try to parse as JSON and extract agent info
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(agent) = parse_agent_state(&json) {
                                agents.push(agent);
                            }
                        }
                    }
                }
            }
        }
    }

    agents
}

/// Parse a single agent state file.
fn parse_agent_state(json: &serde_json::Value) -> Option<DetectedAgent> {
    let name = json.get("name")?.as_str()?.to_string();
    let status = json.get("status").and_then(|v| v.as_str()).unwrap_or("idle").to_string();
    let current_task = json.get("current_task")?.as_str().unwrap_or("").to_string();
    let worktree = json
        .get("worktree")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let started_at = json
        .get("started_at")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let process_name = name.clone();

    Some(DetectedAgent {
        name,
        pid: 0, // Not available from state file
        process_name,
        worktree,
        started_at,
        status,
        current_task,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── format_elapsed ────────────────────────────────────────────────────

    #[test]
    fn test_format_elapsed_seconds() {
        assert_eq!(format_elapsed(0), "0s");
        assert_eq!(format_elapsed(45), "45s");
        assert_eq!(format_elapsed(59), "59s");
    }

    #[test]
    fn test_format_elapsed_minutes() {
        assert_eq!(format_elapsed(60), "1m");
        assert_eq!(format_elapsed(90), "1m");
        assert_eq!(format_elapsed(3599), "59m");
    }

    #[test]
    fn test_format_elapsed_hours() {
        assert_eq!(format_elapsed(3600), "1h 0m");
        assert_eq!(format_elapsed(3661), "1h 1m");
        assert_eq!(format_elapsed(7322), "2h 2m");
    }

    // ── extract_worktree_from_cmdline ─────────────────────────────────────

    #[test]
    fn test_extract_worktree_cwd_flag() {
        let result = extract_worktree_from_cmdline("claude --cwd /repos/AgilePlus");
        assert_eq!(result, Some("/repos/AgilePlus".to_string()));
    }

    #[test]
    fn test_extract_worktree_capital_c_flag() {
        let result = extract_worktree_from_cmdline("git -C /some/path status");
        assert_eq!(result, Some("/some/path".to_string()));
    }

    #[test]
    fn test_extract_worktree_absent() {
        let result = extract_worktree_from_cmdline("claude --some-flag value");
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_worktree_empty_cmdline() {
        let result = extract_worktree_from_cmdline("");
        assert_eq!(result, None);
    }

    // ── extract_task_context ──────────────────────────────────────────────

    #[test]
    fn test_extract_task_context_wp_reference() {
        let result = extract_task_context("claude --task WP13");
        assert!(result.contains("WP13"), "expected WP13 in: {result}");
    }

    #[test]
    fn test_extract_task_context_no_reference() {
        let result = extract_task_context("claude --some-other-arg");
        assert_eq!(result, String::new());
    }

    // ── format_agent_name ─────────────────────────────────────────────────

    #[test]
    fn test_format_agent_name_claude_no_worktree() {
        let result = format_agent_name("claude", &None);
        assert_eq!(result, "claude");
    }

    #[test]
    fn test_format_agent_name_claude_with_worktree() {
        let wt = Some("/repos/AgilePlus-wtrees/my-feature".to_string());
        let result = format_agent_name("claude", &wt);
        assert_eq!(result, "claude-my-feature");
    }

    #[test]
    fn test_format_agent_name_unknown_process() {
        let result = format_agent_name("someotheragent", &None);
        assert_eq!(result, "someotheragent");
    }

    #[test]
    fn test_format_agent_name_gemini() {
        let result = format_agent_name("gemini-cli", &None);
        assert_eq!(result, "gemini");
    }

    // ── parse_agent_state ─────────────────────────────────────────────────

    #[test]
    fn test_parse_agent_state_valid() {
        let json = serde_json::json!({
            "name": "planner-agent",
            "status": "running",
            "current_task": "WP13",
            "worktree": "/repos/AgilePlus",
            "started_at": "2026-03-28T10:00:00Z"
        });
        let agent = parse_agent_state(&json).expect("should parse");
        assert_eq!(agent.name, "planner-agent");
        assert_eq!(agent.status, "running");
        assert_eq!(agent.current_task, "WP13");
        assert_eq!(agent.worktree, Some("/repos/AgilePlus".to_string()));
    }

    #[test]
    fn test_parse_agent_state_missing_name_returns_none() {
        let json = serde_json::json!({ "status": "idle" });
        assert!(parse_agent_state(&json).is_none());
    }

    #[test]
    fn test_parse_agent_state_defaults_status_idle() {
        let json = serde_json::json!({ "name": "test-agent", "current_task": "nothing" });
        let agent = parse_agent_state(&json).expect("should parse");
        assert_eq!(agent.status, "idle");
    }
}
