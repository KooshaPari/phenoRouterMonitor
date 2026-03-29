"""Database helpers for thegent.

Common database utilities.
"""

from __future__ import annotations

import sqlite3
from pathlib import Path
from typing import Any


class Database:
    """Simple SQLite wrapper."""

    def __init__(self, path: str = ":memory:"):
        self.path = path
        self.conn: sqlite3.Connection | None = None

    def connect(self) -> None:
        self.conn = sqlite3.connect(self.path)

    def execute(self, query: str, params: tuple = ()) -> list[dict[str, Any]]:
        if not self.conn:
            self.connect()
        cursor = self.conn.cursor()  # type: ignore
        cursor.execute(query, params)
        if query.strip().upper().startswith("SELECT"):
            rows = cursor.fetchall()
            return [dict(row) for row in rows]
        self.conn.commit()
        return []

    def close(self) -> None:
        if self.conn:
            self.conn.close()


def init_db(path: str) -> Database:
    """Initialize database."""
    db = Database(path)
    db.connect()
    return db
