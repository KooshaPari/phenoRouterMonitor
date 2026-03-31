---
audience: [developers, pms]
---

# Triage & Queue Management

AgilePlus includes a triage system that classifies incoming work (bugs, features, ideas, tasks) and manages a priority-ordered backlog. This enables teams to efficiently intake requests and decide what to work on next.

## Quick Triage

### Classify an Item

```bash
agileplus triage "login page crashes on mobile Safari"
```

Output:

```
Analyzing: "login page crashes on mobile Safari"

Intent:      Bug
Confidence:  92%
Keywords:    crashes, login
Categories:  High Priority, Critical Path

✓ Added to queue as bug item (#42)

Next: agileplus queue show 42
      agileplus queue pop
```

### What It Classifies

The classifier detects four intent categories using keyword matching:

| Intent | Keywords | Default Priority | Example |
|--------|----------|------------------|---------|
| **Bug** | crash, error, fail, broken, exception, bug | High | "login fails for Gmail accounts" |
| **Feature** | add, new, implement, support, enable | Medium | "add dark mode support" |
| **Task** | update, migrate, refactor, optimize, clean | Medium | "update dependencies to latest" |
| **Idea** | what if, could we, brainstorm, consider | Low | "could we cache API responses?" |

### Override Classification

```bash
# Force a specific type
agileplus triage "update dependencies" --type task --priority critical
```

### Dry Run (Preview Only)

```bash
agileplus triage "fix memory leak" --dry-run
```

Output:

```
Analyzing: "fix memory leak"

Intent:      Bug
Confidence:  96%
Keywords:    fix, memory, leak
Categories:  Performance, Reliability

(No item added — use without --dry-run to add to queue)
```

### JSON Output

```bash
agileplus triage "slow database queries" --output json
```

```json
{
  "text": "slow database queries",
  "intent": "Bug",
  "confidence": 0.88,
  "matched_keywords": ["slow", "database"],
  "suggested_priority": "High",
  "suggested_type": "Bug",
  "categorized_as": ["Performance", "Database"]
}
```

## Managing the Queue

The queue is a prioritized backlog of all incoming work. Items flow from queue → specified → implemented.

### Add to Queue

**With auto-classification:**

```bash
agileplus queue add "Add email notifications"
# Auto-classified as Feature (Medium priority)
```

**With explicit type:**

```bash
agileplus queue add \
  --title "Users can't reset password" \
  --type bug \
  --priority critical \
  --description "Password reset emails not being sent"
```

**Bulk add from file:**

```bash
# items.txt
Add dark mode
Fix typo on homepage
Update Python to 3.11

agileplus queue add --from-file items.txt
```

### List Queue Items

**Show all items:**

```bash
agileplus queue list
```

Output:

```
CRITICAL (1)
  [BUG]     #47  Users can't reset password       2 hours ago

HIGH (3)
  [BUG]     #46  Login fails on mobile Safari    5 hours ago
  [FEAT]    #45  Add API rate limiting           1 day ago
  [TASK]    #44  Update Node dependencies        2 days ago

MEDIUM (8)
  [FEAT]    #43  Email notifications            3 days ago
  [FEAT]    #42  Dark mode support              4 days ago
  ...

LOW (12)
  [IDEA]    #35  Consider webhook system        1 week ago
  ...

Total: 24 items | Avg age: 3 days
```

**Filter by type:**

```bash
agileplus queue list --type bug        # Only bugs
agileplus queue list --type feature    # Only features
```

**Filter by priority:**

```bash
agileplus queue list --priority critical,high
```

**Filter by age:**

```bash
agileplus queue list --older-than 7d   # Items added >7 days ago
```

**Sort by different criteria:**

```bash
agileplus queue list --sort priority   # Default: priority
agileplus queue list --sort age        # Oldest first
agileplus queue list --sort impact     # Estimated impact
```

**Output formats:**

```bash
agileplus queue list --output json     # JSON array
agileplus queue list --output csv      # CSV (for spreadsheets)
agileplus queue list --output markdown # Markdown table
```

### Show Item Details

```bash
agileplus queue show 47
```

Output:

```
Issue #47: Users can't reset password

Type:       Bug
Priority:   CRITICAL
Status:     New
Age:        2 hours ago
Submitter:  alice@example.com

Description:
Users report that password reset emails are not being sent.
Affects: All email-dependent flows
Severity: Blocker

Related:
- GitHub issue #1234
- Plane.so issue abc-123

Next:
  agileplus specify 47              # Start work on this
  agileplus queue move 47 --to blocked
```

### Pop Next Item (For Implementation)

Get the highest-priority item from the queue:

```bash
agileplus queue pop
```

Output:

```
Highest priority item:

[CRITICAL] #47: Users can't reset password
Type:      Bug
Queued:    2 hours ago

Next steps:
1. agileplus specify 47              # Create spec from queue item
2. Work through clarify → research → plan → implement

Ready? (Y/n) y

✓ Started specification for item #47
  kitty-specs/001-password-reset-fix/
```

### Move Items Between States

