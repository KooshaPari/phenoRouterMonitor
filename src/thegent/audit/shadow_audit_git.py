"""Shadow audit git operations for tracking agent changes.

WBS: wp-71004-audit-cli
FR Traceability: FR-VER-005 (audit log and diff CLI)
"""

from __future__ import annotations

import sqlite3
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    pass


@dataclass
class AuditEntry:
    """A single audit log entry."""

    id: int
    project_id: int
    sha: str
    message: str
    diff: str | None
    timestamp: datetime


class ShadowAuditGit:
    """Shadow git repository audit for tracking agent git operations.

    Records commits made by agents to enable diff and recovery.
    """

    def __init__(self, db_path: Path) -> None:
        self.db_path = db_path
        self._init_db()

    def _init_db(self) -> None:
        """Initialize the database schema."""
        with sqlite3.connect(self.db_path) as conn:
            conn.execute(
                """
                CREATE TABLE IF NOT EXISTS audit_entries (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    project_id INTEGER NOT NULL,
                    sha TEXT NOT NULL,
                    message TEXT NOT NULL,
                    diff TEXT,
                    timestamp TEXT NOT NULL
                )
                """
            )
            conn.execute(
                """
                CREATE INDEX IF NOT EXISTS idx_project_id
                ON audit_entries(project_id)
                """
            )

    def record_commit(
        self,
        project_id: int,
        sha: str,
        message: str,
        diff: str | None = None,
    ) -> int:
        """Record a commit in the audit log.

        Returns the entry ID.
        """
        timestamp = datetime.utcnow().isoformat()
        with sqlite3.connect(self.db_path) as conn:
            cursor = conn.execute(
                """
                INSERT INTO audit_entries (project_id, sha, message, diff, timestamp)
                VALUES (?, ?, ?, ?, ?)
                """,
                (project_id, sha, message, diff, timestamp),
            )
            conn.commit()
            return cursor.lastrowid

    def get_entries(
        self,
        project_id: int,
        limit: int | None = None,
    ) -> list[AuditEntry]:
        """Get audit entries for a project."""
        query = """
            SELECT id, project_id, sha, message, diff, timestamp
            FROM audit_entries
            WHERE project_id = ?
            ORDER BY id DESC
        """
        params: list[int | None] = [project_id]

        if limit is not None:
            query += " LIMIT ?"
            params.append(limit)

        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row
            cursor = conn.execute(query, params)
            rows = cursor.fetchall()

        return [
            AuditEntry(
                id=row["id"],
                project_id=row["project_id"],
                sha=row["sha"],
                message=row["message"],
                diff=row["diff"],
                timestamp=datetime.fromisoformat(row["timestamp"]),
            )
            for row in rows
        ]

    def get_entry_by_sha(self, project_id: int, sha: str) -> AuditEntry | None:
        """Get a specific entry by SHA."""
        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row
            cursor = conn.execute(
                """
                SELECT id, project_id, sha, message, diff, timestamp
                FROM audit_entries
                WHERE project_id = ? AND sha = ?
                """,
                (project_id, sha),
            )
            row = cursor.fetchone()

        if row is None:
            return None

        return AuditEntry(
            id=row["id"],
            project_id=row["project_id"],
            sha=row["sha"],
            message=row["message"],
            diff=row["diff"],
            timestamp=datetime.fromisoformat(row["timestamp"]),
        )
