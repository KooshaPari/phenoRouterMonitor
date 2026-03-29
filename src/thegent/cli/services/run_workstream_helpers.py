"""Workstream/planning helper service wrappers for impl.py compatibility."""

from __future__ import annotations

import logging
import orjson as json
from pathlib import Path
from typing import Any

_log = logging.getLogger(__name__)


def parse_work_stream_md(work_stream_path: Path) -> dict[str, Any]:
    """Parse WORK_STREAM.md into structured data."""
    if not work_stream_path.exists():
        return {"backlog": [], "claimed": [], "completed": []}

    content = work_stream_path.read_text(encoding="utf-8")
    lines = content.splitlines()

    backlog: list[dict[str, Any]] = []
    claimed: set[str] = set()
    completed: set[str] = set()

    current_section: str | None = None
    in_table = False
    header_seen = False

    for _i, line in enumerate(lines):
        stripped = line.strip()

        if stripped.startswith(("## BACKLOG", "## PENDING")):
            current_section = "backlog"
            in_table = False
            header_seen = False
            continue
        if stripped.startswith("## CLAIMED"):
            current_section = "claimed"
            in_table = False
            header_seen = False
            continue
        if stripped.startswith("## COMPLETED"):
            current_section = "completed"
            in_table = False
            header_seen = False
            continue

        if stripped.startswith("## ") and current_section != "backlog":
            current_section = None
            continue

        if (
            current_section
            and stripped.startswith("|")
            and "ID" in stripped.upper()
            and ("Title" in stripped or "Description" in stripped)
        ):
            header_seen = True
            in_table = True
            continue

        if current_section and stripped.startswith("|") and "|" in stripped[1:]:
            if not header_seen:
                if "ID" in stripped.upper() or "----" in stripped:
                    header_seen = True
                    in_table = True
                    continue
            elif in_table or header_seen:
                parts = [p.strip() for p in stripped.split("|") if p.strip()]
                if len(parts) >= 2:
                    item_id = parts[0]
                    if item_id.startswith("---") or item_id.upper() == "ID":
                        continue

                    row_status = parts[5].upper() if len(parts) >= 6 else ""

                    if current_section == "backlog" or row_status == "PENDING":
                        if "COMPLETED" in row_status:
                            completed.add(item_id)
                            continue
                        if "CLAIMED" in row_status or "IN_PROGRESS" in row_status:
                            claimed.add(item_id)
                            continue

                        title = parts[1] if len(parts) > 1 else ""
                        task_type = parts[2] if len(parts) > 2 else "feature"
                        depends_str = parts[3] if len(parts) > 3 else ""
                        depends = [d.strip() for d in depends_str.split(",") if d.strip()] if depends_str else []

                        backlog.append(
                            {
                                "id": item_id,
                                "title": title,
                                "description": title,
                                "source": task_type,
                                "priority": "P2",
                                "depends": depends,
                            }
                        )
                    elif current_section == "claimed" or "CLAIMED" in row_status or "IN_PROGRESS" in row_status:
                        claimed.add(item_id)
                    elif current_section == "completed" or "COMPLETED" in row_status:
                        completed.add(item_id)

    return {"backlog": backlog, "claimed": claimed, "completed": completed}


def check_dependencies_satisfied(item: dict[str, Any], completed: set[str], claimed: set[str]) -> bool:
    """Check if all dependencies for an item are satisfied (completed or claimed)."""
    depends = item.get("depends", [])
    if not depends:
        return True

    ignore_patterns = ["-", "—", "✅", "COMPLETE", "HYBRID_ENV", "PROMPT_HISTORY"]

    actual_depends: list[str] = []
    for dep in depends:
        dep_clean = dep.strip()
        if not dep_clean:
            continue
        if any(p in dep_clean.upper() for p in ignore_patterns):
            continue
        actual_depends.append(dep_clean)

    if not actual_depends:
        return True

    return all(dep in completed for dep in actual_depends)


def priority_sort_key(priority: str) -> int:
    """Convert priority string (P1, P2, P3) to sortable integer."""
    if priority.startswith("P"):
        try:
            return int(priority[1:])
        except ValueError:
            _log.debug("Invalid priority '%s'; defaulting sort priority to 999.", priority)
            return 999
    return 999


def collect_work_stream_items(work_stream_path: Path, limit: int) -> tuple[list[dict[str, Any]], list[str]]:
    """Collect available items from WORK_STREAM.md. Returns (items, sources_checked)."""
    if not work_stream_path.exists():
        return [], []
    parsed = parse_work_stream_md(work_stream_path)
    backlog = parsed["backlog"]
    claimed = parsed["claimed"]
    completed = parsed["completed"]
    available: list[dict[str, Any]] = []
    for item in backlog:
        item_id = item["id"]
        if item_id in claimed or item_id in completed:
            continue
        if not check_dependencies_satisfied(item, completed, claimed):
            continue
        available.append(item)
    available.sort(key=lambda x: priority_sort_key(x.get("priority", "P2")))
    items = []
    for item in available[:limit]:
        title = item.get("title", item.get("description", item["id"]))
        items.append(
            {
                "id": item["id"],
                "description": title,
                "source": item.get("source", "WORK_STREAM"),
                "priority": item.get("priority", "P2"),
                "prompt_suggestion": f"Complete {item['id']}: {title}",
                "_sort_order": 4,
            }
        )
    return items, ["WORK_STREAM.md"]


