---
audience: [developers]
---

# Contributing

How to set up your development environment, follow the development workflow, and contribute to AgilePlus.

## Prerequisites

- **Rust 1.85+** (edition 2024) — Language toolchain
  ```bash
  rustup default stable
  rustup update stable
  rustup component add rustfmt clippy
  # Verify version
  rustc --version   # must be >= 1.85.0
  ```

- **process-compose** — Service orchestration
  ```bash
  brew install F1bonacci/homebrew-tap/process-compose
  # or: cargo install process-compose
  ```

- **NATS server** — Event bus (required for integration tests)
  ```bash
  brew install nats-server
  nats-server --version   # verify install
  ```

- **Dragonfly** — Redis-compatible cache
  ```bash
  brew install dragonflydb/dragonfly/dragonfly
  ```

- **Bun** — JavaScript runtime for docs build
  ```bash
  curl -fsSL https://bun.sh/install | bash
  ```

- **Git 2.30+** — Version control (git worktree support required)
  ```bash
  git --version   # must be >= 2.30
  ```

- **SQLite 3.35+** — Default database (comes with macOS; check version)
  ```bash
  sqlite3 --version
  ```

**Optional (for full platform integration tests):**

- **Neo4j** — Dependency graph storage
  ```bash
  brew install neo4j
  neo4j start
  ```

- **MinIO** — Artifact storage
  ```bash
  brew install minio/stable/minio
  ```

## Setup

### Clone and Build

```bash
git clone https://github.com/KooshaPari/AgilePlus.git
cd AgilePlus

# Build entire workspace
cargo build --workspace

# Build just the CLI binary
cargo build -p agileplus-cli

# Verify the CLI runs
./target/debug/agileplus --version
# Output: agileplus 0.1.0

# Build docs
bun install
bun run docs:dev   # Start dev server at http://localhost:5173
```

### Verify Setup

```bash
# Run all workspace tests
cargo test --workspace

# Run domain crate tests only (no external deps needed)
cargo test -p agileplus-domain

# Check all clippy warnings (must be zero)
cargo clippy --workspace -- -D warnings

# Check formatting
cargo fmt --all --check

# Test docs build
bun run docs:build
```

### Start Platform Services (for integration tests)

```bash
# Start NATS, Dragonfly, and MinIO for integration testing
agileplus platform up --dev

# Or start services manually:
nats-server --jetstream &
dragonfly --port 6379 &

# Verify services are up
agileplus platform status
```

### Environment Configuration

Create `.env` for local development:

```bash
# .env (never commit this file)
RUST_LOG=debug
AGILEPLUS_DB=.agileplus/agileplus.db

# NATS (start with: nats-server --jetstream)
NATS_URL=nats://localhost:4222

# Dragonfly / Redis
DRAGONFLY_URL=redis://localhost:6379

# Plane.so (optional for sync testing)
PLANE_API_KEY=test_key_for_local_testing

# GitHub (optional for sync testing)
GITHUB_TOKEN=ghp_xxxx

# Claude Code agent
CLAUDE_CODE_PATH=claude

# OpenTelemetry (optional, for tracing)
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
```

## Development Workflow

### 1. Create a Feature Branch

```bash
git checkout main
git pull origin main
git checkout -b feat/descriptive-name
```

Branch naming conventions:
- Feature: `feat/feature-description`
- Bug fix: `fix/issue-description`
- Docs: `docs/topic`
- Refactor: `refactor/component`

Example:
```bash
git checkout -b feat/add-agent-retry-logic
```

### 2. Make Changes

Edit code in:
- Core logic: `crates/agileplus-core/src/`
- Engine: `crates/agileplus-engine/src/`
- CLI: `crates/agileplus-cli/src/`
- Docs: `docs/`

### 3. Write Tests

Add tests alongside your code:

```rust
// src/handlers/login.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_credentials_returns_token() {
        let user = User::new("test@example.com");
        let result = user.verify_password("password123");
        assert!(result.is_ok());
    }

    #[test]
    fn invalid_password_returns_error() {
        let user = User::new("test@example.com");
        let result = user.verify_password("wrong");
        assert!(result.is_err());
    }
}
```

Run tests frequently:

```bash
# All tests
cargo test

# Tests for a specific crate
cargo test -p agileplus-core

# Tests matching a pattern
cargo test handler

# Show println! output
cargo test -- --nocapture
```

### 4. Code Quality

Before committing, ensure code quality:

```bash
# Format
cargo fmt

# Lint
cargo clippy -- -D warnings

# Verify formatting
cargo fmt -- --check

# Full quality check
cargo test && cargo fmt && cargo clippy -- -D warnings
```

### 5. Commit

Follow commit message convention:

