"""MCP Sampling primitive — server-initiated analysis via MCP Sampling.

Traceability: FR-049 / WP14-T084b
"""

from __future__ import annotations

import logging
from typing import Any

logger = logging.getLogger(__name__)


class SamplingHandler:
    """Handles server-initiated sampling requests.

    Implements three sampling workflows:
    1. Auto-triage: analyse agent output and classify bugs/issues.
    2. Governance pre-check: validate before state transitions.
    3. Retrospective generation: analyse feature history.
    """

    def __init__(self, client: Any) -> None:
        self._client = client

    async def auto_triage(self, feature_slug: str, agent_output: str) -> dict[str, Any]:
        """Analyse agent output and classify issues.

        Args:
            feature_slug: The feature the agent was working on.
            agent_output: Raw stdout/stderr from the agent run.

        Returns:
            Triage result with ``severity``, ``category``, and ``remediation``.
        """
        logger.info("Auto-triage sampling for feature %s", feature_slug)
        # In a full implementation this would call the MCP sampling API
        # to have the connected LLM classify the output.
        lines = agent_output.splitlines()
        has_error = any("error" in line.lower() for line in lines)
        has_warning = any("warning" in line.lower() for line in lines)

        severity = "error" if has_error else ("warning" if has_warning else "info")
        return {
            "feature_slug": feature_slug,
            "severity": severity,
            "category": "build_failure" if has_error else "lint_warning" if has_warning else "ok",
            "remediation": "Review agent output and fix failing tests." if has_error else "",
            "raw_lines": len(lines),
        }

    async def governance_pre_check(
        self, feature_slug: str, planned_transition: str
    ) -> dict[str, Any]:
        """Proactively validate governance before a state transition.

        Args:
            feature_slug: The feature about to transition.
            planned_transition: The transition string (e.g. ``implementing->validated``).

        Returns:
            Pre-check result with ``ready`` bool and ``blockers`` list.
        """
        logger.info(
            "Governance pre-check for %s: transition=%s", feature_slug, planned_transition
        )
        gate = await self._client.check_governance_gate(feature_slug, planned_transition)
        return {
            "feature_slug": feature_slug,
            "transition": planned_transition,
            "ready": gate["passed"],
            "blockers": [v["message"] for v in gate.get("violations", [])],
        }

    async def generate_retrospective(self, feature_slug: str) -> dict[str, Any]:
        """Server-initiated retrospective analysis of feature history.

        Args:
            feature_slug: The shipped feature to analyse.

        Returns:
            Retrospective summary with ``highlights``, ``issues``, and ``metrics``.
        """
        logger.info("Retrospective sampling for feature %s", feature_slug)
        audit = await self._client.get_audit_trail(feature_slug, verify=True)
        entries = audit.get("entries", [])
        verification = audit.get("verification", {})

        return {
            "feature_slug": feature_slug,
            "total_transitions": len(entries),
            "audit_valid": verification.get("valid", True),
            "highlights": [
                f"Feature transitioned through {len(entries)} states",
            ],
            "issues": [] if verification.get("valid", True) else ["Audit chain integrity failure"],
            "metrics": {
                "entries_verified": verification.get("entries_verified", len(entries)),
            },
        }
