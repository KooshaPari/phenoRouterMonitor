//! Phenotype MCP Prompts Module
//!
//! Provides prompt templates for MCP server.

use fastmcp::Prompt;

/// Available prompts
pub const PROMPTS: &[(&str, &str, &[&str])] = &[
    (
        "feature_spec_template",
        "Template for creating feature specifications",
        &["title", "description", "requirements"],
    ),
    (
        "code_review_template",
        "Template for code review requests",
        &["repo", "pr_number", "focus_areas"],
    ),
    (
        "agent_task_template",
        "Template for agent task dispatch",
        &["agent_type", "task", "context"],
    ),
];

/// Get a prompt by name
pub async fn get_prompt(name: &str, arguments: &[(&str, &str)]) -> Option<String> {
    match name {
        "feature_spec_template" => Some(get_feature_spec_template(arguments)),
        "code_review_template" => Some(get_code_review_template(arguments)),
        "agent_task_template" => Some(get_agent_task_template(arguments)),
        _ => None,
    }
}

fn get_feature_spec_template(args: &[(&str, &str)]) -> String {
    let title = args.iter().find(|(k, _)| *k == "title").map(|(_, v)| v.as_str()).unwrap_or("[Feature Title]");
    let description = args.iter().find(|(k, _)| *k == "description").map(|(_, v)| v.as_str()).unwrap_or("[Description]");
    let requirements = args.iter().find(|(k, _)| *k == "requirements").map(|(_, v)| v.as_str()).unwrap_or("[Requirements]");

    format!(
        r#"# Feature Specification: {title}

## Description

{description}

## Requirements

{requirements}

## Acceptance Criteria

- [ ] Criterion 1
- [ ] Criterion 2
- [ ] Criterion 3

## Metadata

- **Priority**: [low/medium/high/critical]
- **Estimate**: [story points]
- **Labels**: [relevant labels]
"#
    )
}

fn get_code_review_template(args: &[(&str, &str)]) -> String {
    let repo = args.iter().find(|(k, _)| *k == "repo").map(|(_, v)| v.as_str()).unwrap_or("[Repository]");
    let pr = args.iter().find(|(k, _)| *k == "pr_number").map(|(_, v)| v.as_str()).unwrap_or("[PR #]");
    let focus = args.iter().find(|(k, _)| *k == "focus_areas").map(|(_, v)| v.as_str()).unwrap_or("[Focus Areas]");

    format!(
        r#"# Code Review Request: {repo} PR #{pr}

## Focus Areas

{focus}

## Review Checklist

### Correctness
- [ ] Logic is correct
- [ ] Edge cases handled
- [ ] Error handling appropriate

### Code Quality
- [ ] Follows style guidelines
- [ ] No code smells
- [ ] Adequate comments

### Testing
- [ ] Tests cover main paths
- [ ] Edge cases tested
- [ ] No regressions

### Security
- [ ] No security vulnerabilities
- [ ] Input validation
- [ ] Authentication/authorization correct
"#
    )
}

fn get_agent_task_template(args: &[(&str, &str)]) -> String {
    let agent_type = args.iter().find(|(k, _)| *k == "agent_type").map(|(_, v)| v.as_str()).unwrap_or("[Agent Type]");
    let task = args.iter().find(|(k, _)| *k == "task").map(|(_, v)| v.as_str()).unwrap_or("[Task Description]");
    let context = args.iter().find(|(k, _)| *k == "context").map(|(_, v)| v.as_str()).unwrap_or("[Context]");

    format!(
        r#"# Agent Task: {agent_type}

## Task

{task}

## Context

{context}

## Expected Output

[Describe what success looks like]

## Constraints

- [Any constraints or requirements]

## Priority

[low/normal/high/urgent]
"#
    )
}