```
type(scope): description

feat(dispatch): add retry logic for agent sessions

- Add exponential backoff for failed agent calls
- Maximum 3 retries with 1-10 second delays
- Log retry attempts for debugging

Fixes #147
```

**Types**:
- `feat` — New feature
- `fix` — Bug fix
- `docs` — Documentation
- `refactor` — Code reorganization (no functionality change)
- `test` — Test-only changes
- `chore` — Build, dependency, or tooling changes

**Scope** (optional, but recommended):
- `dispatch` — Agent dispatch logic
- `sync` — Tracker synchronization
- `cli` — Command-line interface
- `core` — Core domain logic
- `engine` — Planning/orchestration engine

**Description**:
- Use imperative mood: "add", not "added"
- Capitalize first letter
- No period at end
- < 72 characters

**Body** (for non-trivial commits):
- Explain **why** (not **what** — code shows that)
- Reference issues: `Fixes #147`, `Relates to #150`
- Include migration instructions if needed

### 6. Push and Open PR

```bash
git push origin feat/add-agent-retry-logic
```

Then open PR on GitHub with:

**Title**: Match commit message format
```
feat(dispatch): add retry logic for agent sessions
```

**Description**: Explain the change

```markdown
## What's Changed

Added exponential backoff retry logic for failed agent dispatches.

## Why

Transient network errors can cause valid work to fail immediately.
Retries with backoff improve reliability without compromising UX.

## How to Test

Run dispatch tests:
```bash
cargo test dispatch
```

Manual testing:
1. Set up Plane API to return 500 error (or use mock)
2. Dispatch agent: `agileplus implement 001 --wp WP02`
3. Should retry 3 times, then fail gracefully

## Checklist

- [x] Tests pass locally
- [x] No clippy warnings
- [x] Code formatted
- [x] Commit messages explain why
- [x] Tests for new functionality
- [x] Docs updated (if needed)
```

## Code Style Guide

### Rust Conventions

**Follow rustfmt + clippy strictly**:

```bash
cargo fmt  # Auto-format all code
cargo clippy -- -D warnings  # No warnings allowed
```

### No `unwrap()` in Library Code

```rust
// ✗ Bad - panics if value is None
let id = value.unwrap();

// ✓ Good - propagates error
let id = value.ok_or(Error::MissingId)?;

// ✓ Good - handles error explicitly
let id = match value {
    Some(v) => v,
    None => return Err(Error::MissingId),
};
```

### Function Length

Keep functions under 50 lines (soft limit):

```rust
// ✗ Bad - 80 lines
fn process_spec(spec: &Spec) -> Result<Plan> {
    // ... lots of logic ...
}

// ✓ Good - breaks into smaller functions
fn process_spec(spec: &Spec) -> Result<Plan> {
    let requirements = extract_requirements(spec)?;
    let packages = decompose_requirements(&requirements)?;
    let dependencies = analyze_dependencies(&packages)?;
    Ok(Plan::new(packages, dependencies))
}
```

### Error Handling

Define custom errors:

```rust
#[derive(Debug)]
pub enum Error {
    SpecNotFound(String),
    InvalidSpec(String),
    DatabaseError(String),
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        Error::DatabaseError(e.to_string())
    }
}
```

Use `Result` type alias:

```rust
pub type Result<T> = std::result::Result<T, Error>;

pub fn read_spec(id: &FeatureId) -> Result<Spec> {
    // Returns Result<Spec, Error>
}
```

### Comments

Comment **why**, not **what**:

```rust
// ✗ Bad - repeats code
let result = multiply(x, y);  // multiply x by y

// ✓ Good - explains decision
// Use multiplication instead of addition because we're computing
// total impact across all features (not cumulative)
let result = multiply(x, y);
```

## Testing

### Unit Tests

Inline tests in the module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_validates_required_fields() {
        // Setup
        let spec = Spec::builder()
            .title("OAuth2")
            .build();

        // Test
        let result = spec.validate();

        // Assert
        assert!(result.is_ok());
    }
}
```

### Integration Tests

Cross-module tests in `tests/`:

```rust
// tests/integration/spec_to_plan.rs

#[test]
fn spec_generates_work_packages() {
    let spec = load_spec("tests/fixtures/oauth-spec.md");
    let plan = generate_plan(&spec).expect("should generate plan");

    assert_eq!(plan.work_packages.len(), 4);
    assert_eq!(plan.work_packages[0].name, "Provider Config");
}
```

Run integration tests:

```bash
cargo test --test integration
```

### Test Data

Store fixtures in `tests/fixtures/`:

```rust
// tests/integration/some_test.rs

