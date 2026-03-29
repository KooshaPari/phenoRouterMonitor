"""Shared work-stream orchestration service for CLI command wrappers."""

from __future__ import annotations

import time
from pathlib import Path
from typing import Any

from thegent.cli.services import pre_work_gate_helpers
from thegent.cli.services import run_workstream_helpers
from thegent.config import ThegentSettings

_log = __import__("logging").getLogger(__name__)


def _normalize_item_id(item_id: str) -> str:
    """Normalize item ids for dedupe/claim handling."""
    return item_id.strip().strip("~")


def do_next_impl(cd: Path | None = None, limit: int = 5) -> dict[str, Any]:
    """Find next actionable work items from work-stream and queue sources."""
    from thegent.cli.commands.impl import _resolve_cwd

    limit = max(1, min(100, limit))
    cwd = _resolve_cwd(cd) or Path.cwd()
    gate_block = pre_work_gate_helpers.enforce_pre_work_hard_gate(cwd)
    if gate_block is not None:
        return {
            **gate_block,
            "next_items": [],
            "count": 0,
            "sources_checked": [],
            "empty_reason": "Pre-work hard gate blocked do-next until evidence is refreshed",
        }

    settings = ThegentSettings()
    work_stream_path = cwd / "docs" / "reference" / "WORK_STREAM.md"
    session_dir = Path(settings.session_dir).expanduser().resolve()

    try:
        from thegent.planning.workstream_db import WorkstreamDB

        db = WorkstreamDB(settings=settings)
        if work_stream_path.exists():
            data = run_workstream_helpers.parse_work_stream_md(work_stream_path)
            db.sync_workstream(data)
        db.sync_from_agileplus(session_dir)
        db.sync_from_queues(session_dir)
        next_items = db.get_next_items(limit=limit)
        if next_items:
            return {
                "next_items": next_items,
                "count": len(next_items),
                "sources_checked": ["workstream.db"],
                "empty_reason": None,
            }
    except Exception as e:
        _log.debug("DB primary failed, falling back to direct sources: %s", e)

    aggregated_items: list[dict[str, Any]] = []
    sources_checked: list[str] = []
    queued, q_sources = run_workstream_helpers.collect_queued_items(settings, limit)
    aggregated_items.extend(queued)
    sources_checked.extend(q_sources)
    ws_items, ws_sources = run_workstream_helpers.collect_work_stream_items(work_stream_path, limit)
    aggregated_items.extend(ws_items)
    sources_checked.extend(ws_sources)
    aggregated_items.sort(
        key=lambda x: (x.pop("_sort_order", 5), run_workstream_helpers.priority_sort_key(x.get("priority", "P2")))
    )
    next_items = aggregated_items[:limit]

    if not next_items:
        example_task_path = cwd / "tasks" / "example-task.md"
        if example_task_path.exists():
            return {
                "next_items": [
                    {
                        "id": "example-task",
                        "description": "This is an example task file demonstrating the YAML frontmatter format.",
                        "source": "TASKS",
                        "prompt_suggestion": "Complete example-task: Example Task",
                    }
                ],
                "count": 1,
                "sources_checked": ["tasks/example-task.md"],
                "empty_reason": "No work stream or queue items; returning example-task",
            }
        if not work_stream_path.exists():
            return {
                "error": f"WORK_STREAM.md not found: {work_stream_path}",
                "next_items": [],
                "count": 0,
                "sources_checked": list(dict.fromkeys(sources_checked)),
                "empty_reason": "No WORK_STREAM.md and no queued items",
            }
        return {
            "next_items": [],
            "count": 0,
            "sources_checked": list(dict.fromkeys(sources_checked)),
            "empty_reason": "No available items in work stream or queues",
        }

    return {
        "next_items": next_items,
        "count": len(next_items),
        "sources_checked": list(dict.fromkeys(sources_checked)),
        "empty_reason": None,
    }


