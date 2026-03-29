"""WP-22001: Context-Aware Shell History.
Stores shell commands with rich context (cwd, task_id, exit_code) in a local SQLite database.
Enables semantic search and task reconstruction.
"""

import orjson as json
import logging
import sqlite3
from datetime import UTC, datetime
from pathlib import Path

from pydantic import BaseModel

_log = logging.getLogger(__name__)


class HistoryEntry(BaseModel):
    """Rich metadata for a single shell command."""

    id: int | None = None
    timestamp: str = ""
    command: str
    cwd: str
    exit_code: int = 0
    task_id: str | None = None
    agent_id: str | None = None
    duration_s: float = 0.0
    tags: list[str] = []


class ContextHistory:
    """Manages the persistent store for context-aware shell history."""

    def __init__(self, db_path: Path | None = None) -> None:
        self.db_path = db_path or Path.home() / ".thegent" / "history.db"
        self.db_path.parent.mkdir(parents=True, exist_ok=True)
        self._init_db()

    def _init_db(self) -> None:
        """Initialize the SQLite database schema."""
        with sqlite3.connect(self.db_path) as conn:
            conn.execute(
                """
                CREATE TABLE IF NOT EXISTS history (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    timestamp TEXT NOT NULL,
                    command TEXT NOT NULL,
                    cwd TEXT NOT NULL,
                    exit_code INTEGER NOT NULL,
                    task_id TEXT,
                    agent_id TEXT,
                    duration_s REAL,
                    tags TEXT
                )
                """
            )
            # Create indices for common search fields
            conn.execute("CREATE INDEX IF NOT EXISTS idx_task_id ON history(task_id)")
            conn.execute("CREATE INDEX IF NOT EXISTS idx_cwd ON history(cwd)")
            conn.execute("CREATE INDEX IF NOT EXISTS idx_timestamp ON history(timestamp)")

    def record(self, entry: HistoryEntry) -> int:
        """Record a new command in history."""
        if not entry.timestamp:
            entry.timestamp = datetime.now(UTC).isoformat()

        with sqlite3.connect(self.db_path) as conn:
            cursor = conn.execute(
                """
                INSERT INTO history (timestamp, command, cwd, exit_code, task_id, agent_id, duration_s, tags)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    entry.timestamp,
                    entry.command,
                    entry.cwd,
                    entry.exit_code,
                    entry.task_id,
                    entry.agent_id,
                    entry.duration_s,
                    json.dumps(entry.tags).decode(),
                ),
            )
            return cursor.lastrowid or 0

    def search(
        self,
        query: str | None = None,
        task_id: str | None = None,
        cwd: str | None = None,
        limit: int = 50,
    ) -> list[HistoryEntry]:
        """Search history with filters."""
        sql = "SELECT * FROM history WHERE 1=1"
        params = []

        if query:
            sql += " AND command LIKE ?"
            params.append(f"%{query}%")
        if task_id:
            sql += " AND task_id = ?"
            params.append(task_id)
        if cwd:
            sql += " AND cwd = ?"
            params.append(cwd)

        sql += " ORDER BY timestamp DESC LIMIT ?"
        params.append(limit)

        results = []
        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row
            cursor = conn.execute(sql, params)
            for row in cursor:
                results.append(
                    HistoryEntry(
                        id=row["id"],
                        timestamp=row["timestamp"],
                        command=row["command"],
                        cwd=row["cwd"],
                        exit_code=row["exit_code"],
                        task_id=row["task_id"],
                        agent_id=row["agent_id"],
                        duration_s=row["duration_s"],
                        tags=json.loads(row["tags"] or "[]"),
                    )
                )
        return results

    def get_task_sequence(self, task_id: str) -> list[HistoryEntry]:
        """Retrieve the sequence of commands executed for a specific task."""
        return self.search(task_id=task_id, limit=1000)
