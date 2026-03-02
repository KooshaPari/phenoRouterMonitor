---
audience: [developers, agents]
---

# Quick Start

Get AgilePlus running and create your first feature specification in under 5 minutes.

## Prerequisites

- Rust toolchain (`rustup`) — [Install](https://rustup.rs/)
- Git 2.x — [Install](https://git-scm.com/)
- An existing project (or create a new one)

## 1. Install (1 minute)

```bash
# Clone and install the CLI
git clone https://github.com/phenotype-org/agileplus.git
cd agileplus
cargo install --path crates/agileplus-cli

# Verify installation
agileplus --version
```

## 2. Initialize Your Project (1 minute)

Navigate to your project and run:

```bash
cd my-existing-project
agileplus init
```

This auto-detects your language, frameworks, and generates:

```
✓ Created .kittify/config.toml
✓ Created AGENTS.md
✓ Created CLAUDE.md
✓ Created kitty-specs/ directory
```

## 3. Create a Feature Specification (2 minutes)

```bash
agileplus specify --title "Add Email Notifications" \
  --description "Send emails when users take actions"
```

During the interview, answer these questions:

```
Project: Add Email Notifications
Primary value: Users receive notifications via email
Key users: Application users
Scope: Send email on user signup, new comment
```

**Output:**

```
✓ Generated kitty-specs/001-email-notifications/spec.md
✓ Feature ready to plan

Next: agileplus clarify 001
     agileplus plan 001
```

## 4. Create a Plan (1 minute)

```bash
agileplus plan 001
```

This generates an implementation blueprint:

```
✓ Generated kitty-specs/001-email-notifications/plan.md
✓ Identified 3 files to create/modify
✓ Created dependency graph

File Changes:
- src/email/mod.rs (CREATE)
- src/email/templates.rs (CREATE)
- src/handlers/signup.rs (MODIFY)
```

## 5. Generate Work Packages (within 5 minutes)

```bash
agileplus tasks 001
```

This breaks the work into parallel work packages:

```
✓ Created WP01-email-setup
✓ Created WP02-templates
✓ Created WP03-integration

Ready to implement:
agileplus implement WP01
```

## Start Implementing

```bash
# Create an isolated worktree for WP01
agileplus implement WP01

# Make changes in the worktree
cd .worktrees/001-email-notifications-WP01
# ... write code ...
git commit -m "feat(WP01): email service setup"

# Move to review when done
agileplus move WP01 --to for_review
```

## Complete the Feature

```bash
# After all WPs are reviewed and done
agileplus accept 001        # Validate spec completion
agileplus merge 001         # Merge to main
```

## From 5 Minutes to Shipping

The quick flow above gets you from zero to a plan in 5 minutes. Full implementation depends on complexity:

- **Simple features (1 WP)**: Add 10-15 minutes
- **Medium features (2-3 WPs)**: Add 30-60 minutes
- **Complex features (4+ WPs)**: Add hours to days (parallel work)

Each work package runs independently, so multiple developers can work in parallel.

## Key Commands Reference

```bash
# Specification phase
agileplus specify "description"    # Create a new feature spec
agileplus clarify 001              # Identify gaps in the spec
agileplus research 001             # Scan codebase for patterns

# Planning phase
agileplus plan 001                 # Create implementation plan
agileplus tasks 001                # Generate work packages

# Implementation phase
agileplus implement WP01           # Start work on a package
agileplus move WP01 --to for_review  # Submit for review

# Review & Merge
agileplus review WP01              # Check work quality
agileplus move WP01 --to done      # Approve
agileplus accept 001               # Validate feature complete
agileplus merge 001                # Merge to main
```

## Common Questions

**Q: Can I work on multiple packages in parallel?**

Yes. AgilePlus creates isolated worktrees, so you can run multiple `implement` commands for different work packages. They won't interfere with each other.

```bash
# Terminal 1
agileplus implement WP01
cd .worktrees/001-feature-WP01
# ... code ...

# Terminal 2 (different directory)
agileplus implement WP02
cd .worktrees/001-feature-WP02
# ... code ...
```

**Q: What if I skip clarify or research?**

You don't have to run every step. Quick features can skip straight to plan:

```bash
agileplus specify "title"
agileplus plan 001          # Skip clarify & research
agileplus tasks 001
agileplus implement WP01
```

**Q: How do I add my project tracker?**

Edit `.kittify/config.toml` to add Plane.so or GitHub Issues sync:

```toml
[sync.plane]
workspace = "my-org"
project = "my-project"
api_key = "${PLANE_API_KEY}"
```

Then sync:

```bash
agileplus sync
```

**Q: What if implementation reveals issues?**

If research or planning missed something, you can backtrack:

```bash
agileplus clarify 001       # Identify new gaps
agileplus plan 001          # Re-plan
agileplus tasks 001         # Re-task
```

Then continue implementing.

## What's Next?

- **[Getting Started](/guide/getting-started)** — Full detailed walkthrough
- **[Core Workflow](/guide/workflow)** — Understand all 7 phases
- **[Work with Trackers](/guide/sync)** — Sync with Plane.so or GitHub
- **[Configuration](/guide/configuration)** — Customize for your team
- **[Spec-Driven Development](/concepts/spec-driven-dev)** — Learn the philosophy
