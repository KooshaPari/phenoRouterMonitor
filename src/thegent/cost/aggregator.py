"""Cost governance scaffolding (G-GP-06).

CostEstimator and CostAggregator for per-run cost tracking.
See docs/governance/COST_GOVERNANCE_DESIGN.md.
"""

from __future__ import annotations

import orjson as json
import logging
from dataclasses import dataclass, field
from datetime import UTC, datetime
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path

logger = logging.getLogger(__name__)

# Default $ per MTok (combined I/O, optimized for Pareto routing)
# Standardized to price per 1 Million tokens
_DEFAULT_PRICING_MTOK: dict[str, float] = {
    "claude-haiku-4.5": 0.50,
    "gemini-3-flash": 0.15,
    "gemini-3.1-pro": 3.50,
    "gpt-5.3-codex-spark": 1.20,
    "gpt-5.3-codex": 3.00,
    "claude-sonnet-4.5": 3.00,
    "claude-opus-4.6": 15.00,
    "gpt-5.3-codex-max": 10.00,
    "minimax-m2.5": 0.40,
    "MiniMax-M2.5": 0.40,
    "glm-5": 0.40,
    "GLM-5": 0.40,
    "z-ai/glm-5": 0.40,
    "kilo-default": 0.50,
    "roo-default": 0.50,
}


@dataclass
class CostEstimator:
    """Estimate run cost from metadata. WP-5003: Cost-aware routing integration."""

    pricing_mtok: dict[str, float] = field(default_factory=lambda: dict(_DEFAULT_PRICING_MTOK))

    def estimate(
        self,
        model: str | None = None,
        tokens_total: int = 0,
        prompt_length: int = 0,
    ) -> float:
        """Estimate cost in USD. Uses pricing table or heuristic fallback."""
        price_per_m: float | None = None

        # Try to get from metadata registry first
        if model:
            try:
                from thegent.utils.routing_impl.model_metadata import get_model_metadata

                metadata = get_model_metadata(model)
                if metadata and "cost_per_mtok" in metadata:
                    price_per_m = metadata["cost_per_mtok"]
            except Exception as exc:
                logger.warning("Failed to read model metadata for pricing (%s): %s", model, exc)

        # Fallback to local pricing table
        if price_per_m is None and model and model in self.pricing_mtok:
            price_per_m = self.pricing_mtok[model]

        if price_per_m is not None:
            count = tokens_total or (prompt_length * 1.5 + 500)
            return (count / 1_000_000.0) * price_per_m

        # Fallback heuristic
        est_total = tokens_total or (prompt_length * 1.5 + 500)
        return (est_total / 1_000_000.0) * 2.0  # $2/MTok fallback


@dataclass
class CostAggregator:
    """Daily cost rollup by owner. G-GP-06 Phase 4."""

    session_dir: Path

    def daily_total(self, owner: str | None, days: int = 1) -> float:
        """Sum cost_usd for owner's runs over the past ``days`` days.

        Returns 0.0 if no cost tracking. When ``days=1`` (default) only today's
        runs are included; higher values look back that many calendar days.
        """
        # Scaffolding: read from run_registry.jsonl finish events with cost_usd
        registry_path = self.session_dir / "run_registry.jsonl"
        if not registry_path.exists():
            return 0.0
        from datetime import timedelta

        cutoff = datetime.now(UTC) - timedelta(days=days)
        total = 0.0
        try:
            with registry_path.open("r", encoding="utf-8") as f:
                for line in f:
                    if not line.strip():
                        continue
                    try:
                        data = json.loads(line)
                        if data.get("event") == "finish" and data.get("cost_usd") is not None:
                            ts_str = data.get("ended_at_utc", data.get("timestamp", ""))
                            if ts_str:
                                try:
                                    ts = datetime.fromisoformat(ts_str.replace("Z", "+00:00"))
                                    if ts >= cutoff:
                                        total += float(data["cost_usd"])
                                except ValueError:
                                    continue
                    except Exception:
                        continue
        except Exception as exc:
            logger.warning("Failed to read monthly total from %s: %s", registry_path, exc)
        return total

    def get_mtd_total(self) -> float:
        """Sum cost_usd for all runs this month. G-GP-06 Phase 4."""
        registry_path = self.session_dir / "run_registry.jsonl"
        if not registry_path.exists():
            return 0.0

        now = datetime.now(UTC)
        current_month = f"{now.year}-{now.month:02d}"
        total = 0.0
        try:
            with registry_path.open("r", encoding="utf-8") as f:
                for line in f:
                    if not line.strip():
                        continue
                    try:
                        data = json.loads(line)
                        if data.get("event") == "finish" and data.get("cost_usd") is not None:
                            ts = data.get("ended_at_utc", data.get("timestamp", ""))
                            if ts and ts.startswith(current_month):
                                total += float(data["cost_usd"])
                    except Exception:
                        continue
        except Exception as exc:
            logger.warning("Failed to read category monthly total from %s: %s", registry_path, exc)
        return total

    def get_category_mtd_total(self, category: str) -> float:
        """Sum cost_usd for a specific category this month.

        Args:
            category: Task category (fast/normal/complex/high_complex)

        Returns:
            Total cost in USD for the category this month
        """
        registry_path = self.session_dir / "run_registry.jsonl"
        if not registry_path.exists():
            return 0.0

        now = datetime.now(UTC)
        current_month = f"{now.year}-{now.month:02d}"
        total = 0.0
        try:
            with registry_path.open("r", encoding="utf-8") as f:
                for line in f:
                    if not line.strip():
                        continue
                    try:
                        data = json.loads(line)
                        if (
                            data.get("event") == "finish"
                            and data.get("cost_usd") is not None
                            and data.get("task_category", "").lower() == category.lower()
                        ):
                            ts = data.get("ended_at_utc", data.get("timestamp", ""))
                            if ts and ts.startswith(current_month):
                                total += float(data["cost_usd"])
                    except Exception:
                        continue
        except Exception as exc:
            logger.warning("Failed to read category monthly total from %s: %s", registry_path, exc)
        return total

    def get_all_categories_mtd(self) -> dict[str, float]:
        """Get MTD cost totals for all categories.

        Returns:
            Dictionary mapping category names to MTD costs
        """
        categories = ["fast", "normal", "complex", "high_complex"]
        return {cat: self.get_category_mtd_total(cat) for cat in categories}
