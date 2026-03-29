"""Confidential Report Mode for minimized metadata exposure.

WL-313: Confidential Report Mode
Provides report sensitivity levels and field redaction for confidential data.
"""

from __future__ import annotations

import re
from datetime import datetime, UTC
from enum import Enum
from typing import Any, ClassVar, cast


class ReportSensitivity(str, Enum):
    """Report sensitivity classification."""

    PUBLIC = "public"
    CONFIDENTIAL = "confidential"


class ConfidentialReportFilter:
    """Filter and redact sensitive fields in reports."""

    REDACT_FIELDS: ClassVar[set[str]] = {
        "token",
        "secret",
        "password",
        "api_key",
        "auth",
        "credential",
    }
    REDACT_VALUE_PATTERNS: ClassVar[tuple[re.Pattern[str], ...]] = (
        re.compile(r"(?i)\b(gh[pousr]_[a-z0-9]{8,}|sk-[a-z0-9]{8,}|xox[baprs]-[a-z0-9-]{8,})\b"),
        re.compile(r"(?i)\b(bearer\s+[a-z0-9._-]{8,})\b"),
    )
    REDACTION_POLICY: ClassVar[dict[str, str]] = {
        "field_substring_policy": "match-any(REDACT_FIELDS)",
        "value_regex_policy": "match-any(REDACT_VALUE_PATTERNS)",
        "replacement": "[REDACTED]",
    }

    @classmethod
    def redact(cls, data: dict[str, Any], sensitivity: ReportSensitivity) -> dict[str, Any]:
        """Redact sensitive fields from data if confidential.

        For CONFIDENTIAL reports, recursively replaces values for keys
        matching any REDACT_FIELDS substring (case-insensitive) with "[REDACTED]".
        For PUBLIC reports, returns data unchanged.

        Args:
            data: Dictionary to potentially redact.
            sensitivity: Report sensitivity level.

        Returns:
            Original data if PUBLIC, redacted copy if CONFIDENTIAL.
        """
        if sensitivity == ReportSensitivity.PUBLIC:
            return data

        return cast("dict[str, Any]", cls._redact_recursive(data))

    @classmethod
    def _redact_recursive(cls, obj: Any) -> Any:
        """Recursively redact sensitive fields.

        Args:
            obj: Object to redact (dict, list, or primitive).

        Returns:
            Redacted copy of the object.
        """
        if isinstance(obj, dict):
            result = {}
            for key, value in obj.items():
                if cls._is_redact_key(key):
                    result[key] = "[REDACTED]"
                else:
                    result[key] = cls._redact_recursive(value)
            return result
        if isinstance(obj, list):
            return [cls._redact_recursive(item) for item in obj]
        if isinstance(obj, str) and cls._is_redact_value(obj):
            return "[REDACTED]"
        return obj

    @classmethod
    def _is_redact_key(cls, key: str) -> bool:
        """Check if a key should be redacted.

        Checks if any REDACT_FIELDS substring appears in the key (case-insensitive).

        Args:
            key: Key to check.

        Returns:
            True if key matches any redact field pattern.
        """
        key_lower = key.lower()
        return any(field in key_lower for field in cls.REDACT_FIELDS)

    @classmethod
    def _is_redact_value(cls, value: str) -> bool:
        """Check if a string value matches sensitive token patterns."""
        return any(pattern.search(value) is not None for pattern in cls.REDACT_VALUE_PATTERNS)

    @classmethod
    def wrap_report(cls, report: dict[str, Any], sensitivity: ReportSensitivity, report_id: str) -> dict[str, Any]:
        """Wrap a report with metadata and redaction.

        Args:
            report: Report data dictionary.
            sensitivity: Report sensitivity level.
            report_id: Unique report identifier.

        Returns:
            Wrapped report with id, sensitivity, redacted data, and timestamp.
        """
        redacted_data = cls.redact(report, sensitivity)
        return {
            "report_id": report_id,
            "sensitivity": sensitivity.value,
            "data": redacted_data,
            "generated_at": datetime.now(UTC).isoformat(),
        }

    @classmethod
    def redact_artifact_payload(cls, payload: Any) -> Any:
        """Apply deterministic redaction to arbitrary artifact payloads."""
        return cls._redact_recursive(payload)