#[test]
fn parses_valid_spec() {
    let spec_content = include_str!("../fixtures/valid-oauth-spec.md");
    let spec = Spec::parse(spec_content).expect("should parse");
    assert_eq!(spec.title, "OAuth2 Authentication");
}
```

## PR Review Checklist

Before opening PR, ensure:

- [ ] `cargo test --all` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo fmt --check` passes
- [ ] Tests added for new functionality
- [ ] No `unwrap()` in library code
- [ ] Commit messages explain **why**
- [ ] PR description explains context and testing

Example passing CI:

```bash
$ cargo test --all
   Compiling agileplus...
    Finished test [unoptimized + debuginfo]
     Running unittests src/lib.rs

test spec::tests::validates_required_fields ... ok
test engine::tests::generates_work_packages ... ok

test result: ok. 14 passed

$ cargo clippy -- -D warnings
warning: unused import
   --> src/main.rs:3:5
    |
3 | use std::collections::HashMap;
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^

Fixing...
Fixed. No warnings.

$ cargo fmt --check
Checking formatting... All files formatted correctly!
```

## Getting Help

**Issues**: Report bugs or suggest features at:
https://github.com/KooshaPari/AgilePlus/issues

**Discussions**: Ask questions or brainstorm at:
https://github.com/KooshaPari/AgilePlus/discussions

**Existing work**: Check planned features in:
`kitty-specs/` — specifications for in-progress or planned features

**Documentation**: Reference docs at:
https://docs.agileplus.dev

## Maintainer Support

If you need help:
1. Comment on the issue you're working on
2. Tag maintainers: `@KooshaPari`
3. Ask in discussions
4. Review architecture docs for context

## Crate-Specific Guidelines

### agileplus-domain (most important crate)

The domain crate must have **zero external I/O dependencies**. Adding `tokio`, `sqlx`, `reqwest`, or any crate that does I/O is not allowed:

```toml
# Allowed in agileplus-domain/Cargo.toml:
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
sha2 = { workspace = true }            # for hash chain
chrono = { workspace = true }          # for timestamps
thiserror = { workspace = true }       # for error types

# NOT allowed:
tokio = ...     # no async runtime in domain
sqlx = ...      # no database in domain
reqwest = ...   # no HTTP in domain
```

### agileplus-sqlite (storage adapter)

All queries must use compile-time checked `sqlx::query!` macros:

```rust
// ✓ Compile-time checked
let feature = sqlx::query_as!(
    FeatureRow,
    "SELECT id, slug, state FROM features WHERE slug = ?",
    slug
)
.fetch_optional(&self.pool)
.await?;

// ✗ Runtime-only string query (no type checking)
let feature = sqlx::query("SELECT * FROM features WHERE slug = ?")
    .bind(slug)
    .fetch_optional(&self.pool)
    .await?;
```

Run `cargo sqlx prepare` to regenerate the offline query cache when changing SQL.

### agileplus-cli (binary crate)

CLI commands must be thin — no business logic here. Commands are orchestrators that call the engine:

```rust
// ✓ Thin command handler
async fn handle_specify(args: SpecifyArgs, ctx: &Context) -> Result<()> {
    let result = ctx.engine.specify_feature(&args.title, &args.description).await?;
    println!("✓ Created feature: {}", result.slug);
    Ok(())
}

// ✗ Business logic in CLI handler
async fn handle_specify(args: SpecifyArgs, ctx: &Context) -> Result<()> {
    // Don't put domain logic here — it belongs in the engine
    let spec = validate_spec(&args.title)?;
    let hash = sha256(spec.as_bytes());
    let feature = Feature::new(&args.title, hash);
    ctx.storage.create_feature(&feature).await?;
    ...
}
```

## Running Benchmarks

Performance-critical paths have Criterion benchmarks:

```bash
# Run all benchmarks
cargo bench --workspace

# Run a specific benchmark
cargo bench -p agileplus-domain -- audit_chain

# Expected benchmark outputs:
# audit_chain/hash_entry     time:   [1.2 µs 1.3 µs 1.4 µs]
# audit_chain/verify_chain   time:   [45 µs 46 µs 47 µs] (100 entries)
# dependency_graph/kahn_sort time:   [8 µs 9 µs 10 µs] (50 WPs)
```

If a PR regresses performance by >10% on any benchmark, CI will flag it.

## Code of Conduct

Be respectful, inclusive, and collaborative. See [CODE_OF_CONDUCT.md](https://github.com/KooshaPari/AgilePlus/blob/main/CODE_OF_CONDUCT.md).

## Next Steps

- [Testing](testing.md) — Test patterns and coverage requirements
- [Extending](extending.md) — Adding storage adapters and CLI subcommands
- [Architecture Overview](../architecture/overview.md) — Understanding the crate structure
- [Environment Variables](../reference/env-vars.md) — Local development configuration
