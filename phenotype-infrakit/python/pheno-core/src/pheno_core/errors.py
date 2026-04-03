"""Error hierarchy for Phenotype Core."""

from typing import Any, Optional


class ZenMCPError(Exception):
    """
    Base exception for all Phenotype errors.

    Provides unified error handling with error codes and context support.
    """

    def __init__(
        self,
        message: str,
        code: Optional[str] = None,
        context: Optional[dict[str, Any]] = None,
    ) -> None:
        """
        Initialize ZenMCPError.

        Args:
            message: Error message.
            code: Optional error code for categorization.
            context: Optional context dictionary with additional error details.
        """
        super().__init__(message)
        self.message = message
        self.code = code
        self.context = context or {}

    def __str__(self) -> str:
        """Return string representation."""
        return self.message

    def __repr__(self) -> str:
        """Return detailed representation."""
        return f"{self.__class__.__name__}(message={self.message!r}, code={self.code!r})"


class ConfigurationError(ZenMCPError):
    """
    Raised when configuration is invalid or missing required values.

    Typically used during config loading, validation, or application startup.
    """

    pass


class ValidationError(ZenMCPError):
    """
    Raised when input or data validation fails.

    Typically used when user input or data doesn't meet requirements.
    """

    pass


class ServiceUnavailableError(ZenMCPError):
    """
    Raised when an external service is unavailable or unresponsive.

    Typically retryable and may include retry-after information in context.
    """

    pass


class CircuitBreakerOpenError(ZenMCPError):
    """
    Raised when a circuit breaker is open and blocking calls.

    Indicates a service has failed too many times and is temporarily unavailable.
    """

    pass


class RetryableError(ZenMCPError):
    """
    Raised when an operation is retryable but failed.

    Includes retry metadata in context.
    """

    pass


class TimeoutError(ZenMCPError):
    """
    Raised when an operation exceeds timeout threshold.

    Includes timeout duration and elapsed time in context.
    """

    pass
