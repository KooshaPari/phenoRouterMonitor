#!/bin/bash
# install.sh: Install git hooks for the repository
# Usage: ./install.sh or bash install.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT=$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel 2>/dev/null || echo "")

if [ -z "$REPO_ROOT" ]; then
  echo "ERROR: Could not determine repository root"
  exit 1
fi

# Set git hooks path
git -C "$REPO_ROOT" config core.hooksPath "$SCRIPT_DIR"

echo "Git hooks installed successfully!"
echo "Hooks directory: $SCRIPT_DIR"
echo ""
echo "Installed hooks:"
ls -1 "$SCRIPT_DIR" | grep -v "install.sh" | while read hook; do
  echo "  - $hook"
done

echo ""
echo "To verify installation:"
echo "  git config core.hooksPath"
