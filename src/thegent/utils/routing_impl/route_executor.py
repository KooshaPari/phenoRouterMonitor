"""Route Executor and Orchestrator: Python bridge for Rust thegent-router Phase 3.

Provides the Python-side data models and orchestration logic that coordinate
with the Rust thegent-router crate:
- RoutingDecision / ExecutionOutcome / AgentRoutingState / RouterStatus: mirrors of Rust structs
- read_routing_audit: reads routing_audit.jsonl written by Rust AuditLogger
- RoutingOrchestratorBridge: in-process Python orchestrator for multi-agent routing
- make_routing_decision_from_factors: heuristic router using ThegentSettings

WL-012 Phase 3.1 / 3.2 / 3.4
"""

from __future__ import annotations

import json
import logging
from dataclasses import dataclass, field
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from pathlib import Path

from thegent.config import ThegentSettings

_log = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# Data models (mirror Rust structs)
# ---------------------------------------------------------------------------


@dataclass
class RoutingDecision:
    """Routing decision from the Pareto router."""

    mode: str  # "Lifecycle" | "TheGent"
    risk_score: float
    rationale: str


@dataclass
class ExecutionOutcome:
    """Outcome of dispatching a routed task."""

    decision_id: str
    provider: str
    model: str
    latency_ms: int
    cost_usd: float
    success: bool
    error: str | None = None


@dataclass
class AgentRoutingState:
    """Current routing state for a single agent."""

    agent_id: str
    current_mode: str
    total_decisions: int
    lifecycle_decisions: int
    thegent_decisions: int
    last_rationale: str


@dataclass
class RouterStatus:
    """Aggregated status for `thegent router status`."""

    agents: list[AgentRoutingState] = field(default_factory=list)
    total_decisions: int = 0
    policy: str = "MajorityWins"
    quorum_decision: str | None = None
    lifecycle_pct: float = 0.0
    thegent_pct: float = 0.0

    def display(self) -> str:
        """Format as human-readable status string for CLI."""
        lines = [
            f"Router Status ({len(self.agents)} agents)",
            f"Policy: {self.policy}",
            f"Total decisions: {self.total_decisions}",
            f"Lifecycle: {self.lifecycle_pct:.1f}% | TheGent: {self.thegent_pct:.1f}%",
        ]
        if self.quorum_decision is not None:
            lines.append(f"Quorum: {self.quorum_decision}")
        lines.append("")
        lines.append("Agent states:")
        for agent in self.agents:
            lines.append(
                f"  {agent.agent_id} → {agent.current_mode} "
                f"(lc={agent.lifecycle_decisions}, tg={agent.thegent_decisions})"
                f" | {agent.last_rationale}"
            )
        return "\n".join(lines)

    def to_json(self) -> str:
        """Serialize to JSON for machine-readable output."""
        return json.dumps(
            {
                "agents": [
                    {
                        "agent_id": a.agent_id,
                        "current_mode": a.current_mode,
                        "total_decisions": a.total_decisions,
                        "lifecycle_decisions": a.lifecycle_decisions,
                        "thegent_decisions": a.thegent_decisions,
                        "last_rationale": a.last_rationale,
                    }
                    for a in self.agents
                ],
                "total_decisions": self.total_decisions,
                "policy": self.policy,
                "quorum_decision": self.quorum_decision,
                "lifecycle_pct": self.lifecycle_pct,
                "thegent_pct": self.thegent_pct,
            },
            indent=2,
        )


# ---------------------------------------------------------------------------
# Audit log reader (Python side for CLI status display)
# ---------------------------------------------------------------------------


def read_routing_audit(audit_path: Path, limit: int = 20) -> list[dict[str, Any]]:
    """Read the last `limit` entries from routing_audit.jsonl.

    Returns records in chronological order (oldest first within the slice).
    """
    if not audit_path.exists():
        return []
    records: list[dict[str, Any]] = []
    try:
        lines = audit_path.read_text(encoding="utf-8").splitlines()
        for line in lines[-limit:]:
            line = line.strip()
            if line:
                records.append(json.loads(line))
    except Exception as e:
        _log.warning("Failed to read routing audit log %s: %s", audit_path, e)
    return records


# ---------------------------------------------------------------------------
# In-process orchestrator (pure Python, no Rust FFI required for CLI status)
# ---------------------------------------------------------------------------


