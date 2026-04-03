#!/bin/bash
set -e

cmd="$1"

case "$cmd" in
  verify)
    echo "Running verification checks..."
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test --all
    echo "All checks passed!"
    ;;
  lint)
    echo "Running linter..."
    cargo clippy --all-targets --all-features -- -D warnings
    ;;
  format)
    echo "Checking formatting..."
    cargo fmt --check
    ;;
  test)
    echo "Running tests..."
    cargo test --all
    ;;
  *)
    echo "Unknown command: $cmd"
    echo "Usage: $0 {verify|lint|format|test}"
    exit 1
    ;;
esac
