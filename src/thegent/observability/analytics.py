"""WP-Y7: Usage analytics integration."""

from __future__ import annotations

import logging

_log = logging.getLogger(__name__)


class AnalyticsIntegration:
    """Lightweight analytics facade for operational event tracking."""

    def __init__(self, provider: str, site_id: str) -> None:
        self.provider = provider
        self.site_id = site_id

    def track_page_view(self, path: str) -> None:
        """Track a logical page view or workflow path."""
        _log.debug(
            "Analytics page view provider=%s site_id=%s path=%s",
            self.provider,
            self.site_id,
            path,
        )

    def track_event(
        self,
        category: str,
        action: str,
        label: str | None = None,
        value: float | None = None,
    ) -> None:
        """Track a structured analytics event."""
        _log.debug(
            "Analytics event provider=%s site_id=%s category=%s action=%s label=%s value=%s",
            self.provider,
            self.site_id,
            category,
            action,
            label,
            value,
        )
