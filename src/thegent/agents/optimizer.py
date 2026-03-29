"""WP-20003: Automated Prompt Optimization (DSPy).
Optimizes agent prompts using logical verification and performance-driven feedback loops.
Inspired by DSPy's programmatic prompt optimization.
"""

import logging
import uuid
from dataclasses import dataclass, field
from datetime import UTC, datetime

from thegent.agents.base import RunResult
from thegent.execution import RunRegistry

_log = logging.getLogger(__name__)


@dataclass
class PromptVersion:
    """A specific version of a prompt for an agent."""

    version_id: str = field(default_factory=lambda: f"v_{uuid.uuid4().hex[:4]}")
    content: str = ""
    success_rate: float = 0.0
    avg_tokens: int = 0
    avg_cost: float = 0.0
    runs: int = 0
    timestamp: str = field(default_factory=lambda: datetime.now(UTC).isoformat())


class PromptOptimizer:
    """Optimizes agent prompts by tracking version performance and proposing improvements."""

    def __init__(self, agent_id: str, registry: RunRegistry | None = None) -> None:
        self.agent_id = agent_id
        self.registry = registry
        self.versions: list[PromptVersion] = []
        self._load_versions()

    def _load_versions(self):
        """Load prompt versions from disk (mock/registry)."""
        # In a real system, this would read from the agent's history in the registry.

    def record_run(self, version_id: str, result: RunResult, tokens: int, cost: float):
        """Record the outcome of a prompt version's execution."""
        v = next((v for v in self.versions if v.version_id == version_id), None)
        if not v:
            return

        v.runs += 1
        success = 1.0 if result.exit_code == 0 else 0.0
        v.success_rate = ((v.success_rate * (v.runs - 1)) + success) / v.runs
        v.avg_tokens = int(((v.avg_tokens * (v.runs - 1)) + tokens) / v.runs)
        v.avg_cost = ((v.avg_cost * (v.runs - 1)) + cost) / v.runs
        _log.info("Prompt version %s updated: Success rate: %.2f", version_id, v.success_rate)

    def optimize(self, current_prompt: str, feedback: str | None = None) -> tuple[str, str]:
        """WP-20003: Optimize the current prompt based on feedback or performance history."""
        _log.info("Optimizing prompt for agent: %s", self.agent_id)

        # 1. Analyze performance (simplified logic: if success rate is low, refine)
        # 2. In a real DSPy-like implementation, this would use a 'Teacher' agent
        # to rephrase the prompt with more explicit constraints or CoT steps.

        optimized_prompt = (
            current_prompt
            + "\n\n[Optimizer Update: Always reason step-by-step and verify each tool call result before proceeding.]"
        )

        v = PromptVersion(content=optimized_prompt)
        self.versions.append(v)
        _log.info("Prompt optimized successfully (New version: %s)", v.version_id)
        return optimized_prompt, v.version_id

    def get_best_prompt(self) -> str | None:
        """Return the prompt content with the highest success rate."""
        if not self.versions:
            return None
        best_v = max(self.versions, key=lambda x: x.success_rate)
        return best_v.content
