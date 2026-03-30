"""Tests for pheno_core.observability module."""

from typing import Any

import pytest

from pheno_core.observability import (
    Alerter,
    HealthChecker,
    HealthStatus,
    Logger,
    Meter,
    Tracer,
)


class TestLogger:
    """Test Logger port/interface."""

    def test_logger_is_abstract(self) -> None:
        """Test that Logger is an abstract base class."""
        assert hasattr(Logger, "__abstractmethods__")

    def test_logger_required_methods(self) -> None:
        """Test Logger has required methods."""
        methods = ["debug", "info", "warning", "error", "critical"]
        for method in methods:
            assert hasattr(Logger, method)

    def test_logger_implementation(self) -> None:
        """Test implementing Logger interface."""
        class TestLogger(Logger):
            def debug(self, message: str, **kwargs: Any) -> None:
                pass

            def info(self, message: str, **kwargs: Any) -> None:
                pass

            def warning(self, message: str, **kwargs: Any) -> None:
                pass

            def error(self, message: str, **kwargs: Any) -> None:
                pass

            def critical(self, message: str, **kwargs: Any) -> None:
                pass

        logger = TestLogger()
        assert isinstance(logger, Logger)


class TestTracer:
    """Test Tracer port/interface."""

    def test_tracer_is_abstract(self) -> None:
        """Test that Tracer is an abstract base class."""
        assert hasattr(Tracer, "__abstractmethods__")

    def test_tracer_required_methods(self) -> None:
        """Test Tracer has required methods."""
        methods = ["start_span", "end_span", "add_event"]
        for method in methods:
            assert hasattr(Tracer, method)

    def test_tracer_implementation(self) -> None:
        """Test implementing Tracer interface."""
        class TestTracer(Tracer):
            def start_span(self, name: str, **kwargs: Any) -> str:
                return "span_id"

            def end_span(self, span_id: str, **kwargs: Any) -> None:
                pass

            def add_event(self, span_id: str, event: str, **kwargs: Any) -> None:
                pass

        tracer = TestTracer()
        assert isinstance(tracer, Tracer)


class TestMeter:
    """Test Meter port/interface."""

    def test_meter_is_abstract(self) -> None:
        """Test that Meter is an abstract base class."""
        assert hasattr(Meter, "__abstractmethods__")

    def test_meter_required_methods(self) -> None:
        """Test Meter has required methods."""
        methods = ["record_counter", "record_histogram", "record_gauge"]
        for method in methods:
            assert hasattr(Meter, method)

    def test_meter_implementation(self) -> None:
        """Test implementing Meter interface."""
        class TestMeter(Meter):
            def record_counter(self, name: str, value: float = 1.0, **kwargs: Any) -> None:
                pass

            def record_histogram(self, name: str, value: float, **kwargs: Any) -> None:
                pass

            def record_gauge(self, name: str, value: float, **kwargs: Any) -> None:
                pass

        meter = TestMeter()
        assert isinstance(meter, Meter)


class TestHealthStatus:
    """Test HealthStatus enum."""

    def test_health_status_values(self) -> None:
        """Test HealthStatus has expected values."""
        assert hasattr(HealthStatus, "HEALTHY")
        assert hasattr(HealthStatus, "DEGRADED")
        assert hasattr(HealthStatus, "UNHEALTHY")

    def test_health_status_enum_values(self) -> None:
        """Test HealthStatus enum values are strings."""
        assert HealthStatus.HEALTHY.value in ["healthy", "HEALTHY", "Healthy"]
        assert HealthStatus.DEGRADED.value in ["degraded", "DEGRADED", "Degraded"]
        assert HealthStatus.UNHEALTHY.value in ["unhealthy", "UNHEALTHY", "Unhealthy"]


class TestHealthChecker:
    """Test HealthChecker port/interface."""

    def test_health_checker_is_abstract(self) -> None:
        """Test that HealthChecker is an abstract base class."""
        assert hasattr(HealthChecker, "__abstractmethods__")

    def test_health_checker_required_methods(self) -> None:
        """Test HealthChecker has required methods."""
        methods = ["check_health"]
        for method in methods:
            assert hasattr(HealthChecker, method)

    def test_health_checker_implementation(self) -> None:
        """Test implementing HealthChecker interface."""
        class TestHealthChecker(HealthChecker):
            async def check_health(self) -> tuple[HealthStatus, str]:
                return HealthStatus.HEALTHY, "OK"

        checker = TestHealthChecker()
        assert isinstance(checker, HealthChecker)

    @pytest.mark.asyncio
    async def test_health_checker_return_type(self) -> None:
        """Test HealthChecker returns correct type."""
        class TestHealthChecker(HealthChecker):
            async def check_health(self) -> tuple[HealthStatus, str]:
                return HealthStatus.HEALTHY, "All systems operational"

        checker = TestHealthChecker()
        status, message = await checker.check_health()

        assert status == HealthStatus.HEALTHY
        assert isinstance(message, str)


