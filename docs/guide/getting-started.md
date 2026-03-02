---
audience: [developers, agents, pms]
---

# Getting Started

Install AgilePlus and run your first spec-driven pipeline. This guide walks you through installation, project setup, and your first feature.

## Prerequisites

Before you begin, ensure you have:

- **Rust 2024 edition** (1.85+) — [Install rustup](https://rustup.rs/)
- **Git 2.x** — [Install git](https://git-scm.com/)
- **Git-capable editor** — VS Code, Vim, or your preferred code editor
- **Project tracker account** (optional but recommended):
  - [Plane.so](https://plane.so) for issue tracking
  - GitHub Issues for lighter workflows

## Installation

### Option 1: From Local Source

Clone the AgilePlus repository and install the CLI:

```bash
# Clone the repository
git clone https://github.com/phenotype-org/agileplus.git
cd agileplus

# Install the CLI binary locally
cargo install --path crates/agileplus-cli

# Verify installation
agileplus --version
```

### Option 2: From Crates.io (When Available)

```bash
cargo install agileplus
```

### Verify Installation

```bash
agileplus --version
# Output: agileplus 0.1.0

agileplus --help
# Shows all available commands
```

## Setting Up Your First Project

### 1. Create a Project Directory

```bash
mkdir my-awesome-project
cd my-awesome-project

# Initialize a git repository if you haven't already
git init
git config user.email "your-email@example.com"
git config user.name "Your Name"
```

### 2. Initialize AgilePlus

```bash
agileplus init
```

This command detects your project type and generates governance files:

```
Scanning project...
✓ Detected brownfield project (8 Rust files)
✓ Detected frameworks: Axum, Tokio
✓ Generating config files...
✓ Creating governance structure...

Generated:
  .kittify/config.toml
  .kittify/metadata.yaml
  .claudeignore
  CLAUDE.md
  AGENTS.md
```

**What was created:**

| File | Purpose |
|------|---------|
| `.kittify/config.toml` | Project configuration (language, frameworks, etc.) |
| `.kittify/metadata.yaml` | Version, timestamp, and platform metadata |
| `CLAUDE.md` | Governance instructions for AI code assistants |
| `AGENTS.md` | Cross-agent coordination rules |
| `.claudeignore` | Context optimization for assistants |
| `kitty-specs/` | Directory for feature specifications |

### 3. Review Project Config

Open `.kittify/config.toml` to see what was detected:

```toml
[project]
name = "my-awesome-project"
type = "brownfield"
root = "."

[vcs]
provider = "git"

[conventions]
languages = ["rust", "typescript"]
frameworks = ["axum", "vitepress"]

[sync]
# Optional: add tracker integration
# [sync.plane]
# workspace = "my-org"
# project = "my-project"
# api_key = "${PLANE_API_KEY}"
```

## Your First Feature: Complete Walkthrough

This section walks you through creating, planning, and implementing your first feature using AgilePlus.

### Step 1: Create a Specification

Specifications are the source of truth for features. They capture requirements, user scenarios, and success criteria.

```bash
agileplus specify --title "User Authentication" \
  --description "Add email/password login and signup"
```

You'll enter an interactive specification interview:

```
Feature name: User Authentication
What is the primary value to users?
> Users can sign up and log in with email and password

Who are the key users?
> Application users

What's the scope?
> Login endpoint, signup endpoint, password hashing
```

**Output:**

```
Generated specification:
  kitty-specs/001-user-authentication/spec.md
  kitty-specs/001-user-authentication/meta.json

Spec markdown:
- 5 user scenarios
- 8 functional requirements
- 4 success criteria
```

View your generated spec:

```bash
cat kitty-specs/001-user-authentication/spec.md
```

### Step 2: Clarify Ambiguities

If your specification has any unclear areas, use clarify to generate targeted questions:

```bash
agileplus clarify 001
```

Example output:

```
Analyzing spec for gaps...

Q1: Password Requirements
Question: What are the password complexity requirements?
Proposed answer: Minimum 8 characters, at least 1 uppercase, 1 number

Q2: Session Duration
Question: How long should user sessions last?
Proposed answer: 24 hours with refresh token support

Accept proposed answers? [Y/n]
```

After accepting, the spec is updated with concrete answers.

### Step 3: Research Feasibility

Before planning, research your codebase to understand existing patterns and dependencies:

```bash
agileplus research 001
```

This scans your codebase and produces:

```
kitty-specs/001-user-authentication/research/
├── codebase-scan.md       # Existing auth code, models
├── feasibility.md         # Risks, dependencies, complexity estimate
└── decisions.md           # Technical decisions
```

Example research output:

```markdown
## Codebase Scan

### Existing Auth Code
- `src/auth/middleware.rs` - Request guard middleware
- `src/models/user.rs` - User model with email field

### Frameworks
- Axum for web framework
- SQLx for database access
- Argon2 for password hashing (already in Cargo.lock)

## Feasibility
- Complexity: Medium
- Risks: Password reset flow not in scope
- Integration: Compatible with existing middleware
```

### Step 4: Generate Implementation Plan

Create a detailed plan that breaks down the work into manageable pieces:

```bash
agileplus plan 001
```

Output:

```
kitty-specs/001-user-authentication/plan.md

## Architecture Decisions
1. Use Argon2 for password hashing (industry standard)
2. JWT tokens for session management
3. RESTful endpoints for signup/login

## File Changes
| File | Action | Purpose |
|------|--------|---------|
| src/auth/handlers.rs | Create | Login/signup handlers |
| src/auth/models.rs | Create | Auth request/response types |
| src/db/migrations/001_users.sql | Create | Users table |

## Build Sequence
1. Create database schema
2. Create User model
3. Implement password hashing
4. Implement signup endpoint
5. Implement login endpoint
6. Add tests
```

### Step 5: Generate Work Packages

Break the plan into parallel-safe work packages:

```bash
agileplus tasks 001
```

Output:

```
kitty-specs/001-user-authentication/tasks/
├── WP01-database.md        # Database schema and migrations
├── WP02-models.md          # User model and types
├── WP03-signup.md          # Signup endpoint
├── WP04-login.md           # Login endpoint
└── WP05-tests.md           # Integration tests
```

View work packages:

```bash
cat kitty-specs/001-user-authentication/tasks/WP01-database.md
```

### Step 6: Start Implementation

Create an isolated worktree for work package WP01:

```bash
agileplus implement WP01
```

This creates a clean workspace:

```
.worktrees/001-authentication-WP01/
├── src/
├── tests/
├── Cargo.toml
└── .git
```

Make changes in the worktree:

```bash
cd .worktrees/001-authentication-WP01

# Create database migrations
mkdir -p migrations
cat > migrations/001_create_users.sql << 'EOF'
CREATE TABLE users (
  id INTEGER PRIMARY KEY,
  email TEXT UNIQUE NOT NULL,
  password_hash TEXT NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
EOF

# Commit your work
git add -A
git commit -m "feat(WP01): create users table schema"
```

### Step 7: Move to Review

When implementation is complete:

```bash
agileplus move WP01 --to for_review
```

### Step 8: Review and Approve

As a reviewer, check the work:

```bash
agileplus review WP01
```

If everything looks good:

```bash
agileplus move WP01 --to done
```

This automatically unblocks dependent work packages.

### Step 9: Accept the Feature

When all work packages are done:

```bash
agileplus accept 001
```

This verifies that all requirements from the spec are met and success criteria are achieved.

### Step 10: Merge to Main

Integrate all work packages into your main branch:

```bash
agileplus merge 001
```

This:
- Merges all WP branches into main
- Removes worktrees
- Cleans up branches
- Updates your tracker (if configured)

**Verification:**

```bash
# You're now on main
git log --oneline | head -5

# Should show the merged feature commits
# feat(WP05): add integration tests for auth
# feat(WP04): implement login endpoint
# feat(WP03): implement signup endpoint
# feat(WP02): add user models
# feat(WP01): create users table schema
```

## Configuration (Optional)

### Connect to Plane.so

To sync work packages with Plane.so:

```bash
# Set API key
export PLANE_API_KEY="your_plane_api_key_here"

# Update config
cat >> .kittify/config.toml << 'EOF'
[sync.plane]
workspace = "my-org"
project = "my-awesome-project"
api_key = "${PLANE_API_KEY}"
EOF

# Run sync
agileplus sync
```

### Connect to GitHub Issues

```bash
# Set GitHub token
export GITHUB_TOKEN="your_github_token_here"

# Update config
cat >> .kittify/config.toml << 'EOF'
[sync.github]
repo = "your-username/my-awesome-project"
token = "${GITHUB_TOKEN}"
EOF

# Run sync
agileplus sync
```

## Troubleshooting

### Installation Issues

**Error: `cargo not found`**

Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

**Error: `git not found`**

Install Git from [git-scm.com](https://git-scm.com/)

### Project Setup Issues

**Error: `agileplus: command not found`**

Make sure installation succeeded:
```bash
cargo install --path crates/agileplus-cli --force
which agileplus
```

**Error: `Permission denied` when running agileplus**

Fix permissions:
```bash
chmod +x ~/.cargo/bin/agileplus
```

### Workflow Issues

**Error: `No specs found`**

Make sure you've run `agileplus specify` first. Specs are stored in `kitty-specs/`.

**Error: `Worktree creation failed`**

Ensure your repository is clean:
```bash
git status  # Should show no uncommitted changes
git stash   # If needed
```

## What's Next

- **[Quick Start](/guide/quick-start)** — 5-minute path to your first spec
- **[Project Initialization](/guide/init)** — Detailed init options and configuration
- **[Core Workflow](/guide/workflow)** — Deep dive into each pipeline stage
- **[Full Pipeline Example](/examples/full-pipeline)** — Complete end-to-end walkthrough
- **[Spec-Driven Development](/concepts/spec-driven-dev)** — Understand the philosophy
- **[CLI Reference](/reference/cli)** — All commands and flags
