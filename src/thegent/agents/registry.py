"""Agent registry."""

from __future__ import annotations

import logging
from pathlib import Path
from typing import TYPE_CHECKING

from thegent.agents.codex_proxy import CodexProxyRunner
from thegent.agents.cursor_api_runner import CursorApiRunner
from thegent.agents.direct_agents import DirectAgentRunner

if TYPE_CHECKING:
    from thegent.agents.teammate_runner import TeammateRunner

AGENT_NAMES = [
    "gemini",
    "codex",
    "copilot",
    "opencode",
    "cursor-agent",
    "cursor-api",
    "claude",
    "antigravity",
    "minimax",
    "glm",
    "zen",
    "cliproxy",
    "roo",
    "kilo",
    "summarizer",
]

_logger = logging.getLogger(__name__)

# Agents with native CLIs - use DirectAgentRunner (no proxy required)
# Note: gemini, copilot, claude, codex can have issues; use antigravity/minimax/etc via proxy instead
_DIRECT_AGENTS = frozenset({"cursor-agent", "opencode"})
# Agents that run via CLIProxyAPIPlus (antigravity, minimax, glm, cliproxy, roo, kilo use same backend)
# codex, claude, copilot, gemini moved here for reliability via proxy
_PROXY_AGENTS = frozenset(
    {"antigravity", "minimax", "glm", "zen", "cliproxy", "roo", "kilo", "codex", "claude", "copilot", "gemini"}
)
# Cursor via cursor-api (wisdgod) - OpenAI-compat HTTP backend
_CURSOR_API_AGENTS = frozenset({"cursor-api"})

# Fallback chain when provider hits usage limit (subscription/quota exhausted).
# Order: try next provider in list. Using proxy-based agents for reliability.
_PROVIDER_FALLBACK_CHAIN: list[list[str]] = [
    ["glm", "zen", "minimax", "antigravity", "cliproxy", "roo", "kilo"],
    ["minimax", "glm", "antigravity", "cliproxy", "roo", "kilo"],
    ["zen", "glm", "minimax", "antigravity", "cliproxy", "roo", "kilo"],
    ["antigravity", "minimax", "glm", "cliproxy", "roo", "kilo"],
    ["cliproxy", "antigravity", "minimax", "glm", "roo", "kilo"],
    ["roo", "kilo", "cliproxy", "antigravity", "minimax", "glm"],
    ["kilo", "roo", "cliproxy", "antigravity", "minimax", "glm"],
    # Native CLIs now use proxy - fallback to proxy agents instead
    ["gemini", "antigravity", "minimax", "glm"],
    ["codex", "antigravity", "minimax", "glm"],
    ["copilot", "antigravity", "minimax", "glm"],
    ["claude", "antigravity", "minimax", "glm"],
    ["cursor-agent", "antigravity", "minimax", "glm"],
    ["cursor-api", "antigravity", "minimax", "glm"],
]

# Label (display/metadata) -> CLI name. Frontmatter/agent_hint use label; run/bg use CLI name.
AGENT_LABELS: dict[str, str] = {"cursor-agent": "cursor", "cursor-api": "cursor-api"}

# Alias (label) -> canonical CLI name.
_AGENT_ALIASES: dict[str, str] = {
    "cursor": "cursor-agent",
    "oc": "opencode",
    "free": "copilot",
    "summarize": "gemini",
    "research": "claude",
    "review": "claude",
    "explain": "gemini",
    "fix": "claude",
    "code": "claude",
    "architect": "claude",
}


def _resolve_agent(agent_name: str) -> str:
    """Resolve alias to canonical CLI name."""
    return _AGENT_ALIASES.get(agent_name, agent_name)