class TestAlerter:
    """Test Alerter port/interface."""

    def test_alerter_is_abstract(self) -> None:
        """Test that Alerter is an abstract base class."""
        assert hasattr(Alerter, "__abstractmethods__")

    def test_alerter_required_methods(self) -> None:
        """Test Alerter has required methods."""
        methods = ["send_alert"]
        for method in methods:
            assert hasattr(Alerter, method)

    def test_alerter_implementation(self) -> None:
        """Test implementing Alerter interface."""
        class TestAlerter(Alerter):
            async def send_alert(self, title: str, message: str, **kwargs: Any) -> None:
                pass

        alerter = TestAlerter()
        assert isinstance(alerter, Alerter)

    @pytest.mark.asyncio
    async def test_alerter_send_alert(self) -> None:
        """Test sending an alert."""
        class TestAlerter(Alerter):
            def __init__(self) -> None:
                self.alerts: list[dict[str, Any]] = []

            async def send_alert(self, title: str, message: str, **kwargs: Any) -> None:
                self.alerts.append({
                    "title": title,
                    "message": message,
                    "extra": kwargs
                })

        alerter = TestAlerter()
        await alerter.send_alert("Test Alert", "This is a test")

        assert len(alerter.alerts) == 1
        assert alerter.alerts[0]["title"] == "Test Alert"
        assert alerter.alerts[0]["message"] == "This is a test"


class TestObservabilityPorts:
    """Test observability ports together."""

    def test_all_ports_are_abstract_bases(self) -> None:
        """Test all observability ports are abstract."""
        from abc import ABC as ABC_BASE
        ports = [Logger, Tracer, Meter, HealthChecker, Alerter]
        for port in ports:
            assert issubclass(port, ABC_BASE)

    def test_ports_can_be_composed(self) -> None:
        """Test that observability ports can be composed."""
        class ComposedObservability:
            def __init__(
                self,
                logger: Logger,
                tracer: Tracer,
                meter: Meter,
                health_checker: HealthChecker,
                alerter: Alerter,
            ) -> None:
                self.logger = logger
                self.tracer = tracer
                self.meter = meter
                self.health_checker = health_checker
                self.alerter = alerter

        # Can instantiate with dummy implementations
        class DummyLogger(Logger):
            def debug(self, message: str, **kwargs: Any) -> None:
                pass

            def info(self, message: str, **kwargs: Any) -> None:
                pass

            def warning(self, message: str, **kwargs: Any) -> None:
                pass

            def error(self, message: str, **kwargs: Any) -> None:
                pass

            def critical(self, message: str, **kwargs: Any) -> None:
                pass

        class DummyTracer(Tracer):
            def start_span(self, name: str, **kwargs: Any) -> str:
                return "span"

            def end_span(self, span_id: str, **kwargs: Any) -> None:
                pass

            def add_event(self, span_id: str, event: str, **kwargs: Any) -> None:
                pass

        class DummyMeter(Meter):
            def record_counter(self, name: str, value: float = 1.0, **kwargs: Any) -> None:
                pass

            def record_histogram(self, name: str, value: float, **kwargs: Any) -> None:
                pass

            def record_gauge(self, name: str, value: float, **kwargs: Any) -> None:
                pass

        class DummyHealthChecker(HealthChecker):
            async def check_health(self) -> tuple[HealthStatus, str]:
                return HealthStatus.HEALTHY, "OK"

        class DummyAlerter(Alerter):
            async def send_alert(self, title: str, message: str, **kwargs: Any) -> None:
                pass

        obs = ComposedObservability(
            logger=DummyLogger(),
            tracer=DummyTracer(),
            meter=DummyMeter(),
            health_checker=DummyHealthChecker(),
            alerter=DummyAlerter(),
        )

        assert obs.logger is not None
        assert obs.tracer is not None
        assert obs.meter is not None
        assert obs.health_checker is not None
        assert obs.alerter is not None
