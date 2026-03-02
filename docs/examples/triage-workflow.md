---
audience: [developers, pms]
---

# Triage Workflow Example

Walk through classifying incoming items and managing the backlog. Triage is the first step for any new request, turning unstructured feedback into actionable work.

## Scenario

Your team receives support tickets, feature requests, and bug reports throughout the day. AgilePlus triages them automatically into the backlog, prioritizing by type and urgency.

## Incoming Items

### Bug Report (High Priority)

A customer files a critical bug:

```bash
agileplus triage "users can't log in after password reset - getting 500 error"
```

```
Analyzing input: "users can't log in after password reset - getting 500 error"

Intent Detection:
  - Primary: Bug (92% confidence)
  - Keywords: error, can't, 500
  - Severity signal: error code + user-facing

Classification
  Type: Bug
  Severity: High
  Priority Score: 95/100

Added to backlog as item #1 [Bug] (Priority: High)
```

The triage system detects:
- **Type**: Bug (broken functionality)
- **Severity**: High (login is critical, affects multiple users)
- **Urgency**: Critical (prevents user access)

### Feature Request (Medium Priority)

```bash
agileplus triage "would be great to have dark mode in the dashboard"
```

```
Analyzing input: "would be great to have dark mode in the dashboard"

Intent Detection:
  - Primary: Feature (78% confidence)
  - Secondary: Enhancement (15%)
  - Keywords: feature, have, dashboard
  - Qualifier: "would be great" indicates nice-to-have, not blocking

Classification
  Type: Feature
  Scope: UI enhancement
  Priority Score: 58/100

Added to backlog as item #2 [Feature] (Priority: Medium)
```

### Vague Idea (Low Priority)

```bash
agileplus triage "what if we integrated with Slack for notifications"
```

```
Analyzing input: "what if we integrated with Slack for notifications"

Intent Detection:
  - Primary: Idea (81% confidence)
  - Secondary: Enhancement (12%)
  - Keywords: what if, integrated, Slack
  - Signal: Speculative language ("what if") indicates early-stage thinking

Classification
  Type: Idea
  Scope: Third-party integration
  Priority Score: 35/100

Added to backlog as item #3 [Idea] (Priority: Low)
  → Requires stakeholder validation before planning
```

### Maintenance Task (Medium Priority)

```bash
agileplus triage "need to update Node from 18 to 22"
```

```
Analyzing input: "need to update Node from 18 to 22"

Intent Detection:
  - Primary: Task (88% confidence)
  - Keywords: update, version bump, maintenance
  - Type signal: Technical maintenance work

Classification
  Type: Task
  Category: Maintenance
  Priority Score: 62/100

Added to backlog as item #4 [Task] (Priority: Medium)
  → Technical debt, affects performance
```

### Duplicate Detection

```bash
agileplus triage "dashboard dark mode is needed"
```

```
Analyzing input: "dashboard dark mode is needed"

Possible duplicate detected:
  Similar to item #2: "would be great to have dark mode in the dashboard"

  Similarity: 94%

Merge options:
  1. agileplus queue merge 2 --with-new  (combines into #2)
  2. agileplus queue add               (keeps separate)

What would you like to do? [merge/add]: merge

✓ Merged into item #2
  Added note: "Also requested by [user]"
```

## Review the Queue

View all incoming items by priority:

```bash
agileplus queue list
```

```
Backlog Queue (sorted by priority)

#1  [Bug]     users can't log in after password reset        Priority: HIGH    Status: new
    Severity: Critical  |  Submitted: 2 hours ago  |  Affects: users

#2  [Feature] dark mode in the dashboard                     Priority: MEDIUM  Status: new
    Scope: UI enhancement  |  Submitted: 4 hours ago  |  Votes: 3

#4  [Task]    update Node from 18 to 22                      Priority: MEDIUM  Status: new
    Category: Maintenance  |  Submitted: 1 day ago  |  Blocks: performance work

#3  [Idea]    integrate with Slack for notifications         Priority: LOW     Status: new
    Scope: Integration  |  Submitted: 2 days ago  |  Requires: validation
```

### Filter by Type

```bash
# Only bugs
agileplus queue list --type bug

# Open bugs and features
agileplus queue list --type bug --type feature --status new

# High-priority items
agileplus queue list --priority high --priority critical
```

### Get Details on a Queue Item

```bash
agileplus queue show 1
```

