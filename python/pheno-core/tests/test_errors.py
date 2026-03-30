"""Tests for pheno_core.errors module."""

import pytest

from pheno_core.errors import (
    CircuitBreakerOpenError,
    ConfigurationError,
    ServiceUnavailableError,
    ValidationError,
    ZenMCPError,
)


class TestZenMCPError:
    """Test ZenMCPError base exception."""

    def test_zen_mcp_error_creation(self) -> None:
        """Test creating a ZenMCPError."""
        error = ZenMCPError("Test error")
        assert str(error) == "Test error"
        assert isinstance(error, Exception)

    def test_zen_mcp_error_with_code(self) -> None:
        """Test ZenMCPError with error code."""
        error = ZenMCPError("Test error", code="ERR_001")
        assert error.code == "ERR_001"
        assert str(error) == "Test error"

    def test_zen_mcp_error_with_context(self) -> None:
        """Test ZenMCPError with context."""
        context = {"service": "test_service", "operation": "test_op"}
        error = ZenMCPError("Test error", context=context)
        assert error.context == context


class TestConfigurationError:
    """Test ConfigurationError exception."""

    def test_configuration_error_creation(self) -> None:
        """Test creating a ConfigurationError."""
        error = ConfigurationError("Invalid config")
        assert str(error) == "Invalid config"
        assert isinstance(error, ZenMCPError)

    def test_configuration_error_inherits_from_zen_mcp(self) -> None:
        """Test that ConfigurationError inherits from ZenMCPError."""
        error = ConfigurationError("Config missing", code="ERR_CONFIG_001")
        assert error.code == "ERR_CONFIG_001"


class TestValidationError:
    """Test ValidationError exception."""

    def test_validation_error_creation(self) -> None:
        """Test creating a ValidationError."""
        error = ValidationError("Invalid input")
        assert str(error) == "Invalid input"
        assert isinstance(error, ZenMCPError)

    def test_validation_error_with_field(self) -> None:
        """Test ValidationError with field information."""
        error = ValidationError("Invalid value", context={"field": "email"})
        assert error.context["field"] == "email"


class TestServiceUnavailableError:
    """Test ServiceUnavailableError exception."""

    def test_service_unavailable_error_creation(self) -> None:
        """Test creating a ServiceUnavailableError."""
        error = ServiceUnavailableError("Service down")
        assert str(error) == "Service down"
        assert isinstance(error, ZenMCPError)

    def test_service_unavailable_error_with_retry_info(self) -> None:
        """Test ServiceUnavailableError with retry information."""
        error = ServiceUnavailableError(
            "Service down",
            context={"retry_after": 30, "attempt": 1}
        )
        assert error.context["retry_after"] == 30


class TestCircuitBreakerOpenError:
    """Test CircuitBreakerOpenError exception."""

    def test_circuit_breaker_open_error_creation(self) -> None:
        """Test creating a CircuitBreakerOpenError."""
        error = CircuitBreakerOpenError("Circuit open")
        assert str(error) == "Circuit open"
        assert isinstance(error, ZenMCPError)

    def test_circuit_breaker_error_with_recovery_time(self) -> None:
        """Test CircuitBreakerOpenError with recovery time."""
        error = CircuitBreakerOpenError(
            "Circuit open",
            context={"recovery_time": 60}
        )
        assert error.context["recovery_time"] == 60


class TestErrorHierarchy:
    """Test error hierarchy and inheritance."""

    def test_all_errors_inherit_from_zen_mcp(self) -> None:
        """Test that all custom errors inherit from ZenMCPError."""
        errors = [
            ConfigurationError("test"),
            ValidationError("test"),
            ServiceUnavailableError("test"),
            CircuitBreakerOpenError("test"),
        ]
        for error in errors:
            assert isinstance(error, ZenMCPError)

    def test_error_catching_by_base_class(self) -> None:
        """Test catching errors by base ZenMCPError class."""
        with pytest.raises(ZenMCPError):
            raise ValidationError("test")

        with pytest.raises(ZenMCPError):
            raise ServiceUnavailableError("test")

    def test_error_code_assignment(self) -> None:
        """Test error code assignment across error types."""
        errors = [
            ("ERR_CONFIG", ConfigurationError("config", code="ERR_CONFIG")),
            ("ERR_VALIDATION", ValidationError("validation", code="ERR_VALIDATION")),
            ("ERR_SERVICE", ServiceUnavailableError("service", code="ERR_SERVICE")),
            ("ERR_CIRCUIT", CircuitBreakerOpenError("circuit", code="ERR_CIRCUIT")),
        ]
        for expected_code, error in errors:
            assert error.code == expected_code
