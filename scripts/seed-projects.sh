#!/usr/bin/env bash
# Seed AgilePlus with all Phenotype ecosystem repos as modules.
# Usage: bash scripts/seed-projects.sh [--db <path>]
# Default DB path: .agileplus/agileplus.db (relative to repo root, or AGILEPLUS_DB env var)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

DB="${AGILEPLUS_DB:-$REPO_ROOT/.agileplus/agileplus.db}"

# Parse --db flag
while [[ $# -gt 0 ]]; do
  case "$1" in
    --db)
      DB="$2"
      shift 2
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if [ ! -f "$DB" ]; then
  echo "ERROR: AgilePlus DB not found at $DB" >&2
  echo "       Run 'agileplus module list' first to initialise the database, or set AGILEPLUS_DB." >&2
  exit 1
fi

echo "Seeding Phenotype ecosystem modules into $DB ..."

# Use agileplus CLI if available, otherwise fall back to raw SQL.
if command -v agileplus &>/dev/null; then
  declare -A REPOS=(
    ["AgilePlus"]="Spec-driven project management tool (self-hosted)"
    ["trace"]="TraceRTM requirements traceability and RTM platform"
    ["heliosApp"]="Helios web application frontend"
    ["thegent"]="The Gent dotfiles manager and agent orchestration platform"
    ["civ"]="CIV platform service"
    ["phenotype-shared"]="Shared libraries and utilities across Phenotype ecosystem"
    ["cliproxyapi-plusplus"]="CLI proxy API plus-plus service"
    ["phenotype-infrakit"]="Phenotype infrastructure kit and tooling"
    ["phenotype-gauge"]="Phenotype metrics and gauge library"
    ["agent-wave"]="Agent wave orchestration service"
    ["phenotype-design"]="Phenotype design system and UI components"
  )

  # Fetch existing slugs to skip duplicates
  EXISTING=$(agileplus --db "$DB" module list 2>/dev/null | grep -oP '\(slug: \K[^)]+' || true)

  for name in "${!REPOS[@]}"; do
    desc="${REPOS[$name]}"
    slug="${name,,}"          # lowercase
    slug="${slug//_/-}"       # underscores to dashes
    if echo "$EXISTING" | grep -qx "$slug"; then
      echo "  SKIP (already exists): $name (slug: $slug)"
    else
      echo "  CREATE: $name"
      agileplus --db "$DB" module create --name "$name" --description "$desc"
    fi
  done
else
  echo "agileplus CLI not found — falling back to direct SQL inserts" >&2
  sqlite3 "$DB" <<'SQL'
INSERT OR IGNORE INTO modules (slug, friendly_name, description, created_at, updated_at) VALUES
  ('agileplus',           'AgilePlus',             'Spec-driven project management tool (self-hosted)',             datetime('now'), datetime('now')),
  ('trace',               'trace',                  'TraceRTM requirements traceability and RTM platform',           datetime('now'), datetime('now')),
  ('heliosapp',           'heliosApp',              'Helios web application frontend',                               datetime('now'), datetime('now')),
  ('thegent',             'thegent',                'The Gent dotfiles manager and agent orchestration platform',    datetime('now'), datetime('now')),
  ('civ',                 'civ',                    'CIV platform service',                                          datetime('now'), datetime('now')),
  ('phenotype-shared',    'phenotype-shared',       'Shared libraries and utilities across Phenotype ecosystem',     datetime('now'), datetime('now')),
  ('cliproxyapi-plusplus','cliproxyapi-plusplus',   'CLI proxy API plus-plus service',                              datetime('now'), datetime('now')),
  ('phenotype-infrakit',  'phenotype-infrakit',     'Phenotype infrastructure kit and tooling',                     datetime('now'), datetime('now')),
  ('phenotype-gauge',     'phenotype-gauge',        'Phenotype metrics and gauge library',                          datetime('now'), datetime('now')),
  ('agent-wave',          'agent-wave',             'Agent wave orchestration service',                             datetime('now'), datetime('now')),
  ('phenotype-design',    'phenotype-design',       'Phenotype design system and UI components',                    datetime('now'), datetime('now'));
SQL
fi

echo ""
echo "Done. Current modules:"
if command -v agileplus &>/dev/null; then
  agileplus --db "$DB" module list
else
  sqlite3 "$DB" "SELECT slug, friendly_name FROM modules ORDER BY slug;"
fi
