---
work_package_id: WP10
title: CLI Sync Commands
lane: "doing"
dependencies: []
base_branch: main
base_commit: e34fef4c66a8f5abee0afeafb9110eb8e0f440c0
created_at: '2026-03-02T12:06:07.348910+00:00'
subtasks: [T059, T060, T061, T062, T063]
shell_pid: "35783"
history:
- date: '2026-03-02'
  action: created
  by: spec-kitty
---

## Objective

Add sync subcommands to `crates/agileplus-subcmds` and wire to `crates/agileplus-cli`.

Implementation command: `spec-kitty implement WP10 --base WP09`

## Subtasks

### T059: `agileplus sync push`

Push all local features/WPs that have changed since last sync to Plane.so.

**Behavior:**
- Use `SyncMapping` to determine what changed
- Display `SyncReport` on completion
- Options:
  - `--feature <slug>` (single feature)
  - `--dry-run` (show what would sync without actually syncing)

**Output:**
```
Pushing 3 features to Plane.so...
✓ Feature 'auth-flow' created (plane_issue_id: #42)
✓ WP 'api-endpoints' updated
⊘ WP 'database-schema' skipped (no changes)
Duration: 2.3s
```

### T060: `agileplus sync pull`

Pull all Plane.so changes since last sync.

**Behavior:**
- Process pending webhook events if any queued
- Display `SyncReport`
- Options:
  - `--feature <slug>`
  - `--dry-run`

**Output:**
```
Pulling changes from Plane.so...
✓ Feature 'auth-flow' updated
✓ Issue #45 imported as new feature
⚠ Conflict detected: 'api-design' (local vs remote)
Duration: 1.8s
```

### T061: `agileplus sync auto [on|off|status]`

Enable/disable auto-sync mode.

**Behavior:**
- `on`: Enable auto-sync. Every feature/WP mutation automatically triggers a push.
- `off`: Disable auto-sync.
- `status`: Show current auto-sync setting.
- Store setting in config file.

**Output:**
```
agileplus sync auto status
Auto-sync is ON

agileplus sync auto off
Auto-sync disabled
```

### T062: `agileplus sync status`

Display sync status for all tracked entities.

**Output (table):**
```
Entity                    | Local State  | Remote State | Last Synced | Match | Conflicts
─────────────────────────────────────────────────────────────────────────────────────
Feature: auth-flow        | implementing | in_progress  | 2m ago      | ✓     | 0
Feature: api-design       | researched   | unstarted    | 2h ago      | ✗     | 1
WP: database-schema       | specified    | backlog      | 1d ago      | ✓     | 0
```

Color-code: green=synced, yellow=pending, red=conflict.

### T063: `agileplus sync resolve <entity-type> <entity-id>`

Interactive conflict resolution.

**Behavior:**
- Show local vs remote values side-by-side
- Prompt user to choose: `(L)ocal wins`, `(R)emote wins`, `(M)erge manually`
- Apply chosen resolution via `SyncOrchestrator`

**Example:**
```
agileplus sync resolve feature 5
Conflict in Feature 'api-design':

Local:
  State: researched
  Description: Initial API design with OAuth

Remote (Plane.so):
  State: unstarted
  Description: API design

Choose resolution:
  (L) Keep local changes
  (R) Accept remote changes
  (M) Merge manually
  (C) Cancel

> L

Applied: Local version wins
SyncMapping updated, event logged
```

## Definition of Done

- All 5 sync commands work end-to-end with Plane.so
- Sync status shows accurate local/remote state
- Push/pull with dry-run correctly reflects what would happen
- Conflict resolution prompts work interactively
- Auto-sync toggles and persists correctly
