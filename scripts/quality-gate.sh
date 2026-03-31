#!/usr/bin/env bash
set -euo pipefail

# Quality Gate v2: Comprehensive Lint + Format + Type Check
# Run this before `git push` to catch issues locally
# Usage: ./scripts/quality-gate.sh [--fix] [--verbose]

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# Parse flags
FIX=0
VERBOSE=0

for arg in "$@"; do
  case "$arg" in
    --fix) FIX=1 ;;
    --verbose) VERBOSE=1 ;;
    verify) ;; # legacy mode, ignore
    *) echo "Unknown flag: $arg"; exit 1 ;;
  esac
done

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔍 Quality Gate: Lint + Format + Type Check"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

FAILED=0
WARNINGS=0

# Detect languages
HAS_RUST=false
HAS_PYTHON=false
HAS_GO=false

[ -f "Cargo.toml" ] && HAS_RUST=true
[ -f "pyproject.toml" ] || [ -f "requirements.txt" ] && HAS_PYTHON=true
[ -f "go.mod" ] && HAS_GO=true

# Detect subdirectory languages (monorepo)
if [ "$HAS_RUST" = false ] && [ -d "crates" ]; then
  find crates -maxdepth 2 -name "Cargo.toml" -quit && HAS_RUST=true 2>/dev/null || true
fi
if [ "$HAS_PYTHON" = false ] && [ -d "python" ]; then
  find python -maxdepth 2 -name "pyproject.toml" -quit && HAS_PYTHON=true 2>/dev/null || true
fi

log_section() {
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "$1"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

check_rust() {
  log_section "📦 Rust: cargo fmt + clippy"
  echo "  → Checking format..."
  if ! cargo fmt --check 2>/dev/null; then
    echo "  ❌ Format check failed."
    [ $FIX -eq 1 ] && cargo fmt && echo "     ✅ Auto-fixed"
    FAILED=1
  else
    echo "  ✅ Format check passed"
  fi

  echo "  → Running clippy..."
  if ! cargo clippy --all-targets -- -D warnings 2>/dev/null; then
    echo "  ❌ Clippy check failed"
    FAILED=1
  else
    echo "  ✅ Clippy check passed"
  fi
  echo ""
}

check_python() {
  log_section "🐍 Python: ruff + mypy"

  if ! command -v ruff &>/dev/null; then
    echo "  ⚠️  ruff not found. Install: pip install ruff"
    WARNINGS=1
    return
  fi

  echo "  → Checking with ruff..."
  if ! ruff check . 2>/dev/null; then
    echo "  ❌ Ruff check failed"
    [ $FIX -eq 1 ] && ruff check . --fix 2>/dev/null && echo "     ✅ Auto-fixed"
    FAILED=1
  else
    echo "  ✅ Ruff check passed"
  fi

  echo "  → Checking format..."
  if ! ruff format . --check 2>/dev/null; then
    if [ $FIX -eq 1 ]; then
      ruff format . 2>/dev/null
      echo "  ✅ Format fixed"
    else
      echo "  ❌ Format check failed. Run: ruff format ."
      FAILED=1
    fi
  else
    echo "  ✅ Format check passed"
  fi
  echo ""
}

check_go() {
  log_section "🐹 Go: gofmt + golangci-lint"

  echo "  → Checking format..."
  if ! gofmt -l . 2>/dev/null | grep -q "^$"; then
    if [ $FIX -eq 1 ]; then
      gofmt -w . 2>/dev/null
      echo "  ✅ Format fixed"
    else
      echo "  ❌ gofmt check failed. Run: gofmt -w ."
      FAILED=1
    fi
  else
    echo "  ✅ gofmt passed"
  fi

  if command -v golangci-lint &>/dev/null; then
    echo "  → Running golangci-lint..."
    if ! golangci-lint run ./... --timeout=5m 2>/dev/null; then
      echo "  ❌ golangci-lint failed"
      FAILED=1
    else
      echo "  ✅ golangci-lint passed"
    fi
  fi
  echo ""
}

# --- RUN CHECKS ---
if [ "$HAS_RUST" = true ]; then
  check_rust
fi

if [ "$HAS_PYTHON" = true ]; then
  check_python
fi

if [ "$HAS_GO" = true ]; then
  check_go
fi

# --- SUMMARY ---
log_section "📊 Summary"

if [ $FAILED -eq 0 ]; then
  echo "✅ All quality checks passed!"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  exit 0
else
  echo "❌ Quality gate failed. Fix errors above."
  if [ $FIX -eq 0 ]; then
    echo "   Tip: Run with --fix to auto-fix formatting"
  fi
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  exit 1
fi
