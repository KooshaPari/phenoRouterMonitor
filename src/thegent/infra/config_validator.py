"""Configuration validation for thegent.

This module provides utilities for validating configuration files and
settings before they are used.
"""

from pathlib import Path
from rich.console import Console
from rich.panel import Panel
from rich.table import Table

from thegent.config import ThegentSettings

console = Console()


class ConfigValidator:
    """Configuration validator."""

    def __init__(self, config_path: Path | None = None) -> None:
        """Initialize the validator.

        Args:
            config_path: Path to .env file (default: .env in current directory)
        """
        self.config_path = config_path or Path(".env")
        self.errors: list[str] = []
        self.warnings: list[str] = []
        self.settings: ThegentSettings | None = None

    def validate(self) -> bool:
        """Validate configuration.

        Returns:
            True if configuration is valid, False otherwise
        """
        self.errors.clear()
        self.warnings.clear()

        # Check if config file exists
        if not self.config_path.exists():
            self.warnings.append(f"Configuration file not found: {self.config_path}")
            self.warnings.append("Using default settings. Run 'thegent setup --wizard' to configure.")
            return True  # Not an error, just using defaults

        # Try to load settings
        try:
            self.settings = ThegentSettings()
        except Exception as e:
            self.errors.append(f"Failed to load configuration: {e}")
            return False

        # Validate individual settings
        self._validate_basic_settings()
        self._validate_model_settings()
        self._validate_performance_settings()
        self._validate_budget_settings()
        self._validate_advanced_settings()

        return len(self.errors) == 0

    def _validate_basic_settings(self) -> None:
        """Validate basic settings."""
        if not self.settings:
            return

        # MCP Host
        if not self.settings.mcp_host:
            self.errors.append("MCP_HOST is required")

        # MCP Port
        if not (1 <= self.settings.mcp_port <= 65535):
            self.errors.append(f"MCP_PORT must be between 1 and 65535 (got {self.settings.mcp_port})")

        # Session directory
        if self.settings.session_dir:
            session_path = self.settings.session_dir.expanduser()
            if not session_path.parent.exists():
                self.warnings.append(f"Session directory parent does not exist: {session_path.parent}")

        # Cache directory
        if self.settings.cache_dir:
            cache_path = self.settings.cache_dir.expanduser()
            if not cache_path.parent.exists():
                self.warnings.append(f"Cache directory parent does not exist: {cache_path.parent}")

    def _validate_model_settings(self) -> None:
        """Validate model settings."""
        if not self.settings:
            return

        # Check that model names are not empty
        models = [
            ("default_cursor_model", self.settings.default_cursor_model),
            ("default_gemini_model", self.settings.default_gemini_model),
            ("default_copilot_model", self.settings.default_copilot_model),
            ("default_claude_model", self.settings.default_claude_model),
            ("default_codex_model", self.settings.default_codex_model),
        ]

        for name, value in models:
            if not value or not value.strip():
                self.errors.append(f"{name} cannot be empty")

    def _validate_performance_settings(self) -> None:
        """Validate performance settings."""
        if not self.settings:
            return

        # Default timeout
        if not (10 <= self.settings.default_timeout <= 3600):
            self.errors.append(
                f"default_timeout must be between 10 and 3600 seconds (got {self.settings.default_timeout})"
            )

        # Max idle seconds
        if not (60 <= self.settings.max_idle_seconds <= 600):
            self.errors.append(
                f"max_idle_seconds must be between 60 and 600 seconds (got {self.settings.max_idle_seconds})"
            )

        # Default routing
        valid_routing = ["prefer_direct", "prefer_proxy", "failover"]
        if self.settings.default_routing not in valid_routing:
            self.errors.append(f"default_routing must be one of {valid_routing} (got {self.settings.default_routing})")

    def _validate_budget_settings(self) -> None:
        """Validate budget settings."""
        if not self.settings:
            return

        # Budget limits must be non-negative
        if self.settings.budget_hourly_limit < 0:
            self.errors.append(f"budget_hourly_limit must be non-negative (got {self.settings.budget_hourly_limit})")

        if self.settings.budget_daily_limit < 0:
            self.errors.append(f"budget_daily_limit must be non-negative (got {self.settings.budget_daily_limit})")

        if self.settings.budget_run_limit < 0:
            self.errors.append(f"budget_run_limit must be non-negative (got {self.settings.budget_run_limit})")

        # Warning threshold
        if not (0.0 <= self.settings.budget_warning_threshold <= 1.0):
            self.errors.append(
                f"budget_warning_threshold must be between 0.0 and 1.0 (got {self.settings.budget_warning_threshold})"
            )

    def _validate_advanced_settings(self) -> None:
        """Validate advanced settings."""
        if not self.settings:
            return

        # Session backend
        valid_backends = ["auto", "zmx", "tmux", "none"]
        if self.settings.session_backend not in valid_backends:
            self.errors.append(f"session_backend must be one of {valid_backends} (got {self.settings.session_backend})")

        # Retention days
        if not (7 <= self.settings.retention_days_sessions <= 365):
            self.errors.append(
                f"retention_days_sessions must be between 7 and 365 days (got {self.settings.retention_days_sessions})"
            )

    def display_results(self) -> None:
        """Display validation results."""
        if len(self.errors) == 0 and len(self.warnings) == 0:
            console.print(Panel("[bold green]✓ Configuration is valid[/bold green]"))
            return

        # Display errors
        if self.errors:
            error_table = Table(title="Errors", show_header=True, header_style="bold red")
            error_table.add_column("Error", style="red")

            for error in self.errors:
                error_table.add_row(error)

            console.print(error_table)

        # Display warnings
        if self.warnings:
            warning_table = Table(title="Warnings", show_header=True, header_style="bold yellow")
            warning_table.add_column("Warning", style="yellow")

            for warning in self.warnings:
                warning_table.add_row(warning)

            console.print(warning_table)


def validate_config(config_path: Path | None = None) -> bool:
    """Validate configuration file.

    Args:
        config_path: Path to .env file (default: .env in current directory)

    Returns:
        True if configuration is valid, False otherwise
    """
    validator = ConfigValidator(config_path)
    is_valid = validator.validate()
    validator.display_results()
    return is_valid
