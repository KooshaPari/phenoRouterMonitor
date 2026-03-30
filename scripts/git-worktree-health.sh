#!/usr/bin/env bash
# Validate git worktree metadata: paths exist, gitdir targets resolve, optional orphan hints.
# Repository root: Phenotype/repos (run from any directory inside the repo).
#
# Usage:
#   bash scripts/git-worktree-health.sh          # strict — exit 1 on any failure
#   bash scripts/git-worktree-health.sh -v       # verbose
#   bash scripts/git-worktree-health.sh -w       # warn on dirty worktrees (non-fatal)
#   GIT_HYGIENE_ORPHAN_SCAN=1 bash scripts/git-worktree-health.sh -v
#
# See: docs/worklogs/INACTIVE_FOLDERS.md (automation spec)

set -uo pipefail

VERBOSE=0
WARN_DIRTY=0
ORPHAN_SCAN="${GIT_HYGIENE_ORPHAN_SCAN:-0}"

usage() {
  echo "Usage: $0 [-v] [-w]" >&2
  echo "  -v  verbose (prune dry-run, extra paths)" >&2
  echo "  -w  warn if a worktree has unstaged/uncommitted changes (exit still 0 unless other errors)" >&2
  echo "  GIT_HYGIENE_ORPHAN_SCAN=1  extra checks under .git/worktrees" >&2
  exit 2
}

while getopts "vwh" opt; do
  case "$opt" in
    v) VERBOSE=1 ;;
    w) WARN_DIRTY=1 ;;
    h) usage ;;
    *) usage ;;
  esac
done

if ! GIT_TOPLEVEL=$(git rev-parse --show-toplevel 2>/dev/null); then
  echo "ERROR: not inside a git repository (git rev-parse --show-toplevel failed)" >&2
  exit 1
fi

cd "$GIT_TOPLEVEL" || exit 1

if ! COMMON_REL=$(git rev-parse --git-common-dir 2>/dev/null); then
  echo "ERROR: git rev-parse --git-common-dir failed" >&2
  exit 1
fi

# Resolve to absolute path for comparisons
COMMON="$GIT_TOPLEVEL/$COMMON_REL"
case "$COMMON_REL" in
  /*) COMMON="$COMMON_REL" ;;
esac

errors=0
warns=0

logv() {
  if [[ "$VERBOSE" -eq 1 ]]; then
    echo "[verbose] $*" >&2
  fi
}

echo "== git worktree health =="
echo "toplevel: $GIT_TOPLEVEL"
echo "git-common-dir: $COMMON"
logv "prune dry-run:"
if [[ "$VERBOSE" -eq 1 ]]; then
  git worktree prune --dry-run 2>&1 || true
fi

# --- Porcelain: every registered worktree path must exist ---
current_path=""
while IFS= read -r line || [[ -n "$line" ]]; do
  case "$line" in
    worktree\ *)
      current_path="${line#worktree }"
      if [[ ! -d "$current_path" ]]; then
        echo "ERROR: worktree path does not exist: $current_path" >&2
        errors=$((errors + 1))
      else
        logv "OK worktree path: $current_path"
        if [[ "$WARN_DIRTY" -eq 1 ]]; then
          if [[ -n "$(git -C "$current_path" status --porcelain 2>/dev/null)" ]]; then
            echo "WARN: dirty worktree: $current_path" >&2
            warns=$((warns + 1))
          fi
        fi
      fi
      ;;
  esac
done < <(git worktree list --porcelain 2>/dev/null)

# --- Metadata: each worktrees/<id>/gitdir must point to an existing file ---
wt_meta="$COMMON/worktrees"
if [[ -d "$wt_meta" ]]; then
  for meta in "$wt_meta"/*; do
    [[ -e "$meta" ]] || continue
    [[ -d "$meta" ]] || continue
    gd="$meta/gitdir"
    if [[ -f "$gd" ]]; then
      # single line, path to the worktree's .git *file*
      gdir_target=$(tr -d '\r\n' <"$gd")
      if [[ -z "$gdir_target" ]]; then
        echo "ERROR: empty $gd" >&2
        errors=$((errors + 1))
        continue
      fi
      if [[ ! -e "$gdir_target" ]]; then
        echo "ERROR: gitdir target missing: $gdir_target (from $gd)" >&2
        errors=$((errors + 1))
      else
        logv "OK gitdir: $gd -> $gdir_target"
      fi
      root_dir=$(dirname "$gdir_target")
      if [[ ! -d "$root_dir" ]]; then
        echo "ERROR: worktree root directory missing: $root_dir (from $gd)" >&2
        errors=$((errors + 1))
      fi
    fi
  done
else
  logv "no $wt_meta (single worktree clone?)"
fi

# --- Optional orphan-style scan: stale admin dirs vs live gitdir ---
if [[ "$ORPHAN_SCAN" == "1" ]]; then
  logv "GIT_HYGIENE_ORPHAN_SCAN=1: checking admin dirs"
  if [[ -d "$wt_meta" ]]; then
    for meta in "$wt_meta"/*; do
      [[ -d "$meta" ]] || continue
      gd="$meta/gitdir"
      [[ -f "$gd" ]] || continue
      gdir_target=$(tr -d '\r\n' <"$gd")
      root_dir=$(dirname "$gdir_target")
      if [[ ! -d "$root_dir" ]] && [[ -d "$meta" ]]; then
        echo "WARN: possible stale worktree admin dir (missing checkout): $meta -> $root_dir" >&2
        warns=$((warns + 1))
      fi
    done
  fi
fi

echo "== summary: errors=$errors warns=$warns =="
if [[ "$errors" -gt 0 ]]; then
  exit 1
fi
exit 0
