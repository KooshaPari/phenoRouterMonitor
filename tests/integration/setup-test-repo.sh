#!/usr/bin/env bash
# Set up a minimal git repository for integration tests.
# Runs inside the test-runner or agileplus-core container.
#
# Traceability: WP16-T095
set -euo pipefail

REPO_PATH="${1:-/repo}"

echo "Initialising test repository at $REPO_PATH ..."
mkdir -p "$REPO_PATH"
cd "$REPO_PATH"

if [ ! -d ".git" ]; then
    git init --initial-branch=main
    git config user.email "test@agileplus.example"
    git config user.name "AgilePlus Test"
    mkdir -p kitty-specs
    echo '{}' > kitty-specs/.gitkeep
    git add .
    git commit -m "Initial test repository"
    echo "Test repository created."
else
    echo "Repository already exists, skipping init."
fi
