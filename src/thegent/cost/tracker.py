"""WP-Y4: Per-run cost aggregation and budget alerts.

Aggregates actual cost per run and provides budget alerting for the orchestration layer.
"""

from __future__ import annotations

import json
import logging
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

from thegent.integrations.base import SerializableMixin


logger = logging.getLogger(__name__)

# Default path for cost logs
DEFAULT_COST_DIR = Path.home() / ".thegent" / "costs"


@dataclass
class CostEntry(SerializableMixin):
    """Single cost entry for a token or API call."""

    timestamp: str
    provider: str
    model: str
    input_tokens: int
    output_tokens: int
    cost_usd: float
    run_id: str
    task_id: str | None = None


class RunCostTracker:
    """Track and aggregate costs across runs."""

    def __init__(self, cost_dir: Path | None = None) -> None:
        """Initialize tracker.

        Args:
            cost_dir: Directory to store cost reports.
        """
        self.cost_dir = cost_dir or DEFAULT_COST_DIR
        self.cost_dir.mkdir(parents=True, exist_ok=True)
        self.current_run: str | None = None
        self.run_entries: list[CostEntry] = []

    def start_run(self, run_id: str) -> None:
        """Start tracking a new run.

        Args:
            run_id: Unique identifier for the run.
        """
        self.current_run = run_id
        self.run_entries = []
        logger.debug("Started cost tracking for run: %s", run_id)

    def record_entry(self, entry: CostEntry) -> None:
        """Record a cost entry.

        Args:
            entry: The cost entry to record.
        """
        self.run_entries.append(entry)

    def end_run(self) -> dict[str, Any]:
        """End run and return aggregated costs.

        Saves a summary JSON for the run and appends to the global aggregate log.

        Returns:
            Dictionary containing run summary.
        """
        if not self.current_run:
            logger.warning("No active run to end.")
            return {}

        total_cost = sum(e.cost_usd for e in self.run_entries)
        total_input = sum(e.input_tokens for e in self.run_entries)
        total_output = sum(e.output_tokens for e in self.run_entries)

        summary = {
            "run_id": self.current_run,
            "total_cost_usd": round(total_cost, 6),
            "total_input_tokens": total_input,
            "total_output_tokens": total_output,
            "providers": {},
            "ended_at": datetime.now(UTC).isoformat(),
        }

        # Aggregate by provider and model
        for entry in self.run_entries:
            provider = entry.provider
            if provider not in summary["providers"]:
                summary["providers"][provider] = {
                    "cost_usd": 0.0,
                    "input_tokens": 0,
                    "output_tokens": 0,
                    "models": {},
                }
            p = summary["providers"][provider]
            p["cost_usd"] += entry.cost_usd
            p["input_tokens"] += entry.input_tokens
            p["output_tokens"] += entry.output_tokens

            model = entry.model
            if model not in p["models"]:
                p["models"][model] = {"cost_usd": 0.0, "calls": 0}
            p["models"][model]["cost_usd"] += entry.cost_usd
            p["models"][model]["calls"] += 1

        # Save run-specific summary
        self._save_run_summary(summary)

        # Append to global aggregate
        self._append_to_aggregate(summary)

        logger.info("Run %s completed. Total cost: $%.6f", self.current_run, total_cost)

        # Clear state
        _run_id = self.current_run
        self.current_run = None
        self.run_entries = []

        return summary

    def _save_run_summary(self, summary: dict[str, Any]) -> None:
        """Save run summary to JSON file."""
        run_file = self.cost_dir / f"{summary['run_id']}.json"
        try:
            run_file.write_text(json.dumps(summary, indent=2), encoding="utf-8")
        except OSError as e:
            logger.error("Failed to save run summary: %s", e)

    def _append_to_aggregate(self, summary: dict[str, Any]) -> None:
        """Append to aggregate.jsonl log."""
        aggregate_file = self.cost_dir / "aggregate.jsonl"
        try:
            with aggregate_file.open("a", encoding="utf-8") as f:
                f.write(
                    json.dumps(
                        {
                            "run_id": summary["run_id"],
                            "total_cost": summary["total_cost_usd"],
                            "ended_at": summary["ended_at"],
                        }
                    )
                    + "\n"
                )
        except OSError as e:
            logger.error("Failed to update aggregate cost log: %s", e)


# Global tracker for orchestration
_tracker: RunCostTracker | None = None


def get_run_cost_tracker() -> RunCostTracker:
    """Get global run cost tracker instance."""
    global _tracker
    if _tracker is None:
        _tracker = RunCostTracker()
    return _tracker
