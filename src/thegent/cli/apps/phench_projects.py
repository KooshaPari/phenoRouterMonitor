"""Phench projects commands stub."""

from typer import Typer

app = Typer()


def register_projects_run(parent: Typer, **kwargs) -> None:  # noqa: ANN003 -> None:
    parent.add_typer(app, name="projects")