use std::io::{self, BufRead};

use anyhow::{Context, Result};
use chrono::Utc;

use agileplus_domain::domain::feature::Feature;
use agileplus_domain::ports::StoragePort;

use super::SpecifyArgs;

/// Collect spec content either from a file or interactive stdin prompts.
pub(crate) async fn gather_spec<S: StoragePort>(
    args: &SpecifyArgs,
    _storage: &S,
) -> Result<(String, String, String)> {
    if let Some(ref path) = args.from_file {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("reading spec from {}", path.display()))?;

        let friendly_name = content
            .lines()
            .find(|l| l.starts_with("# "))
            .map(|l| l.trim_start_matches("# Specification:").trim().to_string())
            .unwrap_or_else(|| "Unnamed Feature".to_string());

        let slug = args
            .feature
            .clone()
            .unwrap_or_else(|| Feature::slug_from_name(&friendly_name));
        Ok((slug, friendly_name, content))
    } else {
        let (friendly_name, spec_content) = run_interview()?;
        let slug = args
            .feature
            .clone()
            .unwrap_or_else(|| Feature::slug_from_name(&friendly_name));
        Ok((slug, friendly_name, spec_content))
    }
}

fn read_line_prompt(msg: &str) -> Result<String> {
    use std::io::Write as _;
    print!("{msg}: ");
    io::stdout().flush()?;
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;
    Ok(line.trim().to_string())
}

fn read_multiline_prompt(msg: &str) -> Result<String> {
    println!("{msg} (enter empty line to finish):");
    let mut lines = Vec::new();
    for line in io::stdin().lock().lines() {
        let l = line?;
        if l.is_empty() {
            break;
        }
        lines.push(l);
    }
    Ok(lines.join("\n"))
}

fn run_interview() -> Result<(String, String)> {
    let name = read_line_prompt("Feature name")?;
    let problem = read_multiline_prompt("What problem does this solve?")?;
    let users = read_line_prompt("Who benefits from this?")?;

    let mut frs = Vec::new();
    let mut fr_idx = 1;
    loop {
        let fr = read_line_prompt(&format!(
            "Functional requirement FR-{fr_idx} (leave empty to stop)"
        ))?;
        if fr.is_empty() {
            break;
        }
        frs.push(fr);
        fr_idx += 1;
    }

    let nfrs = read_multiline_prompt("Non-functional requirements (performance, security, etc.)")?;
    let constraints = read_multiline_prompt("Constraints and dependencies")?;
    let criteria = read_multiline_prompt("Acceptance criteria")?;

    let fr_lines: String = frs
        .iter()
        .enumerate()
        .map(|(i, fr)| format!("- **FR-{}**: {}", i + 1, fr))
        .collect::<Vec<_>>()
        .join("\n");

    let date = Utc::now().format("%Y-%m-%d").to_string();
    let spec_content = format!(
        r"# Specification: {name}
**Slug**: {slug} | **Date**: {date} | **State**: specified

## Problem Statement
{problem}

## Target Users
{users}

## Functional Requirements
{fr_lines}

## Non-Functional Requirements
{nfrs}

## Constraints & Dependencies
{constraints}

## Acceptance Criteria
{criteria}
",
        name = name,
        slug = Feature::slug_from_name(&name),
        date = date,
        problem = problem,
        users = users,
        fr_lines = fr_lines,
        nfrs = nfrs,
        constraints = constraints,
        criteria = criteria,
    );

    Ok((name, spec_content))
}
