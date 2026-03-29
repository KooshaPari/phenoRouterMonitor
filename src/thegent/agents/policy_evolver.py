"""WP-20001: Self-Evolving Policy Controller.
Analyzes run failures and evolves policy thresholds automatically.
"""

import orjson as json
import logging
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

from thegent.execution import PolicyEngine, RunRegistry

_log = logging.getLogger(__name__)


class PolicyEvolver:
    """Analyzes execution logs and proposes automatic policy adjustments."""

    def __init__(self, session_dir: Path, settings: Any) -> None:
        self.session_dir = session_dir
        self.settings = settings
        self.registry = RunRegistry(session_dir)
        self.policy_engine = PolicyEngine(settings)
        self.proposals_path = session_dir / "policy_proposals.jsonl"

    def evolve(self, lookback_runs: int = 100) -> list[dict[str, Any]]:
        """Analyze recent runs and propose policy updates."""
        runs = self.registry.list_runs(limit=lookback_runs)

        # 1. Analyze failure patterns
        failures = [r for r in runs if r.get("status") == "failed"]
        if not failures:
            _log.info("No failures detected; no policy evolution needed.")
            return []

        # 2. Extract common failure types
        failure_counts = {}
        for r in failures:
            err = r.get("error_class", "unknown")
            failure_counts[err] = failure_counts.get(err, 0) + 1

        proposals = []

        # Logic: If many timeouts, suggest increasing timeout policy
        if failure_counts.get("timeout", 0) > 0.2 * len(failures):
            proposals.append(
                {
                    "principle_id": "P-TIMEOUT-ADAPT",
                    "action": "increase_timeout",
                    "reason": f"Timeouts constitute {failure_counts.get('timeout', 0) / len(failures):.1%} of recent failures.",
                    "suggested_value": 600,
                }
            )

        # Logic: If many policy denies, suggest loosening thresholds if confidence is high
        # (This is a simplified example of meta-learning)
        denies = [r for r in runs if r.get("policy_result") == "deny"]
        if len(denies) > 0.5 * lookback_runs:
            proposals.append(
                {
                    "principle_id": "P-THRESHOLD-ADAPT",
                    "action": "lower_trust_threshold",
                    "reason": "High volume of policy denials suggesting overly restrictive thresholds.",
                    "suggested_value": 0.7,
                }
            )

        # Audit proposals
        for p in proposals:
            self._audit_proposal(p)

        return proposals

    def _audit_proposal(self, proposal: dict[str, Any]) -> None:
        """Audit a policy evolution proposal."""
        self.proposals_path.parent.mkdir(parents=True, exist_ok=True)
        event = {"timestamp": datetime.now(UTC).isoformat(), "proposal": proposal}
        with self.proposals_path.open("a", encoding="utf-8") as f:
            f.write(json.dumps(event).decode() + "\n")
        _log.info("Policy evolution proposal recorded: %s", proposal["principle_id"])