def wait_next_impl(
    cd: Path | None = None,
    poll_interval: float = 2.0,
    timeout: float = 0.0,
    sources: tuple[str, ...] = ("do_next",),
) -> dict[str, Any]:
    """Poll do-next until an actionable item exists or timeout is reached."""
    _ = sources
    start_time = time.perf_counter()
    poll_count = 0

    while True:
        elapsed = time.perf_counter() - start_time
        if timeout > 0 and elapsed >= timeout:
            return {
                "action": None,
                "elapsed_s": elapsed,
                "poll_count": poll_count,
                "timeout": True,
            }

        result = do_next_impl(cd=cd, limit=1)
        poll_count += 1

        if "error" in result:
            time.sleep(poll_interval)
            continue

        items = result.get("next_items", [])
        if items:
            return {
                "action": items[0],
                "elapsed_s": elapsed,
                "poll_count": poll_count,
                "timeout": False,
            }

        time.sleep(poll_interval)


def spawn_next_impl(
    cd: Path | None = None,
    limit: int = 10,
    agent: str = "free",
    timeout: int | None = None,
    lane: str = "critical",
    override_reason: str = "manual-next-step",
    claim: bool = True,
) -> dict[str, Any]:
    """Spawn next items as background runs."""
    from thegent.cli.commands.impl import _default_owner_tag, _resolve_cwd, bg_impl
    from thegent.config_provider import get_config_provider

    limit = max(1, min(20, limit))
    cwd = _resolve_cwd(cd) or Path.cwd()
    settings = ThegentSettings()
    effective_timeout = timeout or settings.default_timeout
    effective_timeout = min(effective_timeout, 1800)

    result = do_next_impl(cd=cd, limit=limit)
    if result.get("governance_blocked"):
        return {
            **result,
            "spawned": [],
            "errors": [],
            "count": 0,
        }
    if "error" in result:
        return {"error": result["error"], "spawned": [], "errors": [], "count": 0}
    raw_items = result.get("next_items", [])
    if not raw_items:
        return {
            "spawned": [],
            "errors": [],
            "count": 0,
            "empty_reason": result.get("empty_reason"),
        }
    # Remove duplicate/invalid items before claiming/spawning.
    items: list[dict[str, Any]] = []
    seen_ids: set[str] = set()
    for item in raw_items:
        raw_item_id = str(item.get("id", "")).strip()
        norm_item_id = _normalize_item_id(raw_item_id)
        if not norm_item_id:
            continue
        if norm_item_id in seen_ids:
            continue
        seen_ids.add(norm_item_id)
        item = dict(item)
        item["id"] = norm_item_id
        items.append(item)

    agent_id = "spawn-next"
    try:
        from thegent.discovery import get_current_agent_id

        agent_id = get_current_agent_id() or agent_id
    except Exception as exc:
        _log.debug("Could not resolve current agent id for do-next: %s", exc)

    owner = _default_owner_tag(cwd) if cwd else None
    spawned: list[dict[str, Any]] = []
    errors: list[dict[str, Any]] = []

    for item in items:
        item_id = item.get("id", "?")
        prompt = item.get("prompt_suggestion", "")
        if not prompt:
            errors.append({"item_id": item_id, "error": "No prompt_suggestion"})
            continue

        if claim:
            try:
                claim_result = work_stream_claim_impl(item_id, agent_id, cd=cd)
                if not claim_result.get("success", False):
                    err: dict[str, Any] = {"item_id": item_id, "error": claim_result.get("error", "Claim failed")}
                    if claim_result.get("governance_blocked"):
                        err["governance_blocked"] = True
                        err["remediation"] = claim_result.get("remediation")
                        err["governance_block"] = claim_result.get("governance_block")
                    if claim_result.get("dependency_blocked"):
                        err["dependency_blocked"] = True
                        err["blocked_by"] = claim_result.get("blocked_by", [])
                        err["remediation"] = claim_result.get("remediation")
                    errors.append(err)
                    continue
            except Exception as e:
                errors.append({"item_id": item_id, "error": f"Claim failed: {e}"})
                continue

        res = bg_impl(
            agent=agent,
            prompt=prompt,
            cd=cwd,
            mode="write",
            timeout=effective_timeout,
            full=False,
            model="gpt-5-mini" if agent == "free" else None,
            owner=owner,
            lane=lane,
            override_reason=override_reason,
            config_provider=get_config_provider(),
        )

        if "error" in res:
            errors.append({"item_id": item_id, "error": res["error"]})
            continue
        sid = res.get("session_id", "")
        if sid:
            spawned.append({"item_id": item_id, "session_id": sid})

    return {"spawned": spawned, "errors": errors, "count": len(spawned)}


