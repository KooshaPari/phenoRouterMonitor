---
audience: [developers]
---

# Project Initialization

`agileplus init` bootstraps a project with governance files, agent configurations, git hooks, and directory structure.

## Quick Usage

```bash
# Initialize current directory
agileplus init

# Initialize specific directory
agileplus init --path ./my-project

# Quick mode (skip framework detection)
agileplus init --quick
```

## What It Does

`init` performs these steps:

### 1. Project Type Detection

Scans your directory to determine project type:

```
Scanning project...
✓ Found 12 source files
✓ Detected brownfield project
✓ Detected languages: Rust, TypeScript
✓ Detected frameworks: Axum, Vitepress
```

| Type | Criterion | Behavior |
|------|-----------|----------|
| **Brownfield** | 5+ source files | Scans for languages, frameworks, conventions |
| **Greenfield** | < 5 files | Creates minimal scaffold |
| **Monorepo** | Multiple workspace roots | Detects workspace structure |

### 2. Language & Framework Detection

Init analyzes your codebase to detect:

**Languages** (by file extension):

```
.rs      → Rust
.ts/.tsx → TypeScript
.py      → Python
.go      → Go
.java    → Java
.js/jsx  → JavaScript
.c/.cpp  → C/C++
.rb      → Ruby
```

**Frameworks** (by config files):

```
Cargo.toml          → Rust (with dep analysis)
package.json        → Node.js (with scripts analysis)
Pipfile/setup.py    → Python
go.mod              → Go
pom.xml             → Java (Maven)
requirements.txt    → Python
docker-compose.yml  → Docker (Compose)
.github/workflows/  → GitHub Actions (CI/CD)
```

**Conventions** (by directory structure):

```
src/                → Source code directory
tests/              → Test directory
specs/              → Specification directory
docs/               → Documentation
.eslintrc           → Linter config detected
.github/            → GitHub Actions
Makefile            → Build system
```

### 3. Generated Files & Directories

**Core files created:**

```
.kittify/
├── config.toml           # Project configuration
├── metadata.yaml         # Version, timestamp, platform info
└── .gitignore            # Git ignore patterns

kitty-specs/             # Specification directory (empty initially)

CLAUDE.md                # Governance for Claude Code
AGENTS.md                # Cross-agent coordination rules
.claudeignore            # Context optimization for AI assistants
.git/hooks/pre-commit    # Encoding validation hook
```

**Detailed file contents:**

#### `.kittify/config.toml`

```toml
[project]
name = "my-project"
type = "brownfield"
root = "."
version = "0.1.0"
created_at = "2026-03-01T10:30:00Z"

[detection]
languages = ["rust", "typescript"]
frameworks = ["axum", "vitepress"]
has_tests = true
has_ci = true

[vcs]
provider = "git"
default_branch = "main"

[conventions]
test_dir = "tests"
source_dir = "src"
build_system = "cargo"

[agents]
enabled = ["claude"]

# Optional: Tracker integration
# [sync.plane]
# workspace = "my-org"
# project = "my-project"
# api_key = "${PLANE_API_KEY}"

# [sync.github]
# repo = "user/repo"
# token = "${GITHUB_TOKEN}"
```

#### `CLAUDE.md` (Example)

```markdown
# Project: my-project

## Conventions

- Language: Rust 2024 edition
- Framework: Axum web framework
- Testing: cargo test
- Build: cargo build

## Governance

- Specifications: kitty-specs/ directory
- Work packages: spec-driven implementation
- Code review: feature branch review required
- Merge strategy: squash merge to main

## Domains

- API Server: src/api/
- Database: src/db/
- Models: src/models/
```

#### `AGENTS.md` (Example)

```markdown
# Agent Governance

## Claude Code

- Tool: Claude Code IDE integration
- Role: Primary implementation agent
- Scope: All features, bug fixes, refactoring
- Constraints: Follow CLAUDE.md conventions

## Code Style

- Use project linter configs (.clippy.toml, .eslintrc)
- Follow existing code patterns
- Run tests before committing

## Communication

- Use task system for work assignment
- Commit messages: follow conventional commits
- PR descriptions: reference spec and work packages
```

#### `.claudeignore` (Example)

```
# Large files that should be excluded from AI context
*.pem
*.key
*.env
node_modules/
target/
.git/
dist/
build/

# Large log files
*.log

# Generated files (context bloat)
*.d.ts
*.lock (optional, context-heavy)
```

#### Pre-commit Hook

Validates file encoding before commits:

```bash
#!/usr/bin/env bash
# .git/hooks/pre-commit

# Check file encoding
for file in $(git diff --cached --name-only); do
    if file -i "$file" | grep -q "charset=iso-8859"; then
        echo "Error: $file has invalid encoding (ISO-8859). Please use UTF-8."
        exit 1
    fi
done
exit 0
```

