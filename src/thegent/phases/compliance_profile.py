"""Compliance profile mapping (EU-AI-ACT, US-SEC, SOX, GDPR)."""

from typing import Any, ClassVar


class ComplianceProfile:
    """Compliance profile mapping."""

    PROFILES: ClassVar[dict[str, Any]] = {
        "eu-ai-act": {
            "name": "EU AI Act",
            "requirements": ["risk_assessment", "transparency", "human_oversight"],
        },
        "us-sec": {
            "name": "US SEC",
            "requirements": ["financial_reporting", "audit_trail", "data_retention"],
        },
        "sox": {
            "name": "Sarbanes-Oxley",
            "requirements": ["internal_controls", "audit_trail", "data_retention"],
        },
        "gdpr": {
            "name": "GDPR",
            "requirements": ["data_protection", "right_to_erasure", "consent_management"],
        },
    }

    def __init__(self, profile_name: str) -> None:
        """Initialize compliance profile.

        Args:
            profile_name: Name of compliance profile
        """
        self.profile_name = profile_name
        self.profile = self.PROFILES.get(profile_name.lower(), {})

    def get_requirements(self) -> list[str]:
        """Get requirements for this profile.

        Returns:
            List of requirement names
        """
        return self.profile.get("requirements", [])

    def check_compliance(self, feature: str) -> bool:
        """Check if a feature is compliant.

        Args:
            feature: Feature name

        Returns:
            True if compliant
        """
        return feature in self.get_requirements()


def validate_profile_drift(
    profiles: dict[str, dict[str, str]],
    required_keys: set[str],
    allowlist: set[str] | None = None,
) -> tuple[bool, dict[str, list[str]]]:
    """Validate profile key presence and cross-environment drift."""
    allowlist = allowlist or set()
    drift: dict[str, list[str]] = {}
    envs = sorted(profiles.keys())

    for env in envs:
        missing = sorted(k for k in required_keys if k not in profiles[env])
        if missing:
            drift[f"{env}:missing"] = missing

    for key in sorted(required_keys):
        if key in allowlist:
            continue
        values = {profiles[env].get(key, "") for env in envs}
        if len(values) > 1:
            drift[f"drift:{key}"] = sorted(values)

    return len(drift) == 0, drift
