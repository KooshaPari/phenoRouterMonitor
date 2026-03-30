# WP-004: Provider-Aware Subagent Spawning

**Work Package ID**: WP-004
**Epic**: eco-fork-001 (Custom Providers & Subagent Management)
**Phase**: 1
**Status**: Pending
**Priority**: Critical
**Created**: 2026-03-30

---

## Overview

Add `spawn-agent` CLI subcommand enabling dynamic subagent creation with provider and model selection, full lifecycle tracking, and gRPC dispatch integration.

## Description

Bridge AgilePlus work packages with forgecode provider abstraction. Users can spawn agents to execute work packages using any registered provider (Claude, local, etc.), with full audit trail and lifecycle management.

---

## Objectives

- Create `spawn-agent` CLI subcommand for selecting provider + model + work package
- Persist agent spawn records to SQLite with lifecycle states
- Implement agent status tracking (spawned, running, completed, failed)
- Wire gRPC agent dispatch service for remote execution
- Support provider fallback on dispatch failure

---

## Acceptance Criteria

1. **CLI Command**:
   - `agileplus spawn-agent WP01 --provider claude --model opus` succeeds
   - Returns agent_id for tracking
   - Help text documents all options

2. **Lifecycle Tracking**:
   - Agent status visible in `agileplus agent list`
   - States: spawned → running → completed (or failed)
   - Timestamps recorded for each state transition

3. **Database**:
   - agent_spawns table created with lifecycle columns
   - Queries return current status correctly
   - History retained for audit trail

4. **Fallback**:
   - Provider fallback works (Opus → Sonnet if unavailable)
   - Fallback attempts logged to audit trail
   - Success metric tracks fallback rate

5. **Testing**:
   - `cargo test -p agileplus-cli` all pass
   - Integration test: spawn → status → completion flow
   - Error handling tests

---

## Deliverables

| Deliverable | Description | Acceptance |
|-------------|-------------|-----------|
| spawn-agent CLI | New subcommand | Works end-to-end |
| Agent database | agent_spawns table | Lifecycle tracked |
| Status tracking | `agent list` command | Shows all agents |
| gRPC dispatch | Integration with dispatch service | Calls work correctly |
| Error handling | Invalid providers, missing WPs | Clear errors |
| Tests | Unit + integration | ≥90% coverage |

---

## Dependencies

**Depends On**:
- WP-002 (Claude Provider)
- WP-003 (Local Provider)

**Blocks**:
- WP-05 (Capability Discovery)
- WP-06 (Performance Metrics)

---

## Effort Estimate

- **Estimated LOC**: 420
- **Estimated Tool Calls**: 14-16
- **Estimated Duration**: 4-5 days

---

## Technical Details

### CLI Usage

```bash
# Spawn agent with specific provider
agileplus spawn-agent WP01 --provider claude --model opus

# Spawn with fallback list
agileplus spawn-agent WP01 --provider claude --fallback grok --fallback local

# List all agents
agileplus agent list

# Get status
agileplus agent status <agent_id>

# Get agent logs
agileplus agent logs <agent_id>
```

### Database Schema

```sql
CREATE TABLE agent_spawns (
    agent_id TEXT PRIMARY KEY,
    work_package_id TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    model TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMP,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    failed_at TIMESTAMP,
    error_message TEXT,
    fallback_chain TEXT,
    output_artifact_path TEXT,
    FOREIGN KEY (work_package_id) REFERENCES work_packages(id)
);
```

---

## Subtasks

- [ ] T018: Create `agileplus-cli/src/commands/spawn_agent.rs`
- [ ] T019: Add `SpawnRequest` message to agileplus-proto
- [ ] T020: Create `agent_spawns` SQLite table with lifecycle states
- [ ] T021: Implement spawn logic: resolve WP → create Agent record → invoke dispatch
- [ ] T022: Add agent status tracking in CLI (`agileplus agent status <id>`)
- [ ] T023: Integration test: spawn → track → verify audit trail
- [ ] T024: Error handling for invalid providers, missing WPs, resource limits

---

## Success Metrics

| Metric | Target | Measure |
|--------|--------|---------|
| CLI Usability | Intuitive | Help text clear, example runs |
| Lifecycle Tracking | 100% | All transitions recorded |
| Fallback Success | >95% | Fallback retry succeeds |
| Response Time | <2s | `spawn-agent` command latency |
| Test Coverage | ≥90% | `cargo tarpaulin` |

---

## Integration Points

- **agileplus-cli**: spawn-agent, agent commands
- **agileplus-sqlite**: agent_spawns table + queries
- **agileplus-grpc**: SpawnRequest protobuf
- **forgecode-providers**: Provider registry lookup

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|-----------|
| gRPC dispatch fails | Medium | High | Implement retry with fallback |
| Database contention | Low | Medium | Use WAL mode, connection pool |
| Resource limits | Medium | Medium | Add max concurrent agents config |

---

## Traceability

**Functional Requirements**:
- FR-PROV07: Subagent spawning with provider selection
- FR-SPAWN01: Agent lifecycle tracking

---

## Notes

- Agent ID: UUID v4 for uniqueness
- Fallback chain: e.g., "claude-opus → claude-sonnet → local"
- Future: Real-time agent status streaming via WebSocket

---

**Owner**: TBD
**Last Updated**: 2026-03-30
**Status**: Pending Implementation
