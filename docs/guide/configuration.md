---
audience: [developers]
---

# Configuration Guide

Configure AgilePlus for your team and project. Configuration is stored in `.kittify/config.toml` and can be overridden with environment variables or CLI flags.

## Project Configuration

`agileplus init` generates `.kittify/config.toml` with auto-detected settings. Edit this file to customize behavior.

### Basic Project Settings

```toml
[project]
name = "my-project"
type = "brownfield"           # or "greenfield"
root = "."
version = "0.1.0"
created_at = "2026-03-01T00:00:00Z"

[detection]
languages = ["rust", "typescript"]
frameworks = ["axum", "vitepress"]
has_tests = true
has_ci = true
```

### VCS Configuration

```toml
[vcs]
provider = "git"              # Currently only git supported
default_branch = "main"       # Usually "main" or "master"

[vcs.worktrees]
prefix = ".worktrees"         # Directory for isolated work
branch_prefix = "feat/"       # Branch naming: feat/001-feature
```

### Agent Configuration

```toml
[agents]
enabled = ["claude", "cursor"]  # Which agents are configured
dispatch_strategy = "round-robin"

[agents.claude]
commands_dir = ".claude/commands"

[agents.cursor]
rules_dir = ".cursor/rules"
```

### Code Conventions

```toml
[conventions]
languages = ["rust", "typescript"]
frameworks = ["axum", "vitepress"]
test_dir = "tests"
source_dir = "src"
build_system = "cargo"

[conventions.commit]
style = "conventional"        # or "semantic"
max_line_length = 100
require_scope = true          # Require type(scope): message

[conventions.pr]
max_files_changed = 50
require_tests = true
require_description = true
```

### Feature Specification Settings

```toml
[spec]
min_scenarios = 3             # Minimum user scenarios per spec
min_requirements = 5          # Minimum requirements
min_success_criteria = 2      # Minimum success criteria

[spec.interviews]
strategy = "guided"           # or "freeform"
max_follow_ups = 5            # Max clarification questions
```

### Workflow Settings

```toml
[workflow]
max_files_per_wp = 10         # Max files modified per work package
max_subtasks_per_wp = 15      # Max subtasks per WP
allow_parallel_wps = true     # Allow multiple WPs to run simultaneously

[workflow.merge]
strategy = "squash"           # or "rebase" or "merge"
delete_branches = true        # Delete branches after merge
delete_worktrees = true       # Clean up worktrees after merge
```

## Advanced Configuration

### Tracker Integration

Connect to Plane.so for issue tracking:

```toml
[sync.plane]
enabled = true
api_url = "https://api.plane.so"
workspace = "my-org"
project = "my-project"
api_key = "${PLANE_API_KEY}"

[sync.plane.mapping]
spec_label = "spec"
wp_label = "work-package"
review_label = "in-review"
```

Connect to GitHub Issues:

```toml
[sync.github]
enabled = true
repo = "username/my-project"
token = "${GITHUB_TOKEN}"

[sync.github.mapping]
labels = {
  spec = "spec",
  wp = "work-package",
  ready = "ready-to-implement"
}

[sync.github.pr]
auto_create = true
draft = false
request_review = true
```

Connect to GitHub Projects (v2):

```toml
[sync.github.projects]
enabled = true
project_id = "PVT_1234567890"
status_field = "Status"
```

### Sync Behavior

```toml
[sync]
# How often to sync (minutes)
interval = 5

# Conflict resolution
conflict_strategy = "local-wins"  # or "remote-wins"

# Sync direction
direction = "bidirectional"        # or "push-only" or "pull-only"

# What to sync
sync_labels = true
sync_milestones = true
sync_assignees = true
sync_comments = false             # Don't sync comments
```

### Database Settings

```toml
[database]
type = "sqlite"
path = ".kittify/agileplus.db"

# Backup settings
auto_backup = true
backup_frequency = "daily"        # or "weekly", "never"
backup_retention = 7              # Days to keep backups
```

### Feature Flags

```toml
[features]
enable_ai_features = true         # AI-powered research, planning
enable_governance_checks = true   # Enforce governance
enable_metrics = true             # Collect metrics
enable_audit_logging = true       # Full audit trail
```

## CLI Flags

Global flags available on all commands. These override config file settings:

```bash
agileplus [COMMAND] [FLAGS]
```

### Verbosity

```bash
-v, --verbose                 # Single verbosity level
-vv                           # Double verbosity
-vvv                          # Triple verbosity (debug)

Example:
agileplus specify "title" -vv  # Show more details
```

### Database