```
Item #1: Login 500 Error After Password Reset

Type: Bug
Status: new
Priority: HIGH (95/100)
Severity: CRITICAL
Submitted: 2 hours ago
Source: customer support ticket #CS-4821

Description:
  "users can't log in after password reset - getting 500 error"

Analysis:
  - Affects: Login flow, password reset flow
  - Impact: Critical (blocks user access)
  - Reproducibility: Confirmed in production
  - Workaround: None known

Related Items:
  - None

Suggested Action:
  → Immediate spec creation + emergency release track
  → Estimated effort: 1 WP, ~1 day
```

## Work the Queue

### Pop High-Priority Item

```bash
agileplus queue pop
```

```
Popped item #1: [Bug] users can't log in after password reset

Priority: HIGH (95/100)
Effort estimate: 1–2 work packages
Time to fix: ~1 day

Ready to specify? Type 'agileplus specify' with title and description.
```

### Convert to Specification

Once triaged, move to planning:

```bash
agileplus specify \
  --title "Fix: Login fails with 500 error after password reset" \
  --description "Investigate and fix root cause of 500 error when users attempt to log in immediately after resetting their password. Affects all users." \
  --queue-item 1
```

```
Creating spec from queue item #1...

Spec created: 002-login-500-fix
  kitty-specs/002-login-500-fix/spec.md

Queue item #1 marked as: in_progress

You can now plan and implement:
  agileplus plan 002
  agileplus implement 002
```

## Batch Processing

### Triage Multiple Items at Once

Process a batch from email, Slack, or support tickets:

```bash
# Create a file: incoming.txt
cat > /tmp/incoming.txt <<EOF
Bug: dashboard crashes on large dataset export
Feature: add export to PDF format
Task: migrate database to new schema
Idea: AI-powered test generation
EOF

agileplus triage --batch /tmp/incoming.txt
```

```
Processing batch (4 items)...

✓ [Bug]     dashboard crashes on large dataset export      Priority: HIGH
✓ [Feature] add export to PDF format                       Priority: MEDIUM
✓ [Task]    migrate database to new schema                 Priority: MEDIUM
✓ [Idea]    AI-powered test generation                     Priority: LOW

Added 4 items to backlog
```

## Override Classification

If the automatic classifier gets it wrong, override it:

```bash
agileplus triage "refactor the auth module to be more testable" --type task
```

```
Input: "refactor the auth module to be more testable"

Auto-detected: Feature (65% confidence)
Your override: Task

Added to backlog as item #5 [Task] (Priority: Low)
  Note: Classification was overridden (auto: Feature → actual: Task)
```

Use `--type` to specify exactly:
- `bug` — broken functionality
- `feature` — new capability
- `task` — maintenance, refactoring, or housekeeping
- `idea` — speculative, needs validation

## Weekly Queue Review

Run a weekly status check to monitor backlog health:

```bash
agileplus queue health
```

```
Backlog Health Report (week of Feb 24 – Mar 2)

Items Added: 12
Items Completed: 4
Items In Progress: 3
Items Waiting: 5

Age Analysis:
  New (< 24h):         3 items
  Recent (1–7 days):   6 items
  Aging (7–30 days):   2 items
  Stale (> 30 days):   1 item ⚠

Priority Distribution:
  Critical: 1 item (8%)
  High: 2 items (17%)
  Medium: 6 items (50%)
  Low: 3 items (25%)

Recommendations:
  → 1 stale item needs closure decision
  → 5 items waiting for clarification
  → Process 2 high-priority items this week

Churn Rate: 33% (items completed vs. added)
Avg. time to spec: 2.3 days
Avg. time in progress: 4.1 days
```

## Integration with Issue Trackers

Sync with GitHub, Plane, or other tools:

```bash
# Pull new issues from GitHub
agileplus sync github --direction pull

# This automatically triages GitHub issues into the queue
# Labels on GitHub map to priority and type:
#   type: bug       → Type: Bug
#   priority: high  → Priority: HIGH
```

```
Syncing with GitHub...

Fetched 7 new issues
  5 already in queue (duplicate check passed)
  2 new items added to queue

Added to backlog:
  #6  [Bug]     timeout on large API requests
  #7  [Feature] webhook support for webhooks
```

## Key Takeaways

1. **Early classification** — Triage immediately when items arrive
2. **Duplicate detection** — Merge similar items to avoid work bifurcation
3. **Priority scoring** — Automatic detection of severity and urgency
4. **Queue health** — Monitor aging items and bottlenecks
5. **Integration** — Pull from issue trackers automatically
6. **Override when needed** — Auto-classification is good, but human judgment wins
