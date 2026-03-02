---
audience: [developers, pms]
---

# Synchronization: Plane.so & GitHub

AgilePlus synchronizes specifications, work packages, and status with external trackers, keeping your workflow system and issue tracker in sync.

## Supported Integrations

### Plane.so

Full bi-directional sync with Plane project management:

| What Syncs | Direction | Notes |
|------------|-----------|-------|
| Issues | ← → | Specs create issues, updates sync both ways |
| States | ← → | Done/blocked/done syncs states in Plane |
| Labels | ← → | AgilePlus labels sync to Plane |
| Milestones | ← → | WP groups sync as milestones |
| Assignees | ← → | WP owner syncs to Plane assignee |
| Comments | ← | Plane comments pull into AgilePlus |

### GitHub Issues

Bi-directional sync with GitHub Issues:

| What Syncs | Direction | Notes |
|------------|-----------|-------|
| Issues | ← → | Create issues from specs |
| Labels | ← → | Status labels sync both ways |
| Milestones | ← → | Tie issues to release milestones |
| PR Links | → | Link issues to merged PRs |
| Assignees | ← → | Assignment syncs both ways |

### GitHub Projects (v2)

Push-only sync to GitHub Projects board:

| What Syncs | Direction | Notes |
|------------|-----------|-------|
| Cards | → | WPs become cards |
| Status | → | WP lane → Status field |
| Custom Fields | → | Custom field mapping |

## Setup & Configuration

### 1. Get API Credentials

**For Plane.so:**

