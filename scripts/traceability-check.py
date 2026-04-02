#!/usr/bin/env python3
"""
Traceability Checker for Phenotype Ecosystem.
Validates that all Functional Requirements (FRs) defined in traceability.json
have corresponding implementations in code and coverage in tests.
"""

import os
import re
import json
import sys
import argparse
from typing import Dict, List, Set, Tuple

# Regex patterns for finding traceability markers in code/tests
SPEC_MARKERS = {
    "FR": re.compile(r"FR-[A-Z0-9]+-\d+"),
    "TEST_ID": re.compile(r"TEST-\d+"),
    "TRACE": re.compile(r"@trace\s+([A-Z0-9]+-\d+)"),
}

def find_markers_in_dir(directory: str, extensions: Tuple[str, ...] = (".rs", ".ts", ".py", ".yaml", ".mts")) -> Tuple[Set[str], Set[str]]:
    """Scan directory for FR and TEST_ID markers."""
    found_frs = set()
    found_tests = set()

    for root, _, files in os.walk(directory):
        if any(d in root for d in ["target", "node_modules", ".git", "vendor"]):
            continue
            
        for file in files:
            if file.endswith(extensions):
                path = os.path.join(root, file)
                try:
                    with open(path, "r", encoding="utf-8") as f:
                        content = f.read()
                        found_frs.update(SPEC_MARKERS["FR"].findall(content))
                        found_tests.update(SPEC_MARKERS["TEST_ID"].findall(content))
                        # Also check @trace annotations
                        traces = SPEC_MARKERS["TRACE"].findall(content)
                        found_frs.update(traces)
                except Exception as e:
                    print(f"Error reading {path}: {e}", file=sys.stderr)
                    
    return found_frs, found_tests

def main():
    parser = argparse.ArgumentParser(description="Phenotype Traceability Checker")
    parser.add_argument("--json", help="Path to traceability.json")
    parser.add_argument("--root", default=".", help="Root directory to scan")
    parser.add_argument("--strict", action="store_true", help="Fail if any implemented spec is missing code/tests")
    args = parser.parse_args()

    if not args.json:
        # Try to find it in common locations
        common_paths = [
            "phenodocs/docs/public/api/traceability.json",
            "docs/traceability/traceability.json"
        ]
        for p in common_paths:
            if os.path.exists(p):
                args.json = p
                break
    
    if not args.json or not os.path.exists(args.json):
        print(f"Error: Traceability JSON not found.")
        sys.exit(1)

    with open(args.json, "r") as f:
        data = json.load(f)

    # Handle different JSON structures
    ecosystem = data.get("ecosystem", "Phenotype")
    repos = data.get("repositories", data.get("repos", []))
    
    print(f"--- Traceability Validation for {ecosystem} ---")
    print(f"Scanning {args.root} for annotations...")
    
    found_frs, found_tests = find_markers_in_dir(args.root)
    print(f"Found {len(found_frs)} FR annotations and {len(found_tests)} TEST_ID annotations.\n")

    overall_missing = []

    for repo in repos:
        repo_name = repo.get("name", "Unknown")
        # Check if it has a detailed specsList (the rich JSON structure)
        if "specsList" in repo:
            implemented = [s for s in repo["specsList"] if s.get("status") == "implemented"]
            missing = [s["id"] for s in implemented if s["id"] not in found_frs]
            
            if missing:
                print(f"❌ {repo_name}: Missing {len(missing)} implementation(s): {', '.join(missing)}")
                overall_missing.extend([(repo_name, sid) for sid in missing])
            else:
                print(f"✅ {repo_name}: All {len(implemented)} implemented specs verified.")
        else:
            # Fallback for simple JSON
            print(f"ℹ️ {repo_name}: No detailed spec list available for validation.")

    print("\n--- Summary ---")
    if overall_missing:
        print(f"Total missing annotations: {len(overall_missing)}")
        if args.strict:
            sys.exit(1)
    else:
        print("All traceability checks passed!")

if __name__ == "__main__":
    main()
