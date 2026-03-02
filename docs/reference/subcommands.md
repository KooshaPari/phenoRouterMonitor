---
audience: [agents, developers]
---

# Hidden Sub-commands Reference

AgilePlus includes 40+ hidden sub-commands across 8 categories for advanced agent workflows. These are not shown in `--help` but are available via the MCP `dispatch_command` tool and audit-logged for compliance.

Each invocation is recorded in the append-only JSONL audit log at `.agileplus/audit.jsonl` with full traceability.

## Branch Operations

Hidden under `agileplus branch:*`

### branch:create

Create a new branch from a base ref.

```bash
agileplus branch:create --name feat/001-login-WP01 --base main
```

| Argument | Type | Description |
|----------|------|-------------|
| `--name` | string | Branch name (required) |
| `--base` | string | Base ref to branch from (default: main) |

**Audit Log Entry:**
```json
{
  "command": "branch:create",
  "args": {"name": "feat/001-login-WP01", "base": "main"},
  "actor": "claude-code",
  "timestamp": "2025-01-16T14:22:00Z",
  "exit_code": 0,
  "duration_ms": 145
}
```

### branch:checkout

Switch to an existing branch.

```bash
agileplus branch:checkout --name feat/001-login-WP01
```

| Argument | Type | Description |
|----------|------|-------------|
| `--name` | string | Branch name (required) |

### branch:delete

Delete a branch (local or remote).

```bash
agileplus branch:delete --name feat/001-login-WP01 --force
```

| Argument | Type | Description |
|----------|------|-------------|
| `--name` | string | Branch name (required) |
| `--force` | flag | Force delete even if not merged |
| `--remote` | string | Delete from remote (origin, etc.) |

### branch:list

List all branches matching a pattern.

```bash
agileplus branch:list --pattern "feat/*" --remote
```

## Commit Operations

Hidden under `agileplus commit:*`

### commit:create

Stage files and create a new commit.

```bash
agileplus commit:create \
  --message "WP01: Implement login form" \
  --files "src/auth/login.rs,src/auth/session.rs"
```

| Argument | Type | Description |
|----------|------|-------------|
| `--message` | string | Commit message (required) |
| `--files` | string | Comma-separated file paths |
| `--author` | string | Author (auto-detected from git config) |
| `--allow-empty` | flag | Allow empty commits |

### commit:amend

Modify the last commit.

```bash
agileplus commit:amend \
  --message "WP01: Fix login form validation" \
  --allow-empty
```

| Argument | Type | Description |
|----------|------|-------------|
| `--message` | string | New commit message |
| `--allow-empty` | flag | Keep empty commit |

### commit:fixup

Create a fixup commit for an earlier commit.

```bash
agileplus commit:fixup --target abc123def456 --message "Fix type error"
```

| Argument | Type | Description |
|----------|------|-------------|
| `--target` | string | Commit SHA to fixup (required) |
| `--message` | string | Fixup message |

## Diff Inspection

Hidden under `agileplus diff:*`

### diff:show

Show changes between commits or branches.

```bash
agileplus diff:show --from main --to feat/001-login-WP01
```

| Argument | Type | Description |
|----------|------|-------------|
| `--from` | string | Base ref (default: HEAD~1) |
| `--to` | string | Target ref (default: HEAD) |
| `--stat` | flag | Show stats only |
| `--unified` | int | Context lines (default: 3) |

### diff:stat

Show file-level diffstat.

```bash
agileplus diff:stat --from main --to feat/001-login-WP01
```

## Stash Management

Hidden under `agileplus stash:*`

### stash:push

Save uncommitted changes to stash.

```bash
agileplus stash:push --message "WIP: login form styling"
```

| Argument | Type | Description |
|----------|------|-------------|
| `--message` | string | Stash description |
| `--include-untracked` | flag | Include untracked files |

### stash:pop

Restore and remove a stash entry.

```bash
agileplus stash:pop --index 0
```

| Argument | Type | Description |
|----------|------|-------------|
| `--index` | int | Stash index (0 = latest) |

### stash:list

Show all stash entries.

```bash
agileplus stash:list
```

## Worktree Operations

Hidden under `agileplus worktree:*`

### worktree:add

Create a new worktree for parallel development.

```bash
agileplus worktree:add \
  --path .worktrees/001-login-WP01 \
  --branch feat/001-login-WP01 \
  --feature-slug 001-login \
  --wp-id WP01
```

| Argument | Type | Description |
|----------|------|-------------|
| `--path` | string | Worktree directory path (required) |
| `--branch` | string | Branch name (required) |
| `--feature-slug` | string | Feature identifier (metadata) |
| `--wp-id` | string | Work package ID (metadata) |

### worktree:remove

Delete a worktree.

```bash
agileplus worktree:remove --path .worktrees/001-login-WP01
```

| Argument | Type | Description |
|----------|------|-------------|
| `--path` | string | Worktree directory path (required) |
| `--force` | flag | Force remove even if dirty |

### worktree:list

Show all active worktrees.

```bash
agileplus worktree:list
```

