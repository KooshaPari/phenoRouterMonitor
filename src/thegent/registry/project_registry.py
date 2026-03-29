"""Project registry for tracking agent-managed projects.

WBS: wp-71004-audit-cli
FR Traceability: FR-VER-005 (audit log and diff CLI)
"""

from __future__ import annotations

import sqlite3
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    pass


@dataclass
class Project:
    """A registered project."""

    id: int
    name: str
    path: str


class ProjectRegistry:
    """Registry for projects managed by thegent agents.

    Enables tracking of which projects have audit logs.
    """

    def __init__(self, db_path: Path) -> None:
        self.db_path = db_path
        self._init_db()

    def _init_db(self) -> None:
        """Initialize the database schema."""
        with sqlite3.connect(self.db_path) as conn:
            conn.execute(
                """
                CREATE TABLE IF NOT EXISTS projects (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL UNIQUE,
                    path TEXT NOT NULL
                )
                """
            )

    def register_project(self, name: str, path: str) -> Project:
        """Register a project.

        Returns the project record.
        """
        with sqlite3.connect(self.db_path) as conn:
            cursor = conn.execute(
                """
                INSERT INTO projects (name, path)
                VALUES (?, ?)
                ON CONFLICT(name) DO UPDATE SET path = excluded.path
                RETURNING id, name, path
                """,
                (name, path),
            )
            row = cursor.fetchone()
            conn.commit()

        return Project(id=row[0], name=row[1], path=row[2])

    def get_project(self, name: str) -> Project | None:
        """Get a project by name."""
        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row
            cursor = conn.execute(
                """
                SELECT id, name, path
                FROM projects
                WHERE name = ?
                """,
                (name,),
            )
            row = cursor.fetchone()

        if row is None:
            return None

        return Project(id=row["id"], name=row["name"], path=row["path"])

    def list_projects(self) -> list[Project]:
        """List all registered projects."""
        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row
            cursor = conn.execute(
                "SELECT id, name, path FROM projects ORDER BY name"
            )
            rows = cursor.fetchall()

        return [Project(id=row["id"], name=row["name"], path=row["path"]) for row in rows]
