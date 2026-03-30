#!/bin/bash
# Generate AgilePlus specs from git history

REPO="$1"
SPEC_DIR="$2"

if [[ -z "$REPO" ]] || [[ -z "$SPEC_DIR" ]]; then
  echo "Usage: $0 <repo-path> <spec-id>"
  exit 1
fi

echo "Analyzing git history for $REPO..."
cd "$REPO" || exit 1

# Get unique feature areas from commits
features=$(git log --format="%s" --since="2025-01-01" | grep -E "^(feat|fix|refactor|chore|docs)" | sed 's/:.*//' | cut -d'(' -f2 | cut -d')' -f1 | sort -u | grep -v "^$")

echo "Found features: $features"