1. Go to [Plane Settings](https://app.plane.so/settings/profile)
2. Click **API Tokens**
3. Create new token, copy it

```bash
export PLANE_API_KEY="your_token_here"
```

**For GitHub:**

1. Go to [GitHub Settings → Developer settings → Personal access tokens](https://github.com/settings/tokens)
2. Create new token with scopes: `repo`, `read:org`
3. Copy token

```bash
export GITHUB_TOKEN="ghp_xxxxxxxxxxxxxxxxxxxx"
```

### 2. Configure AgilePlus

Edit `.kittify/config.toml`:

```toml
[sync.plane]
enabled = true
workspace = "my-org"
project = "my-project"
api_key = "${PLANE_API_KEY}"

[sync.github]
enabled = true
repo = "org/my-repo"
token = "${GITHUB_TOKEN}"

[sync.github.pr]
auto_create = true
draft = false
request_review = true

[sync]
interval = 5                    # Sync every 5 minutes
conflict_strategy = "local-wins"
```

### 3. Test Connection

```bash
agileplus sync --test
```

Output:

```
Testing Plane.so connection...
✓ Connected to workspace: my-org
✓ Project: my-project
✓ API key valid

Testing GitHub connection...
✓ Connected to repo: org/my-repo
✓ Token valid
✓ Can create issues and PRs

All connections working!
```

## Sync Workflow

### Automatic Sync

By default, AgilePlus syncs every 5 minutes (configurable):

```
AgilePlus State        Plane.so           GitHub Issues
─────────────────────  ─────────────────  ──────────────
Planned   ──────→      Bug #123           #456
Doing     ──────→      In Progress        In Progress
Done      ──────→      Done               Closed
          ←──────      (Label changes)    (Re-assigned)
```

### Manual Sync

```bash
# Full bi-directional sync
agileplus sync

# Only push local changes
agileplus sync --push-only

# Only pull remote changes
agileplus sync --pull-only

# Verbose output
agileplus sync --verbose

# Test without making changes
agileplus sync --dry-run
```

### One-Way Sync

Create issues in tracker without auto-update:

```toml
[sync]
direction = "push-only"
```

Only pull from tracker without creating issues:

```toml
[sync]
direction = "pull-only"
```

## Mapping: Specs to Issues

When you create a specification, AgilePlus can create a tracker issue:

```bash
agileplus specify "Add two-factor authentication"

# Automatically creates issue in Plane/GitHub if sync enabled
# → [SPEC] Add two-factor authentication #123
```

The sync creates:

```
Issue Title:  [SPEC] Add two-factor authentication
Description:  Feature specification
Label:        spec
Status:       New
Milestone:    (current sprint, if configured)
```

## Mapping: Work Packages to Issues

When you create work packages, they sync to the tracker:

```bash
agileplus tasks 001

# Creates issues for each WP
# → [WP] WP01: Database schema #124
# → [WP] WP02: Models layer #125
# → [WP] WP03: API endpoints #126
```

Each WP issue shows:

```
Title:        [WP] WP01: Database schema
Description:  Work package WP01 for feature 001
Label:        work-package, wp-01
Status:       Planned
Assignee:     (owner when assigned)
Milestone:    Feature 001
```

## Status Synchronization

### AgilePlus → Tracker

When you move a WP through lanes:

```bash
agileplus move WP01 --to doing      # Updates tracker: In Progress
agileplus move WP01 --to for_review # Updates tracker: In Review
agileplus move WP01 --to done       # Updates tracker: Done
```

The tracker issue status updates automatically.

### Tracker → AgilePlus

When you update status in the tracker:

```
Plane.so Status  →  AgilePlus Lane
───────────────────────────────────
Backlog          →  planned
Todo             →  planned
In Progress      →  doing
In Review        →  for_review
Done             →  done
```

The next sync pulls these changes.

## Label Management

### Auto-Generated Labels

AgilePlus creates standard labels:

```
spec              # For specifications
work-package      # For work packages
wp-01, wp-02      # Per-work-package labels
high-priority     # From queue priority
bug, feature      # From issue type
```

### Custom Label Mapping

```toml
[sync.label_mapping]
my-urgent = "critical"
in-review = "for_review"
approved = "done"
```

## Conflict Resolution

When both AgilePlus and tracker make conflicting changes:

**Strategy: local-wins (default)**

```toml
[sync]
conflict_strategy = "local-wins"
```

If AgilePlus says "Done" and tracker says "In Progress", AgilePlus wins.

**Strategy: remote-wins**

```toml
[sync]
conflict_strategy = "remote-wins"
```

If tracker says "In Progress" and AgilePlus says "Done", tracker wins.

**Strategy: merge**

```toml
[sync]
conflict_strategy = "merge"
```

Conflicting labels are merged (both kept). Only status uses winner strategy.

## Advanced: Custom Field Mapping

Map AgilePlus data to custom fields in tracker:

```toml
[sync.github.custom_fields]
complexity = "Estimated Hours"
risk = "Risk Level"
dependencies = "Blocked By"
```

Then use in GitHub Projects:

```bash
# View custom fields
agileplus show WP01 --include-custom-fields
```

## Troubleshooting

### Sync Not Working

```bash
# Check sync status
agileplus sync --status

# Enable debug logging
agileplus sync -vv

# Test credentials
agileplus config show --include-secrets
```

**Check firewall:**

Make sure you can reach API endpoints:

```bash
# Plane.so
curl -H "Authorization: Bearer $PLANE_API_KEY" \
  https://api.plane.so/api/workspaces/

# GitHub
curl -H "Authorization: Bearer $GITHUB_TOKEN" \
  https://api.github.com/user/repos
```

### Issues Not Syncing

```bash
# Verify labels are created
agileplus sync --labels-only

# Check mapping configuration
cat .kittify/config.toml | grep -A 10 "sync"

# Force full sync
agileplus sync --force --verbose
```

### Duplicate Issues Created

This happens if sync runs twice on same feature:

```bash
# Check existing issues in tracker
agileplus sync --dry-run

# If duplicates exist, manually delete in tracker
# Then re-run sync
agileplus sync --force
```

### Token Invalid

```bash
# Re-enter credentials
export PLANE_API_KEY="new_token"
export GITHUB_TOKEN="new_token"

# Test again
agileplus sync --test
```

## Examples

### Example 1: Full Plane.so Integration

```toml
[sync.plane]
enabled = true
workspace = "acme-corp"
project = "backend-api"
api_key = "${PLANE_API_KEY}"

[sync.plane.mapping]
spec_label = "spec"
wp_label = "work-package"
priority_map = {
  critical = "urgent",
  high = "high",
  medium = "medium",
  low = "low"
}

[sync]
interval = 5
conflict_strategy = "local-wins"
```

Run sync:

```bash
agileplus sync --verbose

# Output:
# ✓ Synced 3 specs as issues
# ✓ Synced 12 work packages
# ✓ Updated 5 issue states
# ✓ Pulled 2 label changes from Plane.so
```

### Example 2: GitHub Issues Only (No Auto PR)

```toml
[sync.github]
enabled = true
repo = "acme-corp/api"
token = "${GITHUB_TOKEN}"

[sync.github.pr]
auto_create = false          # Don't auto-create PRs

[sync.github.mapping]
labels = {
  spec = "spec",
  wp = "work-package"
}
```

### Example 3: Push-Only (Don't Pull Tracker Changes)

```toml
[sync]
direction = "push-only"

[sync.plane]
enabled = true
workspace = "my-org"
project = "my-project"
api_key = "${PLANE_API_KEY}"
```

This creates issues in tracker but doesn't pull status updates back.

## Best Practices

**1. Sync Before Merging**

```bash
agileplus sync --pull-only
# Make sure no conflicting changes from tracker
agileplus merge 001
```

**2. Regular Sync Intervals**

Don't set sync interval too low (API rate limits):

```toml
[sync]
interval = 5    # 5 minutes is reasonable
```

**3. Link Everything**

When creating specs, link to tracker:

```bash
agileplus specify "title" --tracker-issue https://github.com/org/repo/issues/123
```

**4. Archive Old Issues**

Clean up closed issues to reduce clutter:

```bash
agileplus sync --archive-closed
```

**5. Review Mapping Regularly**

Check that labels and statuses still match:

```bash
agileplus config show --include sync
```

## What's Next

- **[Configuration](/guide/configuration)** — Detailed config options
- **[Getting Started](/guide/getting-started)** — Full setup walkthrough
- **[Core Workflow](/guide/workflow)** — Understand the pipeline