def work_stream_claim_impl(item_id: str, agent_id: str, cd: Path | None = None) -> dict[str, Any]:
    """Claim a work item (move from BACKLOG to CLAIMED in WORK_STREAM.md)."""
    from thegent.cli.commands.impl import _resolve_cwd
    from thegent.planning.work_stream import WorkStreamManager

    cwd = _resolve_cwd(cd) or Path.cwd()
    gate_block = pre_work_gate_helpers.enforce_pre_work_hard_gate(cwd)
    if gate_block is not None:
        return {
            "success": False,
            "item_id": item_id,
            "agent_id": agent_id,
            **gate_block,
        }

    settings = ThegentSettings()
    manager = WorkStreamManager(settings, base_dir=cwd)
    return manager.claim(item_id, agent_id)


def work_stream_complete_impl(item_id: str, agent_id: str, cd: Path | None = None) -> dict[str, Any]:
    """Complete a work item (move from CLAIMED to COMPLETED in WORK_STREAM.md)."""
    from thegent.cli.commands.impl import _resolve_cwd
    from thegent.planning.work_stream import WorkStreamManager

    cwd = _resolve_cwd(cd) or Path.cwd()
    settings = ThegentSettings()
    manager = WorkStreamManager(settings, base_dir=cwd)
    return manager.complete(item_id, agent_id)


def incorporate_impl(cd: Path | None = None, dry_run: bool = False) -> dict[str, Any]:
    """Merge fragments into WORK_STREAM.md and sync with task files."""
    from thegent.cli.commands.impl import _resolve_cwd
    from thegent.task.sync import WorkStreamSync

    cwd = _resolve_cwd(cd) or Path.cwd()
    work_stream_path = cwd / "docs" / "reference" / "WORK_STREAM.md"
    tasks_dir = cwd / "tasks"

    if not work_stream_path.exists():
        return {"error": f"WORK_STREAM.md not found: {work_stream_path}"}

    source_files = [
        cwd / "docs" / "reference" / "02-UNIFIED-WBS.md",
        cwd / "docs" / "reference" / "03-UNIFIED-WBS.md",
        cwd / "docs" / "reference" / "UNIFIED-WBS.md",
    ]

    valid_sources = [f for f in source_files if f.exists()]
    if not valid_sources:
        return {
            "merged": 0,
            "message": "No source WBS files found to incorporate.",
            "dry_run": dry_run,
        }

    merged_count = 0
    validation_errors: list[dict[str, Any]] = []

    if tasks_dir.exists():
        task_files = list(tasks_dir.glob("*.md"))
        for tf in task_files:
            _validate_task_and_record_errors(tf, validation_errors)

    if not dry_run:
        try:
            sync = WorkStreamSync(work_stream_path, tasks_dir)
            sync_result = sync.update_work_stream_from_tasks()
            merged_count = sync_result.get("updated", 0)
        except Exception as e:
            return {"error": f"Sync failed during incorporation: {e}", "merged": 0}

    return {
        "merged": merged_count,
        "sources": [str(f.name) for f in valid_sources],
        "target": str(work_stream_path.relative_to(cwd)),
        "message": f"Incorporated {merged_count} tasks and synchronized with WORK_STREAM.md.",
        "validation_errors": validation_errors,
        "dry_run": dry_run,
    }


def _validate_task_and_record_errors(tf: Path, validation_errors: list[dict[str, Any]]) -> None:
    """Validate a single task file and append validation errors."""
    from thegent.task.validator import validate_task_file

    try:
        result = validate_task_file(tf)
        if not result.valid:
            validation_errors.append({"file": str(tf.name), "errors": result.errors})
    except Exception as e:
        validation_errors.append({"file": str(tf.name), "error": str(e)})


def continuity_snapshot_impl(
    owner: str,
    run_ids: list[str],
    state_summary: dict[str, Any] | None = None,
    next_steps: list[str] | None = None,
) -> dict[str, Any]:
    """Create a continuity snapshot for shift handoff (WP-1009)."""
    from thegent.execution import HandoffManager

    settings = ThegentSettings()
    hm = HandoffManager(settings.session_dir)

    snapshot_id = hm.create_snapshot(
        owner,
        run_ids,
    )

    return {
        "snapshot_id": snapshot_id,
        "owner": owner,
        "run_ids": run_ids,
        "state_summary": state_summary,
        "next_steps": next_steps,
    }
