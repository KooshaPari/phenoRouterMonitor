"""Cost tracking for LiteLLM routing.

Tracks LLM costs across sessions with budget alerts and JSONL logging
for integration with the Donut Architecture harvest system.
"""

from __future__ import annotations

import orjson as json
import logging
from dataclasses import dataclass, field
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

logger = logging.getLogger(__name__)

# Default path for cost logs
DEFAULT_COST_LOG_PATH = Path.home() / ".thegent" / "costs.jsonl"


@dataclass
class CostEntry:
    """Single cost tracking entry."""

    timestamp: str
    provider: str
    model: str
    input_tokens: int
    output_tokens: int
    cost_usd: float
    latency_ms: float
    session_id: str | None = None

    def to_json(self) -> dict[str, Any]:
        """Serialize entry to JSON dict."""
        return {
            "timestamp": self.timestamp,
            "provider": self.provider,
            "model": self.model,
            "input_tokens": self.input_tokens,
            "output_tokens": self.output_tokens,
            "cost_usd": self.cost_usd,
            "latency_ms": self.latency_ms,
            "session_id": self.session_id,
        }


@dataclass
class RoutingStats:
    """Routing statistics summary."""

    total_calls: int = 0
    total_cost_usd: float = 0.0
    daily_spend_usd: float = 0.0
    total_tokens: int = 0
    avg_latency_ms: float = 0.0
    budget_remaining: float | None = None
    errors: int = 0
    fallbacks: int = 0
    requests_by_model: dict[str, int] = field(default_factory=dict)
    requests_by_provider: dict[str, int] = field(default_factory=dict)


class CostTracker:
    """Track LLM costs across sessions."""

    def __init__(
        self,
        log_path: Path | None = None,
        daily_budget: float | None = None,
    ) -> None:
        """Initialize cost tracker.

        Args:
            log_path: Path to JSONL cost log file.
            daily_budget: Optional daily budget limit in USD.
        """
        self._log_path = log_path or DEFAULT_COST_LOG_PATH
        self._daily_budget = daily_budget
        self._entries: list[CostEntry] = []
        self._daily_spend: float = 0.0
        self._total_latency_ms: float = 0.0
        self._errors: int = 0
        self._fallbacks: int = 0
        self._requests_by_model: dict[str, int] = {}
        self._requests_by_provider: dict[str, int] = {}

    @property
    def log_path(self) -> Path:
        """Path to the cost log file."""
        return self._log_path

    @property
    def daily_budget(self) -> float | None:
        """Configured daily budget."""
        return self._daily_budget

    def track(
        self,
        provider: str,
        model: str,
        usage: dict[str, int],
        cost: float,
        latency_ms: float,
        session_id: str | None = None,
        is_error: bool = False,
        is_fallback: bool = False,
    ) -> CostEntry:
        """Track a single LLM call cost.

        Args:
            provider: Provider name (e.g., "openai", "anthropic")
            model: Model name (e.g., "gpt-4", "claude-opus")
            usage: Dict with prompt_tokens and completion_tokens
            cost: Cost in USD
            latency_ms: Request latency in milliseconds
            session_id: Optional session identifier
            is_error: Whether the request resulted in an error
            is_fallback: Whether this was a fallback routing

        Returns:
            The created CostEntry
        """
        entry = CostEntry(
            timestamp=datetime.now(UTC).isoformat(),
            provider=provider,
            model=model,
            input_tokens=usage.get("prompt_tokens", 0),
            output_tokens=usage.get("completion_tokens", 0),
            cost_usd=cost,
            latency_ms=latency_ms,
            session_id=session_id,
        )

        self._entries.append(entry)
        self._daily_spend += cost
        self._total_latency_ms += latency_ms

        # Track by model and provider
        self._requests_by_model[model] = self._requests_by_model.get(model, 0) + 1
        self._requests_by_provider[provider] = self._requests_by_provider.get(provider, 0) + 1

        if is_error:
            self._errors += 1
        if is_fallback:
            self._fallbacks += 1

        # Append to log file
        self._append_to_log(entry)

        # Check budget
        if self._daily_budget and self._daily_spend > self._daily_budget:
            logger.warning(
                "Daily budget exceeded: $%.2f / $%.2f",
                self._daily_spend,
                self._daily_budget,
            )

        return entry

    def _append_to_log(self, entry: CostEntry) -> None:
        """Append entry to JSONL log file."""
        try:
            self._log_path.parent.mkdir(parents=True, exist_ok=True)
            with self._log_path.open("a", encoding="utf-8") as f:
                f.write(json.dumps(entry.to_json()).decode("utf-8") + "\n")
        except OSError as e:
            logger.warning("Failed to write cost log %s: %s", self._log_path, e)

    def get_daily_spend(self) -> float:
        """Get today's total spend in USD."""
        return self._daily_spend

    def is_over_budget(self) -> bool:
        """Check if daily budget is exceeded."""
        return self._daily_budget is not None and self._daily_spend > self._daily_budget

    def get_budget_remaining(self) -> float | None:
        """Get remaining budget, or None if no budget set."""
        if self._daily_budget is None:
            return None
        return max(0.0, self._daily_budget - self._daily_spend)

    def get_budget_burn_ratio(self) -> float | None:
        """Get budget burn ratio (0-1). None if no budget set. 0.85+ triggers degraded mode."""
        if self._daily_budget is None or self._daily_budget <= 0:
            return None
        return min(1.0, self._daily_spend / self._daily_budget)

    def get_stats(self) -> RoutingStats:
        """Get cost statistics summary."""
        total_tokens = sum(e.input_tokens + e.output_tokens for e in self._entries)
        avg_latency = (self._total_latency_ms / len(self._entries)) if self._entries else 0.0

        return RoutingStats(
            total_calls=len(self._entries),
            total_cost_usd=sum(e.cost_usd for e in self._entries),
            daily_spend_usd=self._daily_spend,
            total_tokens=total_tokens,
            avg_latency_ms=avg_latency,
            budget_remaining=self.get_budget_remaining(),
            errors=self._errors,
            fallbacks=self._fallbacks,
            requests_by_model=dict(self._requests_by_model),
            requests_by_provider=dict(self._requests_by_provider),
        )

    def clear(self) -> None:
        """Reset all tracking state."""
        self._entries.clear()
        self._daily_spend = 0.0
        self._total_latency_ms = 0.0
        self._errors = 0
        self._fallbacks = 0
        self._requests_by_model.clear()
        self._requests_by_provider.clear()


# Global tracker instance
_tracker: CostTracker | None = None


def get_cost_tracker() -> CostTracker:
    """Get global cost tracker instance.

    Initializes with settings from config on first call.
    """
    global _tracker
    if _tracker is None:
        try:
            from thegent.config import ThegentSettings

            settings = ThegentSettings()
            _tracker = CostTracker(daily_budget=settings.litellm_cost_budget)
        except Exception:
            # Fallback without config
            _tracker = CostTracker()
    return _tracker


def reset_cost_tracker() -> None:
    """Reset the global cost tracker (useful for testing)."""
    global _tracker
    _tracker = None
