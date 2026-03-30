#!/usr/bin/env bash
# generate-evidence.sh — Generates an evidence bundle for a feature.
# Usage: ./scripts/generate-evidence.sh <feature-id>
# Outputs: .agileplus/evidence/<feature-id>/bundle.json
#
# Requires: cargo, git, gh (GitHub CLI), jq

set -euo pipefail

FEATURE_ID="${1:-}"
if [[ -z "$FEATURE_ID" ]]; then
  echo "Usage: $0 <feature-id>" >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
EVIDENCE_DIR="$REPO_ROOT/.agileplus/evidence/$FEATURE_ID"
TIMESTAMP="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

mkdir -p "$EVIDENCE_DIR"

echo "[evidence] Generating bundle for feature: $FEATURE_ID"

# ── 1. Run cargo tests ─────────────────────────────────────────────────────
echo "[evidence] Running cargo test..."
TEST_RESULTS_FILE="$EVIDENCE_DIR/test-results.txt"
TEST_PASSED=false
TEST_SUMMARY=""

if cargo test 2>&1 | tee "$TEST_RESULTS_FILE"; then
  TEST_PASSED=true
  TEST_SUMMARY="All tests passed"
else
  TEST_SUMMARY="Some tests failed — see test-results.txt"
fi

# Count pass/fail
TESTS_PASSED_COUNT=$(grep -c '^test .* \.\.\. ok$' "$TEST_RESULTS_FILE" 2>/dev/null || echo "0")
TESTS_FAILED_COUNT=$(grep -c '^test .* \.\.\. FAILED$' "$TEST_RESULTS_FILE" 2>/dev/null || echo "0")

# ── 2. Collect git log ─────────────────────────────────────────────────────
echo "[evidence] Collecting git log..."
GIT_LOG_FILE="$EVIDENCE_DIR/git-log.txt"
GIT_REMOTE_URL="$(git -C "$REPO_ROOT" remote get-url origin 2>/dev/null || echo "")"

# Extract org/repo from remote URL for link building
GH_REPO=""
if [[ "$GIT_REMOTE_URL" =~ github\.com[:/]([^/]+/[^/.]+)(\.git)?$ ]]; then
  GH_REPO="${BASH_REMATCH[1]}"
fi

git -C "$REPO_ROOT" log --oneline -10 2>/dev/null | tee "$GIT_LOG_FILE" || true

# Build structured git entries
GIT_ENTRIES="[]"
if command -v jq &>/dev/null; then
  GIT_ENTRIES=$(git -C "$REPO_ROOT" log --format="%H|%s|%ai|%an" -10 2>/dev/null | \
    jq -R -s 'split("\n") | map(select(length > 0)) | map(
      split("|") |
      {
        hash: .[0],
        short_hash: (.[0] // "" | .[0:7]),
        subject: .[1],
        date: .[2],
        author: .[3],
        url: (if env.GH_REPO != "" then "https://github.com/\(env.GH_REPO)/commit/\(.[0])" else "" end)
      }
    )' GH_REPO="${GH_REPO}" 2>/dev/null || echo "[]")
fi

# ── 3. Collect open PRs ────────────────────────────────────────────────────
echo "[evidence] Collecting PRs..."
PR_FILE="$EVIDENCE_DIR/prs.json"
PR_ENTRIES="[]"

if command -v gh &>/dev/null && [[ -n "$GH_REPO" ]]; then
  # Look for PRs mentioning the feature ID or on feature branches
  PR_ENTRIES=$(gh pr list \
    --repo "$GH_REPO" \
    --json number,title,url,state,createdAt,headRefName \
    --limit 20 2>/dev/null | \
    jq --arg fid "$FEATURE_ID" '[.[] | select(
      (.title | ascii_downcase | contains($fid | ascii_downcase)) or
      (.headRefName | ascii_downcase | contains($fid | ascii_downcase))
    )]' 2>/dev/null || echo "[]")
  echo "$PR_ENTRIES" > "$PR_FILE"
else
  echo "[]" > "$PR_FILE"
fi

# ── 4. Collect CI links ────────────────────────────────────────────────────
echo "[evidence] Collecting CI run links..."
CI_LINKS="[]"
if command -v gh &>/dev/null && [[ -n "$GH_REPO" ]]; then
  CI_LINKS=$(gh run list \
    --repo "$GH_REPO" \
    --limit 5 \
    --json databaseId,displayTitle,status,conclusion,url,createdAt 2>/dev/null | \
    jq '[.[] | {
      id: .databaseId,
      title: .displayTitle,
      status: .status,
      conclusion: (.conclusion // "pending"),
      url: .url,
      created_at: .createdAt
    }]' 2>/dev/null || echo "[]")
fi

# ── 5. Write bundle.json ───────────────────────────────────────────────────
echo "[evidence] Writing bundle.json..."
BUNDLE_FILE="$EVIDENCE_DIR/bundle.json"

# Read test output (limit to 4KB for bundle)
TEST_OUTPUT_RAW=""
if [[ -f "$TEST_RESULTS_FILE" ]]; then
  TEST_OUTPUT_RAW=$(head -c 4096 "$TEST_RESULTS_FILE" | sed 's/"/\\"/g' | tr '\n' '\\' | sed 's/\\/\\n/g' | sed 's/\\n$//') || true
fi

cat > "$BUNDLE_FILE" <<EOF
{
  "feature_id": "$FEATURE_ID",
  "timestamp": "$TIMESTAMP",
  "test_results": {
    "passed": $TEST_PASSED,
    "passed_count": $TESTS_PASSED_COUNT,
    "failed_count": $TESTS_FAILED_COUNT,
    "summary": "$TEST_SUMMARY",
    "output_snippet": "$TEST_OUTPUT_RAW"
  },
  "git_log": $GIT_ENTRIES,
  "prs": $PR_ENTRIES,
  "ci_links": $CI_LINKS,
  "repo": "$GH_REPO"
}
EOF

echo "[evidence] Bundle written to: $BUNDLE_FILE"
echo "[evidence] Done."
