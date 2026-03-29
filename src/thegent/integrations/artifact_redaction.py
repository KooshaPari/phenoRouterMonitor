"""Artifact redaction pipeline for sensitive field masking.

# @trace WL-276
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any


@dataclass
class RedactionRule:
    """Represents a field redaction rule."""

    field_path: str
    replacement: str = "[REDACTED]"


class ArtifactRedactionPipeline:
    """Pipeline for redacting sensitive fields from artifacts."""

    def __init__(self) -> None:
        """Initialize the redaction pipeline."""
        self._rules: list[RedactionRule] = []

    def add_rule(self, field_path: str, replacement: str = "[REDACTED]") -> RedactionRule:
        """Add a redaction rule.

        Args:
            field_path: The top-level key to redact.
            replacement: The replacement value (default: "[REDACTED]").

        Returns:
            The created RedactionRule.
        """
        rule = RedactionRule(field_path=field_path, replacement=replacement)
        self._rules.append(rule)
        return rule

    def redact(self, data: dict[str, Any]) -> dict[str, Any]:
        """Redact sensitive fields from a dictionary.

        Args:
            data: The input data dictionary.

        Returns:
            A copy of the data with matching top-level keys replaced.
        """
        result = data.copy()
        for rule in self._rules:
            if rule.field_path in result:
                result[rule.field_path] = rule.replacement
        return result

    def rules(self) -> list[RedactionRule]:
        """Return all redaction rules.

        Returns:
            A list of all registered RedactionRule objects.
        """
        return list(self._rules)
