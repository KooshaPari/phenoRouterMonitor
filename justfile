# phenotype-infrakit — Archived
# Tier-0 task runner for maintenance and verification

set dotenv-load := false

# ── Workspace ────────────────────────────────────────────────────────────────

# Run all workspace checks
check-all: check fmt clippy deny test

# Cargo check (fast compilation check)
check:
  cargo check --workspace

# Format check
fmt:
  cargo fmt --all --check

# Clippy lint
clippy:
  cargo clippy --workspace -- -D warnings

# Run all tests
test:
  cargo test --workspace

# Cargo deny audit
deny:
  cargo deny check

# Clean build artifacts
clean:
  cargo clean

# ── Single-crate shortcuts ──────────────────────────────────────────────────

check-bdd:
  cargo check -p phenotype-bdd

check-compliance:
  cargo check -p phenotype-compliance-scanner

# ── Info ─────────────────────────────────────────────────────────────────────

# Show workspace members
members:
  cargo metadata --format-version 1 --no-deps | jq -r '.packages[] | "\(.name) (\(.manifest_path))"'