**Output:**
```json
[
  {
    "path": ".worktrees/001-login-WP01",
    "branch": "feat/001-login-WP01",
    "feature_slug": "001-login",
    "wp_id": "WP01"
  }
]
```

## Artifact File Operations

Hidden under `agileplus artifact:*`

### artifact:write

Write a feature artifact (spec, plan, evidence, etc.).

```bash
agileplus artifact:write \
  --feature-slug 001-login \
  --relative-path spec.md \
  --content "# Login Feature\n\n..."
```

| Argument | Type | Description |
|----------|------|-------------|
| `--feature-slug` | string | Feature ID (required) |
| `--relative-path` | string | Path relative to feature dir (required) |
| `--content` | string | File content |
| `--from-file` | string | Read content from file instead of stdin |

### artifact:read

Read a feature artifact.

```bash
agileplus artifact:read \
  --feature-slug 001-login \
  --relative-path spec.md
```

### artifact:hash

Compute hash of an artifact (for integrity checking).

```bash
agileplus artifact:hash \
  --feature-slug 001-login \
  --relative-path spec.md
```

**Output:**
```json
{
  "path": "001-login/spec.md",
  "hash": "sha256:abc123...",
  "size_bytes": 4567
}
```

## Governance Operations

Hidden under `agileplus governance:*`

### governance:check

Validate whether a feature can transition to a new state.

```bash
agileplus governance:check \
  --feature-slug 001-login \
  --transition REVIEW
```

| Argument | Type | Description |
|----------|------|-------------|
| `--feature-slug` | string | Feature ID (required) |
| `--transition` | string | Target state (required) |
| `--verbose` | flag | Show detailed violations |

**Output (success):**
```json
{
  "passed": true,
  "violations": []
}
```

**Output (failure):**
```json
{
  "passed": false,
  "violations": [
    {
      "rule_id": "FR-REVIEW-001",
      "message": "At least one approved review required",
      "remediation": "Request code review from @reviewer"
    }
  ]
}
```

### governance:enforce

Enforce a governance rule (admin use).

```bash
agileplus governance:enforce \
  --feature-slug 001-login \
  --rule FR-REVIEW-001 \
  --action enable
```

| Argument | Type | Description |
|----------|------|-------------|
| `--feature-slug` | string | Feature ID (required) |
| `--rule` | string | Rule ID (required) |
| `--action` | string | enable, disable, reset |

## Audit Trail Operations

Hidden under `agileplus audit:*`

### audit:log

Query audit log entries.

```bash
agileplus audit:log \
  --feature-slug 001-login \
  --limit 20 \
  --output json
```

| Argument | Type | Description |
|----------|------|-------------|
| `--feature-slug` | string | Filter by feature |
| `--actor` | string | Filter by actor (claude-code, etc.) |
| `--since` | string | ISO 8601 timestamp (e.g., 2025-01-16T00:00:00Z) |
| `--limit` | int | Max entries (default: 50) |
| `--output` | string | `json`, `table`, or `markdown` |

**Example JSON output:**
```json
[
  {
    "id": 42,
    "feature_slug": "001-login",
    "wp_sequence": 1,
    "timestamp": "2025-01-16T14:22:00Z",
    "actor": "claude-code",
    "transition": "PLANNED -> DOING",
    "evidence_refs": ["pr/42", "commit/abc123"],
    "prev_hash": "...",
    "hash": "..."
  }
]
```

### audit:query

Advanced query on audit log (supports filtering, aggregation).

```bash
agileplus audit:query \
  --where "actor = 'claude-code' AND transition LIKE '%REVIEW%'" \
  --group-by "feature_slug" \
  --order-by "timestamp DESC"
```

### audit:verify

Verify the cryptographic integrity of the audit chain.

```bash
agileplus audit:verify --feature-slug 001-login
```

**Output:**
```json
{
  "valid": true,
  "entries_verified": 127,
  "first_invalid_id": null,
  "error_message": null
}
```

## Audit Log Format

All subcommands are recorded in `.agileplus/audit.jsonl`:

```jsonl
{"command":"branch:create","args":{"name":"feat/001"},"actor":"claude-code","exit_code":0,"duration_ms":145,"timestamp":"2025-01-16T14:22:00Z"}
{"command":"commit:create","args":{"message":"WP01: init","files":"src/main.rs"},"actor":"claude-code","exit_code":0,"duration_ms":234,"timestamp":"2025-01-16T14:23:15Z"}
```

Each line is complete JSON (no multiline records).

## Accessibility via MCP

All subcommands are callable via the MCP `dispatch_command` tool:

```json
{
  "command": "dispatch_command",
  "feature_slug": "001-login",
  "args": {
    "command": "branch:create",
    "name": "feat/001-login-WP02",
    "base": "feat/001-login-WP01"
  }
}
```

## Stability & Versioning

Subcommands follow semantic versioning:

- **Existing args are never removed** (backward compatible)
- **New args are optional with sensible defaults**
- **New subcommands can be added freely**
- **Deprecated subcommands move to `legacy:*` namespace**

This ensures agent scripts never break unexpectedly.