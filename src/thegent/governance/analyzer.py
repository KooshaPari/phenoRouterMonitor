"""Finding prioritisation and ranking for governance scans.

Takes a ScanResult produced by scanner.py and produces a severity-ranked
list of Finding objects that downstream components (backlog, remediation
planner) consume.
"""

from __future__ import annotations

import json
import logging
from typing import TYPE_CHECKING
from uuid import uuid4

from pydantic import BaseModel, Field

if TYPE_CHECKING:
    from pathlib import Path

    from thegent.governance.scanner import DimensionScan, ScanResult

_log = logging.getLogger(__name__)

# Scanner dimension name -> health-targets.json dimension name.
# Where names already match the mapping is identity; explicit mapping
# avoids silent mismatches if either side renames a dimension.
_SCANNER_TO_TARGET: dict[str, str] = {
    "test_coverage": "test_coverage",
    "lint_violations": "lint_violations",
    "doc_disorganization": "doc_organization",
    "fragmented_research": "doc_organization",
    "missing_specs": "spec_traceability",
    "technical_debt": "complexity_index",
    "stale_items": "freshness",
    "agent_failure": "agent_health",
}

# Rough effort estimates (tool calls) per dimension remediation.
_EFFORT_ESTIMATES: dict[str, int] = {
    "test_coverage": 2,
    "lint_violations": 1,
    "doc_disorganization": 1,
    "fragmented_research": 1,
    "missing_specs": 2,
    "technical_debt": 3,
    "stale_items": 1,
    "agent_failure": 2,
}


class Finding(BaseModel):
    """A single actionable finding produced by the analyser."""

    finding_id: str = Field(default_factory=lambda: f"f_{uuid4().hex[:8]}")
    dimension: str
    severity: float = Field(ge=0.0, le=1.0)
    priority: float = Field(ge=0.0)
    current_value: float
    target_value: float
    delta: float
    description: str
    affected_files: list[str] = Field(default_factory=list)
    estimated_effort_tool_calls: int = 1


class HealthAnalyzer:
    """Converts raw scan results into a prioritised list of findings."""

    def __init__(self, health_targets_path: Path) -> None:
        with open(health_targets_path) as fh:
            data = json.load(fh)
        self._targets: dict[str, dict] = data["dimensions"]
        _log.debug(
            "analyzer loaded %d dimension configs from %s",
            len(self._targets),
            health_targets_path,
        )

    # ------------------------------------------------------------------
    # public API
    # ------------------------------------------------------------------

    def analyze(
        self,
        scan_result: ScanResult,
        backlog_items: list[dict] | None = None,
    ) -> list[Finding]:
        """Produce a ranked list of findings from *scan_result*.

        Green dimensions (at or exceeding target) are excluded.

        Args:
            scan_result: output of CodebaseScanner.scan_all().
            backlog_items: optional list of previous backlog entries used to
                boost priority of repeatedly-attempted dimensions.  Each
                entry is expected to carry a ``dimension`` key.

        Returns:
            Findings sorted descending by priority (highest first).
        """
        past_attempts = _count_past_attempts(backlog_items) if backlog_items else {}
        findings: list[Finding] = []

        for dim_name, scan in scan_result.dimensions.items():
            target_key = _SCANNER_TO_TARGET.get(dim_name)
            if target_key is None:
                _log.warning("no target mapping for scanner dimension %r", dim_name)
                continue

            dim_cfg = self._targets.get(target_key)
            if dim_cfg is None:
                _log.warning("dimension %r not in health-targets", target_key)
                continue

            if self._is_green(scan, dim_cfg):
                continue

            severity = self._calculate_severity(
                scan.current_value,
                scan.target_value,
                dim_cfg["direction"],
            )
            weight = dim_cfg["weight"]
            attempts = past_attempts.get(dim_name, 0)
            priority = severity * weight * (1 + 0.1 * attempts)

            findings.append(
                Finding(
                    dimension=dim_name,
                    severity=round(severity, 4),
                    priority=round(priority, 6),
                    current_value=scan.current_value,
                    target_value=scan.target_value,
                    delta=scan.delta,
                    description=_describe(dim_name, scan),
                    affected_files=scan.affected_files,
                    estimated_effort_tool_calls=self._estimate_effort(dim_name),
                ),
            )

        return self._rank_findings(findings)

    # ------------------------------------------------------------------
    # internals
    # ------------------------------------------------------------------

    @staticmethod
    def _is_green(scan: DimensionScan, dim_cfg: dict) -> bool:
        """Return True when the dimension meets or exceeds its target."""
        direction = dim_cfg["direction"]
        if direction == "higher_is_better":
            return scan.current_value >= scan.target_value
        return scan.current_value <= scan.target_value

    @staticmethod
    def _calculate_severity(
        current: float,
        target: float,
        direction: str,
    ) -> float:
        """Map the gap between *current* and *target* to 0.0-1.0 severity.

        For ``higher_is_better``: severity = max(0, 1 - current/target).
        For ``lower_is_better``: severity = min(1, current/10) when target=0,
        otherwise min(1, current/target).
        """
        if direction == "higher_is_better":
            if target <= 0:
                return 0.0
            return max(0.0, min(1.0, 1.0 - current / target))

        # lower_is_better
        if target == 0:
            if current == 0:
                return 0.0
            return min(1.0, current / 10.0)
        return min(1.0, current / target)

    @staticmethod
    def _estimate_effort(dimension: str) -> int:
        """Return estimated tool-call effort for remediating *dimension*."""
        return _EFFORT_ESTIMATES.get(dimension, 3)

    @staticmethod
    def _rank_findings(findings: list[Finding]) -> list[Finding]:
        """Sort findings descending by priority (highest urgency first)."""
        return sorted(findings, key=lambda f: f.priority, reverse=True)


# ------------------------------------------------------------------
# helpers
# ------------------------------------------------------------------


def _count_past_attempts(backlog: list[dict]) -> dict[str, int]:
    """Count how many backlog entries reference each dimension."""
    counts: dict[str, int] = {}
    for entry in backlog:
        dim = entry.get("dimension")
        if dim:
            counts[dim] = counts.get(dim, 0) + 1
    return counts


def _describe(dimension: str, scan: DimensionScan) -> str:
    """Generate a human-readable description for a finding."""
    c = scan.current_value
    t = scan.target_value
    descriptions: dict[str, str] = {
        "test_coverage": f"Test coverage at {c:.0f}% (target: {t:.0f}%)",
        "lint_violations": f"{c:.0f} lint violation(s) found (target: {t:.0f})",
        "doc_disorganization": f"{c:.0f} required doc dir(s) missing (target: {t:.0f})",
        "fragmented_research": f"{c:.0f} research file(s) outside docs/research/ (target: {t:.0f})",
        "missing_specs": f"{c:.0f} approved feature(s) without SPEC.md (target: {t:.0f})",
        "technical_debt": f"Avg cyclomatic complexity at {c:.1f} (target: {t:.0f})",
        "stale_items": f"{c:.0f} stale file(s) in specs/ (target: {t:.0f})",
        "agent_failure": f"{c:.0f} open circuit breaker(s) (target: {t:.0f})",
    }
    return descriptions.get(dimension, f"{dimension} at {c} (target: {t})")
