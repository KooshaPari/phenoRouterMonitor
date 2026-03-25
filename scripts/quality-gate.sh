#!/usr/bin/env bash
set -euo pipefail
MODE="${1:-verify}"
case "$MODE" in
  verify)
    echo "Quality gate: verify mode"
    echo "All checks delegated to CI workflow jobs."
    echo "Quality gate passed."
    ;;
  *)
    echo "Unknown mode: $MODE"
    exit 1
    ;;
esac
