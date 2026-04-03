#!/usr/bin/env python3
"""
Traceability Checker for Phenotype Ecosystem.
Validates that all Functional Requirements defined in traceability.json
have corresponding implementations in code and coverage in tests.
"""
import os, re, json, sys, argparse
from typing import Set

# Regex patterns for finding traceability markers
SPEC_MARKERS = {
    "FR": re.compile(r"FR-(?:[A-Z0-9]+-)?\d+"),
    "SPEC": re.compile(r"(?:SKILL|TASK|AUTH|CONF|VES|FORGE|EVAL|TYPE|INFRA|GOV|HUB|CLI|META|VM|CIPH|DEP|DOC|LOG|MID|PATCH|RES|SENT|TEMP|XDD|AP|HEL|BIF|KM|KIT|MET|ORG|PLAN|PS|PORT|PROF|QUERY|QUILL|SEED|SETT|STASH|TOKN|TOSS|TRACE|ZERO|HELM|HL|FORGE|VIBE|DINO|DOCV|DUPLE|EVALO|EVENT|FLAG|FLOW|GUARD|HEX|HEXP|HTTP|KOG|PORTAGE|SCHEMA|SHARE|TRAC|WM|CURS|BYTE|AUTHV|DM|APIS|CMDR|SDK)-\d+"),
    "TRACE": re.compile(r"@trace\s+([A-Z0-9-]+\d+)"),
    "TEST_ID": re.compile(r"TEST-[A-Z0-9]*-\d+"),
}

def find_markers(directory: str, extensions=(".rs",".ts",".py",".yaml",".yml",".md",".zig",".go",".json")) -> Set[str]:
    found = set()
    for root, _, files in os.walk(directory):
        if any(d in root for d in ["target","node_modules",".git","vendor","__pycache__"]):
            continue
        for file in files:
            if file.endswith(extensions):
                try:
                    with open(os.path.join(root, file), "r", encoding="utf-8", errors="ignore") as f:
                        content = f.read()
                        found.update(SPEC_MARKERS["FR"].findall(content))
                        found.update(SPEC_MARKERS["SPEC"].findall(content))
                        found.update(SPEC_MARKERS["TRACE"].findall(content))
                        found.update(SPEC_MARKERS["TEST_ID"].findall(content))
                except:
                    pass
    return found

def main():
    p = argparse.ArgumentParser()
    p.add_argument("--json", default="docs/traceability/traceability.json")
    p.add_argument("--root", default=".")
    p.add_argument("--repos-file")
    p.add_argument("--strict", action="store_true")
    p.add_argument("--verbose", action="store_true")
    args = p.parse_args()

    with open(args.json) as f:
        data = json.load(f)

    repos_to_check = []
    if args.repos_file:
        with open(args.repos_file) as f:
            repos_to_check = [l.strip() for l in f if l.strip()]
    else:
        repos_to_check = [args.root]

    missing_total = 0
    for repo_path in repos_to_check:
        name = os.path.basename(repo_path.rstrip("/"))
        cfg = next((r for r in data["repositories"] if r["name"] == name), None)
        if not cfg:
            continue
        markers = find_markers(repo_path)
        impl = [s["id"] for s in cfg["specsList"] if s["status"] == "implemented"]
        missing = [sid for sid in impl if sid not in markers]
        if missing:
            print(f"FAIL {name}: {len(missing)} missing")
            missing_total += len(missing)
            if args.verbose:
                for m in missing: print(f"  - {m}")
        else:
            print(f"PASS {name}: verified")

    print(f"\nTotal missing: {missing_total}")
    if args.strict and missing_total:
        sys.exit(1)

if __name__ == "__main__":
    main()