```bash
# Mark as blocked (waiting on external event)
agileplus queue move 47 --to blocked

# Mark as duplicate
agileplus queue move 47 --to duplicate --notes "See issue #45"

# Mark as blocked (external dependency)
agileplus queue move 47 --to blocked --reason "Waiting for API documentation"

# Un-block an item
agileplus queue move 47 --to open
```

**Item states:**

```
new → open → blocked/in-progress → done/duplicate/won't-fix
```

### Priority Management

**Auto-assign priority:**

Priority is assigned based on type and keywords:

```
Type    | Keyword Analysis | Default Priority
--------|------------------|------------------
Bug     | Any              | High
Bug     | crash, blocker   | Critical
Feature | standard         | Medium
Task    | standard         | Medium
Idea    | standard         | Low
```

**Override priority:**

```bash
agileplus queue add "add emoji picker" --type feature --priority critical
```

**Reprioritize existing item:**

```bash
agileplus queue move 42 --priority high
```

**Priority order (for pop):**

```
Critical > High > Medium > Low

Within same priority: FIFO (oldest first)
```

### Bulk Operations

**Move multiple items to a state:**

```bash
agileplus queue move 42 44 45 --to done
```

**Update priority for multiple items:**

```bash
agileplus queue set-priority 40-50 --priority medium
# Sets items #40-#50 to medium priority
```

**Archive old items:**

```bash
agileplus queue archive --older-than 30d --status new
# Archives items older than 30 days that are still 'new'
```

## Workflow: From Queue to Specification

The typical flow from queue intake to implementation:

```
1. Incoming request
   ↓ (triage)
2. Queue item (e.g., #47)
   ↓ (review & prioritize)
3. Pop from queue
   ↓ (specify 47)
4. Create specification
   ↓ (clarify, research, plan, etc.)
5. Implementation begins
```

### Example: Bug Fix from Queue

```bash
# Step 1: Triage incoming bug
agileplus triage "Users can't save drafts"
# → Added as bug #51

# Step 2: Review queue
agileplus queue list --priority critical
# → See #51 at top

# Step 3: Pop for work
agileplus queue pop
# → Shows #51

# Step 4: Create detailed spec
agileplus specify 51 \
  --title "Draft save broken" \
  --description "Reproduce: edit post, autosave fails silently"
# → Creates kitty-specs/001-draft-save-fix/

# Step 5: Continue pipeline
agileplus clarify 001
agileplus plan 001
agileplus tasks 001
agileplus implement WP01
```

## Analytics & Insights

**Queue health:**

```bash
agileplus queue stats
```

Output:

```
Queue Statistics

Total Items:     127
By Type:
  - Bugs:        34 (27%)
  - Features:    52 (41%)
  - Tasks:       24 (19%)
  - Ideas:       17 (13%)

By Priority:
  - Critical:     2 (1.6%)
  - High:        18 (14%)
  - Medium:      67 (53%)
  - Low:         40 (31%)

Age Analysis:
  - <1 day:      15
  - 1-7 days:    42
  - 1-4 weeks:   45
  - >1 month:    25

Bottlenecks:
  - 25 items blocked (waiting on dependencies)
  - 12 items have no assigned owner
  - Average time to specification: 2.3 days
```

**Forecast:**

```bash
agileplus queue forecast --items 5
```

Output:

```
Next 5 items to implement:

1. [CRITICAL BUG]    #47  Users can't reset password       Est. 4 hours
2. [HIGH BUG]        #46  Login fails on mobile Safari     Est. 6 hours
3. [HIGH FEATURE]    #45  API rate limiting               Est. 16 hours
4. [MEDIUM TASK]     #44  Update Node dependencies         Est. 3 hours
5. [MEDIUM FEATURE]  #43  Email notifications             Est. 24 hours

Total estimated: 53 hours (6-7 days with team of 2)
```

## Best Practices

**1. Triage Daily**

```bash
# Add all incoming requests to queue
agileplus triage "incoming item"
```

**2. Regular Queue Review**

```bash
# Weekly: review and reprioritize
agileplus queue list --sort age
# Look for old items stuck in 'new' state
```

**3. Keep Priority Stable**

Don't constantly reprioritize. Set priority once, review weekly.

**4. Link to External Systems**

When triaging from GitHub/Plane.so, include issue links:

```bash
agileplus queue add "title" \
  --github-issue https://github.com/org/repo/issues/123
```

**5. Archive Regularly**

Clean up old items:

```bash
agileplus queue archive --older-than 60d --status won't-fix
```

## Troubleshooting

**Misclassified item?**

```bash
agileplus queue move 42 --type feature
# Changes type and recalculates priority
```

**Wrong priority?**

```bash
agileplus queue move 42 --priority critical
# Overrides auto-assigned priority
```

**Item was a duplicate?**

```bash
agileplus queue move 42 --to duplicate --notes "Duplicate of #35"
```

## What's Next

- **[Triage Workflow Example](/examples/triage-workflow)** — Full example with screenshots
- **[Core Workflow](/guide/workflow)** — From queue item to shipped feature
- **[Getting Started](/guide/getting-started)** — Complete getting started guide
