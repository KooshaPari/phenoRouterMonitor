"""Shared helpers for pre-work governance hard gate enforcement."""

from __future__ import annotations

import time
from pathlib import Path
from typing import Any

from thegent.infra import yaml_load


def pre_work_gate_defaults() -> dict[str, Any]:
    """Return default regression spiral guard thresholds for pre-work hard gate."""
    return {
        "require_e2e_first": True,
        "max_test_evidence_age_minutes": 90,
        "max_build_evidence_age_minutes": 90,
        "max_e2e_evidence_age_minutes": 180,
    }


def pre_work_gate_thresholds(project_dir: Path) -> tuple[dict[str, Any], str]:
    """Load pre-work gate thresholds from hooks/hook-config.yaml with required defaults."""
    defaults = pre_work_gate_defaults()
    config_path = project_dir / "hooks" / "hook-config.yaml"
    if not config_path.exists():
        return defaults, "defaults"

    raw_config = yaml_load(config_path) or {}
    settings_block = raw_config.get("settings", {})
    guard_block = settings_block.get("regression_spiral_guard", {}) if isinstance(settings_block, dict) else {}
    if not isinstance(guard_block, dict):
        return defaults, str(config_path)

    thresholds = defaults.copy()

    require_e2e_raw = guard_block.get("require_e2e_first")
    if isinstance(require_e2e_raw, bool):
        thresholds["require_e2e_first"] = require_e2e_raw
    elif isinstance(require_e2e_raw, str):
        lowered = require_e2e_raw.strip().lower()
        if lowered in {"true", "1", "yes", "on"}:
            thresholds["require_e2e_first"] = True
        elif lowered in {"false", "0", "no", "off"}:
            thresholds["require_e2e_first"] = False

    for key in (
        "max_test_evidence_age_minutes",
        "max_build_evidence_age_minutes",
        "max_e2e_evidence_age_minutes",
    ):
        raw_value = guard_block.get(key)
        if raw_value is None:
            continue
        try:
            value = int(raw_value)
        except (TypeError, ValueError):
            continue
        thresholds[key] = max(0, value)

    return thresholds, str(config_path)


def evidence_age_minutes(path: Path) -> int:
    """Return age of evidence file in minutes based on mtime."""
    now_epoch = time.time()
    age_seconds = max(0.0, now_epoch - path.stat().st_mtime)
    return int(age_seconds // 60)


def pre_work_governance_block_payload(
    *,
    project_dir: Path,
    thresholds: dict[str, Any],
    violations: list[dict[str, Any]],
    config_source: str,
) -> dict[str, Any]:
    """Build structured governance block payload for pre-work hard gate failures."""
    remediation_steps = [
        "Refresh async test evidence: run your async test workflow so ~/.claude/.async-test-results.json is updated.",
        "Refresh build/env evidence: run verification so .claude/verification/qa-state.json is updated.",
    ]
    if bool(thresholds.get("require_e2e_first", True)):
        remediation_steps.append(
            "Refresh e2e evidence: run e2e/attestation workflow so .claude/verification/qa-attestation.json is updated."
        )
    remediation_steps.append("Re-run do-next or plan claim after evidence is fresh.")
    remediation = " ".join(remediation_steps)

    return {
        "governance_blocked": True,
        "governance_block": {
            "gate": "WP-HG-05.pre_work_hard_gate",
            "project_dir": str(project_dir),
            "config_source": config_source,
            "thresholds": thresholds,
            "violations": violations,
            "remediation_steps": remediation_steps,
        },
        "remediation": remediation,
        "error": "Pre-work hard gate blocked new work start: missing or stale verification evidence.",
    }


def enforce_pre_work_hard_gate(project_dir: Path) -> dict[str, Any] | None:
    """Enforce freshness evidence before do-next/claim starts new work."""
    thresholds, config_source = pre_work_gate_thresholds(project_dir)

    evidence_checks: list[tuple[str, Path, int]] = [
        (
            "test",
            Path("~/.claude/.async-test-results.json").expanduser(),
            int(thresholds["max_test_evidence_age_minutes"]),
        ),
        (
            "build",
            project_dir / ".claude" / "verification" / "qa-state.json",
            int(thresholds["max_build_evidence_age_minutes"]),
        ),
    ]
    if bool(thresholds["require_e2e_first"]):
        evidence_checks.append(
            (
                "e2e",
                project_dir / ".claude" / "verification" / "qa-attestation.json",
                int(thresholds["max_e2e_evidence_age_minutes"]),
            )
        )

    violations: list[dict[str, Any]] = []
    for evidence_type, evidence_path, max_age_minutes in evidence_checks:
        if not evidence_path.exists():
            violations.append(
                {
                    "evidence_type": evidence_type,
                    "status": "missing",
                    "path": str(evidence_path),
                    "max_age_minutes": max_age_minutes,
                }
            )
            continue

        age_minutes = evidence_age_minutes(evidence_path)
        if age_minutes > max_age_minutes:
            violations.append(
                {
                    "evidence_type": evidence_type,
                    "status": "stale",
                    "path": str(evidence_path),
                    "age_minutes": age_minutes,
                    "max_age_minutes": max_age_minutes,
                }
            )

    if not violations:
        return None

    return pre_work_governance_block_payload(
        project_dir=project_dir,
        thresholds=thresholds,
        violations=violations,
        config_source=config_source,
    )
