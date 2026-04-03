---
audience: [pms, developers]
---

# Roadmap

Public roadmap of AgilePlus development. Current focus is on Foundation phase completion, followed by Ecosystem expansion.

## Current Phase: Foundation (Q1 2026)

Building the core engine and essential integrations for spec-driven development.

### Completed ✓

| Component | Status | Details |
|-----------|--------|---------|
| **Spec Engine** | ✓ Done | Parse, validate, and manage feature specifications |
| **Plan Generation** | ✓ Done | Decompose specs into work packages with dependency graphs |
| **Worktree Management** | ✓ Done | Isolated development environments per work package |
| **Agent Dispatch** | ✓ Done | Claude Code harness with prompt construction |
| **Governance Framework** | ✓ Done | Audit trail, hash-chained commits, constraint enforcement |
| **Triage & Queue** | ✓ Done | Priority queue, auto-classification, backlog management |
| **CLI** | ✓ Done | Complete command set (status, implement, review, ship, etc.) |
| **Plane.so Sync** | ✓ Done | Bi-directional issue synchronization |
| **GitHub Sync** | ✓ Done | Issues, pull requests, project boards |

### In Progress 🔄

| Component | Status | ETA | Details |
|-----------|--------|-----|---------|
| **gRPC API** | In Progress | Feb 2026 | Programmatic access to all CLI operations |
| **MCP Server** | In Progress | Feb 2026 | Tool interface for AI agents (Claude, ChatGPT) |

### Planned for Foundation 📅

| Component | Status | ETA | Details |
|-----------|--------|-----|---------|
| **Multi-language Support** | Planned | Mar 2026 | Python, JavaScript/TypeScript, Go adapters |
| **Database Storage** | Planned | Mar 2026 | PostgreSQL, SQLite backend options |
| **Retrospective Reports** | Planned | Mar 2026 | Structured post-ship analysis and metrics |

## Next Phase: Ecosystem (Q2–Q3 2026)

Expanding integrations, improving developer experience, and adding team capabilities.

### Ecosystem Priorities

#### 1. Web Dashboard (Q2)
- **Interactive kanban board** — drag-and-drop WP lanes
- **Spec editor** — visual requirements/success criteria editor
- **Timeline visualization** — Gantt charts and dependency graphs
- **Real-time updates** — WebSocket-based live status

#### 2. Multi-Agent Orchestration (Q2)
- **Parallel WP execution** — coordinate multiple agents per feature
- **Agent load balancing** — distribute work based on availability
- **Dependency resolution** — unblock WPs automatically
- **Agent performance metrics** — track effectiveness per harness

#### 3. Plugin System (Q2–Q3)
- **Custom storage backends** — user-supplied database adapters
- **Custom VCS providers** — Mercurial, Perforce, etc.
- **Custom agent harnesses** — proprietary or self-hosted models
- **Plugin marketplace** — discover and install community plugins

#### 4. Analytics & Metrics (Q2–Q3)
- **Cycle time dashboard** — track Spec → Ship duration trends
- **Velocity metrics** — WPs/day, feature throughput
- **Quality metrics** — test coverage, rework rate, bug trends
- **Agent performance** — effectiveness, pass rate per harness
- **Team insights** — utilization, bottlenecks, growth areas

#### 5. IDE Integration (Q3)
- **VSCode extension** — spec editor, plan visualization, agent dispatch
- **GitHub Copilot integration** — AI-assisted spec writing
- **JetBrains plugin** — IDE-native AgilePlus commands

## Future: Enterprise & Advanced (2027+)

### Team & Multi-User (Q4 2026 – Q1 2027)
- **User authentication** — local accounts, SSO (OKTA, Azure AD)
- **Role-based access** — architect, developer, reviewer, observer
- **Team workspaces** — shared specs, plans, and execution context
- **Permissions system** — approve, merge, and deploy controls
- **Audit logs** — who did what, when, why

