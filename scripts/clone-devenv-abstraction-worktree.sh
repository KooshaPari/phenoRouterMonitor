#!/usr/bin/env sh
# Clone KooshaPari/devenv-abstraction into the Phenotype worktree layout.
# Run from your machine (not assumed to run in agent sandboxes).
set -eu

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEST="${DEVENV_WT:-$ROOT/worktrees/devenv-abstraction/main}"
REPO="${DEVENV_REPO:-https://github.com/KooshaPari/devenv-abstraction.git}"

if test -e "$DEST/.git"; then
  echo "already cloned: $DEST" >&2
  exit 0
fi

mkdir -p "$(dirname "$DEST")"
git clone --depth 1 "$REPO" "$DEST"
echo "cloned to $DEST"
echo "Open that folder in Cursor for feature work (keep canonical repos on main per AGENTS.md)."
