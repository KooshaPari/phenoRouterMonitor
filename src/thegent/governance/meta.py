"""WP-20004: Meta-Governance & Constitutional AI.
Provides high-level, human-aligned rules (constitution) for all agent operations.
Inspired by Constitutional AI principles (Anthropic).
"""

import json
import logging
from dataclasses import dataclass, field
from enum import StrEnum
from pathlib import Path

_log = logging.getLogger(__name__)


class ConstitutionalPrinciple(StrEnum):
    SAFETY = "safety"
    HELPFULNESS = "helpfulness"
    HONESTY = "honesty"
    DATA_MINIMIZATION = "data_minimization"
    NO_HARMFUL_CONTENT = "no_harmful_content"
    PRIVACY = "privacy"
    TRANSPARENCY = "transparency"


@dataclass
class Rule:
    """A high-level governance rule aligned with a constitutional principle."""

    rule_id: str
    principle: ConstitutionalPrinciple
    description: str
    is_active: bool = True
    metadata: dict[str, str] = field(default_factory=dict)


class MetaGovernance:
    """Manages the agent constitution and high-level governance rules."""

    def __init__(self, constitution_path: Path | None = None) -> None:
        self.path = constitution_path or Path.home() / ".thegent" / "constitution.json"
        self.rules: list[Rule] = []
        self._load_constitution()

    def _load_constitution(self):
        """Load the agent constitution from disk."""
        if self.path.exists():
            try:
                data = json.loads(self.path.read_text(encoding="utf-8"))
                self.rules = [
                    Rule(
                        rule_id=r["rule_id"],
                        principle=ConstitutionalPrinciple(r["principle"]),
                        description=r["description"],
                        is_active=r.get("is_active", True),
                        metadata=r.get("metadata", {}),
                    )
                    for r in data
                ]
            except Exception as e:
                _log.error("Failed to load constitution: %s", e)
        else:
            # Default constitutional rules
            self.rules = [
                Rule("G-META-01", ConstitutionalPrinciple.SAFETY, "Never delete core system configuration files."),
                Rule(
                    "G-META-02",
                    ConstitutionalPrinciple.PRIVACY,
                    "Never upload user credentials or secrets to public APIs.",
                ),
                Rule(
                    "G-META-03",
                    ConstitutionalPrinciple.HELPFULNESS,
                    "Always prioritize user intent and clear communication.",
                ),
                Rule(
                    "G-META-04",
                    ConstitutionalPrinciple.TRANSPARENCY,
                    "Always provide a clear reasoning for destructive actions.",
                ),
            ]
            self.save_constitution()

    def save_constitution(self):
        """Save the constitution to disk."""
        self.path.parent.mkdir(parents=True, exist_ok=True)
        # Convert dataclass objects to dicts manually to handle Enum serialization
        data = []
        for rule in self.rules:
            r_dict = {
                "rule_id": rule.rule_id,
                "principle": rule.principle.value,
                "description": rule.description,
                "is_active": rule.is_active,
                "metadata": rule.metadata,
            }
            data.append(r_dict)
        self.path.write_text(json.dumps(data, indent=2), encoding="utf-8")

    def validate_action(self, action_description: str, tags: set[str]) -> tuple[bool, str | None]:
        """Validate an agent's intended action against the constitution."""
        _log.info("Validating action against meta-governance: %s", action_description[:50])

        # Simple keyword matching for validation
        for rule in self.rules:
            if not rule.is_active:
                continue

            if rule.principle == ConstitutionalPrinciple.SAFETY and (
                "delete" in action_description.lower() or "config" in action_description.lower()
            ):
                if "system" in action_description.lower():
                    return False, f"Action violates {rule.rule_id} ({rule.principle.value}): {rule.description}"

            if rule.principle == ConstitutionalPrinciple.PRIVACY and ("secret" in tags or "credential" in tags):
                return False, f"Action violates {rule.rule_id} ({rule.principle.value}): {rule.description}"

        return True, None

    def get_constitution_summary(self) -> str:
        """Return a formatted summary of the agent constitution."""
        lines = ["# 📜 TheGent Agent Constitution", ""]
        for rule in self.rules:
            status = "✅ Active" if rule.is_active else "❌ Inactive"
            lines.append(f"### {rule.rule_id} - {rule.principle.value.upper()}")
            lines.append(f"**Status**: {status}")
            lines.append(f"**Description**: {rule.description}")
            lines.append("")
        return "\n".join(lines)
