---
audience: [developers, agents]
---

# CLI Commands Reference

AgilePlus CLI is the main entry point for the spec-driven development workflow. All commands are async and support structured output formats.

## Global Flags

```bash
agileplus [GLOBAL_OPTIONS] COMMAND [COMMAND_OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `-v, --verbose` | (none) | Increase verbosity: `-v` (debug), `-vv` (trace) |
| `--db <PATH>` | `.agileplus/agileplus.db` | Path to SQLite database |
| `--repo <PATH>` | (current dir) | Path to git repository root |

## Core Workflow Commands

### agileplus specify

Create or revise a feature specification.

```bash
agileplus specify [OPTIONS] <FEATURE_SLUG> [DESCRIPTION]
```

| Argument | Description |
|----------|-------------|
| `FEATURE_SLUG` | Unique identifier (e.g., `001-user-login`) |
| `DESCRIPTION` | Optional natural-language feature description |

| Option | Description |
|--------|-------------|
| `--interactive` | Interactive prompt mode (default if no description) |
| `--agent <AGENT>` | Use specific agent to draft spec (claude, cursor) |
| `--template <PATH>` | Use custom spec template |
| `--no-git` | Don't create git branch |
| `-v, --verbose` | Increase logging verbosity |

Examples:

```bash
# Interactive mode (prompts for details)
agileplus specify 001-login

# Provide description directly
agileplus specify 001-login "Implement user login with OAuth"

# Use specific agent for spec generation
agileplus specify 001-login --agent claude --interactive

# Dry-run (don't commit spec)
agileplus specify 001-login --no-git
```

### agileplus research

Research feasibility or gather context for a feature.

```bash
agileplus research [OPTIONS] <FEATURE_SLUG>
```

| Option | Description |
|--------|-------------|
| `--pre-specify` | Scan codebase before spec created (discovery mode) |
| `--post-specify` | Analyze feasibility after spec (validation mode) |
| `--scope <PATHS>` | Comma-separated files/dirs to scan |
| `--output <FORMAT>` | `markdown` (default), `json`, or `yaml` |

Examples:

```bash
# Pre-specify research (before writing spec)
agileplus research 001-login --pre-specify --scope src/auth

# Post-specify feasibility check
agileplus research 001-login --post-specify

# Generate JSON report
agileplus research 001-login --output json > report.json
```

### agileplus plan

Generate a work breakdown structure (work packages) for a feature.

```bash
agileplus plan [OPTIONS] <FEATURE_SLUG>
```

| Option | Description |
|--------|-------------|
| `--parallelizable` | Allow independent work packages (default: sequential) |
| `--max-packages <N>` | Maximum work packages to generate (default: 8) |
| `--file-scope` | Include file-level scope in each WP |
| `--agent <AGENT>` | Use specific agent for planning |

Examples:

```bash
# Generate sequential work packages
agileplus plan 001-login

# Generate parallelizable packages with file scope
agileplus plan 001-login --parallelizable --file-scope

# Limit to 5 packages
agileplus plan 001-login --max-packages 5
```

### agileplus implement

Dispatch agents to implement work packages.

```bash
agileplus implement [OPTIONS] <FEATURE_SLUG> [WP_FILTERS]
```

| Option | Description |
|--------|-------------|
| `--wp <ID>` | Implement specific work package (e.g., `WP01`, `WP02`) |
| `--agent <AGENT>` | Agent to dispatch (claude-code, codex) |
| `--timeout <SECS>` | Agent session timeout (default: 1800s = 30m) |
| `--skip-review` | Skip PR review cycle (dangerous) |
| `--max-review-cycles <N>` | Max review/fix iterations (default: 5) |
| `--parallel <N>` | Dispatch N agents in parallel (1–3, default: 1) |

Examples:

```bash
# Implement all work packages sequentially
agileplus implement 001-login

# Implement specific work package
agileplus implement 001-login --wp WP01

# Implement with custom agent and timeout
agileplus implement 001-login --agent claude-code --timeout 3600

# Dispatch 2 agents in parallel for independent packages
agileplus implement 001-login --parallel 2
```

### agileplus validate

Check governance compliance before merging.

```bash
agileplus validate [OPTIONS] <FEATURE_SLUG>
```

| Option | Description |
|--------|-------------|
| `--gate <NAME>` | Validate specific gate (specify, plan, implement, review, merge) |
| `--strict` | Fail on warnings (not just errors) |
| `--verbose` | Show detailed violation list |

Examples:

```bash
# Validate all gates for a feature
agileplus validate 001-login

# Validate only review gate
agileplus validate 001-login --gate review