### 4. Optional: Agent Configuration

If you specify `--agents`, init creates agent-specific configs:

```bash
agileplus init --agents claude,cursor
```

**Claude Code:**

```
.claude/
├── commands/
│   ├── specify.md
│   ├── plan.md
│   ├── review.md
│   ├── test.md
│   ├── document.md
│   ├── refactor.md
│   └── debug.md
```

**Cursor:**

```
.cursor/
└── rules/
    ├── typescript.md
    ├── testing.md
    └── architecture.md

.cursorrules          # Main cursor rules file
```

### 5. Directory Structure Created

```
my-project/
├── .kittify/
│   ├── config.toml
│   ├── metadata.yaml
│   └── .gitignore
├── kitty-specs/                # Specifications (empty)
├── .git/
│   └── hooks/
│       └── pre-commit          # File encoding check
├── CLAUDE.md                   # Governance
├── AGENTS.md                   # Agent coordination
├── .claudeignore               # Context optimization
├── src/                        # (existing)
├── tests/                      # (existing)
├── Cargo.toml                  # (existing)
└── ...
```

## Usage Examples

### Standard Initialization

```bash
# Initialize current directory (interactive)
agileplus init
```

Output:

```
Scanning project...
✓ Detected brownfield project
✓ Found: Rust (src/), TypeScript (ui/)
✓ Frameworks: Axum, React
✓ Build system: Cargo

Select agents to configure: [C]laude, [Cu]rsor, [Co]pilot? > C
Creating config files...
✓ .kittify/config.toml
✓ CLAUDE.md
✓ AGENTS.md
✓ .claude/commands/ (7 commands)

Project initialized!

Next steps:
1. Review .kittify/config.toml
2. Review CLAUDE.md governance
3. agileplus specify "your first feature"
```

### Non-Interactive Mode

```bash
agileplus init --non-interactive
# Uses defaults without prompts
```

### Quick Mode (Minimal)

```bash
agileplus init --quick
# Skips framework detection, creates only essentials
```

Output:

```
✓ .kittify/config.toml (minimal)
✓ CLAUDE.md (stub)
✓ AGENTS.md
✓ kitty-specs/

Quick setup complete. Minimal configuration created.
```

### Specific Path

```bash
agileplus init --path ./backend
# Initializes ./backend/ instead of current directory
```

### Force Overwrite

```bash
agileplus init --force
# Overwrites existing config files
```

### Select Agents

```bash
agileplus init --agents claude,cursor,copilot
# Configure multiple agents
```

## Customizing Configuration After Init

After initialization, edit `.kittify/config.toml` to customize:

### Add Tracker Integration

```toml
[sync.plane]
workspace = "my-org"
project = "my-project"
api_key = "${PLANE_API_KEY}"

[sync.github]
repo = "username/repo"
token = "${GITHUB_TOKEN}"
```

### Add Custom Conventions

```toml
[conventions]
commit_style = "conventional"
pr_prefix = "feat|fix|docs|test"
max_files_per_wp = 10
```

### Configure Test Directories

```toml
[test_paths]
unit = ["tests/unit/", "src/**/test.rs"]
integration = ["tests/integration/"]
e2e = ["tests/e2e/"]
```

## Verification

After init, verify setup:

```bash
# Check generated files
ls -la .kittify/
ls -la kitty-specs/

# View config
cat .kittify/config.toml

# Verify git hook
cat .git/hooks/pre-commit

# Test with first spec
agileplus specify "Test feature"
ls kitty-specs/001-test-feature/
```

## Troubleshooting

### Issue: Permission Denied on Pre-commit Hook

```bash
chmod +x .git/hooks/pre-commit
```

### Issue: Init Detects Wrong Language

```bash
# Override in config after init
cat .kittify/config.toml
# Edit manually to fix detected languages
```

### Issue: Want to Re-initialize

```bash
agileplus init --force
# Overwrites all generated files with fresh versions
```

## Key Flags Reference

```bash
agileplus init                          # Standard initialization
agileplus init --path ./my-project      # Custom path
agileplus init --agents claude,cursor   # Select agents
agileplus init --non-interactive        # No prompts
agileplus init --quick                  # Minimal setup
agileplus init --force                  # Overwrite existing
```

## What's Next

- **[Getting Started](/guide/getting-started)** — Full installation and first feature walkthrough
- **[Configuration](/guide/configuration)** — All config options explained
- **[Quick Start](/guide/quick-start)** — 5-minute path to your first spec
- **[Core Workflow](/guide/workflow)** — Understand the spec-driven pipeline
