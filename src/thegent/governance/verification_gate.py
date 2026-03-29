"""Post-agent verification gate for AgilePlus cycles.

Re-runs targeted scanner dimensions after each agent task completes to verify
improvement, detect regressions, and determine pass/fail verdicts.
"""

from __future__ import annotations

import logging
from enum import StrEnum
from typing import Any, Protocol

from pydantic import BaseModel

_log = logging.getLogger(__name__)

DEFAULT_MAX_REROLLS = 2

# Agent tier escalation order for rerolls
AGENT_TIER_ESCALATION = ("writer_fast", "writer_standard", "writer_high")


class VerificationVerdict(StrEnum):
    """Outcome of post-task verification."""

    PASS = "pass"  # noqa: S105
    FAIL = "fail"
    NEUTRAL = "neutral"
    REGRESSION = "regression"


class TaskVerification(BaseModel):
    """Result of verifying a single task's effect on codebase health."""

    task_id: str
    verdict: VerificationVerdict
    metrics_before: dict[str, float]
    metrics_after: dict[str, float]
    deltas: dict[str, float]
    regressions: list[str]
    evidence_id: str


class DimensionScanResult(Protocol):
    """Protocol for a single dimension's scan output."""

    dimension: str
    score: float
    raw_metrics: dict[str, Any]


class ScanResultProtocol(Protocol):
    """Protocol for a full codebase scan result."""

    dimensions: dict[str, Any]

    def get_dimension(self, dimension: str) -> DimensionScanResult | None: ...


class ScannerProtocol(Protocol):
    """Protocol for CodebaseScanner -- only scan_dimension is needed here."""

    def scan_dimension(self, dimension: str) -> Any: ...

    def scan(self) -> Any: ...


class HealthComputerProtocol(Protocol):
    """Protocol for HealthScoreComputer -- used to get dimension weights."""

    def compute(self, dimension_values: dict[str, float]) -> Any: ...


class RemediationTaskProtocol(Protocol):
    """Protocol for a remediation task from the planner."""

    task_id: str
    dimension: str
    agent_tier: str


class TaskExecutionProtocol(Protocol):
    """Protocol for the result of executing a remediation task."""

    task_id: str
    exit_code: int
    run_id: str


class VerificationGate:
    """Verifies that agent tasks actually improved the targeted dimension.

    After each task execution, re-scans the targeted dimension and compares
    against the pre-scan baseline. Detects regressions in other dimensions.
    """

    def __init__(
        self,
        scanner: ScannerProtocol,
        health_computer: HealthComputerProtocol,
        max_rerolls: int = DEFAULT_MAX_REROLLS,
    ) -> None:
        self.scanner = scanner
        self.health_computer = health_computer
        self.max_rerolls = max_rerolls

    def verify_task(
        self,
        task: RemediationTaskProtocol,
        execution: TaskExecutionProtocol,
        pre_scan: ScanResultProtocol,
    ) -> TaskVerification:
        """Verify a completed task by re-scanning its target dimension.

        Compares post-execution metrics against the pre-scan baseline to
        determine whether the task improved, regressed, or had no effect.
        """
        dimension = task.dimension

        pre_dim = pre_scan.get_dimension(dimension)
        pre_score = pre_dim.score if pre_dim else 0.0
        pre_metrics = dict(pre_dim.raw_metrics) if pre_dim else {}

        post_dim = self.scanner.scan_dimension(dimension)
        post_score = post_dim.score
        post_metrics = dict(post_dim.raw_metrics)

        metrics_before = {"score": pre_score, **pre_metrics}
        metrics_after = {"score": post_score, **post_metrics}

        deltas: dict[str, float] = {}
        for key in set(metrics_before) | set(metrics_after):
            before_val = metrics_before.get(key, 0.0)
            after_val = metrics_after.get(key, 0.0)
            deltas[key] = after_val - before_val

        post_scan_partial = {dimension: post_dim}
        regressions = self._check_regressions(pre_scan, post_scan_partial)

        verdict = self._determine_verdict(
            pre_score=pre_score,
            post_score=post_score,
            regressions=regressions,
        )

        evidence_id = f"verify_{task.task_id}_{execution.run_id}"

        verification = TaskVerification(
            task_id=task.task_id,
            verdict=verdict,
            metrics_before=metrics_before,
            metrics_after=metrics_after,
            deltas=deltas,
            regressions=regressions,
            evidence_id=evidence_id,
        )

        _log.info(
            "Verification for task %s: %s (delta=%.4f, regressions=%d)",
            task.task_id,
            verdict,
            deltas.get("score", 0.0),
            len(regressions),
        )

        return verification

    def _determine_verdict(
        self,
        pre_score: float,
        post_score: float,
        regressions: list[str],
    ) -> VerificationVerdict:
        """Determine the verification verdict based on score changes and regressions."""
        score_delta = post_score - pre_score

        if regressions:
            return VerificationVerdict.REGRESSION

        if score_delta > 0:
            return VerificationVerdict.PASS

        if score_delta == 0:
            return VerificationVerdict.NEUTRAL

        return VerificationVerdict.FAIL

    def _check_regressions(
        self,
        pre_scan: ScanResultProtocol,
        post_scan_partial: dict[str, DimensionScanResult],
    ) -> list[str]:
        """Check if any non-target dimensions regressed.

        Only checks dimensions that were re-scanned (present in post_scan_partial).
        For a full regression check, the caller should do a complete post-scan.
        """
        regressions: list[str] = []

        for dim_name, post_dim in post_scan_partial.items():
            pre_dim = pre_scan.get_dimension(dim_name)
            if pre_dim is None:
                continue
            if post_dim.score < pre_dim.score:
                regressions.append(dim_name)
                _log.warning(
                    "Regression in dimension %s: %.4f -> %.4f",
                    dim_name,
                    pre_dim.score,
                    post_dim.score,
                )

        return regressions

    def get_escalated_tier(self, current_tier: str) -> str | None:
        """Return the next agent tier for reroll escalation.

        Returns None if already at the highest tier.
        """
        if current_tier not in AGENT_TIER_ESCALATION:
            return None
        idx = AGENT_TIER_ESCALATION.index(current_tier)
        if idx + 1 >= len(AGENT_TIER_ESCALATION):
            return None
        return AGENT_TIER_ESCALATION[idx + 1]

    def should_reroll(self, attempts: int) -> bool:
        """Return True if the task should be retried based on attempt count."""
        return attempts < self.max_rerolls
