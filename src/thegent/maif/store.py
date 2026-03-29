"""MAIF artifact storage.

WBS: MAIF implementation
FR Traceability: FR-AUDIT-001
"""

from __future__ import annotations

import json
import sqlite3
from pathlib import Path
from typing import Any

from thegent.maif.artifacts import MAIFArtifact


class MAIFArtifactStore:
    """SQLite-based storage for MAIF artifacts."""

    def __init__(self, db_path: Path) -> None:
        self.db_path = db_path
        self._init_db()

    def _init_db(self) -> None:
        """Initialize the database schema."""
        with sqlite3.connect(self.db_path) as conn:
            conn.execute(
                """
                CREATE TABLE IF NOT EXISTS artifacts (
                    artifact_id TEXT PRIMARY KEY,
                    action_type TEXT NOT NULL,
                    payload TEXT NOT NULL,
                    agent_id TEXT NOT NULL,
                    session_id TEXT NOT NULL,
                    signature TEXT,
                    timestamp TEXT NOT NULL
                )
                """
            )
            conn.execute(
                """
                CREATE INDEX IF NOT EXISTS idx_session_id
                ON artifacts(session_id)
                """
            )

    def store(self, artifact: MAIFArtifact) -> None:
        """Store an artifact."""
        with sqlite3.connect(self.db_path) as conn:
            conn.execute(
                """
                INSERT OR REPLACE INTO artifacts
                (artifact_id, action_type, payload, agent_id, session_id, signature, timestamp)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    artifact.artifact_id,
                    artifact.action_type,
                    json.dumps(artifact.payload),
                    artifact.agent_id,
                    artifact.session_id,
                    artifact.signature,
                    artifact.timestamp,
                ),
            )
            conn.commit()

    def get(self, artifact_id: str) -> MAIFArtifact | None:
        """Retrieve an artifact by ID."""
        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row
            cursor = conn.execute(
                """
                SELECT artifact_id, action_type, payload, agent_id, session_id, signature, timestamp
                FROM artifacts
                WHERE artifact_id = ?
                """,
                (artifact_id,),
            )
            row = cursor.fetchone()

        if row is None:
            return None

        return MAIFArtifact(
            artifact_id=row["artifact_id"],
            action_type=row["action_type"],
            payload=json.loads(row["payload"]),
            agent_id=row["agent_id"],
            session_id=row["session_id"],
            signature=row["signature"],
            timestamp=row["timestamp"],
        )

    def list_by_session(self, session_id: str) -> list[MAIFArtifact]:
        """List all artifacts for a session."""
        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row
            cursor = conn.execute(
                """
                SELECT artifact_id, action_type, payload, agent_id, session_id, signature, timestamp
                FROM artifacts
                WHERE session_id = ?
                ORDER BY timestamp
                """,
                (session_id,),
            )
            rows = cursor.fetchall()

        return [
            MAIFArtifact(
                artifact_id=row["artifact_id"],
                action_type=row["action_type"],
                payload=json.loads(row["payload"]),
                agent_id=row["agent_id"],
                session_id=row["session_id"],
                signature=row["signature"],
                timestamp=row["timestamp"],
            )
            for row in rows
        ]

    def list_by_agent(self, agent_id: str) -> list[MAIFArtifact]:
        """List all artifacts for an agent."""
        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row
            cursor = conn.execute(
                """
                SELECT artifact_id, action_type, payload, agent_id, session_id, signature, timestamp
                FROM artifacts
                WHERE agent_id = ?
                ORDER BY timestamp DESC
                """,
                (agent_id,),
            )
            rows = cursor.fetchall()

        return [
            MAIFArtifact(
                artifact_id=row["artifact_id"],
                action_type=row["action_type"],
                payload=json.loads(row["payload"]),
                agent_id=row["agent_id"],
                session_id=row["session_id"],
                signature=row["signature"],
                timestamp=row["timestamp"],
            )
            for row in rows
        ]
