"""Observability ports and interfaces for Phenotype Core."""

from abc import ABC, abstractmethod
from enum import Enum
from typing import Any, Optional


class HealthStatus(str, Enum):
    """Health status enumeration."""

    HEALTHY = "healthy"
    DEGRADED = "degraded"
    UNHEALTHY = "unhealthy"


class Logger(ABC):
    """
    Abstract base class for logging port.

    Defines the interface that all logger implementations must follow.
    """

    @abstractmethod
    def debug(self, message: str, **kwargs: Any) -> None:
        """Log debug message."""
        pass

    @abstractmethod
    def info(self, message: str, **kwargs: Any) -> None:
        """Log info message."""
        pass

    @abstractmethod
    def warning(self, message: str, **kwargs: Any) -> None:
        """Log warning message."""
        pass

    @abstractmethod
    def error(self, message: str, **kwargs: Any) -> None:
        """Log error message."""
        pass

    @abstractmethod
    def critical(self, message: str, **kwargs: Any) -> None:
        """Log critical message."""
        pass


class Tracer(ABC):
    """
    Abstract base class for tracing port.

    Defines the interface for distributed tracing implementations.
    """

    @abstractmethod
    def start_span(self, name: str, **kwargs: Any) -> str:
        """
        Start a new span.

        Args:
            name: Span name.
            **kwargs: Additional span attributes.

        Returns:
            Span ID for later reference.
        """
        pass

    @abstractmethod
    def end_span(self, span_id: str, **kwargs: Any) -> None:
        """
        End a span.

        Args:
            span_id: ID of span to end.
            **kwargs: Additional end span attributes.
        """
        pass

    @abstractmethod
    def add_event(self, span_id: str, event: str, **kwargs: Any) -> None:
        """
        Add an event to a span.

        Args:
            span_id: ID of span to add event to.
            event: Event description.
            **kwargs: Additional event attributes.
        """
        pass


class Meter(ABC):
    """
    Abstract base class for metrics/measurements port.

    Defines the interface for metric collection implementations.
    """

    @abstractmethod
    def record_counter(self, name: str, value: float = 1.0, **kwargs: Any) -> None:
        """
        Record a counter metric (monotonically increasing).

        Args:
            name: Metric name.
            value: Value to add to counter (default 1.0).
            **kwargs: Additional metric attributes.
        """
        pass

    @abstractmethod
    def record_histogram(self, name: str, value: float, **kwargs: Any) -> None:
        """
        Record a histogram metric (distribution of values).

        Args:
            name: Metric name.
            value: Value to record.
            **kwargs: Additional metric attributes.
        """
        pass

    @abstractmethod
    def record_gauge(self, name: str, value: float, **kwargs: Any) -> None:
        """
        Record a gauge metric (instantaneous measurement).

        Args:
            name: Metric name.
            value: Current gauge value.
            **kwargs: Additional metric attributes.
        """
        pass


class HealthChecker(ABC):
    """
    Abstract base class for health checking port.

    Defines the interface for health check implementations.
    """

    @abstractmethod
    async def check_health(self) -> tuple[HealthStatus, str]:
        """
        Check service health.

        Returns:
            Tuple of (HealthStatus, detailed_message).
        """
        pass


class Alerter(ABC):
    """
    Abstract base class for alerting port.

    Defines the interface for alert/notification implementations.
    """

    @abstractmethod
    async def send_alert(self, title: str, message: str, **kwargs: Any) -> None:
        """
        Send an alert/notification.

        Args:
            title: Alert title.
            message: Alert message.
            **kwargs: Additional alert metadata (severity, tags, etc).
        """
        pass


class Registry(ABC):
    """
    Abstract base class for service registry port.

    Defines the interface for service discovery and registration.
    """

    @abstractmethod
    def register(self, service_name: str, metadata: dict[str, Any]) -> None:
        """
        Register a service.

        Args:
            service_name: Name of the service.
            metadata: Service metadata (address, port, tags, etc).
        """
        pass

    @abstractmethod
    def deregister(self, service_name: str) -> None:
        """
        Deregister a service.

        Args:
            service_name: Name of the service to deregister.
        """
        pass

    @abstractmethod
    def discover(self, service_name: str) -> list[dict[str, Any]]:
        """
        Discover instances of a service.

        Args:
            service_name: Name of the service to discover.

        Returns:
            List of service instances with metadata.
        """
        pass


class SearchableRegistry(Registry):
    """
    Registry with search capabilities.

    Extends Registry with search and filtering functionality.
    """

    @abstractmethod
    def search(self, query: str) -> list[str]:
        """
        Search for services by name or metadata.

        Args:
            query: Search query string.

        Returns:
            List of matching service names.
        """
        pass


class ObservableRegistry(Registry):
    """
    Registry with observability capabilities.

    Extends Registry with health and metrics observations.
    """

    @abstractmethod
    def get_metrics(self, service_name: str) -> dict[str, Any]:
        """
        Get metrics for a registered service.

        Args:
            service_name: Name of the service.

        Returns:
            Dictionary of service metrics.
        """
        pass

    @abstractmethod
    def watch(self, service_name: str, callback: Any) -> None:
        """
        Watch a service for changes.

        Args:
            service_name: Name of the service to watch.
            callback: Callable to invoke on service changes.
        """
        pass
