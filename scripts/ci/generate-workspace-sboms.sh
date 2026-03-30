#!/usr/bin/env bash
# Generate CycloneDX JSON (spec 1.5) for every package in the repo root Cargo workspace.
# Used by .github/workflows/sbom.yml and release.yml — single source of truth vs duplicating matrix YAML.
#
# Usage: generate-workspace-sboms.sh [REPO_ROOT] [OUTPUT_DIR]
#   REPO_ROOT   — directory containing root Cargo.toml (default: .)
#   OUTPUT_DIR  — flat directory for cyclonedx-sbom-<crate>.json (default: sbom-out)
#
# Requires: cargo, cargo-cyclonedx on PATH, jq.
set -euo pipefail

ROOT="${1:-.}"
OUT_DIR="${2:-sbom-out}"

cd "$ROOT"

if ! command -v jq >/dev/null 2>&1; then
  echo "generate-workspace-sboms.sh: jq is required" >&2
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "generate-workspace-sboms.sh: cargo is required" >&2
  exit 1
fi

if ! cargo cyclonedx --version >/dev/null 2>&1; then
  echo "generate-workspace-sboms.sh: cargo-cyclonedx is required (cargo install cargo-cyclonedx --version 0.5.9)" >&2
  exit 1
fi

mkdir -p "$OUT_DIR"

while IFS=$'\t' read -r name path; do
  [[ -z "${name:-}" ]] || [[ -z "${path:-}" ]] && continue
  path=$(realpath "$path")
  echo "SBOM: ${name} (${path})"
  cargo cyclonedx \
    --manifest-path "$path" \
    -f json \
    --spec-version 1.5 \
    --override-filename "sbom-${name}"
  mdir=$(dirname "$path")
  src="${mdir}/sbom-${name}.json"
  if [[ ! -f "$src" ]]; then
    echo "generate-workspace-sboms.sh: expected output missing: ${src}" >&2
    exit 1
  fi
  cp "$src" "${OUT_DIR}/cyclonedx-sbom-${name}.json"
done < <(cargo metadata --no-deps --format-version 1 | jq -r '
  .workspace_members[] as $id
  | .packages[]
  | select(.id == $id)
  | "\(.name)\t\(.manifest_path)"
')

shopt -s nullglob
files=("${OUT_DIR}"/cyclonedx-sbom-*.json)
if [[ ${#files[@]} -lt 1 ]]; then
  echo "generate-workspace-sboms.sh: no SBOM files produced under ${OUT_DIR}" >&2
  exit 1
fi
echo "generate-workspace-sboms.sh: wrote ${#files[@]} file(s) to ${OUT_DIR}"
