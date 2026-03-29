"""Retry classification for workstream autosync.

Extracts retry classification logic.
"""

from thegent.integrations.workstream_autosync_shared import RetryClass


class RetryClassifier:
    """Classifies errors for retry behavior."""

    @staticmethod
    def classify(message: str) -> RetryClass:
        """Classify an error message for retry behavior."""
        lowered = message.lower()

        # Rate limiting - retry with backoff
        if "429" in lowered or "rate limit" in lowered or "quota" in lowered:
            return RetryClass.RATE_LIMIT

        # Auth failures - may need re-auth
        if "401" in lowered or "403" in lowered or "auth" in lowered:
            return RetryClass.TRANSIENT

        # Server errors - retry
        if "500" in lowered or "502" in lowered or "503" in lowered or "504" in lowered:
            return RetryClass.TRANSIENT

        # Timeout - retry
        if "timeout" in lowered or "timed out" in lowered:
            return RetryClass.TRANSIENT

        # Network - retry
        if "network" in lowered or "connection" in lowered or "econnreset" in lowered:
            return RetryClass.TRANSIENT

        # Default - no retry
        return RetryClass.PERMANENT


__all__ = ["RetryClassifier"]
