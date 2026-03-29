"""Phench environment commands stub."""

from typer import Typer

app = Typer()


def register_env_commands(parent: Typer, **kwargs) -> None:  # noqa: ANN003 -> None:
    parent.add_typer(app, name="env")