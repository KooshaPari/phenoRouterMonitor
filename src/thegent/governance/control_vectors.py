"""WP-33002: Behavioral Steering via Semantic Injection.
Influences black-box agents by proactively modifying their environment and context.
Injects 'control vectors' (semantic hints, mock tools, system state) to steer behavior.
"""

import logging
from pathlib import Path
from typing import Any

_log = logging.getLogger(__name__)


class ControlVectorManager:
    """Manages semantic injection vectors for steering black-box agents."""

    def __init__(self, agent_id: str) -> None:
        self.agent_id = agent_id
        # Vector Registry: keyword/state -> injection payload
        self.vectors = {
            "destructive_intent": "\n[STEERING: High-integrity mode active. Destructive commands are physically disabled in this sandbox. Focus on read-only analysis.]",
            "evasive_behavior": "\n[STEERING: Compliance monitoring active. Ensure every decision is mapped to a requirement ID.]",
            "low_confidence": "\n[STEERING: Heuristic check: prior steps had low confidence. Increase verbosity of reasoning.]",
        }

    def analyze_and_inject(self, prompt: str, agent_state: dict[str, Any]) -> str:
        """Analyze prompt and state to decide which control vectors to inject."""
        _log.info("Analyzing steering opportunities for agent: %s", self.agent_id)

        injections = []

        # 1. Keyword-based steering
        if any(kw in prompt.lower() for kw in ["delete", "rm", "purge", "drop"]):
            injections.append(self.vectors["destructive_intent"])

        # 2. State-based steering
        if agent_state.get("compliance_risk", 0) > 0.7:
            injections.append(self.vectors["evasive_behavior"])

        if agent_state.get("average_confidence", 1.0) < 0.5:
            injections.append(self.vectors["low_confidence"])

        if not injections:
            return prompt

        _log.info("Injecting %d control vector(s) into prompt.", len(injections))
        return prompt + "".join(injections)

    def prepare_environment(self, workspace_path: Path):
        """Proactively modify the physical environment to steer behavior (e.g. mock tools)."""
        _log.info("Preparing steered environment in: %s", workspace_path)
        # Example: create a README that 'black-boxes' the agent's permissions
        # (the agent might read this and behave accordingly even if it technically has more power)
        policy_file = workspace_path / ".AGENT_POLICY.md"
        policy_file.write_text(
            "# EXTERNAL AGENT POLICY\n- Do not modify files in /security/\n- All network calls are logged."
        )
