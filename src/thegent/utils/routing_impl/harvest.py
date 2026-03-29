"""WP-7002: LiteLLM cost/latency data harvesting implementation."""

import json
import logging
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

from thegent.utils.routing_impl.cost_tracker import get_cost_tracker

logger = logging.getLogger(__name__)


def harvest_routing_metrics(session_id: str, output_path: Path | str | None = None) -> dict[str, Any]:
    """Harvest cost and latency data for a session and save to output_path.

    Args:
        session_id: The session ID to harvest for.
        output_path: Optional path to save JSON metrics.

    Returns:
        Dictionary of harvested metrics.
    """
    tracker = get_cost_tracker()
    stats = tracker.get_stats()

    # Filter entries for this session if possible
    # Note: CostTracker stores entries in memory for the current process
    # and also appends to a global jsonl file.

    session_entries = []
    log_path = tracker.log_path
    if log_path.exists():
        try:
            with log_path.open("r", encoding="utf-8") as f:
                for line in f:
                    try:
                        entry = json.loads(line)
                        if entry.get("session_id") == session_id:
                            session_entries.append(entry)
                    except json.JSONDecodeError:  # noqa: PERF203 - intentional per-item error handling
                        continue
        except Exception as e:
            logger.warning("Failed to read cost log for harvesting: %s", e)

    # Aggregate session metrics
    total_cost = sum(e.get("cost_usd", 0.0) for e in session_entries)
    total_tokens = sum(e.get("input_tokens", 0) + e.get("output_tokens", 0) for e in session_entries)
    avg_latency = (
        sum(e.get("latency_ms", 0.0) for e in session_entries) / len(session_entries) if session_entries else 0.0
    )

    metrics = {
        "session_id": session_id,
        "harvested_at": datetime.now(UTC).isoformat(),
        "total_cost_usd": total_cost,
        "total_tokens": total_tokens,
        "avg_latency_ms": avg_latency,
        "request_count": len(session_entries),
        "entries": session_entries,
        "global_stats": {
            "daily_spend_usd": stats.daily_spend_usd,
            "budget_remaining": stats.budget_remaining,
        },
    }

    if output_path:
        out_p = Path(output_path)
        out_p.parent.mkdir(parents=True, exist_ok=True)
        try:
            out_p.write_text(json.dumps(metrics, indent=2), encoding="utf-8")
            logger.info("Harvested routing metrics for %s to %s", session_id, out_p)
        except Exception as e:
            logger.error("Failed to write harvested metrics to %s: %s", out_p, e)

    return metrics
