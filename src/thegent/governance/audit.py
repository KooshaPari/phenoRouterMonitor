"""Immutable audit trail and query interface (WP-3004, FR-012)."""

from __future__ import annotations

from typing import TYPE_CHECKING

from thegent.execution import Auditor, RunRegistry

if TYPE_CHECKING:
    from pathlib import Path


def verify_chain(session_dir: Path) -> dict[str, object]:
    """Verify hash chain integrity of the run registry."""
    registry_path = session_dir / "run_registry.jsonl"
    auditor = Auditor(registry_path)
    return auditor.verify_registry()


def query_events(
    session_dir: Path,
    run_id: str | None = None,
    event_type: str | None = None,
    limit: int = 100,
) -> list[dict[str, object]]:
    """Query audit events from the registry."""
    registry = RunRegistry(session_dir)
    runs = registry.list_runs(limit=limit * 2)
    events: list[dict[str, object]] = []
    for r in runs:
        if run_id and r.get("run_id") != run_id:
            continue
        ev = r.get("event") or "start"
        if event_type and ev != event_type:
            continue
        events.append(r)
        if len(events) >= limit:
            break
    return events
