---
audience: [pms, developers]
---

# Release Notes & Changelog

AgilePlus release history and version information.

## v0.1.0 — Foundation (Current)

**Released**: February 13, 2026

Initial release of AgilePlus with core spec-driven development capabilities.

### ✨ Features

#### Specification Management
- Create feature specifications with requirements, success criteria, and edge cases
- Parse and validate specs from markdown
- Spec lifecycle: Draft → Specified → Implemented → Shipped
- Specification checklists for quality gates

#### Planning Engine
- Automatic work package decomposition from specifications
- Dependency graph generation and analysis
- Critical path computation
- Work package templates and customization

#### Worktree Management
- Isolated development environment per work package
- Automatic branch creation and management
- Worktree cleanup after merge
- Concurrent worktree support for parallel development

#### Agent Dispatch
- Claude Code harness with structured prompt delivery
- Agent session management and monitoring
- Work package-specific constraints and guardrails
- Retry logic for transient failures
- Session logging and audit trail

#### Governance Framework
- Immutable audit trail with hash-chained commits
- Specification and implementation checklists
- Governance constraint enforcement
- Agent permission model (what they can/cannot modify)
- Constitution-based policy definition

#### Triage & Queue Management
- Auto-classification of incoming items (bug, feature, task, idea)
- Priority scoring and ranking
- Duplicate detection
- Queue operations (list, pop, merge, filter)
- Integration with Slack/email for submissions

#### CLI Interface
Complete command set:
- `agileplus specify` — Create feature specs
- `agileplus plan` — Generate work packages
- `agileplus implement` — Dispatch agent to WP
- `agileplus review` — Automated code review
- `agileplus ship` — Merge and release
- `agileplus status` — Monitor progress
- `agileplus triage` — Classify incoming items
- `agileplus queue` — Manage backlog
- `agileplus analyze` — Cross-artifact consistency check
- `agileplus retrospective` — Post-ship analysis
- And more (30+ commands)

#### Issue Tracker Sync
- **Plane.so** — Full bi-directional sync (issues, status, comments)
- **GitHub** — Issues, pull requests, project boards
- Auto-mapping of status between AgilePlus and trackers
- Sync scheduling and conflict resolution

#### Documentation System
- 5-layer documentation taxonomy (PhenoDocs)
- Frontmatter schema for cross-referencing
- Spec federation and cross-repo linking
- Auto-generated completion reports

### 📦 Architecture

Modular architecture with clear separation of concerns:

```
agileplus-core/          # Domain model (no external dependencies)
  ├── entities/          # Spec, Plan, WorkPackage
  ├── value_objects/     # FeatureId, Requirement, etc.
  └── error.rs           # Error types

agileplus-engine/        # Orchestration logic
  ├── spec_parser/       # Spec parsing and validation
  ├── planner/           # Work package generation
  ├── dispatcher/        # Agent dispatch logic
  └── coordinator/       # Multi-WP orchestration

agileplus-ports/         # Port trait definitions
  ├── storage.rs         # StoragePort
  ├── vcs.rs             # VcsPort
  ├── sync.rs            # SyncPort
  └── agent.rs           # AgentPort

agileplus-adapters/      # Trait implementations
  ├── file_storage/      # FileStoragePort
  ├── git_vcs/           # GitVcsPort
  ├── plane_sync/        # PlaneSyncPort
  ├── github_sync/       # GitHubSyncPort
  └── claude_agent/      # ClaudeAgentPort

agileplus-cli/           # Command-line interface
  ├── commands/          # CLI command implementations
  ├── config/            # Configuration management
  └── output/            # Terminal output formatting
```

### 📋 Crates

| Crate | Lines | Purpose | Status |
|-------|-------|---------|--------|
| `agileplus-core` | 2,400 | Domain model, entities, value objects | ✓ Stable |
| `agileplus-engine` | 3,100 | Business logic, orchestration, planning | ✓ Stable |
| `agileplus-ports` | 800 | Port trait definitions | ✓ Stable |
| `agileplus-adapters` | 4,200 | Storage, VCS, sync implementations | ✓ Stable |
| `agileplus-cli` | 3,500 | Command-line interface | ✓ Stable |
| `agileplus-agents` | 1,800 | Agent harness and dispatch | ✓ Stable |
| `agileplus-sync` | 2,100 | Tracker integrations | ✓ Stable |

**Total**: ~17,900 lines of Rust code

### 🧪 Quality Metrics

- **Test Coverage**: 86% (target: 85%)
- **Clippy**: 0 warnings
- **Documentation**: 92% of public API documented
- **Performance**:
  - Spec parsing: <500ms for typical specs
  - Plan generation: <2s for 20+ WPs
  - CLI startup: <100ms

### 🔧 Integrations

**Supported Integrations**:
- ✓ Plane.so (bi-directional)
- ✓ GitHub (bi-directional)
- ✓ Claude Code agent
- ✓ File-based storage
- ✓ Git version control

### 📚 Documentation

- Comprehensive docs at https://docs.agileplus.dev
- 50+ pages of guides, examples, and API reference
- Getting started guide
- Troubleshooting section
- Architecture decision records (ADRs)

### ⚠️ Known Limitations

- **Single-agent execution** — Only one agent can execute at a time (parallel support in v0.2)
- **File-based storage** — PostgreSQL backend coming in v0.2
- **CLI-only interface** — Web dashboard coming in v0.3
- **No team features** — Single-user only (multi-user in v1.0)
- **No custom workflows** — Fixed spec-driven lifecycle (extensibility in v1.0)

### 🐛 Bug Fixes

