#!/usr/bin/env bash
set -euo pipefail

# Quality-gate script for AgilePlus CI.
# Usage: ./scripts/quality-gate.sh verify

CMD="${1:-verify}"

case "$CMD" in
  verify)
    echo "Running quality gate verification..."

    echo "--- Rust format check ---"
    cargo fmt --all -- --check

    echo "--- Clippy ---"
    cargo clippy --workspace -- -D warnings

    echo "--- Rust build ---"
    cargo build --workspace

    echo "--- Rust tests ---"
    cargo test --workspace

    echo "--- Rust docs ---"
    cargo doc --no-deps --workspace

    if command -v taplo >/dev/null 2>&1; then
      echo "--- TOML format check ---"
      taplo format --check Cargo.toml rust/Cargo.toml python/pyproject.toml
    fi

    if [ -d python ]; then
      echo "--- Python lint ---"
      cd python
      if command -v uvx >/dev/null 2>&1; then
        uvx ruff format --check .
        uvx ruff check .
      fi
      cd ..
    fi

    echo "Quality gate passed."
    ;;
  *)
    echo "Unknown command: $CMD"
    exit 1
    ;;
esac
