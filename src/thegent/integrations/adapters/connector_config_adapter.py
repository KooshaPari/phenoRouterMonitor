"""Connector configuration adapter for workstream autosync.

Handles connector-specific configuration (timeouts, circuit breakers, etc).
"""

from typing import Any

from thegent.integrations.error_budget import ErrorBudgetConfig, ErrorBudgetTracker
from thegent.integrations.rate_limit_backoff import RateLimitBackoffManager, RateLimitConfig
from thegent.utils.routing_impl.circuit_breaker import (
    ProviderCircuitBreakerConfig,
    ProviderCircuitBreakerRegistry,
)


class ConnectorConfigAdapter:
    """Adapter for connector-specific configuration."""

    def __init__(self, config: Any):
        self.config = config
        self._breaker_registry = ProviderCircuitBreakerRegistry.get_instance()
        self._error_budgets: dict[str, ErrorBudgetTracker] = {}

    def get_connector_breaker(self, connector: str) -> Any:
        """Get circuit breaker for connector."""
        cb_config = ProviderCircuitBreakerConfig(
            failure_threshold=max(1, self.config.connector_circuit_breaker_failure_threshold),
            success_threshold=max(1, self.config.connector_circuit_breaker_success_threshold),
            timeout_sec=max(0.1, self.config.connector_circuit_breaker_timeout_seconds),
        )
        return self._breaker_registry.get(connector, config=cb_config)

    def get_connector_timeout(self, connector: str, direction: str) -> float:
        """Get timeout for connector operation."""
        if connector == "github" and direction == "write":
            return max(0.001, self.config.github_write_timeout_seconds)
        if connector == "github" and direction == "read":
            return max(0.001, self.config.github_read_timeout_seconds)
        if connector == "linear" and direction == "write":
            return max(0.001, self.config.linear_write_timeout_seconds)
        if connector == "linear" and direction == "read":
            return max(0.001, self.config.linear_read_timeout_seconds)
        raise ValueError(f"Unsupported connector timeout target: {connector}/{direction}")

    def get_error_budget(self, connector: str) -> ErrorBudgetTracker:
        """Get error budget tracker for connector."""
        normalized = connector.lower()
        if normalized not in self._error_budgets:
            self._error_budgets[normalized] = ErrorBudgetTracker(
                ErrorBudgetConfig(
                    max_consecutive_failures=self.config.error_budget_max_consecutive_failures,
                    max_failure_rate=self.config.error_budget_max_failure_rate,
                    escalation_after=self.config.error_budget_escalation_after,
                ),
            )
        return self._error_budgets[normalized]

    def create_rate_limiter(self) -> RateLimitBackoffManager:
        """Create rate limiter with config."""
        return RateLimitBackoffManager(
            RateLimitConfig(
                max_retries=max(0, int(self.config.rate_limit_max_retries)),
                initial_wait=max(0.01, float(self.config.rate_limit_initial_wait)),
                max_wait=max(0.01, float(self.config.rate_limit_max_wait)),
                multiplier=max(1.0, float(self.config.rate_limit_multiplier)),
            )
        )


__all__ = ["ConnectorConfigAdapter"]
