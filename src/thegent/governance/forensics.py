"""Forensic incident replay and post-mortem analysis (WP-15002)."""

import orjson as json
from typing import Any

from thegent.governance.ledger import IncidentLedger


class IncidentReplayer:
    """Replays agent execution traces from immutable ledger entries (WP-15002)."""

    def __init__(self, ledger: IncidentLedger) -> None:
        self.ledger = ledger

    def replay(self, run_id: str) -> dict[str, Any]:
        """Reconstruct the execution trace for a specific run from the ledger.

        Args:
            run_id: The ID of the run to replay

        Returns:
            Reconstructed trace dictionary
        """
        artifacts = self.ledger.get_run_artifacts(run_id)
        if not artifacts:
            return {"run_id": run_id, "actions": [], "status": "not_found"}

        # Sort by timestamp if available, or assume ledger order
        actions = []
        for art in artifacts:
            action_entry = {
                "type": art.get("action"),
                "payload": art.get("payload"),
                "hash": art.get("rolling_hash"),
            }
            actions.append(action_entry)

        return {
            "run_id": run_id,
            "actions": actions,
            "replayed_at": "2026-02-19T10:25:00Z",  # Placeholder for real-time
            "ledger_verified": self.ledger.verify_integrity(),
        }

    def generate_incident_report(self, run_id: str) -> str:
        """Generate a human-readable incident report from replayed data."""
        trace = self.replay(run_id)
        if trace.get("status") == "not_found":
            return f"Incident {run_id} not found in ledger."

        report = [
            f"# Incident Report: {run_id}",
            f"Ledger Integrity: {'VALID' if trace['ledger_verified'] else 'COMPROMISED'}",
            "",
            "## Execution Trace",
        ]

        for i, action in enumerate(trace["actions"]):
            report.append(f"{i + 1}. {action['type']} (Hash: {action['hash'][:8]}...)")
            report.append(f"   Payload: {json.dumps(action['payload']).decode()}")

        return "\n".join(report)
