#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Scan Rust sources for public error-style enums; write JSON for worklogs.

Usage:
  python3 scripts/generate_error_enums_index.py [--root DIR] [--scope workspace|all]

Output:
  docs/worklogs/data/error_enums_index.json

Skips: target/, .git/, node_modules/, vendor/, *-wtrees path segments

--scope workspace (default): crates/, libs/, rust/, tools/ under root.
--scope all: entire repo root (same skips).
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

SKIP_DIR_NAMES = frozenset(
    {
        "target",
        ".git",
        "node_modules",
        "vendor",
        ".worktrees",
    }
)

# Typical error enums: *Error, or plain Error in error*.rs
ENUM_LINE_RE = re.compile(r"^\s*pub\s+enum\s+(\w+)\b")


def skip_path(path: Path) -> bool:
    parts = path.parts
    if any(p in SKIP_DIR_NAMES for p in parts):
        return True
    if "target" in parts:
        return True
    # Legacy / agent worktree hubs (duplicate crate trees)
    if any(p.endswith("-wtrees") or p.endswith("_wtrees") for p in parts):
        return True
    return False


def include_enum(name: str, rel_path: str) -> bool:
    lower = rel_path.replace("\\", "/").lower()
    if name.endswith("Error") or name.endswith("Errors"):
        return True
    if name == "Error" and (
        "/error" in lower
        or lower.endswith("error.rs")
        or "/errors/" in lower
        or lower.endswith("/errors.rs")
    ):
        return True
    return False


def collect_rs_paths(root: Path, scope: str) -> list[Path]:
    rs_files: list[Path] = []
    if scope == "workspace":
        scan_roots = [
            root / "crates",
            root / "libs",
            root / "rust",
            root / "tools",
        ]
        for sub in scan_roots:
            if sub.is_dir():
                for path in sub.rglob("*.rs"):
                    if not skip_path(path):
                        rs_files.append(path)
        return rs_files
    if scope == "all":
        for path in root.rglob("*.rs"):
            if skip_path(path):
                continue
            rs_files.append(path)
        return rs_files
    raise ValueError(f"unknown scope: {scope!r}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parents[1],
        help="Repository root (default: parent of scripts/)",
    )
    parser.add_argument(
        "--scope",
        choices=("workspace", "all"),
        default="workspace",
        help="workspace: crates/libs/rust/tools; all: full tree under root (default: workspace)",
    )
    args = parser.parse_args()
    root: Path = args.root.resolve()

    if not root.is_dir():
        print(f"ERROR: root is not a directory: {root}", file=sys.stderr)
        return 1

    rs_files = collect_rs_paths(root, args.scope)

    entries: list[dict[str, int | str]] = []
    for path in rs_files:
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError as e:
            print(f"WARN: skip {path}: {e}", file=sys.stderr)
            continue
        rel = path.relative_to(root).as_posix()
        for lineno, line in enumerate(text.splitlines(), 1):
            m = ENUM_LINE_RE.match(line)
            if not m:
                continue
            name = m.group(1)
            if not include_enum(name, rel):
                continue
            entries.append({"path": rel, "line": lineno, "enum": name})

    entries.sort(key=lambda e: (str(e["path"]), int(e["line"]), str(e["enum"])))

    out_path = root / "docs" / "worklogs" / "data" / "error_enums_index.json"
    out_path.parent.mkdir(parents=True, exist_ok=True)
    payload = {
        "schema": "error_enums_index.v1",
        "repo_root": root.name,
        "scan_scope": args.scope,
        "count": len(entries),
        "enums": entries,
    }
    out_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(f"Wrote {len(entries)} entries to {out_path.relative_to(root)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
