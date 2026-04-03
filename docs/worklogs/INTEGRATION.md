# Integration Worklogs

**Category:** INTEGRATION | **Updated:** 2026-03-29

---

## 2026-03-29 - Cross-Project Integration Recommendations

**Project:** [cross-repo]
**Category:** integration
**Status:** in_progress
**Priority:** P1

### Summary

Recommendations for integrating new repositories (from starred repo research) with existing Phenotype ecosystem.

### Integration Matrix

| New Repo | AgilePlus | thegent | heliosCLI | heliosApp |
|----------|-----------|---------|-----------|-----------|
| `knowledge-base` | Semantic search | Research context | Help system | N/A |
| `harbor-skills` | Agent dispatch | Agent roles | CLI skills | N/A |
| `pathway-xpack` | Event processing | Research streams | N/A | N/A |
| `llm-eval` | Agent validation | Governance | Quality gates | N/A |
| `nitro-agent` | MCP deployment | N/A | Serverless | N/A |
| `worktrunk-sync` | PM sync | N/A | N/A | PM features |

### Integration Tasks

#### knowledge-base Integration

| Task | Depends On | Status |
|------|------------|--------|
| Create spec indexer | knowledge-base repo | pending |
| Create plan indexer | knowledge-base repo | pending |
| Add RAG to agent dispatch | spec indexer | pending |
| Wire to MCP tools | indexers | pending |

#### harbor-skills Integration

| Task | Depends On | Status |
|------|------------|--------|
| Create AgilePlus skill definitions | harbor-skills repo | pending |
| Add skill dispatch to agent-dispatch | skill definitions | pending |
| Wire skills to CLI commands | skill dispatch | pending |
| Document skill patterns | all above | pending |

#### pathway-xpack Integration

| Task | Depends On | Status |
|------|------------|--------|
| Create NATS connector | pathway-xpack repo | pending |
| Add event stream processor | NATS connector | pending |
| Wire to agileplus-events | stream processor | pending |
| Add RAG connector | event processor | pending |

### Tasks Completed

- [x] Identified integration points
- [x] Documented dependency matrix
- [x] Created integration recommendations

### Next Steps

- [ ] Create integration specs for each new repo
- [ ] Prioritize knowledge-base integration
- [ ] Define API contracts

### Related

- Plan: `plans/2026-03-29-CROSS_PROJECT_DUPLICATION_PLAN-v1.md`
- Architecture: `worklogs/ARCHITECTURE.md`

---

## 2026-03-29 - MCP Server Integration Review

**Project:** [AgilePlus]
**Category:** integration
**Status:** completed
**Priority:** P1

### Summary

Review of MCP server integration between agileplus-mcp and thegent-mcp.

### Current State

| Aspect | agileplus-mcp | thegent-mcp |
|--------|---------------|-------------|
| Tools | 15+ | 8+ |
| Skills | Basic | Advanced |
| Streaming | Not implemented | N/A |
| gRPC backend | Yes | No |

### Integration Opportunities

1. **Share common MCP utilities** - Create `phenotype-mcp-core`
2. **Adopt skill framework** - Import from thegent
3. **Add streaming** - Implement SSE for responses
4. **Unified tool registry** - Share tool definitions

### Recommendations

1. Create `libs/phenotype-mcp-core` for shared utilities
2. Migrate thegent MCP tools to agileplus-mcp
3. Add skill-based tool organization
4. Implement response streaming

### Related

- MCP Server: `agileplus-mcp/src/agileplus_mcp/server.py`
- TheGent MCP: `thegent/src/thegent/mcp/`

---

## 2026-03-28 - Plane.so Integration Status

**Project:** [AgilePlus]
**Category:** integration
**Status:** in_progress
**Priority:** P0

### Summary

Status update on Plane.so integration. Work paused pending G037 fork decision.

### Current Implementation

| Component | Status | Location |
|-----------|--------|----------|
| API client | Partial | `crates/agileplus-plane/src/client.rs` |
| Sync mapping | Partial | `crates/agileplus-plane/src/sync.rs` |
| Bidirectional sync | Partial | `crates/agileplus-plane/src/bidirectional.rs` |
| Conflict detection | Not started | pending |

### Dependencies

- G037-WP1: Fork Plane repo → blocked
- G037-WP2: Define API boundary → blocked
- G037-WP3: Migrate dashboard code → blocked

### Related

- Spec: `.agileplus/specs/008-plane-shared-pm-substrate/`
- Session: `docs/sessions/20260327-plane-fork-pm-substrate/`

---

## 2026-03-27 - NATS Event Bus Integration

**Project:** [AgilePlus]
**Category:** integration
**Status:** in_progress
**Priority:** P1

### Summary

NATS JetStream integration for inter-service event transport.

### Implementation Status

| Component | Status | File |
|-----------|--------|------|
| NATS client | Partial | `crates/agileplus-nats/src/client.rs` |
| Event publisher | Partial | `crates/agileplus-nats/src/publish.rs` |
| Event subscriber | Partial | `crates/agileplus-nats/src/subscribe.rs` |
| Stream management | Partial | `crates/agileplus-nats/src/streams.rs` |

### Next Steps

- [ ] Implement subject mapping
- [ ] Add message serialization
- [ ] Configure stream retention
- [ ] Add health checks

### Related

- Crate: `crates/agileplus-nats/`

---

## 2026-03-26 - GitHub Integration Status

**Project:** [AgilePlus]
**Category:** integration
**Status:** in_progress
**Priority:** P1

### Summary

GitHub API integration for PR and issue linking.

### Implementation Status

| Component | Status | File |
|-----------|--------|------|
| GitHub client | Partial | `crates/agileplus-github/src/client.rs` |
| PR linking | Partial | `crates/agileplus-github/src/pr.rs` |
| Issue linking | Partial | `crates/agileplus-github/src/issue.rs` |
| Status sync | Not started | pending |

### Related

- Crate: `crates/agileplus-github/`

---