def get_runner(
    agent_name: str,
) -> DirectAgentRunner | CodexProxyRunner | CursorApiRunner | TeammateRunner | None:
    """Get runner for agent. Returns None for unknown."""
    canonical = _resolve_agent(agent_name)
    if canonical in _DIRECT_AGENTS:
        return DirectAgentRunner(canonical, default_model="")
    if canonical in _PROXY_AGENTS or canonical == "summarizer":
        return CodexProxyRunner(canonical)
    if canonical in _CURSOR_API_AGENTS:
        return CursorApiRunner()

    # WP-16001: Support teammates
    from thegent.agents.teammate_runner import TeammateRunner

    try:
        return TeammateRunner(agent_name)
    except ValueError as exc:
        _logger.debug("No teammate found for '%s': %s", agent_name, exc)
        return None
    except Exception as exc:
        raise RuntimeError(f"Failed to create teammate runner for '{agent_name}'") from exc

    return None


def get_fallback_agents(agent_name: str) -> list[str]:
    """Return fallback agents when this provider hits usage limit. Excludes current agent."""
    canonical = _resolve_agent(agent_name)
    for chain in _PROVIDER_FALLBACK_CHAIN:
        if chain and chain[0] == canonical:
            return [a for a in chain[1:] if a != canonical]
    return []


def resolve_agent(agent_name: str | None) -> str | None:
    """Resolve label/alias to canonical CLI name. E.g. 'cursor' -> 'cursor-agent'."""
    if agent_name is None:
        return None
    return _AGENT_ALIASES.get(agent_name, agent_name)


def list_agent_names() -> list[str]:
    """List available agent names (canonical CLI names)."""
    return list(AGENT_NAMES)


def list_droid_names(droids_dir: Path) -> list[str]:
    """List available droid names from .md files (legacy; droids disabled)."""
    d = droids_dir.expanduser().resolve()
    if not d.exists():
        return []
    return [f.stem for f in d.glob("*.md")]


class LearningCandidate:
    """Represents a candidate model or configuration for autonomous learning."""

    def __init__(self, model_id: str, baseline_id: str) -> None:
        self.model_id = model_id
        self.baseline_id = baseline_id
        self.trust_score = 0.0
        self.calibration = 0.0
        self.metrics: dict[str, list[float]] = {}

    def add_metric(self, name: str, value: float):
        if name not in self.metrics:
            self.metrics[name] = []
        self.metrics[name].append(value)


class LearningRegistry:
    """Registry for autonomous learning models and metrics (WP-14001)."""

    def __init__(self) -> None:
        self.canaries: dict[str, LearningCandidate] = {}
        self.active_model: str = "baseline-v1"

    def register_canary(self, canary_id: str, baseline_id: str):
        """Register a new canary model for testing."""
        self.canaries[canary_id] = LearningCandidate(canary_id, baseline_id)

    def record_metric(self, model_id: str, name: str, value: float):
        """Record a performance metric for a model."""
        if model_id in self.canaries:
            self.canaries[model_id].add_metric(name, value)

    def should_rollback(self, canary_id: str) -> bool:
        """Determine if a canary model should be rolled back to baseline."""
        candidate = self.canaries.get(canary_id)
        if not candidate:
            return False

        # Simple heuristic: if any latency > 2s, rollback
        latencies = candidate.metrics.get("latency", [])
        if latencies and any(l > 2.0 for l in latencies):
            self.active_model = candidate.baseline_id
            return True
        return False

    def get_active_model(self) -> str:
        """Get the currently active model ID."""
        return self.active_model

    def promote(self, canary_id: str, require_approval: bool = True) -> bool:
        """Promote a canary model to default status."""
        if require_approval:
            # In a real impl, this would check a HITL signal
            return False

        if canary_id in self.canaries:
            self.active_model = canary_id
            return True
        return False

    def record_feedback(self, model_id: str, success: bool, quality_score: float):
        """Record human or system feedback for a learning candidate."""
        candidate = self.canaries.get(model_id)
        if candidate:
            candidate.trust_score += 0.1 if success else -0.2
            candidate.calibration = (candidate.calibration + quality_score) / 2.0

    def get_candidate(self, model_id: str) -> LearningCandidate | None:
        """Get candidate metadata."""
        return self.canaries.get(model_id)
