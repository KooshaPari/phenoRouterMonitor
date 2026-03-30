# WP-005: Router CLI & Status Dashboard

**Work Package ID**: WP-005
**Epic**: eco-fork-002 (Consolidated API Monitoring & Routing)
**Phase**: 2
**Status**: Pending
**Priority**: Medium
**Created**: 2026-03-30

---

## Overview

Add `router` CLI subcommand to AgilePlus with status dashboard showing routes, backends, and health.

## Description

Provide real-time visibility into router state via CLI commands and optional TUI dashboard.

---

## Objectives

- Add `router` CLI subcommand to AgilePlus
- Show status of all routes, backends, health
- List aggregated metrics
- Enable/disable routes without restarting
- Add status dashboard (TUI with ratatui)

---

## Acceptance Criteria

1. **CLI Commands**:
   - `agileplus router status` renders table with all routes
   - `agileplus router backends <route>` shows health of each backend
   - `agileplus router metrics <route>` shows latency percentiles
   - Can disable/enable route via CLI

2. **Dashboard**:
   - Real-time status display
   - Refresh updates every 5 seconds
   - Shows routes, backends, health, metrics

3. **Testing**:
   - `cargo test -p agileplus-cli` all pass
   - CLI output format tests

---

## Deliverables

| Deliverable | Description | Acceptance |
|-------------|-------------|-----------|
| CLI commands | router status/backends/metrics | All work |
| Admin API | Enable/disable routes | Immediate effect |
| TUI dashboard | ratatui-based dashboard | Responsive |
| Tests | Unit + integration | ≥85% coverage |

---

## Dependencies

**Depends On**:
- WP-001-WP-004 (All core router WPs)

**Blocks**: None

---

## Effort Estimate

- **Estimated LOC**: 320
- **Estimated Tool Calls**: 9-11
- **Estimated Duration**: 3-4 days

---

## Subtasks

- [ ] T033: Create `agileplus-cli/src/commands/router.rs`
- [ ] T034: Implement `router status` command
- [ ] T035: Implement `router backends <route>` command
- [ ] T036: Implement `router metrics <route>` command
- [ ] T037: Implement `router disable <route>` / `enable` commands
- [ ] T038: HTTP API endpoints for router admin
- [ ] T039: TUI dashboard showing router status (ratatui)
- [ ] T040: Integration test: verify CLI shows accurate metrics

---

**Owner**: TBD
**Last Updated**: 2026-03-30
**Status**: Pending Implementation
