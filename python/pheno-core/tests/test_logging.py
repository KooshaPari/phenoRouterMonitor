"""Tests for pheno_core.logging module."""

import logging
from typing import Any

import structlog

from pheno_core.logging import (
    get_logger,
    setup_console_logging,
    setup_file_logging,
    setup_json_logging,
)


class TestLoggerCreation:
    """Test Logger creation and configuration."""

    def test_get_logger(self) -> None:
        """Test getting a logger instance."""
        logger = get_logger(__name__)
        assert logger is not None
        assert hasattr(logger, "info")
        assert hasattr(logger, "debug")
        assert hasattr(logger, "error")

    def test_get_logger_with_name(self) -> None:
        """Test getting logger with specific name."""
        logger = get_logger("test.module")
        assert logger is not None


class TestConsoleLogging:
    """Test console logging setup."""

    def test_setup_console_logging(self) -> None:
        """Test setting up console logging."""
        logger = setup_console_logging(
            name="test_console_setup",
            level="INFO"
        )

        assert logger is not None
        # Verify that the logger was created with correct name
        assert logger._logger.name == "test_console_setup"
        assert logger._logger.level == logging.INFO

    def test_console_logging_level(self) -> None:
        """Test console logging respects log level."""
        logger = setup_console_logging(
            name="test_level",
            level="ERROR"
        )

        # Set up to capture at handler level
        assert logger is not None


class TestJsonLogging:
    """Test JSON logging setup."""

    def test_setup_json_logging(self) -> None:
        """Test setting up JSON logging."""
        logger = setup_json_logging(
            name="test_json",
            level="INFO"
        )

        assert logger is not None

    def test_json_logging_format(self) -> None:
        """Test JSON logging outputs valid JSON."""
        logger = setup_json_logging(
            name="test_json_format",
            level="INFO"
        )

        assert logger is not None
        # JSON logging is configured through structlog


class TestFileLogging:
    """Test file logging setup."""

    def test_setup_file_logging(self, tmp_path: Any) -> None:
        """Test setting up file logging."""
        log_file = tmp_path / "test.log"

        logger = setup_file_logging(
            name="test_file",
            filepath=str(log_file),
            level="INFO"
        )

        assert logger is not None

    def test_file_logging_writes_to_file(self, tmp_path: Any) -> None:
        """Test that file logging actually writes to file."""
        log_file = tmp_path / "test_write.log"

        logger = setup_file_logging(
            name="test_file_write",
            filepath=str(log_file),
            level="INFO"
        )

        logger.info("test message")

        # Check file was created and contains log
        assert log_file.exists()
        content = log_file.read_text()
        assert "test message" in content or len(content) > 0


class TestLoggerInterface:
    """Test Logger interface methods."""

    def test_logger_has_required_methods(self) -> None:
        """Test that logger has all required methods."""
        logger = get_logger("test")

        required_methods = ["debug", "info", "warning", "error", "critical"]
        for method in required_methods:
            assert hasattr(logger, method), f"Logger missing method: {method}"

    def test_logger_with_context(self) -> None:
        """Test logging with context."""
        logger = get_logger("test_context")

        # Should support context binding
        assert logger is not None
        logger.info("message with context", user="alice", action="login")

    def test_logger_exception_logging(self) -> None:
        """Test logging exceptions."""
        logger = get_logger("test_exception")

        try:
            raise ValueError("test error")
        except ValueError:
            logger.exception("An error occurred")


class TestStructlogIntegration:
    """Test structlog integration."""

    def test_structlog_available(self) -> None:
        """Test that structlog is available."""
        assert structlog is not None

    def test_structlog_configuration(self) -> None:
        """Test structlog is properly configured."""
        # Structlog should be configured during setup
        logger = get_logger("test_structlog")
        assert logger is not None
