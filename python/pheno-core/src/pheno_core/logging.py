"""Logging setup for Phenotype Core."""

import logging
import sys
from pathlib import Path
from typing import Any, Optional, Union

import structlog

# Configure structlog
structlog.configure(
    processors=[
        structlog.stdlib.filter_by_level,
        structlog.stdlib.add_logger_name,
        structlog.stdlib.add_log_level,
        structlog.stdlib.PositionalArgumentsFormatter(),
        structlog.processors.TimeStamper(fmt="iso"),
        structlog.processors.StackInfoRenderer(),
        structlog.processors.format_exc_info,
        structlog.processors.UnicodeDecoder(),
        structlog.processors.JSONRenderer()
    ],
    context_class=dict,
    logger_factory=structlog.stdlib.LoggerFactory(),
    cache_logger_on_first_use=True,
)


class Logger:
    """
    High-level logger interface.

    Wraps Python logging with convenience methods.
    """

    def __init__(self, name: str) -> None:
        """
        Initialize Logger.

        Args:
            name: Logger name.
        """
        self._logger = logging.getLogger(name)
        self._structlog = structlog.get_logger(name)

    def debug(self, message: str, **kwargs: Any) -> None:
        """Log debug message."""
        self._logger.debug(message)
        self._structlog.debug(message, **kwargs)

    def info(self, message: str, **kwargs: Any) -> None:
        """Log info message."""
        self._logger.info(message)
        self._structlog.info(message, **kwargs)

    def warning(self, message: str, **kwargs: Any) -> None:
        """Log warning message."""
        self._logger.warning(message)
        self._structlog.warning(message, **kwargs)

    def error(self, message: str, **kwargs: Any) -> None:
        """Log error message."""
        self._logger.error(message)
        self._structlog.error(message, **kwargs)

    def critical(self, message: str, **kwargs: Any) -> None:
        """Log critical message."""
        self._logger.critical(message)
        self._structlog.critical(message, **kwargs)

    def exception(self, message: str, **kwargs: Any) -> None:
        """Log exception with traceback."""
        self._logger.exception(message)
        self._structlog.exception(message, **kwargs)


def get_logger(name: str) -> Logger:
    """
    Get a logger instance.

    Args:
        name: Logger name (typically __name__ from calling module).

    Returns:
        Configured Logger instance.
    """
    return Logger(name)


def setup_console_logging(
    name: str,
    level: str = "INFO",
    format: Optional[str] = None,
) -> Logger:
    """
    Set up console (stdout) logging.

    Args:
        name: Logger name.
        level: Logging level as string (DEBUG, INFO, WARNING, ERROR, CRITICAL).
        format: Optional custom log format string.

    Returns:
        Configured Logger instance.
    """
    logger = logging.getLogger(name)
    logger.setLevel(getattr(logging, level.upper()))

    if not logger.handlers:
        handler = logging.StreamHandler(sys.stdout)

        if format:
            formatter = logging.Formatter(format)
        else:
            formatter = logging.Formatter(
                "%(asctime)s - %(name)s - %(levelname)s - %(message)s"
            )

        handler.setFormatter(formatter)
        logger.addHandler(handler)

    return Logger(name)


def setup_json_logging(
    name: str,
    level: str = "INFO",
) -> Logger:
    """
    Set up JSON-formatted logging via structlog.

    Args:
        name: Logger name.
        level: Logging level as string.

    Returns:
        Configured Logger instance.
    """
    logger = logging.getLogger(name)
    logger.setLevel(getattr(logging, level.upper()))

    if not logger.handlers:
        handler = logging.StreamHandler(sys.stdout)
        handler.setFormatter(logging.Formatter("%(message)s"))
        logger.addHandler(handler)

    return Logger(name)


def setup_file_logging(
    name: str,
    filepath: Union[str, Path],
    level: str = "INFO",
    format: Optional[str] = None,
    max_bytes: int = 10485760,  # 10MB
    backup_count: int = 5,
) -> Logger:
    """
    Set up file-based logging with rotation.

    Args:
        name: Logger name.
        filepath: Path to log file.
        level: Logging level as string.
        format: Optional custom log format string.
        max_bytes: Maximum bytes before rotation (default 10MB).
        backup_count: Number of backup files to keep.

    Returns:
        Configured Logger instance.
    """
    from logging.handlers import RotatingFileHandler

    logger = logging.getLogger(name)
    logger.setLevel(getattr(logging, level.upper()))

    if not logger.handlers:
        # Ensure parent directory exists
        path = Path(filepath)
        path.parent.mkdir(parents=True, exist_ok=True)

        handler = RotatingFileHandler(
            str(filepath),
            maxBytes=max_bytes,
            backupCount=backup_count,
        )

        if format:
            formatter = logging.Formatter(format)
        else:
            formatter = logging.Formatter(
                "%(asctime)s - %(name)s - %(levelname)s - %(message)s"
            )

        handler.setFormatter(formatter)
        logger.addHandler(handler)

    return Logger(name)


def setup_syslog_logging(
    name: str,
    level: str = "INFO",
    facility: Optional[int] = None,
) -> Logger:
    """
    Set up syslog logging.

    Args:
        name: Logger name.
        level: Logging level as string.
        facility: Syslog facility (default LOG_USER).

    Returns:
        Configured Logger instance.
    """
    from logging.handlers import SysLogHandler

    if facility is None:
        facility = SysLogHandler.LOG_USER

    logger = logging.getLogger(name)
    logger.setLevel(getattr(logging, level.upper()))

    if not logger.handlers:
        handler = SysLogHandler(facility=facility)
        formatter = logging.Formatter(
            "%(name)s[%(process)d]: %(levelname)s - %(message)s"
        )
        handler.setFormatter(formatter)
        logger.addHandler(handler)

    return Logger(name)