```bash
--db <PATH>                   # Custom database path
# Default: .kittify/agileplus.db

Example:
agileplus list --db /tmp/agileplus.db
```

### Repository

```bash
--repo <PATH>                 # Repository root (if not current dir)
# Default: current directory

Example:
agileplus list --repo /path/to/project
```

### Output Format

```bash
--output <FORMAT>             # Output format
# Options: text, json, csv, markdown

Example:
agileplus queue list --output json
```

### Color Output

```bash
--color <WHEN>                # When to use colors
# Options: auto (default), always, never

Example:
agileplus list --color never
```

### Non-Interactive

```bash
--non-interactive             # No prompts, use defaults

Example:
agileplus init --non-interactive
```

## Environment Variables

Override settings via environment variables:

```bash
# Database
export AGILEPLUS_DB=/path/to/db
export AGILEPLUS_REPO=/path/to/repo

# Tracker Integration
export PLANE_API_KEY=your-api-key-here
export PLANE_WORKSPACE=my-org
export GITHUB_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxx
export GITHUB_REPO=username/repo

# Debug
export AGILEPLUS_LOG=debug    # debug, info, warn, error
export AGILEPLUS_LOG_FILE=/tmp/agileplus.log
```

**Precedence** (highest to lowest):

1. CLI flags
2. Environment variables
3. `.kittify/config.toml`
4. Default values

## Configuration Examples

### Minimal Setup (For Solo Developers)

```toml
[project]
name = "my-app"
type = "brownfield"

[vcs]
provider = "git"
default_branch = "main"

[agents]
enabled = []

[workflow]
allow_parallel_wps = false  # Simpler for one person
```

### Team Setup (With Plane.so)

```toml
[project]
name = "team-app"
type = "brownfield"

[vcs]
provider = "git"

[agents]
enabled = ["claude", "cursor"]

[sync.plane]
enabled = true
workspace = "my-org"
project = "team-app"
api_key = "${PLANE_API_KEY}"

[sync]
interval = 5
conflict_strategy = "local-wins"

[workflow]
allow_parallel_wps = true
max_files_per_wp = 20

[conventions.commit]
style = "conventional"
require_scope = true
```

### CI/CD Integration (GitHub Actions)

```toml
[project]
name = "github-app"

[sync.github]
enabled = true
repo = "org/my-app"
token = "${GITHUB_TOKEN}"

[sync.github.pr]
auto_create = true
draft = false
request_review = true

[features]
enable_governance_checks = true
enable_audit_logging = true
```

### Multi-Language Monorepo

```toml
[project]
name = "monorepo"
type = "brownfield"

[detection]
languages = ["rust", "typescript", "python"]
frameworks = ["axum", "react", "fastapi"]

[conventions]
test_dir = "tests"
source_dir = "packages"

[workspace]
type = "mono"
roots = ["packages/api", "packages/web", "packages/sdk"]
```

## Viewing Current Config

```bash
# Show effective configuration (merged: file + env + CLI)
agileplus config show

# Show only file configuration
agileplus config show --file

# Show only environment variables
agileplus config show --env

# Show in JSON format
agileplus config show --output json
```

## Validating Configuration

```bash
# Validate config file syntax
agileplus config validate

# Check all required settings are present
agileplus config check

# Full diagnostic
agileplus config diagnose
```

## Updating Configuration

```bash
# Interactive editor
agileplus config edit

# Set a specific value
agileplus config set project.name "new-name"
agileplus config set agents.enabled "['claude']"

# Unset a value (revert to default)
agileplus config unset agents.enabled
```

## Configuration Reference

See [Reference: Environment Variables](/reference/env-vars) for complete list of all environment variables.

See [Reference: CLI](/reference/cli) for complete list of all CLI flags.

## Troubleshooting

**Config not being read?**

```bash
# Check where config is being loaded from
agileplus config show --verbose

# Show actual file contents
cat .kittify/config.toml
```

**Settings not taking effect?**

Remember the precedence:

```
CLI flags > Environment > Config file > Defaults
```

Check that your setting isn't being overridden by a higher-precedence source.

**Invalid config format?**

```bash
# Validate TOML syntax
agileplus config validate
```

**Reset to defaults?**

```bash
# Backup current config
cp .kittify/config.toml .kittify/config.toml.bak

# Re-init to regenerate
agileplus init --force
```

## What's Next

- **[Guide: Getting Started](/guide/getting-started)** — Complete walkthrough
- **[Guide: Initialization](/guide/init)** — Project setup details
- **[Reference: Environment Variables](/reference/env-vars)** — All env var options
- **[Reference: CLI](/reference/cli)** — All command line flags
