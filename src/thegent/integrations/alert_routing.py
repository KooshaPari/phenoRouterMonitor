"""Alert Routing Hooks for pluggable alert handling.

WL-318: Alert Routing Hooks
Provides pluggable alert routing hooks for webhook, email, and event bus integration.
"""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
from enum import Enum
from typing import Callable


class AlertSeverity(str, Enum):
    """Alert severity classification."""

    INFO = "info"
    WARN = "warn"
    CRITICAL = "critical"


@dataclass
class Alert:
    """An alert event with severity, message, and context."""

    alert_id: str
    severity: AlertSeverity
    message: str
    context: dict
    timestamp: datetime


class AlertRouter:
    """Router for pluggable alert handling hooks."""

    def __init__(self) -> None:
        """Initialize the alert router."""
        self._hooks: dict[str, Callable[[Alert], None]] = {}

    def register_hook(self, name: str, fn: Callable[[Alert], None]) -> None:
        """Register an alert routing hook.

        Args:
            name: Unique name for the hook.
            fn: Callable that accepts an Alert and returns None.
        """
        self._hooks[name] = fn

    def unregister_hook(self, name: str) -> None:
        """Unregister an alert routing hook.

        Args:
            name: Name of hook to remove.

        Raises:
            KeyError: If hook with given name does not exist.
        """
        if name not in self._hooks:
            raise KeyError(f"Hook not found: {name}")
        del self._hooks[name]

    def route(self, alert: Alert) -> int:
        """Route an alert to all registered hooks.

        Args:
            alert: Alert to route.

        Returns:
            Number of hooks called.
        """
        for hook_fn in self._hooks.values():
            hook_fn(alert)
        return len(self._hooks)

    def list_hooks(self) -> list[str]:
        """List all registered hook names.

        Returns:
            Sorted list of hook names.
        """
        return sorted(self._hooks.keys())
