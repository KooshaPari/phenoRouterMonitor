"""Constitutional AI and alignment enforcement for thegent (WP-3001)."""

import logging
from pathlib import Path
from typing import Any

from pydantic import BaseModel

from thegent.infra import yaml_load

_log = logging.getLogger(__name__)


class ConstitutionalViolation(BaseModel):
    principle_id: str
    reason: str
    remediation: str


class ProofOfAlignment(BaseModel):
    """Verifiable proof that an action aligns with the constitution."""

    verified_principles: list[str]
    critique_hash: str
    aligned: bool


class ConstitutionManager:
    """WP-3001: Manages project principles and critique logic."""

    def __init__(self, constitution_path: Path) -> None:
        self.path = constitution_path
        self._load()

    def _load(self) -> None:
        if not self.path.exists():
            self.principles = []
            return
        data = yaml_load(self.path)
        self.principles = data.get("principles", []) if isinstance(data, dict) else []

    def critique_action(self, action: dict[str, Any]) -> list[ConstitutionalViolation]:
        """WP-3001: Pre-execution critique of a proposed agent action."""
        violations = []

        # Simple rule-based critique for this phase
        prompt = str(action.get("prompt", "")).lower()

        for p in self.principles:
            if p["id"] == "P1-SAFETY":
                if any(kw in prompt for kw in ["rm -rf", "force push", "delete all"]):
                    violations.append(
                        ConstitutionalViolation(
                            principle_id=p["id"],
                            reason="Destructive command detected in prompt.",
                            remediation="Use simulation mode or get explicit human sign-off.",
                        )
                    )

            if p["id"] == "P2-PRIVACY":
                if any(kw in prompt for kw in ["password", "api_key", "secret_key"]):
                    violations.append(
                        ConstitutionalViolation(
                            principle_id=p["id"],
                            reason="Potential secret leakage detected.",
                            remediation="Mask sensitive values before submitting.",
                        )
                    )

        return violations

    def generate_poa(self, action_id: str, aligned: bool) -> ProofOfAlignment:
        """Generate a Proof of Alignment for a MAIF artifact."""
        import hashlib

        verified = [p["id"] for p in self.principles]
        critique_data = f"{action_id}|{aligned}|{','.join(verified)}"
        c_hash = hashlib.sha256(critique_data.encode()).hexdigest()

        return ProofOfAlignment(verified_principles=verified, critique_hash=c_hash, aligned=aligned)
