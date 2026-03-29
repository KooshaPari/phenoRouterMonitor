"""Interactive configuration wizard for thegent setup.

This module provides a step-by-step wizard for configuring thegent with
sensible defaults and validation at each step.
"""

from pathlib import Path
from typing import Any

from rich.console import Console
from rich.panel import Panel
from rich.prompt import Confirm, Prompt
from rich.table import Table

from thegent.config import ThegentSettings
from thegent.infra.progress import print_section, print_status, print_step

console = Console()


class ConfigWizard:
    """Interactive configuration wizard."""

    def __init__(self, config_path: Path | None = None) -> None:
        """Initialize the wizard.

        Args:
            config_path: Path to .env file (default: .env in current directory)
        """
        self.config_path = config_path or Path(".env")
        self.config: dict[str, Any] = {}
        self.settings = ThegentSettings()

    def run(self) -> bool:
        """Run the configuration wizard.

        Returns:
            True if configuration was successful, False otherwise
        """
        console.print(
            Panel("[bold cyan]thegent Configuration Wizard[/bold cyan]\n[dim]Let's set up your environment[/dim]")
        )

        try:
            # Step 1: Welcome and overview
            self._step_welcome()

            # Step 2: Basic configuration
            self._step_basic_config()

            # Step 3: Model defaults
            self._step_model_defaults()

            # Step 4: Performance settings
            self._step_performance()

            # Step 5: Budget settings
            self._step_budget()

            # Step 6: Advanced settings
            if Confirm.ask("\n[bold]Configure advanced settings?[/bold]", default=False):
                self._step_advanced()

            # Step 7: Review and save
            return self._step_review_and_save()

        except KeyboardInterrupt:
            console.print("\n[yellow]Configuration cancelled by user.[/yellow]")
            return False
        except Exception as e:
            console.print(f"\n[red]Error during configuration: {e}[/red]")
            return False

    def _step_welcome(self) -> None:
        """Welcome step with overview."""
        print_section("Welcome")
        console.print(
            "This wizard will help you configure thegent for optimal performance.\n"
            "You can skip any step by pressing Enter to use defaults.\n"
            "Press Ctrl+C at any time to cancel.\n"
        )

    def _step_basic_config(self) -> None:
        """Basic configuration step."""
        print_step(1, 6, "Basic Configuration")
        print_section("Basic Settings")

        # MCP Host
        default_host = str(self.settings.mcp_host)
        host = Prompt.ask("[bold]MCP Server Host[/bold]", default=default_host)
        self.config["THGENT_MCP_HOST"] = host

        # MCP Port
        default_port = str(self.settings.mcp_port)
        port = Prompt.ask("[bold]MCP Server Port[/bold]", default=default_port)
        try:
            port_int = int(port)
            if not (1 <= port_int <= 65535):
                raise ValueError("Port must be between 1 and 65535")
            self.config["THGENT_MCP_PORT"] = port
        except ValueError as _e:
            console.print(f"[yellow]Invalid port, using default: {default_port}[/yellow]")
            self.config["THGENT_MCP_PORT"] = default_port

        # Session directory
        default_session_dir = str(self.settings.session_dir.expanduser())
        session_dir = Prompt.ask("[bold]Session Directory[/bold]", default=default_session_dir)
        self.config["THGENT_SESSION_DIR"] = session_dir

        # Cache directory
        default_cache_dir = str(self.settings.cache_dir.expanduser())
        cache_dir = Prompt.ask("[bold]Cache Directory[/bold]", default=default_cache_dir)
        self.config["THGENT_CACHE_DIR"] = cache_dir

        print_status("Basic configuration complete", "success")

    def _step_model_defaults(self) -> None:
        """Model defaults configuration step."""
        print_step(2, 6, "Model Defaults")
        print_section("Default Models")

        models = {
            "Cursor": ("THGENT_DEFAULT_CURSOR_MODEL", self.settings.default_cursor_model),
            "Gemini": ("THGENT_DEFAULT_GEMINI_MODEL", self.settings.default_gemini_model),
            "Copilot": ("THGENT_DEFAULT_COPILOT_MODEL", self.settings.default_copilot_model),
            "Claude": ("THGENT_DEFAULT_CLAUDE_MODEL", self.settings.default_claude_model),
            "Codex": ("THGENT_DEFAULT_CODEX_MODEL", self.settings.default_codex_model),
        }

        console.print("[dim]Configure default models for each agent type.[/dim]\n")

        for name, (key, default) in models.items():
            value = Prompt.ask(f"[bold]{name} Model[/bold]", default=default)
            self.config[key] = value

        print_status("Model defaults configured", "success")

    def _step_performance(self) -> None:
        """Performance settings step."""
        print_step(3, 6, "Performance Settings")
        print_section("Performance Configuration")

        # Default timeout
        default_timeout = str(self.settings.default_timeout)
        timeout = Prompt.ask("[bold]Default Agent Timeout (seconds)[/bold]", default=default_timeout)
        try:
            timeout_int = int(timeout)
            if not (10 <= timeout_int <= 3600):
                raise ValueError("Timeout must be between 10 and 3600 seconds")
            self.config["THGENT_DEFAULT_TIMEOUT"] = timeout
        except ValueError:
            console.print(f"[yellow]Invalid timeout, using default: {default_timeout}[/yellow]")
            self.config["THGENT_DEFAULT_TIMEOUT"] = default_timeout

        # Max idle seconds
        default_idle = str(self.settings.max_idle_seconds)
        idle = Prompt.ask("[bold]Max Idle Seconds[/bold]", default=default_idle)
        try:
            idle_int = int(idle)
            if not (60 <= idle_int <= 600):
                raise ValueError("Idle must be between 60 and 600 seconds")
            self.config["THGENT_MAX_IDLE_SECONDS"] = idle
        except ValueError:
            console.print(f"[yellow]Invalid idle, using default: {default_idle}[/yellow]")
            self.config["THGENT_MAX_IDLE_SECONDS"] = default_idle

        # Default routing
        default_routing = self.settings.default_routing
        routing_options = ["prefer_direct", "prefer_proxy", "failover"]
        routing = Prompt.ask(
            "[bold]Default Routing Policy[/bold]",
            choices=routing_options,
            default=default_routing,
        )
        self.config["THGENT_DEFAULT_ROUTING"] = routing

        print_status("Performance settings configured", "success")

    def _step_budget(self) -> None:
        """Budget settings step."""
        print_step(4, 6, "Budget Settings")
        print_section("Budget Configuration")

        if not Confirm.ask("[bold]Configure budget limits?[/bold]", default=True):
            return

        # Hourly limit
        default_hourly = str(self.settings.budget_hourly_limit)
        hourly = Prompt.ask("[bold]Hourly Budget Limit (USD)[/bold]", default=default_hourly)
        try:
            hourly_float = float(hourly)
            if hourly_float < 0:
                raise ValueError("Budget must be non-negative")
            self.config["THGENT_BUDGET_HOURLY_LIMIT"] = hourly
        except ValueError:
            console.print(f"[yellow]Invalid budget, using default: {default_hourly}[/yellow]")
            self.config["THGENT_BUDGET_HOURLY_LIMIT"] = default_hourly

        # Daily limit
        default_daily = str(self.settings.budget_daily_limit)
        daily = Prompt.ask("[bold]Daily Budget Limit (USD)[/bold]", default=default_daily)
        try:
            daily_float = float(daily)
            if daily_float < 0:
                raise ValueError("Budget must be non-negative")
            self.config["THGENT_BUDGET_DAILY_LIMIT"] = daily
        except ValueError:
            console.print(f"[yellow]Invalid budget, using default: {default_daily}[/yellow]")
            self.config["THGENT_BUDGET_DAILY_LIMIT"] = default_daily

        # Per-run limit
        default_run = str(self.settings.budget_run_limit)
        run_limit = Prompt.ask("[bold]Per-Run Budget Limit (USD)[/bold]", default=default_run)
        try:
            run_float = float(run_limit)
            if run_float < 0:
                raise ValueError("Budget must be non-negative")
            self.config["THGENT_BUDGET_RUN_LIMIT"] = run_limit
        except ValueError:
            console.print(f"[yellow]Invalid budget, using default: {default_run}[/yellow]")
            self.config["THGENT_BUDGET_RUN_LIMIT"] = default_run

        print_status("Budget settings configured", "success")

    def _step_advanced(self) -> None:
        """Advanced settings step."""
        print_step(5, 6, "Advanced Settings")
        print_section("Advanced Configuration")

        # Session backend
        backend_options = ["auto", "zmx", "tmux", "none"]
        default_backend = self.settings.session_backend
        backend = Prompt.ask(
            "[bold]Session Backend[/bold]",
            choices=backend_options,
            default=default_backend,
        )
        self.config["THGENT_SESSION_BACKEND"] = backend

        # Retention days
        default_retention = str(self.settings.retention_days_sessions)
        retention = Prompt.ask("[bold]Session Retention (days)[/bold]", default=default_retention)
        try:
            retention_int = int(retention)
            if not (7 <= retention_int <= 365):
                raise ValueError("Retention must be between 7 and 365 days")
            self.config["THGENT_RETENTION_DAYS_SESSIONS"] = retention
        except ValueError:
            console.print(f"[yellow]Invalid retention, using default: {default_retention}[/yellow]")
            self.config["THGENT_RETENTION_DAYS_SESSIONS"] = default_retention

        print_status("Advanced settings configured", "success")

    def _step_review_and_save(self) -> bool:
        """Review and save configuration."""
        print_step(6, 6, "Review & Save")
        print_section("Configuration Review")

        # Display configuration summary
        table = Table(title="Configuration Summary", show_header=True, header_style="bold cyan")
        table.add_column("Setting", style="cyan")
        table.add_column("Value", style="green")

        for key, value in sorted(self.config.items()):
            table.add_row(key, str(value))

        console.print(table)

        # Confirm save
        if not Confirm.ask("\n[bold]Save this configuration?[/bold]", default=True):
            console.print("[yellow]Configuration not saved.[/yellow]")
            return False

        # Save to .env file
        try:
            self._save_config()
            print_status(f"Configuration saved to {self.config_path}", "success")
            return True
        except Exception as e:
            console.print(f"[red]Error saving configuration: {e}[/red]")
            return False

    def _save_config(self) -> None:
        """Save configuration to .env file."""
        lines = []
        lines.append("# thegent Configuration")
        lines.append("# Generated by configuration wizard")
        lines.append("")

        for key, value in sorted(self.config.items()):
            lines.append(f"{key}={value}")

        # Write to file
        self.config_path.parent.mkdir(parents=True, exist_ok=True)
        self.config_path.write_text("\n".join(lines) + "\n")


def run_wizard(config_path: Path | None = None) -> bool:
    """Run the configuration wizard.

    Args:
        config_path: Path to .env file (default: .env in current directory)

    Returns:
        True if configuration was successful, False otherwise
    """
    wizard = ConfigWizard(config_path)
    return wizard.run()
