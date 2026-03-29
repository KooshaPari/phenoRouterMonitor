"""Multi-agent execution modes and selection policy.

Defines coordination patterns for multi-agent workflows (FR-032).
"""

from enum import StrEnum
from typing import Any

from pydantic import BaseModel


class ExecutionMode(StrEnum):
    """Coordination patterns for multi-agent execution."""

    SEQUENTIAL_DELEGATION = "sequential_delegation"
    PARALLEL_CONSENSUS = "parallel_consensus"
    REVIEW_LOOP = "review_loop"
    ARBITRATION_QUORUM = "arbitration_quorum"
    SOLO = "solo"


class ModeCapability(BaseModel):
    """Metadata for an execution mode."""

    mode: ExecutionMode
    description: str
    min_agents: int = 1
    supports_streaming: bool = True
    coordination_logic: str


# Mode Catalog (XK2)
MODE_CATALOG: dict[ExecutionMode, ModeCapability] = {
    ExecutionMode.SEQUENTIAL_DELEGATION: ModeCapability(
        mode=ExecutionMode.SEQUENTIAL_DELEGATION,
        description="Agents run in sequence, passing state forward.",
        min_agents=2,
        coordination_logic="Output of Agent N is injected into prompt of Agent N+1.",
    ),
    ExecutionMode.PARALLEL_CONSENSUS: ModeCapability(
        mode=ExecutionMode.PARALLEL_CONSENSUS,
        description="Multiple agents run in parallel; output is merged or voted.",
        min_agents=2,
        coordination_logic="Quorum-based arbitration of CSM fields.",
    ),
    ExecutionMode.REVIEW_LOOP: ModeCapability(
        mode=ExecutionMode.REVIEW_LOOP,
        description="Primary agent produces output; secondary agent reviews and retries.",
        min_agents=2,
        coordination_logic="Loop continues until reviewer issues empty issues list.",
    ),
    ExecutionMode.ARBITRATION_QUORUM: ModeCapability(
        mode=ExecutionMode.ARBITRATION_QUORUM,
        description="Quorum-based decision gating across heterogeneous providers.",
        min_agents=3,
        coordination_logic="Majority vote on CSM status and success fields.",
    ),
    ExecutionMode.SOLO: ModeCapability(
        mode=ExecutionMode.SOLO,
        description="Single agent execution (default).",
        min_agents=1,
        coordination_logic="Direct execution via runner.",
    ),
}


def get_mode_capability(mode: str) -> ModeCapability | None:
    """Get capability metadata for a mode string."""
    try:
        m = ExecutionMode(mode)
        return MODE_CATALOG.get(m)
    except ValueError:
        return None


def list_modes() -> list[dict[str, Any]]:
    """List all available execution modes."""
    return [m.model_dump() for m in MODE_CATALOG.values()]
