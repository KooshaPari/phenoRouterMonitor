"""Vetter data models: VetterPolicy, VetterCheck, VetterResult, VetterCheckResult.

Foundation for the Vetter governance layer — pure data types and the VetterCheck Protocol.
Fail fast, fail loudly. No silent fallbacks, no legacy shims.

# @trace WL-090
"""

from __future__ import annotations

import time
from enum import Enum
from typing import Any, Literal, Protocol, runtime_checkable

from pydantic import BaseModel, ConfigDict, Field


class VetterConfigError(Exception):
    """Raised when Vetter is misconfigured (e.g. SafetyCheck with no firewall).

    # @trace WL-090
    """


class VetterVerdict(str, Enum):
    """Four-verdict taxonomy for all vetting decisions.

    # @trace WL-090
    """

    APPROVED = "approved"
    REJECTED = "rejected"
    ESCALATED = "escalated"
    REVISION_REQUESTED = "revision_requested"


class VetterCheckResult(BaseModel):
    """Per-check result returned by each VetterCheck implementation.

    Frozen: immutable after construction.
    # @trace WL-090
    """

    model_config = ConfigDict(frozen=True)

    check_name: str
    passed: bool
    score: float | None = None
    message: str = ""
    metadata: dict[str, Any] = Field(default_factory=dict)


class VetterResult(BaseModel):
    """Aggregate result returned after running all checks against a policy.

    Frozen: immutable after construction.
    # @trace WL-090
    """

    model_config = ConfigDict(frozen=True)

    run_id: str
    verdict: VetterVerdict
    check_results: list[VetterCheckResult]
    duration_ms: int = 0
    revision_prompt: str | None = None
    escalation_reason: str | None = None
    timestamp: float = Field(default_factory=time.time)


class VetterPolicy(BaseModel):
    """Policy defining which checks to run and how to interpret results.

    Frozen: immutable after construction.
    # @trace WL-090
    """

    model_config = ConfigDict(frozen=True)

    checks: list[str]
    fail_fast: bool = False
    on_fail: Literal["reject", "escalate"] = "reject"
    escalation_lane: str = "standard"
    escalate_on: list[str] = Field(default_factory=list)
    thresholds: dict[str, float] = Field(default_factory=dict)
    max_revision_rounds: int = 3
    bypass_checks: list[str] = Field(default_factory=list)


@runtime_checkable
class VetterCheck(Protocol):
    """Structural subtyping interface for all Vetter check implementations.

    All conforming classes must expose:
    - name: str
    - async check(run_id, output, context) -> VetterCheckResult

    Uses Protocol (not ABC) for structural subtyping — duck typing with isinstance support.
    # @trace WL-090
    """

    name: str

    async def check(
        self,
        run_id: str,
        output: str,
        context: dict[str, Any],
    ) -> VetterCheckResult:
        """Run this check against agent output. Fail fast, fail loudly."""
        ...
