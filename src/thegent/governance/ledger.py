"""WP-5006: Ledger integrity verification."""

import hashlib
import json
import logging
from pathlib import Path
from typing import Any

_log = logging.getLogger(__name__)


class LedgerVerifier:
    """Verifies the integrity of the action ledger using rolling hashes."""

    def __init__(self, ledger_path: Path) -> None:
        self.ledger_path = ledger_path

    def verify_integrity(self) -> dict[str, Any]:
        """Verify the rolling hash chain in the ledger."""
        report = {"valid": True, "count": 0, "errors": []}

        if not self.ledger_path.exists():
            return report

        last_hash = ""
        with self.ledger_path.open("r", encoding="utf-8") as f:
            for i, line in enumerate(f):
                last_hash = self._process_ledger_line(line, i, last_hash, report)

        return report

    def _process_ledger_line(self, line: str, i: int, last_hash: str, report: dict[str, Any]) -> str:
        """Helper to process a single ledger line."""
        try:
            entry = json.loads(line)
            # WP-3002: Action signatures
            # Expect entry to have 'rolling_hash' and 'prev_hash'
            prev_hash = entry.get("prev_hash", "")
            if prev_hash != last_hash:
                report["valid"] = False
                report["errors"].append(f"Hash mismatch at line {i + 1}")

            # Compute rolling hash of current entry (minus its own hash)
            content = line.strip().split(',"rolling_hash"')[0] + "}"
            current_hash = hashlib.sha256(content.encode()).hexdigest()
            report["count"] += 1
            return current_hash
        except Exception as e:
            report["valid"] = False
            report["errors"].append(f"Error at line {i + 1}: {e}")
            return last_hash


class IncidentLedger(LedgerVerifier):
    """Immutable incident ledger with rolling hash chain (WP-15002)."""

    def __init__(self, ledger_path: Path) -> None:
        super().__init__(ledger_path)
        self._last_hash = ""
        if self.ledger_path.exists():
            with self.ledger_path.open("r", encoding="utf-8") as f:
                for line in f:
                    self._last_hash = self._get_last_hash_from_line(line, self._last_hash)

    def _get_last_hash_from_line(self, line: str, current_last_hash: str) -> str:
        """Helper to extract last hash from a line."""
        try:
            entry = json.loads(line)
            return entry.get("rolling_hash", current_last_hash)
        except Exception:
            return current_last_hash

    def record_artifact(self, run_id: str, action: str, payload: dict[str, Any]) -> str:
        """Append artifact with rolling hash; return computed hash."""
        prev_hash = self._last_hash
        entry = {
            "run_id": run_id,
            "action": action,
            "payload": payload,
            "prev_hash": prev_hash,
        }
        content = json.dumps(entry, sort_keys=True, separators=(",", ":"))
        current_hash = hashlib.sha256(content.encode()).hexdigest()
        self._last_hash = current_hash
        # Write rolling_hash last so LedgerVerifier split captures full content
        line = content[:-1] + ',"rolling_hash":"' + current_hash + '"}\n'
        self.ledger_path.parent.mkdir(parents=True, exist_ok=True)
        with self.ledger_path.open("a", encoding="utf-8") as f:
            f.write(line)
        return current_hash

    def get_run_artifacts(self, run_id: str) -> list[dict[str, Any]]:
        """Return all artifacts for run_id."""
        out: list[dict[str, Any]] = []
        if not self.ledger_path.exists():
            return out
        with self.ledger_path.open("r", encoding="utf-8") as f:
            for line in f:
                entry = self._parse_line_if_run_id(line, run_id)
                if entry:
                    out.append(entry)
        return out

    def _parse_line_if_run_id(self, line: str, run_id: str) -> dict[str, Any] | None:
        """Helper to parse line and check run_id."""
        try:
            entry = json.loads(line)
            if entry.get("run_id") == run_id:
                return entry
        except Exception:
            pass
        return None

    def verify_integrity(self) -> bool:
        report = super().verify_integrity()
        return report.get("valid", False)