# Strict mode: fail on any violations
agileplus validate 001-login --strict
```

### agileplus ship

Merge all completed work packages to main.

```bash
agileplus ship [OPTIONS] <FEATURE_SLUG>
```

| Option | Description |
|--------|-------------|
| `--auto-resolve` | Automatically resolve simple merge conflicts |
| `--no-verify` | Skip pre-merge hooks |
| `--dry-run` | Show what would be merged without doing it |
| `--target <BRANCH>` | Target branch (default: main) |

Examples:

```bash
# Dry-run: show what would be shipped
agileplus ship 001-login --dry-run

# Ship with auto-conflict resolution
agileplus ship 001-login --auto-resolve

# Ship to develop branch instead of main
agileplus ship 001-login --target develop
```

### agileplus retrospective

Generate a post-ship report.

```bash
agileplus retrospective [OPTIONS] <FEATURE_SLUG>
```

| Option | Description |
|--------|-------------|
| `--output <PATH>` | Save report to file (default: stdout) |
| `--format <FORMAT>` | `markdown` (default), `html`, `json` |
| `--include-metrics` | Include performance/coverage metrics |

Examples:

```bash
# Generate markdown report to file
agileplus retrospective 001-login --output RETROSPECTIVE.md

# Generate HTML report
agileplus retrospective 001-login --format html --output report.html

# Include metrics in report
agileplus retrospective 001-login --include-metrics
```

## Triage & Queue Commands

### agileplus triage

Classify incoming issues/requests.

```bash
agileplus triage [OPTIONS] <TEXT>
```

| Argument | Description |
|----------|-------------|
| `TEXT` | Issue description (quoted string or stdin) |

| Option | Description |
|--------|-------------|
| `--type <TYPE>` | Override auto-classification (bug, feature, idea, task) |
| `--dry-run` | Classify without adding to backlog |
| `--output <FORMAT>` | `table` (default), `json`, `yaml` |
| `--priority <P>` | Set priority: `critical`, `high`, `medium`, `low` |

Examples:

```bash
# Classify from stdin
echo "Users can't log in on mobile" | agileplus triage --type bug

# Direct classification
agileplus triage "Add dark mode support" --type feature

# Dry-run to preview classification
agileplus triage "Fix typo in docs" --dry-run

# JSON output
agileplus triage "Performance is slow" --output json
```

### agileplus queue add

Add item to backlog queue.

```bash
agileplus queue add [OPTIONS] <TEXT>
```

| Option | Description |
|--------|-------------|
| `--type <TYPE>` | Item type (bug, feature, idea, task) |
| `--priority <P>` | Priority: critical, high, medium, low |
| `--tags <TAGS>` | Comma-separated labels |

Examples:

```bash
agileplus queue add "Implement two-factor auth" --type feature --priority high

agileplus queue add "Database migration needed" --type task --priority critical
```

### agileplus queue list

List all backlog items.

```bash
agileplus queue list [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--filter <TYPE>` | Filter by type (bug, feature, idea, task) |
| `--priority <P>` | Filter by priority |
| `--output <FORMAT>` | `table` (default), `json`, `yaml` |
| `--limit <N>` | Show top N items (default: 20) |

Examples:

```bash
# List all items
agileplus queue list

# List only critical bugs
agileplus queue list --filter bug --priority critical

# JSON output
agileplus queue list --output json
```

### agileplus queue show

Show details of a backlog item.

```bash
agileplus queue show <ITEM_ID>
```

### agileplus queue pop

Remove and return highest-priority item.

```bash
agileplus queue pop [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--count <N>` | Pop N items (default: 1) |

Examples:

```bash
# Pop next item for implementation
agileplus queue pop

# Pop next 3 items
agileplus queue pop --count 3
```

## Verbosity & Output

### Log Levels

Set via `-v` flag:

```bash
agileplus specify 001-login              # INFO level
agileplus specify 001-login -v            # DEBUG level
agileplus specify 001-login -vv           # TRACE level
```

### Output Formats

Most commands support:

```bash
--output table    # Human-readable table (default)
--output json     # Machine-readable JSON
--output yaml     # YAML format
--output markdown # Markdown (for reports)
```

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | General error |
| `2` | Invalid arguments |
| `3` | Governance violation (can't proceed) |
| `4` | VCS error (git operation failed) |
| `5` | Agent dispatch error |

## Environment Variables

All commands respect:

```bash
AGILEPLUS_PROJECT     # Override --repo path
AGILEPLUS_CONFIG      # Override config file location
AGILEPLUS_LOG_LEVEL   # Override -v verbosity
AGILEPLUS_AGENT       # Default agent to use
```
