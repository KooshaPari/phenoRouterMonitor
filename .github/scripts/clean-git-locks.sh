#!/bin/bash
#
# Clean stale Git lock files
#
# Description:
#   Detects and removes stale .git/index.lock files that are 0 bytes
#   (indicating abandoned locks from crashed or concurrent git operations).
#   This script is useful as a safety net before running git operations.
#
# Usage:
#   ./.github/scripts/clean-git-locks.sh [repo_path]
#
# Arguments:
#   repo_path (optional): Path to git repository. Defaults to current directory.
#
# Exit Codes:
#   0 - Success (lock cleaned or not present)
#   1 - Error accessing repository

set -euo pipefail

REPO_PATH="${1:-.}"
LOCK_FILE="$REPO_PATH/.git/index.lock"

# Check if we're in a git repository
if [[ ! -d "$REPO_PATH/.git" ]]; then
    echo "Error: $REPO_PATH is not a git repository" >&2
    exit 1
fi

# Check if lock file exists
if [[ ! -f "$LOCK_FILE" ]]; then
    echo "No .git/index.lock found in $REPO_PATH"
    exit 0
fi

# Get file size
lock_size=$(stat -c%s "$LOCK_FILE" 2>/dev/null || stat -f%z "$LOCK_FILE" 2>/dev/null || echo "-1")

# Check if it's a stale 0-byte lock file
if [[ "$lock_size" -eq 0 ]]; then
    echo "Found stale .git/index.lock (0 bytes) in $REPO_PATH"
    rm -f "$LOCK_FILE"
    if [[ ! -f "$LOCK_FILE" ]]; then
        echo "Successfully removed stale git lock file"
        exit 0
    else
        echo "Error: Failed to remove git lock file" >&2
        exit 1
    fi
else
    echo ".git/index.lock exists but is $lock_size bytes (not stale, skipping)"
    exit 0
fi