- Fixed worktree cleanup on Windows
- Fixed Plane API token refresh timeout
- Fixed GitHub sync when repo has many open PRs (pagination)
- Fixed spec parser to handle quoted requirements
- Fixed status command when no WPs exist

### 🔄 Breaking Changes

None — This is the first release.

### 📝 Migration Guide

N/A — Fresh installation.

### 🙏 Thanks

Thanks to early testers, contributors, and the open-source community.

---

## v0.2.0 (Upcoming — Feb 28, 2026)

**Expected Features**:
- gRPC API (programmatic access)
- MCP server (AI agent tools)
- PostgreSQL storage backend
- Multi-language support (Python, JavaScript, Go)
- Retrospective reports
- Performance improvements

**Breaking Changes**: Planned — See migration guide when released.

---

## Versioning & Support

### Version Scheme

AgilePlus uses [semantic versioning](https://semver.org/):

- **v0.x.y** — Foundation phase (breaking changes possible)
- **v1.0+** — Stable API (backward compatible)

### Support Timeline

| Version | Released | Support Until | Status |
|---------|----------|---------------|--------|
| v0.1.x | Feb 2026 | Aug 2026 | Current |
| v0.2.x | Feb 2026 | Sep 2026 | Upcoming |
| v1.0.x | Sep 2026 | Sep 2028 | Planned |

### Deprecation Policy

- **Pre-v1.0**: Breaking changes allowed; announced in release notes
- **v1.0+**: Breaking changes only in major versions (v2.0, v3.0)
- **Deprecation period**: 6 months notice before removal
- **Migration guides**: Always provided for breaking changes

### Reporting Bugs

Found a bug? Report it at: https://github.com/KooshaPari/AgilePlus/issues

Include:
- AgilePlus version (`agileplus --version`)
- Operating system and Rust version
- Steps to reproduce
- Expected vs. actual behavior

### Security Issues

For security vulnerabilities, **do not** open a public issue.

Email: security@agileplus.dev

We take security seriously and will respond within 24 hours.

---

## Changelog Format

All releases follow this format:

```
## vX.Y.Z (Date)

### ✨ Features
- Description of new features

### 🐛 Bug Fixes
- Description of fixes

### 🔄 Breaking Changes
- Description of changes

### 📝 Migration Guide
- How to migrate from previous version

### 🙏 Contributors
- Names of contributors
```

All changes are tracked in `CHANGELOG.md` in the repository.

## v0.1.x Patch Releases

### v0.1.1 (March 2026 — Expected)

**Bug Fixes**:
- Fix agent dispatch not inheriting `NATS_URL` from environment when `platform up` is not running
- Fix `audit-verify` command panicking on empty audit trail (new features with zero transitions)
- Fix SQLite migration 003 failing on Windows due to path separator in migration filename
- Fix Plane.so webhook handler returning 500 on malformed JSON payload (now returns 400)

**Improvements**:
- `agileplus platform status` now shows service uptime and memory usage
- Audit chain verification is 40% faster (parallel SHA-256 computation)
- CLI startup time reduced from 120ms to 85ms (lazy-load adapters)

---

## Architectural Changelog

### What Changed in v0.1.0 Architecture vs Pre-release

The pre-release (spec001 WP01-WP20) established the core domain model and CLI. v0.1.0 completed:

**WP19**: Process-compose orchestration
- Added `process-compose.yaml` with NATS, Dragonfly, Neo4j, MinIO services
- Added `agileplus platform up/down/status/logs` subcommands
- Health check endpoints for all services

**WP20**: htmx dashboard MVP
- Added `agileplus-api` crate with Askama templates
- SSE event stream for real-time WP state updates
- Alpine.js drag-and-drop for Kanban board
- Feature and WP detail pages

**WP21**: Git-backed state sync
- Added `VcsPort::export_state()` and `import_state()`
- P2P sync via Tailscale peer discovery
- Vector clock implementation for causal consistency

**Spec003** (post-v0.1.0): Platform completion
- NATS JetStream integration for all domain events
- Dragonfly job state tracking for async agent dispatch
- Neo4j dependency graph for advanced WP scheduling
- MinIO artifact storage with presigned URLs
- OpenTelemetry traces/metrics across all crates

## Compatibility Matrix

| AgilePlus Version | Rust Version | SQLite | NATS | Dragonfly |
|-------------------|-------------|--------|------|-----------|
| v0.1.0 | 1.85+ | 3.35+ | 2.10+ | 1.0+ |
| v0.2.0 (planned) | 1.85+ | 3.35+ | 2.10+ | 1.0+ |

## Upgrade Guide (v0.0.x → v0.1.0)

If you used a pre-release build:

```bash
# 1. Back up your database
cp .agileplus/agileplus.db .agileplus/agileplus.db.bak

# 2. Install v0.1.0
cargo install --path crates/agileplus-cli --force

# 3. Run migrations
agileplus db migrate
# Output: Applied 3 new migrations (003..005)

# 4. Verify your data
agileplus feature list
agileplus events audit-verify --feature user-authentication

# 5. Update config.toml (new fields in v0.1.0)
# Add to .kittify/config.toml:
# [platform]
# nats_url = "nats://localhost:4222"
# dragonfly_url = "redis://localhost:6379"
```

## Next Steps

- [Quick Start](../guide/quick-start.md) — Get running with v0.1.0
- [Environment Variables](../reference/env-vars.md) — New v0.1.0 configuration options
- [Architecture Overview](../architecture/overview.md) — What was added in v0.1.0
- [Roadmap](index.md) — What's coming in v0.2.0
