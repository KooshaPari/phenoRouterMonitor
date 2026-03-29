"""Alerting integration for LiteLLM routing.

Provides alert management for routing events including budget exceeded,
high latency, and provider errors. Supports webhook notifications.
"""

from __future__ import annotations

import logging
from dataclasses import dataclass, field
from typing import Any, ClassVar

import httpx

logger = logging.getLogger(__name__)


@dataclass
class Alert:
    """Routing alert."""

    alert_type: str  # "budget_exceeded", "high_latency", "provider_error", "cooldown_triggered"
    severity: str  # "warning", "critical", "info"
    message: str
    data: dict[str, Any] = field(default_factory=dict)
    timestamp: str | None = None

    def to_json(self) -> dict[str, Any]:
        """Serialize alert to JSON dict."""
        return {
            "alert_type": self.alert_type,
            "severity": self.severity,
            "message": self.message,
            "data": self.data,
            "timestamp": self.timestamp,
        }


class AlertManager:
    """Manage routing alerts with webhook support."""

    # Severity levels for comparison
    SEVERITY_LEVELS: ClassVar[dict[str, int]] = {"info": 0, "warning": 1, "critical": 2}

    def __init__(
        self,
        webhook_url: str | None = None,
        min_severity: str = "warning",
    ) -> None:
        """Initialize alert manager.

        Args:
            webhook_url: Optional webhook URL for alert delivery.
            min_severity: Minimum severity level to send (info, warning, critical).
        """
        self._webhook_url = webhook_url
        self._min_severity = min_severity
        self._pending_alerts: list[Alert] = []

    @property
    def webhook_url(self) -> str | None:
        """Configured webhook URL."""
        return self._webhook_url

    def _should_send(self, severity: str) -> bool:
        """Check if alert severity meets minimum threshold."""
        min_level = self.SEVERITY_LEVELS.get(self._min_severity, 1)
        alert_level = self.SEVERITY_LEVELS.get(severity, 0)
        return alert_level >= min_level

    def send_alert(self, alert: Alert) -> bool:
        """Send alert to configured webhook.

        Args:
            alert: The alert to send.

        Returns:
            True if sent successfully, False otherwise.
        """
        if not self._webhook_url:
            logger.warning("No webhook configured, would send alert: %s", alert.message)
            self._pending_alerts.append(alert)
            return False

        if not self._should_send(alert.severity):
            logger.debug("Alert below min severity: %s", alert.message)
            return False

        try:
            resp = httpx.post(
                self._webhook_url,
                json=alert.to_json(),
                timeout=5,
            )
            return resp.status_code == 200
        except httpx.HTTPStatusError as e:
            logger.error("Alert webhook HTTP error: %s %s", e.response.status_code, e.response.reason_phrase)
            return False
        except httpx.RequestError as e:
            logger.error("Alert webhook HTTP error: %s", e)
            return False
        except Exception as e:
            logger.error("Failed to send alert: %s", e)
            return False

    def alert_budget_exceeded(self, daily_spend: float, budget: float) -> Alert:
        """Create and send budget exceeded alert.

        Args:
            daily_spend: Current daily spend in USD.
            budget: Configured budget limit in USD.

        Returns:
            The created Alert.
        """
        alert = Alert(
            alert_type="budget_exceeded",
            severity="critical",
            message=f"Daily budget exceeded: ${daily_spend:.2f} / ${budget:.2f}",
            data={"daily_spend": daily_spend, "budget": budget},
        )
        self.send_alert(alert)
        return alert

    def alert_high_latency(
        self,
        model: str,
        latency_ms: float,
        threshold_ms: float,
        provider: str | None = None,
    ) -> Alert:
        """Create and send high latency alert.

        Args:
            model: The model that had high latency.
            latency_ms: Observed latency in milliseconds.
            threshold_ms: Configured threshold in milliseconds.
            provider: Optional provider name.

        Returns:
            The created Alert.
        """
        alert = Alert(
            alert_type="high_latency",
            severity="warning",
            message=f"High latency on {model}: {latency_ms:.0f}ms (threshold: {threshold_ms:.0f}ms)",
            data={
                "model": model,
                "latency_ms": latency_ms,
                "threshold_ms": threshold_ms,
                "provider": provider,
            },
        )
        self.send_alert(alert)
        return alert

    def alert_provider_error(
        self,
        provider: str,
        error: str,
        model: str,
        is_rate_limit: bool = False,
    ) -> Alert:
        """Create and send provider error alert.

        Args:
            provider: The provider that had an error.
            error: Error message or type.
            model: The model being used.
            is_rate_limit: Whether this was a rate limit error.

        Returns:
            The created Alert.
        """
        severity = "warning" if is_rate_limit else "critical"
        alert = Alert(
            alert_type="provider_error",
            severity=severity,
            message=f"Provider {provider} error on {model}: {error}",
            data={
                "provider": provider,
                "error": error,
                "model": model,
                "is_rate_limit": is_rate_limit,
            },
        )
        self.send_alert(alert)
        return alert

    def alert_cooldown_triggered(
        self,
        model: str,
        provider: str,
        cooldown_seconds: float,
        reason: str,
    ) -> Alert:
        """Create and send cooldown triggered alert.

        Args:
            model: The model in cooldown.
            provider: The provider.
            cooldown_seconds: Duration of cooldown.
            reason: Why cooldown was triggered.

        Returns:
            The created Alert.
        """
        alert = Alert(
            alert_type="cooldown_triggered",
            severity="info",
            message=f"Cooldown triggered for {model} on {provider}: {reason}",
            data={
                "model": model,
                "provider": provider,
                "cooldown_seconds": cooldown_seconds,
                "reason": reason,
            },
        )
        self.send_alert(alert)
        return alert

    def get_pending_alerts(self) -> list[Alert]:
        """Get list of alerts that weren't sent (no webhook configured)."""
        return list(self._pending_alerts)

    def clear_pending_alerts(self) -> None:
        """Clear pending alerts list."""
        self._pending_alerts.clear()


# Global alert manager instance
_alert_manager: AlertManager | None = None


def get_alert_manager() -> AlertManager:
    """Get global alert manager instance.

    Initializes with settings from config on first call.
    """
    global _alert_manager
    if _alert_manager is None:
        try:
            from thegent.config import ThegentSettings

            settings = ThegentSettings()
            _alert_manager = AlertManager(webhook_url=settings.litellm_alert_webhook)
        except Exception:
            # Fallback without config
            _alert_manager = AlertManager()
    return _alert_manager


def reset_alert_manager() -> None:
    """Reset the global alert manager (useful for testing)."""
    global _alert_manager
    _alert_manager = None