### Enterprise Features (2027+)
- **Compliance** — SOC2, HIPAA, regulatory audit trails
- **Advanced RBAC** — fine-grained permissions per resource
- **Data residency** — on-premise deployment, air-gapped environments
- **Advanced integrations** — Jira, Azure DevOps, Atlassian ecosystem
- **Custom workflows** — user-defined phase sequences beyond standard

### Custom Missions (2027+)
- **Product management** — feature prioritization, roadmap planning
- **Infrastructure as Code** — spec-driven Terraform/Pulumi generation
- **Data pipeline orchestration** — spec-driven data workflow design
- **Research projects** — experiment design, hypothesis testing workflow
- **Non-software domains** — extensible to any structured planning domain

## Release Schedule

### v0.1.0 (Current)
- Foundation phase core
- CLI interface
- Plane & GitHub sync
- See [Release Notes](/roadmap/release-notes)

### v0.2.0 (Feb 2026)
- gRPC API complete
- MCP server complete
- Multi-language support (Python, JS/TS, Go)
- PostgreSQL storage adapter
- Retrospective reports

### v0.3.0 (Mar 2026)
- Web dashboard (interactive)
- Multi-agent orchestration
- Performance improvements
- Community plugins (first 5)

### v0.4.0 (May 2026)
- IDE integrations (VSCode, JetBrains)
- Advanced analytics
- Plugin marketplace
- Copilot integration

### v1.0.0 (Sep 2026)
- Team & multi-user support
- Enterprise features (RBAC, compliance)
- Production-ready infrastructure
- Stable API guarantees

## How to Propose Features

### Submit Issues

Have an idea? [Open a GitHub Issue](https://github.com/KooshaPari/AgilePlus/issues):

```
Title: [Feature] Short description

Description:
- Why you need this
- How it would help
- Examples of usage

Example:
[Feature] Multi-agent parallel execution
- Would reduce cycle time by 30%
- Enable faster feature delivery
- Critical for large teams with multiple agents
```

### Create Specifications

For detailed proposals, create a feature specification:

```bash
agileplus specify \
  --title "Multi-Agent Orchestration" \
  --description "Execute independent WPs in parallel across multiple agents"
```

Then push to GitHub as a discussion or PR.

### Vote on Roadmap Items

Upvote planned features (GitHub reactions):
- 👍 for "implement this"
- 🔥 for "urgent/blocking"
- ❓ for "tell us more"

## Funding & Support

AgilePlus is developed by the Phenotype organization.

- **Community support**: GitHub Issues, Discussions
- **Commercial support**: Contact Phenotype for enterprise licensing
- **Sponsorships**: Enterprise clients funding features

## Versioning

AgilePlus follows [semantic versioning](https://semver.org/):

- **v0.x.y** — Foundation phase (breaking changes possible)
- **v1.0+** — Stable API, backward compatibility guaranteed

## Stability Commitments

### v1.0 Stability Promise

Once v1.0 is released (Sep 2026):

- ✓ CLI command syntax will not change (may add new commands)
- ✓ Data format on disk will remain compatible
- ✓ API contract will be maintained (backward compatible)
- ✗ Internal implementation may change
- ✗ Undocumented features may change

### Breaking Changes

Pre-v1.0 breaking changes are possible. We will:
- Announce in release notes
- Provide migration guides
- Tag with `BREAKING` in changelog

After v1.0:
- Breaking changes only in major versions (v2.0, v3.0)
- 6-month deprecation period before removal
- Detailed migration guides provided

## Feedback

Questions about the roadmap?

- **Discussions**: Ask in [GitHub Discussions](https://github.com/KooshaPari/AgilePlus/discussions)
- **Roadmap**: Comment on [roadmap issues](https://github.com/KooshaPari/AgilePlus/issues?q=label%3Aroadmap)
- **Feature requests**: [Open an issue](https://github.com/KooshaPari/AgilePlus/issues/new)
