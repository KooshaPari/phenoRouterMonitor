"""Configuration management commands for thegent.

This module provides commands for validating, showing, and migrating configuration.
"""

from pathlib import Path

import typer
from rich.console import Console

from thegent.infra.config_validator import validate_config
from thegent.infra.config_wizard import run_wizard

console = Console()


def config_validate_cmd(
    config_path: str = typer.Option(".env", "--config", "-c", help="Path to configuration file"),
) -> None:
    """Validate configuration file.

    Examples:
        thegent config validate
        thegent config validate --config .env.production
    """
    path = Path(config_path)
    is_valid = validate_config(path)
    raise typer.Exit(0 if is_valid else 1)


def config_show_cmd(
    config_path: str = typer.Option(".env", "--config", "-c", help="Path to configuration file"),
) -> None:
    """Show current configuration.

    Examples:
        thegent config show
        thegent config show --config .env.production
    """
    from rich.table import Table

    from thegent.config import ThegentSettings

    path = Path(config_path)
    if not path.exists():
        console.print(f"[yellow]Configuration file not found: {path}[/yellow]")
        console.print("[dim]Using default settings. Run 'thegent setup --wizard' to configure.[/dim]")
        raise typer.Exit(0)

    try:
        # Use _env_file private parameter for pydantic-settings v2
        settings = ThegentSettings(_env_file=str(path))  # type: ignore[call-arg]

        table = Table(title="Current Configuration", show_header=True, header_style="bold cyan")
        table.add_column("Setting", style="cyan")
        table.add_column("Value", style="green")
        table.add_column("Source", style="yellow")

        # Show key settings
        key_settings = [
            ("MCP Host", settings.mcp_host, "THGENT_MCP_HOST"),
            ("MCP Port", settings.mcp_port, "THGENT_MCP_PORT"),
            ("Session Directory", settings.session_dir, "THGENT_SESSION_DIR"),
            ("Cache Directory", settings.cache_dir, "THGENT_CACHE_DIR"),
            ("Default Timeout", settings.default_timeout, "THGENT_DEFAULT_TIMEOUT"),
            ("Default Routing", settings.default_routing, "THGENT_DEFAULT_ROUTING"),
            ("Budget Hourly Limit", settings.budget_hourly_limit, "THGENT_BUDGET_HOURLY_LIMIT"),
            ("Budget Daily Limit", settings.budget_daily_limit, "THGENT_BUDGET_DAILY_LIMIT"),
            ("Session Backend", settings.session_backend, "THGENT_SESSION_BACKEND"),
        ]

        for name, value, env_key in key_settings:
            source = "env" if env_key in str(path.read_text()) else "default"
            table.add_row(name, str(value), source)

        console.print(table)
    except Exception as e:
        console.print(f"[red]Error loading configuration: {e}[/red]")
        raise typer.Exit(1)


def config_wizard_cmd(
    config_path: str = typer.Option(".env", "--config", "-c", help="Path to configuration file"),
) -> None:
    """Run interactive configuration wizard.

    Examples:
        thegent config wizard
        thegent config wizard --config .env.production
    """
    path = Path(config_path)
    success = run_wizard(path)
    raise typer.Exit(0 if success else 1)


def config_migrate_cmd(
    source: str = typer.Option(".env", "--source", "-s", help="Source configuration file"),
    target: str = typer.Option(".env", "--target", "-t", help="Target configuration file"),
    dry_run: bool = typer.Option(False, "--dry-run", help="Show what would be migrated without making changes"),
) -> None:
    """Migrate configuration from old format to new format.

    Examples:
        thegent config migrate
        thegent config migrate --source .env.old --target .env.new --dry-run
    """
    from rich.panel import Panel

    source_path = Path(source)
    target_path = Path(target)

    if not source_path.exists():
        console.print(f"[red]Source configuration file not found: {source_path}[/red]")
        raise typer.Exit(1)

    # Read source config
    try:
        source_content = source_path.read_text()
    except Exception as e:
        console.print(f"[red]Error reading source configuration: {e}[/red]")
        raise typer.Exit(1)

    # Parse and migrate
    migrated_lines = []
    migrated_lines.append("# thegent Configuration")
    migrated_lines.append("# Migrated configuration")
    migrated_lines.append("")

    # Simple migration: copy all THGENT_* variables
    for line in source_content.splitlines():
        line = line.strip()
        if (line.startswith("THGENT_") and "=" in line) or (line.startswith("#") and "THGENT" in line):
            migrated_lines.append(line)

    migrated_content = "\n".join(migrated_lines) + "\n"

    if dry_run:
        console.print(Panel(migrated_content, title="Migration Preview (Dry Run)", border_style="yellow"))
        console.print("[yellow]Dry run: No changes made. Remove --dry-run to apply migration.[/yellow]")
    else:
        try:
            target_path.parent.mkdir(parents=True, exist_ok=True)
            target_path.write_text(migrated_content)
            console.print(f"[green]✓ Configuration migrated to {target_path}[/green]")

            # Validate migrated config
            console.print("\n[dim]Validating migrated configuration...[/dim]")
            is_valid = validate_config(target_path)
            if not is_valid:
                console.print("[yellow]⚠ Migrated configuration has validation errors. Please review.[/yellow]")
        except Exception as e:
            console.print(f"[red]Error writing target configuration: {e}[/red]")
            raise typer.Exit(1)
