#!/usr/bin/env python3
# SPDX-License-Identifier: MIT
"""Export Claude Code + Cursor session artifacts into a single JSON file.

Mirrors the shape of ``docs/worklogs/data/phenotype_session_extract_*.json``:
``meta``, ``user_prompts``, ``action_plans`` (assistant plan-like text).

Usage:
  python3 scripts/export_phenotype_session_artifacts.py \\
    [--home DIR] [--output PATH] [--cutoff DATE] [--cwd-substr SUBSTR] [--repo-root DIR]

Defaults:
  --home         ``Path.home()``
  --cutoff       7 days ago (UTC date)
  --cwd-substr   ``CodeProjects/Phenotype`` (Claude CWD pass filter)
  --output       ``docs/worklogs/data/phenotype_session_extract_<cutoff>_<today>.json``
                 under ``--repo-root`` (default: parent of ``scripts/``)

Requires: Python 3.10+
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from collections.abc import Iterator
from datetime import UTC, date, datetime, timedelta
from pathlib import Path
from typing import Any

CLAUDE_PROJECTS = Path(".claude") / "projects"
CURSOR_PROJECTS = Path(".cursor") / "projects"


def _dedupe_key(text: str) -> str:
    normalized = " ".join(text.split()).strip()[:8000]
    return hashlib.sha256(normalized.encode("utf-8", errors="replace")).hexdigest()[:16]


def _iter_jsonl(path: Path) -> Iterator[tuple[int, dict[str, Any]]]:
    try:
        raw = path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return
    for lineno, line in enumerate(raw.splitlines(), 1):
        line = line.strip()
        if not line:
            continue
        try:
            yield lineno, json.loads(line)
        except json.JSONDecodeError:
            continue


def _extract_user_text(obj: dict[str, Any]) -> str | None:
    """Best-effort user message extraction from Claude Code JSONL."""
    if obj.get("type") == "user":
        msg = obj.get("message")
        if isinstance(msg, dict):
            content = msg.get("content")
            if isinstance(content, str) and content.strip():
                return content.strip()
            if isinstance(content, list):
                parts: list[str] = []
                for block in content:
                    if isinstance(block, dict) and block.get("type") == "text":
                        t = block.get("text")
                        if isinstance(t, str):
                            parts.append(t)
                if parts:
                    return "\n".join(parts).strip()
    role = obj.get("role")
    if role == "user":
        c = obj.get("content")
        if isinstance(c, str) and c.strip():
            return c.strip()
    return None


def _extract_assistant_plan(obj: dict[str, Any]) -> str | None:
    """Heuristic: long assistant text with plan-like structure."""
    if obj.get("type") != "assistant":
        return None
    msg = obj.get("message")
    if not isinstance(msg, dict):
        return None
    content = msg.get("content")
    text = ""
    if isinstance(content, str):
        text = content
    elif isinstance(content, list):
        for block in content:
            if isinstance(block, dict) and block.get("type") == "text":
                t = block.get("text")
                if isinstance(t, str):
                    text += t + "\n"
    text = text.strip()
    if len(text) < 400:
        return None
    plan_markers = (
        "\n## ",
        "\n### ",
        "\n1.",
        "\n- [",
        "Action items",
        "Implementation",
        "Next steps",
    )
    if not any(m in text for m in plan_markers):
        return None
    return text[:120_000]


def _extract_cwd(obj: dict[str, Any]) -> str | None:
    for key in ("cwd", "working_directory", "workspace"):
        v = obj.get(key)
        if isinstance(v, str) and v.strip():
            return v.strip()
    session = obj.get("session")
    if isinstance(session, dict):
        for key in ("cwd", "working_directory"):
            v = session.get(key)
            if isinstance(v, str) and v.strip():
                return v.strip()
    return None


def _extract_ts(obj: dict[str, Any]) -> str | None:
    for key in ("timestamp", "ts", "created_at", "time"):
        v = obj.get(key)
        if isinstance(v, str) and v.strip():
            return v.strip()
    return None


def _cursor_user_blocks(path: Path) -> Iterator[tuple[int, str, str | None]]:
    """Yield (line, text, cwd_guess) from Cursor-style transcripts."""
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return
    for m in re.finditer(
        r"<user_query>\s*(.*?)\s*</user_query>",
        text,
        flags=re.DOTALL | re.IGNORECASE,
    ):
        block = m.group(1).strip()
        if len(block) < 2:
            continue
        yield m.start(), block, None


def scan_claude(
    home: Path,
    cwd_substr: str,
    cutoff: datetime,
    phenotype_named: list[str],
    cwd_pass_rows: list[dict[str, Any]],
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    user_rows: list[dict[str, Any]] = []
    plan_rows: list[dict[str, Any]] = []
    base = home / CLAUDE_PROJECTS
    if not base.is_dir():
        return user_rows, plan_rows

    for proj_dir in sorted(base.iterdir()):
        if not proj_dir.is_dir():
            continue
        rel_store = str(CLAUDE_PROJECTS / proj_dir.name)
        name_has_phenotype = "Phenotype" in proj_dir.name or "phenotype" in proj_dir.name.lower()

        for jsonl in sorted(proj_dir.rglob("*.jsonl")):
            for lineno, obj in _iter_jsonl(jsonl):
                cwd = _extract_cwd(obj)
                ts_s = _extract_ts(obj)
                try:
                    if ts_s:
                        event_ts = datetime.fromisoformat(ts_s.replace("Z", "+00:00"))
                    else:
                        event_ts = datetime.fromtimestamp(
                            jsonl.stat().st_mtime, tz=UTC
                        )
                except (ValueError, OSError):
                    event_ts = datetime.fromtimestamp(jsonl.stat().st_mtime, tz=UTC)
                if event_ts < cutoff:
                    continue

                if not name_has_phenotype:
                    if not cwd or cwd_substr not in cwd.replace("\\", "/"):
                        continue
                    ut = _extract_user_text(obj)
                    if ut:
                        cwd_pass_rows.append(
                            {
                                "kind": "user_prompt",
                                "store": rel_store,
                                "file": jsonl.name,
                                "line": lineno,
                                "event_ts": event_ts.isoformat(),
                                "cwd": cwd or "",
                                "text": ut,
                                "dedupe_key": _dedupe_key(ut),
                            }
                        )
                    pt = _extract_assistant_plan(obj)
                    if pt:
                        cwd_pass_rows.append(
                            {
                                "kind": "action_plan",
                                "store": rel_store,
                                "file": jsonl.name,
                                "line": lineno,
                                "event_ts": event_ts.isoformat(),
                                "cwd": cwd or "",
                                "text": pt,
                                "dedupe_key": _dedupe_key(pt),
                            }
                        )
                    continue

                if name_has_phenotype and rel_store not in phenotype_named:
                    phenotype_named.append(rel_store)

                ut = _extract_user_text(obj)
                if ut:
                    user_rows.append(
                        {
                            "kind": "user_prompt",
                            "store": rel_store,
                            "file": jsonl.name,
                            "line": lineno,
                            "event_ts": event_ts.isoformat(),
                            "cwd": cwd or "",
                            "text": ut,
                            "dedupe_key": _dedupe_key(ut),
                        }
                    )
                pt = _extract_assistant_plan(obj)
                if pt:
                    plan_rows.append(
                        {
                            "kind": "action_plan",
                            "store": rel_store,
                            "file": jsonl.name,
                            "line": lineno,
                            "event_ts": event_ts.isoformat(),
                            "cwd": cwd or "",
                            "text": pt,
                            "dedupe_key": _dedupe_key(pt),
                        }
                    )
    return user_rows, plan_rows


def scan_cursor(
    home: Path,
    cutoff: datetime,
    cursor_roots: list[str],
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    user_rows: list[dict[str, Any]] = []
    base = home / CURSOR_PROJECTS
    if not base.is_dir():
        return user_rows, []

    for proj_dir in sorted(base.iterdir()):
        if not proj_dir.is_dir():
            continue
        if "Phenotype" not in proj_dir.name and "phenotype" not in proj_dir.name.lower():
            continue
        at = proj_dir / "agent-transcripts"
        if not at.is_dir():
            continue
        rel_root = str(CURSOR_PROJECTS / proj_dir.name / "agent-transcripts")
        if rel_root not in cursor_roots:
            cursor_roots.append(rel_root)

        for tfile in sorted(at.rglob("*")):
            if tfile.suffix.lower() not in (".jsonl", ".json", ".txt", ".md"):
                continue
            try:
                mtime = datetime.fromtimestamp(tfile.stat().st_mtime, tz=UTC)
            except OSError:
                continue
            if mtime < cutoff:
                continue
            for line_off, block, _ in _cursor_user_blocks(tfile):
                user_rows.append(
                    {
                        "kind": "user_prompt",
                        "store": rel_root,
                        "file": tfile.name,
                        "line": line_off,
                        "event_ts": mtime.isoformat(),
                        "cwd": "",
                        "text": block,
                        "dedupe_key": _dedupe_key(block),
                    }
                )
    return user_rows, []


def dedupe_rows(rows: list[dict[str, Any]]) -> list[dict[str, Any]]:
    seen: set[str] = set()
    out: list[dict[str, Any]] = []
    for r in sorted(rows, key=lambda x: (x.get("event_ts", ""), x.get("store", ""))):
        k = r.get("dedupe_key")
        if not isinstance(k, str) or k in seen:
            continue
        seen.add(k)
        out.append(r)
    return out


def substantive_filter(text: str) -> bool:
    t = text.strip()
    if len(t) < 12:
        return False
    noise = (
        "<local-command",
        "<command-name>",
        "<command-message>",
        "/clear",
        "/compact",
    )
    if any(n in t for n in noise):
        return False
    return True


def git_head(repo: Path) -> str | None:
    try:
        p = subprocess.run(
            ["git", "-C", str(repo), "rev-parse", "--short", "HEAD"],
            capture_output=True,
            text=True,
            timeout=10,
        )
        if p.returncode == 0:
            return p.stdout.strip() or None
    except (OSError, subprocess.TimeoutExpired):
        pass
    return None


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--home", type=Path, default=Path.home())
    parser.add_argument("--repo-root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument(
        "--cutoff",
        type=str,
        default="",
        help="UTC date YYYY-MM-DD (default: 7 days ago)",
    )
    parser.add_argument(
        "--cwd-substr",
        default="CodeProjects/Phenotype",
        help="Substring for Claude CWD pass filter",
    )
    parser.add_argument("--output", type=Path, default=None)
    args = parser.parse_args()

    home: Path = args.home.expanduser().resolve()
    repo: Path = args.repo_root.resolve()

    if args.cutoff:
        cutoff_d = date.fromisoformat(args.cutoff)
        cutoff = datetime(cutoff_d.year, cutoff_d.month, cutoff_d.day, tzinfo=UTC)
    else:
        cutoff = datetime.now(tz=UTC) - timedelta(days=7)

    today = datetime.now(tz=UTC).date().isoformat()
    cutoff_s = cutoff.date().isoformat()
    out_path = args.output
    if out_path is None:
        out_path = (
            repo
            / "docs"
            / "worklogs"
            / "data"
            / f"phenotype_session_extract_{cutoff_s}_{today}.json"
        )
    out_path = out_path.resolve()
    out_path.parent.mkdir(parents=True, exist_ok=True)

    phenotype_named: list[str] = []
    cwd_pass_accum: list[dict[str, Any]] = []
    u1, p1 = scan_claude(home, args.cwd_substr, cutoff, phenotype_named, cwd_pass_accum)
    cursor_roots: list[str] = []
    u2, _ = scan_cursor(home, cutoff, cursor_roots)

    cwd_users = [r for r in cwd_pass_accum if r.get("kind") == "user_prompt"]
    cwd_plans = [r for r in cwd_pass_accum if r.get("kind") == "action_plan"]
    user_all = dedupe_rows(u1 + u2 + cwd_users)
    user_sub = [r for r in user_all if substantive_filter(str(r.get("text", "")))]
    plans = dedupe_rows(p1 + cwd_plans)

    cwd_user = sum(
        1
        for r in cwd_pass_accum
        if r.get("kind") == "user_prompt" and substantive_filter(str(r.get("text", "")))
    )
    cwd_plan = sum(1 for r in cwd_pass_accum if r.get("kind") == "action_plan")

    payload: dict[str, Any] = {
        "meta": {
            "generated_at": datetime.now(tz=UTC).isoformat(),
            "cutoff_utc": cutoff.isoformat(),
            "home": str(home),
            "repo_head": git_head(repo),
            "claude_project_roots_phenotype_named": sorted(set(phenotype_named)),
            "cursor_transcript_roots": sorted(set(cursor_roots)),
            "cwd_phenotype_pass": {
                "description": (
                    "Claude JSONL from dirs whose name omits 'Phenotype', "
                    f"included only when line cwd contains .../{args.cwd_substr}/"
                ),
                "user_prompt_rows_substantive": cwd_user,
                "action_plan_rows": cwd_plan,
            },
            "counts": {
                "user_prompts_deduped": len(user_all),
                "user_prompts_substantive": len(user_sub),
                "action_plans_deduped": len(plans),
            },
        },
        "user_prompts": user_sub,
        "action_plans": plans,
    }

    out_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(f"Wrote {out_path} ({len(user_sub)} substantive prompts, {len(plans)} plans)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
