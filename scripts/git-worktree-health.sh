#!/usr/bin/env bash
# Verify every path registered in `git worktree list` is usable.
# Fails loudly when admin files point at removed trees (common after manual deletes).
#
# Usage: run from any worktree of the repo, or: bash scripts/git-worktree-health.sh
# Remediation: remove stale dirs, then `git worktree prune`, or re-add with `git worktree add`.

set -euo pipefail

verbose=0
if [[ "${1:-}" == "-v" || "${1:-}" == "--verbose" ]]; then
  verbose=1
fi

if ! root="$(git rev-parse --show-toplevel 2>/dev/null)"; then
  echo "GIT_HYGIENE FAIL: not inside a git repository (run from a checkout)." >&2
  exit 1
fi

cd "$root"

if [[ "$verbose" -eq 1 ]]; then
  echo "=== git worktree list (human) ==="
  git worktree list
  echo ""
fi

rc=0
paths=()
while IFS= read -r line || [[ -n "${line:-}" ]]; do
  case "$line" in
    worktree\ *)
      wtpath="${line#worktree }"
      paths+=("$wtpath")
      ;;
  esac
done < <(git worktree list --porcelain 2>/dev/null || true)

if [[ "${#paths[@]}" -eq 0 ]]; then
  echo "GIT_HYGIENE FAIL: could not parse worktree list (is this a git repo?)." >&2
  exit 1
fi

for wtpath in "${paths[@]}"; do
  if [[ ! -d "$wtpath" ]]; then
    echo "GIT_HYGIENE FAIL: registered worktree path missing on disk: $wtpath" >&2
    rc=1
    continue
  fi
  if ! git -C "$wtpath" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "GIT_HYGIENE FAIL: path is not a valid git worktree: $wtpath" >&2
    rc=1
    continue
  fi
  if ! git -C "$wtpath" rev-parse HEAD >/dev/null 2>&1; then
    echo "GIT_HYGIENE FAIL: cannot resolve HEAD in worktree: $wtpath" >&2
    rc=1
  fi
done

if [[ "$rc" -ne 0 ]]; then
  echo "" >&2
  echo "Remediation: remove stale worktree dirs if any, then from the main checkout run:" >&2
  echo "  git worktree prune" >&2
  echo "Re-add a tree with: git worktree add -b <branch> <path> [<start-point>]" >&2
fi

exit "$rc"
