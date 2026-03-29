"""WP-15005: End-user support mode with automatic PII and secret redaction."""

import re
from typing import Any


class SupportRedactor:
    """Automatic redaction of PII and secrets for support mode (WP-15005)."""

    def __init__(self) -> None:
        # Common secret/PII patterns
        self._patterns = {
            "api_key": re.compile(r"(?i)(sk-|key-|token-|bearer\s+)[a-z0-9]{20,}"),
            "email": re.compile(r"[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+"),
            "ip_address": re.compile(r"\b(?:\d{1,3}\.){3}\d{1,3}\b"),
            "credit_card": re.compile(r"\b(?:\d{4}-){3}\d{4}\b"),
        }

    def redact_text(self, text: str) -> str:
        """Apply all redaction patterns to the provided text."""
        redacted = text
        for label, pattern in self._patterns.items():
            redacted = pattern.sub(f"[REDACTED_{label.upper()}]", redacted)
        return redacted

    def redact_payload(self, payload: dict[str, Any]) -> dict[str, Any]:
        """Recursively redact strings within a nested dictionary payload."""
        import json

        dumped = json.dumps(payload)
        redacted_json = self.redact_text(dumped)
        return json.loads(redacted_json)


class SupportModeSession:
    """Least-privilege session for platform support engineers."""

    def __init__(self, engineer_id: str) -> None:
        self.engineer_id = engineer_id
        self.redactor = SupportRedactor()
        self.active = True

    def get_view(self, raw_output: str) -> str:
        """Return a redacted view of the system output."""
        if not self.active:
            return raw_output
        return self.redactor.redact_text(raw_output)
