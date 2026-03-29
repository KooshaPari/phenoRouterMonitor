"""Phench timeline commands stub."""

from typer import Typer

app = Typer()


def register_timeline_commands(parent: Typer, **kwargs) -> None:  # noqa: ANN003 -> None:
    parent.add_typer(app, name="timeline")