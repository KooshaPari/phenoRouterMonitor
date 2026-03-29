"""DAG helper service wrappers for impl.py compatibility."""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING, Any

from thegent.cli.commands import dag_impl

if TYPE_CHECKING:
    from thegent.cli.commands.dag_impl import DagDocument


def parse_dag_full(path: Path) -> "DagDocument":
    return dag_impl._parse_dag_full(path)


def serialize_dag(doc: "DagDocument") -> str:
    return dag_impl._serialize_dag(doc)


def parse_dag_session(path: Path) -> tuple[dict[str, str], list[dict[str, str]]]:
    return dag_impl._parse_dag_session(path)


def validate_task_id(task_id: str) -> str | None:
    return dag_impl._validate_task_id(task_id)


def validate_agent(agent: str) -> str | None:
    return dag_impl._validate_agent(agent)


def validate_dag(doc: "DagDocument") -> list[str]:
    return dag_impl._validate_dag(doc)


def dag_update_task(
    doc: "DagDocument",
    task_id: str,
    *,
    status: str | None = None,
    session_id: str | None = None,
    prompt: str | None = None,
    agent: str | None = None,
    depends_on: str | None = None,
    retry_count: int | None = None,
    contract_version: str | None = None,
) -> bool:
    return dag_impl._dag_update_task(
        doc=doc,
        task_id=task_id,
        status=status,
        session_id=session_id,
        prompt=prompt,
        agent=agent,
        depends_on=depends_on,
        retry_count=retry_count,
        contract_version=contract_version,
    )


def parse_depends_on(dep_str: str) -> list[str]:
    return dag_impl._parse_depends_on(dep_str)


def get_ready_task_ids(tasks: list[dict[str, str]]) -> list[str]:
    return dag_impl._get_ready_task_ids(tasks)


def dag_ready_impl(cd: Path | None = None) -> dict[str, Any]:
    return dag_impl.dag_ready_impl(cd)
