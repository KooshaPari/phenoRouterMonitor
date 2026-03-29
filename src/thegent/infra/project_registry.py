"""ProjectRegistry: hierarchical project state backed by SQLite (wp-71001).

Models: Product > Milestone > Sprint > Task > Episode.
DB stored at ``~/.thegent/registry.db`` by default.

# @trace FR-VCS-001
"""

from __future__ import annotations

import logging
import sqlite3
import uuid
from datetime import UTC, datetime
from pathlib import Path

from pydantic import BaseModel, field_validator

_log = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Pydantic models
# ---------------------------------------------------------------------------

_DEFAULT_DB_PATH = Path.home() / ".thegent" / "registry.db"


class Product(BaseModel):
    """Top-level product entity."""

    id: str
    name: str
    created_at: str = ""

    @field_validator("name")
    @classmethod
    def _name_not_empty(cls, v: str) -> str:
        if not v.strip():
            raise ValueError("Product name must not be empty")
        return v.strip()


class Milestone(BaseModel):
    """A milestone within a product."""

    id: str
    product_id: str
    name: str
    created_at: str = ""


class Sprint(BaseModel):
    """A sprint within a milestone."""

    id: str
    milestone_id: str
    name: str
    created_at: str = ""


class Task(BaseModel):
    """A task within a sprint."""

    id: str
    sprint_id: str
    name: str
    created_at: str = ""


class Episode(BaseModel):
    """An episode (agent work session) within a task."""

    id: str
    task_id: str
    status: str = "active"
    created_at: str = ""


# ---------------------------------------------------------------------------
# SQL schema
# ---------------------------------------------------------------------------

_SCHEMA = """\
CREATE TABLE IF NOT EXISTS products (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS milestones (
    id TEXT PRIMARY KEY,
    product_id TEXT NOT NULL REFERENCES products(id),
    name TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS sprints (
    id TEXT PRIMARY KEY,
    milestone_id TEXT NOT NULL REFERENCES milestones(id),
    name TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    sprint_id TEXT NOT NULL REFERENCES sprints(id),
    name TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS episodes (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL REFERENCES tasks(id),
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL
);
"""


def _now_iso() -> str:
    return datetime.now(UTC).isoformat()


def _new_id() -> str:
    return str(uuid.uuid4())


# ---------------------------------------------------------------------------
# ProjectRegistry
# ---------------------------------------------------------------------------