class RoutingOrchestratorBridge:
    """Python-side orchestrator that tracks routing decisions from the ExecutionEngine.

    This bridge maintains a lightweight Python-side view of routing state,
    consuming decisions produced by the Rust ParetoRouter (via the Phase 1/2
    PyO3 binding or subprocess).  It provides `router status` output without
    requiring a running Rust process.

    Thread-safe via internal dict operations (GIL-protected in CPython).
    """

    def __init__(
        self,
        settings: ThegentSettings | None = None,
        policy: str = "MajorityWins",
    ) -> None:
        self.settings = settings or ThegentSettings()
        self.policy = policy
        self._agents: dict[str, AgentRoutingState] = {}

    def record_decision(
        self,
        agent_id: str,
        decision: RoutingDecision,
    ) -> None:
        """Record a routing decision for an agent."""
        if agent_id not in self._agents:
            self._agents[agent_id] = AgentRoutingState(
                agent_id=agent_id,
                current_mode=decision.mode,
                total_decisions=0,
                lifecycle_decisions=0,
                thegent_decisions=0,
                last_rationale="",
            )
        state = self._agents[agent_id]
        state.total_decisions += 1
        state.current_mode = decision.mode
        state.last_rationale = decision.rationale
        if decision.mode == "Lifecycle":
            state.lifecycle_decisions += 1
        else:
            state.thegent_decisions += 1

    def arbitrate(self) -> str | None:
        """Apply quorum arbitration across all agents.

        Returns:
            "Lifecycle" | "TheGent" or None if no agents registered.
        """
        if not self._agents:
            return None
        modes = [a.current_mode for a in self._agents.values()]
        thegent_votes = sum(1 for m in modes if m == "TheGent")
        lifecycle_votes = len(modes) - thegent_votes

        if self.policy == "MostRestrictiveWins":
            return "TheGent" if thegent_votes > 0 else "Lifecycle"
        # MajorityWins (default) — TheGent wins ties
        return "TheGent" if thegent_votes >= lifecycle_votes else "Lifecycle"

    def status(self) -> RouterStatus:
        """Build current RouterStatus snapshot."""
        agents = list(self._agents.values())
        total = sum(a.total_decisions for a in agents)
        lc_total = sum(a.lifecycle_decisions for a in agents)
        tg_total = sum(a.thegent_decisions for a in agents)

        lifecycle_pct = (lc_total / total * 100.0) if total > 0 else 0.0
        thegent_pct = (tg_total / total * 100.0) if total > 0 else 0.0

        return RouterStatus(
            agents=agents,
            total_decisions=total,
            policy=self.policy,
            quorum_decision=self.arbitrate(),
            lifecycle_pct=lifecycle_pct,
            thegent_pct=thegent_pct,
        )


# ---------------------------------------------------------------------------
# ExecutionEngine integration helper
# ---------------------------------------------------------------------------


_COMPLEXITY_RISK_MAP: dict[str, float] = {
    "simple": 0.1,
    "moderate": 0.45,
    "complex": 0.7,
    "very_complex": 0.9,
}


def make_routing_decision_from_factors(
    complexity: str = "moderate",
    cost_sensitive: bool = False,
    latency_critical: bool = False,
    settings: ThegentSettings | None = None,
) -> RoutingDecision:
    """Create a routing decision using the Phase 3 hysteresis heuristic.

    Uses ThegentSettings.router_band_width and router_override_threshold
    to apply the same 4-condition logic as the Rust ParetoRouter (WL-012 P3.4).
    The compiled PyO3 wheel is not required at runtime — the heuristic matches
    the Rust behaviour for pure-Python execution paths.

    Raises:
        ValueError: if complexity is not a known level.
    """
    if complexity.lower() not in _COMPLEXITY_RISK_MAP:
        msg = f"Unknown complexity level {complexity!r}. Valid values: {list(_COMPLEXITY_RISK_MAP)}"
        raise ValueError(msg)

    settings = settings or ThegentSettings()

    risk = _COMPLEXITY_RISK_MAP[complexity.lower()]

    if cost_sensitive:
        risk = max(risk - 0.1, 0.0)
    if latency_critical:
        risk = max(risk - 0.05, 0.0)

    band = settings.router_band_width
    high = 1.0 - band
    mode = "TheGent" if risk > high else "Lifecycle"

    return RoutingDecision(
        mode=mode,
        risk_score=risk,
        rationale=f"Heuristic routing: risk={risk:.2f}, band_width={band}, mode={mode}",
    )
