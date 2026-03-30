#!/usr/bin/env bash
# Run ripgrep with conservative excludes for Phenotype monorepo audits (agents / CI).
# Avoids scanning build artifacts, nested worktree hubs, and large vendored trees.
#
# Usage (from repo root):
#   bash scripts/audit_scope.sh -n "pattern" crates libs
#   bash scripts/audit_scope.sh --files-with-matches Error crates --type rust
#
# Override root: REPO_ROOT=/path/to/repos bash scripts/audit_scope.sh ...

set -euo pipefail

ROOT="${REPO_ROOT:-}"
if [[ -z "$ROOT" ]]; then
  ROOT=$(git rev-parse --show-toplevel 2>/dev/null || true)
fi
if [[ -z "$ROOT" ]] || [[ ! -d "$ROOT" ]]; then
  echo "ERROR: set REPO_ROOT or run inside a git checkout" >&2
  exit 1
fi
cd "$ROOT"

if ! command -v rg >/dev/null 2>&1; then
  echo "ERROR: ripgrep (rg) not found" >&2
  exit 1
fi

exec rg \
  --glob '!**/target/**' \
  --glob '!**/.git/**' \
  --glob '!**/node_modules/**' \
  --glob '!**/.venv/**' \
  --glob '!**/dist/**' \
  --glob '!**/docs-dist/**' \
  --glob '!**/worktrees/**' \
  --glob '!**/*-wtrees/**' \
  --glob '!**/isolated/**' \
  "$@"