def collect_queued_items(settings: Any, limit: int) -> tuple[list[dict[str, Any]], list[str]]:
    """Collect defers and other queued work from PromptQueue, EscalationQueue, DeferralQueue, BacklogManager."""
    items: list[dict[str, Any]] = []
    sources: list[str] = []
    session_dir = Path(settings.session_dir).expanduser().resolve()

    try:
        from thegent.queue.storage import PromptQueue

        pq = PromptQueue(session_dir)
        all_items = pq.list_all(include_done=False, include_expired=True, limit=limit)
        pending_items = [(it["id"], it) for it in all_items if it.get("status") == "pending"]
        for queue_item_id, p in pending_items:
            prompt = p.get("prompt", "")
            project = p.get("project", "")
            items.append(
                {
                    "id": f"defer-{queue_item_id}",
                    "description": prompt[:80] + ("..." if len(prompt) > 80 else ""),
                    "source": "PROMPT_QUEUE",
                    "priority": "P1",
                    "prompt_suggestion": prompt,
                    "queue_item_id": queue_item_id,
                    "project": project,
                    "_sort_order": 1,
                }
            )
        if pending_items:
            sources.append("PROMPT_QUEUE")
    except Exception as exc:
        _log.debug("Failed to collect prompt-queue items: %s", exc)

    try:
        from thegent.execution import EscalationQueue

        eq = EscalationQueue(session_dir)
        past_sla = eq.list_pending(past_sla_only=True, limit=limit)
        for e in past_sla:
            run_id = e.get("run_id", "?")
            reason = e.get("reason", "")
            items.append(
                {
                    "id": f"escalation-{run_id}",
                    "description": f"Resolve escalation: {reason[:60]}",
                    "source": "ESCALATION",
                    "priority": "P0",
                    "prompt_suggestion": f"Resolve escalation {run_id}: {reason}",
                    "run_id": run_id,
                    "_sort_order": 0,
                }
            )
        if past_sla:
            sources.append("ESCALATION")
    except Exception as exc:
        _log.debug("Failed to collect escalation queue items: %s", exc)

    try:
        from thegent.orchestration.resilience.deferral import DeferralManager

        dm = DeferralManager(settings)
        deferred = dm.list_deferred()
        for d in deferred[:limit]:
            task_id = d.get("task_id", "?")
            reason = d.get("reason", "")
            items.append(
                {
                    "id": f"deferral-{task_id}",
                    "description": f"Resume deferred: {reason[:60]}",
                    "source": "DEFERRAL",
                    "priority": "P1",
                    "prompt_suggestion": f"Resume deferred task {task_id}",
                    "task_id": task_id,
                    "_sort_order": 2,
                }
            )
        dq_path = session_dir / "deferral_queue.jsonl"
        if dq_path.exists():
            with dq_path.open("r", encoding="utf-8") as f:
                for line in f:
                    if not line.strip():
                        continue
                    try:
                        d = json.loads(line)
                        if d.get("status") != "deferred":
                            continue
                        run_id = d.get("run_id", "?")
                        reason = d.get("reason", "")
                        items.append(
                            {
                                "id": f"deferral-{run_id}",
                                "description": f"Resume deferred run: {reason[:60]}",
                                "source": "DEFERRAL",
                                "priority": "P1",
                                "prompt_suggestion": f"Resume deferred run {run_id}",
                                "run_id": run_id,
                                "_sort_order": 2,
                            }
                        )
                        if len([i for i in items if i.get("source") == "DEFERRAL"]) >= limit:
                            break
                    except Exception:
                        continue
        if any(i.get("source") == "DEFERRAL" for i in items):
            sources.append("DEFERRAL")
    except Exception as exc:
        _log.debug("Failed to collect deferral queue items: %s", exc)

    try:
        from thegent.governance.backlog import BacklogManager

        bm = BacklogManager(session_dir)
        pending = bm.get_pending()
        for p in pending[:limit]:
            item_id = p.item_id
            desc = p.description[:60] + ("..." if len(p.description) > 60 else "")
            items.append(
                {
                    "id": f"backlog-{item_id}",
                    "description": desc,
                    "source": "BACKLOG",
                    "priority": "P2",
                    "prompt_suggestion": f"Address finding {p.finding_id}: {p.description}",
                    "backlog_item_id": item_id,
                    "_sort_order": 3,
                }
            )
        if pending:
            sources.append("BACKLOG")
    except Exception as exc:
        _log.debug("Failed to collect backlog items: %s", exc)

    return items, sources