class ProjectRegistry:
    """Hierarchical project registry backed by SQLite.

    Args:
        db_path: Path to the SQLite database file.
            Defaults to ``~/.thegent/registry.db``.
    """

    def __init__(self, db_path: Path = _DEFAULT_DB_PATH) -> None:
        self._db_path = Path(db_path)
        self._db_path.parent.mkdir(parents=True, exist_ok=True)
        self._conn = sqlite3.connect(str(self._db_path))
        self._conn.execute("PRAGMA journal_mode=WAL")
        self._conn.execute("PRAGMA foreign_keys=ON")
        self._init_schema()

    def _init_schema(self) -> None:
        cur = self._conn.cursor()
        cur.executescript(_SCHEMA)
        self._conn.commit()

    # -- Products -----------------------------------------------------------

    def create_product(self, name: str) -> Product:
        """Create a new product."""
        product = Product(id=_new_id(), name=name, created_at=_now_iso())
        self._conn.execute(
            "INSERT INTO products (id, name, created_at) VALUES (?, ?, ?)",
            (product.id, product.name, product.created_at),
        )
        self._conn.commit()
        _log.debug("Created product %s: %s", product.id, product.name)
        return product

    # -- Milestones ---------------------------------------------------------

    def create_milestone(self, product_id: str, name: str) -> Milestone:
        """Create a new milestone under a product."""
        self._require_exists("products", product_id, "Product")
        milestone = Milestone(
            id=_new_id(),
            product_id=product_id,
            name=name,
            created_at=_now_iso(),
        )
        self._conn.execute(
            "INSERT INTO milestones (id, product_id, name, created_at) VALUES (?, ?, ?, ?)",
            (milestone.id, milestone.product_id, milestone.name, milestone.created_at),
        )
        self._conn.commit()
        _log.debug("Created milestone %s: %s", milestone.id, milestone.name)
        return milestone

    # -- Sprints ------------------------------------------------------------

    def create_sprint(self, milestone_id: str, name: str) -> Sprint:
        """Create a new sprint under a milestone."""
        self._require_exists("milestones", milestone_id, "Milestone")
        sprint = Sprint(
            id=_new_id(),
            milestone_id=milestone_id,
            name=name,
            created_at=_now_iso(),
        )
        self._conn.execute(
            "INSERT INTO sprints (id, milestone_id, name, created_at) VALUES (?, ?, ?, ?)",
            (sprint.id, sprint.milestone_id, sprint.name, sprint.created_at),
        )
        self._conn.commit()
        _log.debug("Created sprint %s: %s", sprint.id, sprint.name)
        return sprint

    # -- Tasks --------------------------------------------------------------

    def create_task(self, sprint_id: str, name: str) -> Task:
        """Create a new task under a sprint."""
        self._require_exists("sprints", sprint_id, "Sprint")
        task = Task(
            id=_new_id(),
            sprint_id=sprint_id,
            name=name,
            created_at=_now_iso(),
        )
        self._conn.execute(
            "INSERT INTO tasks (id, sprint_id, name, created_at) VALUES (?, ?, ?, ?)",
            (task.id, task.sprint_id, task.name, task.created_at),
        )
        self._conn.commit()
        _log.debug("Created task %s: %s", task.id, task.name)
        return task

    # -- Episodes -----------------------------------------------------------

    def create_episode(self, task_id: str) -> Episode:
        """Create a new active episode under a task."""
        self._require_exists("tasks", task_id, "Task")
        episode = Episode(
            id=_new_id(),
            task_id=task_id,
            status="active",
            created_at=_now_iso(),
        )
        self._conn.execute(
            "INSERT INTO episodes (id, task_id, status, created_at) VALUES (?, ?, ?, ?)",
            (episode.id, episode.task_id, episode.status, episode.created_at),
        )
        self._conn.commit()
        _log.debug("Created episode %s for task %s", episode.id, task_id)
        return episode

    def get_active_episode(self) -> Episode | None:
        """Return the most recently created active episode, or None."""
        row = self._conn.execute(
            "SELECT id, task_id, status, created_at FROM episodes "
            "WHERE status = 'active' ORDER BY created_at DESC LIMIT 1",
        ).fetchone()
        if row is None:
            return None
        return Episode(id=row[0], task_id=row[1], status=row[2], created_at=row[3])

    def complete_episode(self, episode_id: str) -> None:
        """Mark an episode as completed."""
        self._conn.execute(
            "UPDATE episodes SET status = 'completed' WHERE id = ?",
            (episode_id,),
        )
        self._conn.commit()

    def list_episodes(self, task_id: str) -> list[Episode]:
        """List all episodes for a given task."""
        rows = self._conn.execute(
            "SELECT id, task_id, status, created_at FROM episodes WHERE task_id = ? ORDER BY created_at",
            (task_id,),
        ).fetchall()
        return [Episode(id=r[0], task_id=r[1], status=r[2], created_at=r[3]) for r in rows]

    # -- Listing methods ----------------------------------------------------

    def get_first_product_id(self) -> str | None:
        """Return the ID of the first product, or None if no products exist."""
        row = self._conn.execute(
            "SELECT id FROM products ORDER BY created_at LIMIT 1",
        ).fetchone()
        return row[0] if row else None

    def get_first_milestone_id(self) -> str | None:
        """Return the ID of the first milestone, or None if none exist."""
        row = self._conn.execute(
            "SELECT id FROM milestones ORDER BY created_at LIMIT 1",
        ).fetchone()
        return row[0] if row else None

    def list_milestones(self) -> list[Milestone]:
        """List all milestones ordered by creation time."""
        rows = self._conn.execute(
            "SELECT id, product_id, name, created_at FROM milestones ORDER BY created_at",
        ).fetchall()
        return [Milestone(id=r[0], product_id=r[1], name=r[2], created_at=r[3]) for r in rows]

    def list_sprints(self) -> list[Sprint]:
        """List all sprints ordered by creation time."""
        rows = self._conn.execute(
            "SELECT id, milestone_id, name, created_at FROM sprints ORDER BY created_at",
        ).fetchall()
        return [Sprint(id=r[0], milestone_id=r[1], name=r[2], created_at=r[3]) for r in rows]

    # -- Helpers ------------------------------------------------------------

    def _require_exists(self, table: str, row_id: str, label: str) -> None:
        """Raise ValueError if a row with the given id does not exist."""
        row = self._conn.execute(
            f"SELECT 1 FROM {table} WHERE id = ?",  # noqa: S608 -- table name is internal, not user input
            (row_id,),
        ).fetchone()
        if row is None:
            raise ValueError(f"{label} '{row_id}' not found")
