"""Phench observability commands stub."""

from typer import Typer

app = Typer()


def register_observability_commands(parent: Typer, **kwargs) -> None:  # noqa: ANN003 -> None:
    parent.add_typer(app, name="observability")