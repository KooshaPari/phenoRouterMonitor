"""Automated compliance reporting."""

import json
import logging
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

logger = logging.getLogger(__name__)


class ComplianceReporter:
    """Generate automated compliance reports."""

    def __init__(self) -> None:
        """Initialize compliance reporter."""
        self.reports: list[dict[str, Any]] = []

    def generate_report(
        self,
        compliance_data: dict[str, Any],
        format: str = "json",
    ) -> str:
        """Generate compliance report.

        Args:
            compliance_data: Compliance data dictionary
            format: Report format (json, markdown, html)

        Returns:
            Report content as string
        """
        report = {
            "timestamp": datetime.now(UTC).isoformat(),
            "compliance": compliance_data,
        }

        if format == "json":
            return json.dumps(report, indent=2)
        if format == "markdown":
            return self._generate_markdown(report)
        if format == "html":
            return self._generate_html(report)
        raise ValueError("Unsupported compliance report format")

    def generate_governance_rollup(self, evidence: list[dict[str, Any]]) -> dict[str, Any]:
        """Build deterministic governance rollup aggregates."""
        by_kind: dict[str, int] = {}
        by_actor: dict[str, int] = {}
        action_required = 0
        for item in evidence:
            kind = str(item.get("kind", "unknown"))
            actor = str(item.get("actor", "unknown"))
            by_kind[kind] = by_kind.get(kind, 0) + 1
            by_actor[actor] = by_actor.get(actor, 0) + 1
            payload = item.get("payload")
            if isinstance(payload, dict) and bool(payload.get("requires_action")):
                action_required += 1
        return {
            "total_records": len(evidence),
            "action_required_records": action_required,
            "by_kind": dict(sorted(by_kind.items())),
            "by_actor": dict(sorted(by_actor.items())),
        }

    def build_governance_queue(self, evidence: list[dict[str, Any]]) -> list[dict[str, Any]]:
        """Create action queue ordered by severity then time."""
        severity_rank = {"critical": 0, "high": 1, "medium": 2, "low": 3}
        queue: list[dict[str, Any]] = []
        for item in evidence:
            payload = item.get("payload")
            if not isinstance(payload, dict) or not bool(payload.get("requires_action")):
                continue
            severity = str(payload.get("severity", "low")).lower()
            queue.append(
                {
                    "evidence_id": item.get("evidence_id"),
                    "timestamp_utc": item.get("timestamp_utc"),
                    "severity": severity,
                    "reason": payload.get("reason", ""),
                }
            )
        queue.sort(key=lambda x: (severity_rank.get(str(x["severity"]), 99), str(x["timestamp_utc"])))
        return queue

    def generate_governance_telemetry(self, *, rollup: dict[str, Any], queue: list[dict[str, Any]]) -> dict[str, Any]:
        """Project key telemetry counters from rollup and queue."""
        return {
            "total_records": int(rollup.get("total_records", 0)),
            "unique_kinds": len(dict(rollup.get("by_kind", {}))),
            "unique_actors": len(dict(rollup.get("by_actor", {}))),
            "queue_depth": len(queue),
            "action_required_records": int(rollup.get("action_required_records", 0)),
        }

    def _generate_markdown(self, report: dict[str, Any]) -> str:
        """Generate markdown report.

        Args:
            report: Report dictionary

        Returns:
            Markdown string
        """
        lines = ["# Compliance Report", ""]
        lines.append(f"**Generated**: {report['timestamp']}")
        lines.append("")
        lines.append("## Compliance Status")
        lines.append("")

        compliance = report.get("compliance", {})
        for key, value in compliance.items():
            lines.append(f"- **{key}**: {value}")

        return "\n".join(lines)

    def _generate_html(self, report: dict[str, Any]) -> str:
        """Generate HTML report.

        Args:
            report: Report dictionary

        Returns:
            HTML string
        """
        html = ["<html><head><title>Compliance Report</title></head><body>"]
        html.append("<h1>Compliance Report</h1>")
        html.append(f"<p><strong>Generated</strong>: {report['timestamp']}</p>")
        html.append("<h2>Compliance Status</h2>")
        html.append("<ul>")

        compliance = report.get("compliance", {})
        for key, value in compliance.items():
            html.append(f"<li><strong>{key}</strong>: {value}</li>")

        html.append("</ul></body></html>")
        return "\n".join(html)

    def export_report(
        self,
        compliance_data: dict[str, Any],
        output_path: Path,
        format: str = "json",
    ) -> Path:
        """Export compliance report to file.

        Args:
            compliance_data: Compliance data
            output_path: Output file path
            format: Report format

        Returns:
            Path to exported file
        """
        content = self.generate_report(compliance_data, format)
        output_path.write_text(content)
        logger.info(f"Exported compliance report: {output_path}")
        return output_path
