"""Data protection commands."""

from __future__ import annotations


def data_protection_cmd(format: str | None = None) -> None:
    """Show data protection status."""
    from thegent.cli.commands.impl import get_data_protection_status_impl

    status = get_data_protection_status_impl()

    if format == "json":
        import orjson as json

        from thegent.cli import console

        console.print(json.dumps(status))
    else:
        from thegent.cli import console
        from rich.table import Table

        table = Table(title="Data Protection Status")
        table.add_column("Setting", style="cyan")
        table.add_column("Value", style="green")

        for key, value in status.items():
            table.add_row(key, str(value))

        console.print(table)
