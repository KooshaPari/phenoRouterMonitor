---
audience: [developers]
---

# Project Setup

`agileplus init` bootstraps a project with governance files, agent configs, and git hooks.

## Usage

```bash
agileplus init
```

## What It Does

### 1. Project Detection

Init scans your project root to determine the project type:

- **Brownfield** — 5 or more source files detected. Init scans for languages and frameworks.
- **Greenfield** — Fewer than 5 source files. Init creates a minimal scaffold.

### 2. Language & Framework Scan

For brownfield projects, init detects:

| Detection | Method |
|-----------|--------|
| Languages | File extension counting (`.rs`, `.ts`, `.py`, etc.) |
| Frameworks | Config file markers (`Cargo.toml` → Rust, `package.json` → Node, etc.) |
| Conventions | Existing linter configs, test directories, CI files |

### 3. Generated Files

```
.agileplus/
├── config.toml        # Project configuration
└── metadata.toml      # Version, timestamp, platform

CLAUDE.md              # Claude Code governance (project-aware)
AGENTS.md              # Cross-agent governance rules
.claudeignore          # Context optimization for AI assistants
.git/hooks/pre-commit  # Encoding check hook
```

### 4. Agent Configs

Based on `--agents` selection:

| Agent | Generated |
|-------|-----------|
| Claude | `.claude/commands/` (7 slash commands) |
| Cursor | `.cursorrules`, `.cursor/rules/` |
| Codex | `.codex/` config |
| Copilot | `.github/prompts/` |

## Flags

```bash
agileplus init --path ./my-project     # Non-default root
agileplus init --agents claude,cursor  # Select agents
agileplus init --non-interactive       # Skip prompts
agileplus init --force                 # Overwrite existing
agileplus init --quick                 # Lightweight scaffold only
```

## Quick Mode

`--quick` skips framework detection and generates only the essentials:
- `.agileplus/config.toml`
- `.gitignore` block
- `CLAUDE.md` stub
