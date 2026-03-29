"""Automatic PII and secret redaction for support mode and compliance (WP-15003)."""

import re
from typing import ClassVar


class PIIRedactor:
    """Redacts PII and secrets from text outputs."""

    # Patterns for common PII and secrets
    # WP-15003: Expand patterns for global compliance
    PATTERNS: ClassVar[dict[str, str]] = {
        "email": r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b",
        "phone": r"\b\d{3}-\d{3}-\d{4}\b",
        "ssn": r"\b\d{3}-\d{2}-\d{4}\b",
        "api_key_sk": r"sk-[a-zA-Z0-9_-]{20,}",
        "api_key_gen": r"key-[a-zA-Z0-9_-]{20,}",
        "token": r"token-[a-zA-Z0-9_-]{20,}",
        "auth_header": r"Authorization:\s*(Bearer|Basic)\s+[a-zA-Z0-9._~+/-]+=*",
    }

    def __init__(self, custom_patterns: dict[str, str] | None = None) -> None:
        self.patterns = self.PATTERNS.copy()
        if custom_patterns:
            self.patterns.update(custom_patterns)

        self._compiled_re = {name: re.compile(pattern, re.IGNORECASE) for name, pattern in self.patterns.items()}

    def redact(self, text: str, mode: str = "support") -> str:
        """Redact PII and secrets from text.

        Args:
            text: Input text to redact
            mode: Redaction mode ("support", "audit", "standard")

        Returns:
            Redacted text
        """
        if not text:
            return ""

        redacted = text
        for name, rex in self._compiled_re.items():
            # Standard redaction label
            replacement = f"[REDACTED_{name.upper()}]" if mode == "audit" else "[REDACTED]"
            redacted = rex.sub(replacement, redacted)

        return redacted

    def contains_pii(self, text: str) -> bool:
        """Check if text contains any PII or secrets."""
        return any(rex.search(text) for rex in self._compiled_re.values())
