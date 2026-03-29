"""WP-2004: Automated recovery and self-healing orchestration."""

import logging

from thegent.agents.base import RunResult
from thegent.agents.resilience import RecoveryEngine, classify_failure

_log = logging.getLogger(__name__)


class StabilityTracker:
    """Monitors session stability and performance over time."""

    def __init__(self, window_size: int = 10) -> None:
        self.window_size = window_size
        self.results: list[RunResult] = []

    def record_result(self, result: RunResult) -> None:
        """Record a run result and prune old history."""
        self.results.append(result)
        if len(self.results) > self.window_size:
            self.results.pop(0)

    def get_stability_score(self) -> float:
        """Calculate stability score (0.0 - 1.0) based on success rate."""
        if not self.results:
            return 1.0
        successes = sum(1 for r in self.results if r.exit_code == 0)
        return successes / len(self.results)


class RecoveryRouter:
    """Routes failed runs to automated recovery playbooks."""

    def __init__(self) -> None:
        self.engine = RecoveryEngine()

    def attempt_recovery(self, result: RunResult) -> str | None:
        """Suggest a recovery action based on the failure classification."""
        kind = classify_failure(result)
        playbook = self.engine.suggest_playbook(str(kind))
        _log.info("Recovery engine suggests: %s", playbook)

        # In a real implementation, this might return a modified prompt or model override
        if "quota" in playbook.lower() or "usage" in playbook.lower():
            return "Switch to fallback provider (e.g. Gemini Flash)"
        if "context" in playbook.lower():
            return "Compress history and retry"

        return playbook

    def back_project_failure(self, run_id: str, prompt: str, failure_type: str) -> str:
        """WP-16002: Analyze failure and project fix into instructions."""
        _log.info("Back-projecting failure analysis for run %s", run_id)

        # Simulated logic: if failure_type is 'API_QUOTA', suggest batching or rate limiting instructions.
        # This would typically use an LLM to generate a better prompt.
        if "quota" in failure_type.lower():
            instruction_fix = " [Self-Correction: Add explicit instruction to batch API calls to avoid rate limits]"
            updated_prompt = prompt + instruction_fix
        elif "auth" in failure_type.lower():
            instruction_fix = " [Self-Correction: Verify credentials before starting task]"
            updated_prompt = prompt + instruction_fix
        else:
            instruction_fix = " [Self-Correction: Add more granular logging for debugging]"
            updated_prompt = prompt + instruction_fix

        _log.info("Updated prompt for next run: %s", updated_prompt)
        return updated_prompt
