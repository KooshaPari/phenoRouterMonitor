"""Native governance scanner (obfuscated triggers, Rust built)."""

import logging
from typing import Any

logger = logging.getLogger(__name__)


class NativeGovernanceScanner:
    """Native governance scanner with obfuscated triggers."""

    def __init__(self) -> None:
        """Initialize native governance scanner."""
        self.triggers: list[str] = []
        self.obfuscated_patterns: list[str] = []

    def scan(self, content: str) -> dict[str, Any]:
        """Scan content for governance violations.

        Args:
            content: Content to scan

        Returns:
            Scan results
        """
        violations = []

        # Check obfuscated patterns
        for pattern in self.obfuscated_patterns:
            if pattern in content:
                violations.append(
                    {
                        "pattern": pattern,
                        "type": "obfuscated_trigger",
                    }
                )

        logger.info(f"Scanned content: {len(violations)} violations found")

        return {
            "violations": violations,
            "status": "complete",
        }

    def add_trigger(self, trigger: str, obfuscated: bool = False) -> None:
        """Add a trigger pattern.

        Args:
            trigger: Trigger pattern
            obfuscated: Whether pattern is obfuscated
        """
        if obfuscated:
            self.obfuscated_patterns.append(trigger)
        else:
            self.triggers.append(trigger)
