# CLI Commands

## Core Pipeline

| Command | Description |
|---------|-------------|
| `agileplus init` | Initialize project with governance files and agent configs |
| `agileplus specify` | Create or revise a feature specification |
| `agileplus research` | Research feasibility (pre- or post-specify) |
| `agileplus plan` | Generate work packages for a feature |
| `agileplus implement` | Execute work packages |
| `agileplus validate` | Check governance compliance |
| `agileplus ship` | Merge WP branches to main |
| `agileplus retrospective` | Generate post-ship report |

## Triage & Queue

| Command | Description |
|---------|-------------|
| `agileplus triage <text>` | Classify input text (bug/feature/idea/task) |
| `agileplus queue add` | Add item to backlog |
| `agileplus queue list` | List backlog items |
| `agileplus queue show <id>` | Show item details |
| `agileplus queue pop` | Pop highest-priority item |

## Triage Flags

| Flag | Description |
|------|-------------|
| `--type <TYPE>` | Override classification (bug, feature, idea, task) |
| `--dry-run` | Classify without adding to backlog |
| `--output <FORMAT>` | Output format: table (default) or json |

## Init Flags

| Flag | Description |
|------|-------------|
| `--path <DIR>` | Project root (default: current directory) |
| `--agents <LIST>` | Comma-separated agent list (claude, cursor, codex, copilot) |
| `--non-interactive` | Skip prompts, use defaults |
| `--force` | Overwrite existing files |
| `--quick` | Lightweight scaffold only |
