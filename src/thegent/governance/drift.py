"""WP-3005: Policy drift detection and sweep."""

import json
import logging
import time
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

from thegent.config import ThegentSettings
from thegent.governance.overrides import OverrideManager

_log = logging.getLogger(__name__)


class DriftDetector:
    """Detects drift in policy state and cleans up stale overrides."""

    def __init__(self, settings: ThegentSettings) -> None:
        self.settings = settings
        self.om = OverrideManager(settings)
        self.drift_log = settings.session_dir / "policy_drift.jsonl"

    def detect_drift(self) -> dict[str, Any]:
        """
        Check for drift between current state and baseline.
        Returns a report of detected issues.
        """
        report = {
            "timestamp": datetime.now(UTC).isoformat(),
            "expired_overrides": [],
            "policy_mismatches": [],
            "drift_detected": False,
            "baseline_established": False,
        }

        # 1. Check for expired overrides that haven't been cleaned up
        overrides_dir = self.settings.session_dir / "overrides"
        if overrides_dir.exists():
            for f in overrides_dir.glob("*.json"):
                self._check_override_file(f, report)

        # 2. Check policy contract drift against persisted baseline
        baseline_path = self.settings.session_dir / "policy_contracts_baseline.json"
        current_contracts_dir = self.settings.session_dir / "contracts"
        current_contracts: dict[str, str] = {}
        if current_contracts_dir.exists():
            for p in sorted(current_contracts_dir.glob("*.json")):
                current_contracts[p.name] = p.read_text(encoding="utf-8")

        if not baseline_path.exists():
            baseline_path.write_text(
                json.dumps(
                    {
                        "generated_at_utc": datetime.now(UTC).isoformat(),
                        "contracts": current_contracts,
                    },
                    indent=2,
                    sort_keys=True,
                )
                + "\n",
                encoding="utf-8",
            )
            report["baseline_established"] = True
        else:
            baseline_doc = json.loads(baseline_path.read_text(encoding="utf-8"))
            contracts_obj = baseline_doc.get("contracts")
            if not isinstance(contracts_obj, dict):
                raise ValueError("Invalid policy baseline format")
            baseline_contracts: dict[str, str] = {
                str(k): str(v) for k, v in contracts_obj.items() if isinstance(k, str) and isinstance(v, str)
            }
            for name, base_content in baseline_contracts.items():
                if name not in current_contracts:
                    report["policy_mismatches"].append(
                        {"contract": name, "type": "removed", "diff": f"baseline/{name}"}
                    )
                    report["drift_detected"] = True
                    continue
                cur_content = current_contracts[name]
                if cur_content != base_content:
                    report["policy_mismatches"].append(
                        {"contract": name, "type": "changed", "diff": f"baseline/{name} -> current/{name}"}
                    )
                    report["drift_detected"] = True
            for name in current_contracts:
                if name not in baseline_contracts:
                    report["policy_mismatches"].append({"contract": name, "type": "added", "diff": f"current/{name}"})
                    report["drift_detected"] = True

        if report["drift_detected"]:
            self._log_drift(report)

        return report

    def sweep(self) -> dict[str, int]:
        """
        Perform a sweep to correct detected drift.
        Returns counts of corrected items.
        """
        # Cleanup expired overrides
        cleaned = self.om.cleanup_expired()
        return {"overrides_cleaned": cleaned}

    def _log_drift(self, report: dict[str, Any]) -> None:
        """Append drift report to log."""
        self.settings.session_dir.mkdir(parents=True, exist_ok=True)
        with self.drift_log.open("a", encoding="utf-8") as f:
            f.write(json.dumps(report) + "\n")

    def _check_override_file(self, f: Path, report: dict[str, Any]) -> None:
        """Helper to check a single override file for drift."""
        try:
            with f.open("r") as f_in:
                data = json.load(f_in)
                # WP-3003 stores as float time
                expires_at = data.get("expires_at")
                if expires_at and isinstance(expires_at, (int | float)):
                    if expires_at < time.time():
                        report["expired_overrides"].append(
                            {
                                "id": data.get("policy_id", f.stem),
                                "by": data.get("by", "unknown"),
                                "expiry": expires_at,
                            }
                        )
                        report["drift_detected"] = True
                elif isinstance(expires_at, str):
                    # Backward compat or manual entry
                    expiry_dt = datetime.fromisoformat(expires_at)
                    if expiry_dt.timestamp() < time.time():
                        report["expired_overrides"].append(
                            {
                                "id": data.get("policy_id", f.stem),
                                "by": data.get("by", "unknown"),
                                "expiry": expires_at,
                            }
                        )
                        report["drift_detected"] = True
        except Exception as e:
            _log.error("Failed to check override %s: %s", f, e)
