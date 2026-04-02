#!/bin/bash
# Setup script for AI Testing Infrastructure secrets
# Run this script after obtaining API keys from each service

set -e

echo "=== AI Testing Infrastructure Secrets Setup ==="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to add secret
add_secret() {
    local repo=$1
    local name=$2
    local value=$3

    if [ -z "$value" ]; then
        echo -e "${YELLOW}Skipping $name - not set${NC}"
        return 1
    fi

    echo -n "Adding $name to $repo... "
    echo "$value" | gh secret set "$name" --repo "$repo" 2>/dev/null
    echo -e "${GREEN}Done${NC}"
}

# Check authentication
echo "Checking GitHub authentication..."
if ! gh auth status >/dev/null 2>&1; then
    echo -e "${RED}Error: Not authenticated with GitHub. Run 'gh auth login' first.${NC}"
    exit 1
fi
echo -e "${GREEN}Authenticated as $(gh api user -q .login)${NC}"
echo ""

# NanoVMS secrets
echo "--- NanoVMS ---"
add_secret "KooshaPari/nanovms" "QODO_API_KEY" "${QODO_API_KEY}"

# AgilePlus secrets
echo ""
echo "--- AgilePlus ---"
add_secret "KooshaPari/AgilePlus" "APPLITOOLS_API_KEY" "${APPLITOOLS_API_KEY}"
add_secret "KooshaPari/AgilePlus" "TESTRIGOR_API_KEY" "${TESTRIGOR_API_KEY}"

# thegent secrets
echo ""
echo "--- thegent ---"
add_secret "KooshaPari/thegent" "QODO_API_KEY" "${QODO_API_KEY}"

echo ""
echo "=== Setup Complete ==="
echo ""
echo "To set API keys before running:"
echo "  export QODO_API_KEY='your-qodo-key'"
echo "  export APPLITOOLS_API_KEY='your-applitools-key'"
echo "  export TESTRIGOR_API_KEY='your-testrigor-key'"
echo "  ./scripts/setup-ai-testing-secrets.sh"
