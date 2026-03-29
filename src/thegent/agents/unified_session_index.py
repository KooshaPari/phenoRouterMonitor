"""Unified Agent Session Index.

Aggregates sessions from multiple agent harnesses with dual-redundancy:
1. SQLite DB snapshot for persistent query
2. Live file watching for real-time updates

Harnesses: Cursor, Codex, Claude, Ante, Droid
"""

import json
import logging
import sqlite3
import threading
import time
from dataclasses import dataclass, field
from datetime import UTC, datetime
from enum import Enum
from pathlib import Path
from typing import Any, Callable, ClassVar, cast

from thegent.infra.fast_file_watcher import FastFileWatcher
from thegent.integrations.base import SerializableMixin

_log = logging.getLogger(__name__)


class HarnessType(Enum):
    CURSOR = "cursor"
    CODEX = "codex"
    CLAUDE = "claude"
    ANTE = "ante"
    DROID = "droid"
    UNKNOWN = "unknown"


@dataclass
class AgentSession(SerializableMixin):
    """Unified agent session representation."""

    session_id: str
    harness: HarnessType
    project_path: str | None
    started_at: datetime
    ended_at: datetime | None = None
    prompt_tokens: int = 0
    completion_tokens: int = 0
    messages: list[dict[str, Any]] = field(default_factory=list)
    metadata: dict[str, Any] = field(default_factory=dict)
    summary: str | None = None


class UnifiedSessionIndex:
    """Unified session index with DB + live watching dual-redundancy."""

    def __init__(self, db_path: Path | None = None) -> None:
        self.db_path = db_path or Path.home() / ".thegent" / "session_index.db"
        self.db_path.parent.mkdir(parents=True, exist_ok=True)
        self._lock = threading.RLock()
        self._init_db()

        # Harness paths
        self._harness_paths: dict[HarnessType, Path] = {
            HarnessType.CURSOR: Path.home() / ".cursor",
            HarnessType.CODEX: Path.home() / ".codex",
            HarnessType.CLAUDE: Path.home() / ".claude-code",
            HarnessType.ANTE: Path.home() / ".ante",
        }

        # SQLite DBs per harness
        self._sqlite_dbs: dict[HarnessType, list[Path]] = {
            HarnessType.CURSOR: self._find_dbs(Path.home() / ".cursor", "*.db"),
            HarnessType.CODEX: self._find_dbs(Path.home() / ".codex", "*.db"),
            HarnessType.CLAUDE: self._find_dbs(Path.home() / ".claude-code", "*.db"),
            HarnessType.ANTE: self._find_dbs(Path.home() / ".ante", "*.db"),
        }

        self._watchers: dict[HarnessType, FastFileWatcher] = {}
        self._live_callbacks: list[Callable[..., Any]] = []

    def _find_dbs(self, root: Path, pattern: str) -> list[Path]:
        """Find SQLite DBs in directory tree."""
        if not root.exists():
            return []
        return list(root.rglob(pattern))

    def _init_db(self) -> None:
        """Initialize SQLite database."""
        with sqlite3.connect(self.db_path) as conn:
            conn.execute("""
                CREATE TABLE IF NOT EXISTS sessions (
                    session_id TEXT PRIMARY KEY,
                    harness TEXT NOT NULL,
                    project_path TEXT,
                    started_at TEXT NOT NULL,
                    ended_at TEXT,
                    prompt_tokens INTEGER DEFAULT 0,
                    completion_tokens INTEGER DEFAULT 0,
                    messages_json TEXT,
                    metadata_json TEXT,
                    summary TEXT,
                    indexed_at TEXT NOT NULL
                )
            """)
            conn.execute("""
                CREATE INDEX IF NOT EXISTS idx_harness ON sessions(harness)
            """)
            conn.execute("""
                CREATE INDEX IF NOT EXISTS idx_started ON sessions(started_at)
            """)
            conn.execute("""
                CREATE INDEX IF NOT EXISTS idx_project ON sessions(project_path)
            """)

    def index_all(self) -> int:
        """Full index of all harnesses. Returns count."""
        count = 0
        count += self._index_cursor()
        count += self._index_codex()
        count += self._index_ante()
        count += self._index_zmx()
        return count

    def _index_zmx(self) -> int:
        """Index active zmx sessions and verify live processes."""
        count = 0
        try:
            from thegent.session.zmx_backend import ZmxBackend

            backend = ZmxBackend()
            if backend.available:
                sessions = backend.list()
                for s in sessions:
                    # Dual-redundancy: check if PID is actually alive
                    is_alive = False
                    if s.pid:
                        try:
                            import os

                            os.kill(s.pid, 0)
                            is_alive = True
                        except (OSError, ProcessLookupError):
                            is_alive = False

                    session = AgentSession(
                        session_id=s.name,
                        harness=HarnessType.UNKNOWN,
                        project_path=None,
                        started_at=datetime.now(UTC),
                        metadata={"source": "zmx", "pid": s.pid, "state": s.state, "cmd": s.cmd, "live": is_alive},
                    )
                    self._upsert_session(session)
                    count += 1
        except Exception as e:
            _log.debug(f"ZMX index error: {e}")
        return count

    def _index_cursor(self) -> int:
        """Index Cursor sessions from SQLite + live files."""
        count = 0
        cursor_root = self._harness_paths[HarnessType.CURSOR]
        chats_dir = cursor_root / "chats"

        if not chats_dir.exists():
            return 0

        for session_dir in chats_dir.iterdir():
            if not session_dir.is_dir():
                continue

            # Check for store.db
            store_db = None
            for subdir in session_dir.iterdir():
                if subdir.is_dir():
                    db = subdir / "store.db"
                    if db.exists():
                        store_db = db
                        break

            if store_db:
                try:
                    sessions = self._parse_cursor_db(store_db)
                    for session in sessions:
                        self._upsert_session(session)
                        count += 1
                except Exception as e:
                    _log.debug(f"Cursor DB parse error: {e}")

            # Also check for transcript JSONL
            transcript = cursor_root / "projects" / "agent-transcripts"
            if transcript.exists():
                for jsonl in transcript.glob("*.jsonl"):
                    sessions = self._parse_cursor_transcript(jsonl)
                    for session in sessions:
                        self._upsert_session(session)
                        count += 1

        return count

    def _parse_cursor_db(self, db_path: Path) -> list[AgentSession]:
        """Parse Cursor session from store.db."""
        sessions = []
        try:
            with sqlite3.connect(db_path) as conn:
                conn.row_factory = sqlite3.Row
                cur = conn.execute("SELECT name FROM sqlite_master WHERE type='table'")
                tables = [r["name"] for r in cur.fetchall()]

                # Try to find conversation/session tables
                for table in tables:
                    if "conversation" in table.lower():
                        session = AgentSession(
                            session_id=db_path.parent.name,
                            harness=HarnessType.CURSOR,
                            project_path=str(db_path.parent.parent.parent),
                            started_at=datetime.now(UTC),
                            metadata={"source": "store.db", "table": table},
                        )
                        sessions.append(session)
                        break
        except Exception as e:
            _log.debug(f"Failed to parse {db_path}: {e}")

        return sessions

    def _parse_cursor_transcript(self, path: Path) -> list[AgentSession]:
        """Parse Cursor agent transcript JSONL."""
        sessions = []
        try:
            with open(path) as f:
                for line in f:
                    if not line.strip():
                        continue
                    data = json.loads(line)
                    session = AgentSession(
                        session_id=data.get("session_id", path.stem),
                        harness=HarnessType.CURSOR,
                        project_path=data.get("project"),
                        started_at=datetime.fromisoformat(data.get("timestamp", datetime.now(UTC).isoformat())),
                        messages=data.get("messages", []),
                        metadata={"source": "transcript"},
                    )
                    sessions.append(session)
        except Exception as e:
            _log.debug(f"Failed to parse transcript {path}: {e}")

        return sessions

    def _index_codex(self) -> int:
        """Index Codex sessions."""
        count = 0
        codex_root = self._harness_paths[HarnessType.CODEX]

        # Parse Codex SQLite
        sqlite_db = codex_root / "sqlite" / "codex-dev.db"
        if sqlite_db.exists():
            try:
                with sqlite3.connect(sqlite_db) as conn:
                    conn.row_factory = sqlite3.Row
                    cur = conn.execute("SELECT name FROM sqlite_master WHERE type='table'")
                    for row in cur.fetchall():
                        table = row["name"]
                        if "automation" in table.lower():
                            session = AgentSession(
                                session_id=f"{sqlite_db.stem}:{table}",
                                harness=HarnessType.CODEX,
                                project_path=str(codex_root),
                                started_at=datetime.now(UTC),
                                metadata={"source": "codex-dev.db", "table": table},
                            )
                            self._upsert_session(session)
                            count += 1
            except Exception as e:
                _log.debug(f"Codex DB error: {e}")

        return count

    def _index_ante(self) -> int:
        """Index Ante sessions from both session files and user input history."""
        count = 0
        ante_root = self._harness_paths[HarnessType.ANTE]

        # Parse individual session JSON files
        sessions_dir = ante_root / "sessions"
        if sessions_dir.exists():
            try:
                for session_file in sorted(sessions_dir.glob("*.json"), key=lambda x: x.stat().st_mtime, reverse=True):
                    parsed = self._parse_ante_session(session_file)
                    if parsed:
                        self._upsert_session(parsed)
                        count += 1
            except Exception as e:
                _log.debug(f"Ante sessions directory error: {e}")

        # Parse user_input_history.jsonl for prompts and metadata
        history_file = ante_root / "user_input_history.jsonl"
        if history_file.exists():
            try:
                with open(history_file) as f:
                    for line in f:
                        if not line.strip():
                            continue
                        data = json.loads(line)
                        if not isinstance(data, dict):
                            continue
                        prompt = data.get("prompt")
                        timestamp = data.get("timestamp")
                        if prompt and timestamp:
                            # Generate session ID from prompt + timestamp hash
                            session_id = f"ante-prompt-{hash((prompt, timestamp)) & 0x7FFFFFFF:08x}"
                            try:
                                session = AgentSession(
                                    session_id=session_id,
                                    harness=HarnessType.ANTE,
                                    project_path=None,
                                    started_at=datetime.fromisoformat(timestamp.replace("Z", "+00:00")),
                                    messages=[{"role": "user", "content": prompt}],
                                    metadata={"source": "user_input_history"},
                                )
                                self._upsert_session(session)
                                count += 1
                            except Exception as parse_err:
                                _log.debug(f"Error parsing Ante history line: {parse_err}")
            except Exception as e:
                _log.debug(f"Ante history error: {e}")

        return count

    def _parse_ante_session(self, session_file: Path) -> AgentSession | None:
        """Parse a single Ante session JSON file into AgentSession.

        Ante stores sessions as JSON with usage stats, timestamps, and model info.
        """
        try:
            with open(session_file) as f:
                data = json.load(f)

            if not isinstance(data, dict):
                return None

            session_id = data.get("id")
            if not session_id:
                return None

            # Extract usage stats
            usage = data.get("usage", {})
            input_tokens = int(usage.get("input_tokens", 0)) if isinstance(usage, dict) else 0
            output_tokens = int(usage.get("output_tokens", 0)) if isinstance(usage, dict) else 0

            # Extract timestamps
            started_time = data.get("started_time")
            if not started_time:
                return None

            # Parse ISO timestamp
            try:
                started_at = datetime.fromisoformat(started_time.replace("Z", "+00:00"))
            except Exception:
                started_at = datetime.now(UTC)

            # Ante doesn't store explicit end time; we infer from start + duration
            ended_at = None
            duration = data.get("duration", {})
            if isinstance(duration, dict):
                secs = int(duration.get("secs", 0))
                nanos = int(duration.get("nanos", 0))
                duration_secs = secs + (nanos / 1_000_000_000)
                if duration_secs > 0:
                    ended_at = datetime.fromtimestamp(started_at.timestamp() + duration_secs, tz=UTC)

            # Extract model and provider info
            model_info = data.get("model", {})
            model = model_info.get("name", "unknown") if isinstance(model_info, dict) else str(model_info or "unknown")

            provider_info = data.get("provider", {})
            provider = (
                provider_info.get("name", "unknown")
                if isinstance(provider_info, dict)
                else str(provider_info or "unknown")
            )

            # Project directory
            project_dir = data.get("dir")

            session = AgentSession(
                session_id=session_id,
                harness=HarnessType.ANTE,
                project_path=project_dir,
                started_at=started_at,
                ended_at=ended_at,
                prompt_tokens=input_tokens,
                completion_tokens=output_tokens,
                messages=[],  # Ante doesn't store full message history in session JSON
                metadata={
                    "source": "ante_session_json",
                    "model": model,
                    "provider": provider,
                    "thinking": model_info.get("thinking", "Unknown") if isinstance(model_info, dict) else None,
                },
            )
            return session
        except Exception as e:
            _log.debug(f"Failed to parse Ante session {session_file}: {e}")
            return None

    def _upsert_session(self, session: AgentSession) -> None:
        """Insert or update session in DB."""
        with self._lock:
            with sqlite3.connect(self.db_path) as conn:
                conn.execute(
                    """
                    INSERT OR REPLACE INTO sessions
                    (session_id, harness, project_path, started_at, ended_at,
                     prompt_tokens, completion_tokens, messages_json, metadata_json, summary, indexed_at)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    """,
                    (
                        session.session_id,
                        session.harness.value,
                        session.project_path,
                        session.started_at.isoformat(),
                        session.ended_at.isoformat() if session.ended_at else None,
                        session.prompt_tokens,
                        session.completion_tokens,
                        json.dumps(session.messages),
                        json.dumps(session.metadata),
                        session.summary,
                        datetime.now(UTC).isoformat(),
                    ),
                )

    def search(
        self,
        query: str | None = None,
        harness: HarnessType | None = None,
        project: str | None = None,
        limit: int = 100,
    ) -> list[AgentSession]:
        """Search sessions with filters."""
        with self._lock:
            with sqlite3.connect(self.db_path) as conn:
                conn.row_factory = sqlite3.Row

                sql = "SELECT * FROM sessions WHERE 1=1"
                params: list[Any] = []

                if harness:
                    sql += " AND harness = ?"
                    params.append(harness.value)

                if project:
                    sql += " AND project_path LIKE ?"
                    params.append(f"%{project}%")

                if query:
                    sql += " AND (messages_json LIKE ? OR metadata_json LIKE ?)"
                    params.extend([f"%{query}%", f"%{query}%"])

                sql += " ORDER BY started_at DESC LIMIT ?"
                params.append(limit)

                rows = conn.execute(sql, params).fetchall()
                return [self._row_to_session(row) for row in rows]

    def _row_to_session(self, row: sqlite3.Row) -> AgentSession:
        """Convert DB row to AgentSession."""
        return AgentSession(
            session_id=row["session_id"],
            harness=HarnessType(row["harness"]),
            project_path=row["project_path"],
            started_at=datetime.fromisoformat(row["started_at"]),
            ended_at=datetime.fromisoformat(row["ended_at"]) if row["ended_at"] else None,
            prompt_tokens=row["prompt_tokens"],
            completion_tokens=row["completion_tokens"],
            messages=json.loads(row["messages_json"] or "[]"),
            metadata=json.loads(row["metadata_json"] or "{}"),
            summary=row["summary"],
        )

    # === Live Watching ===

    def watch_start(self) -> None:
        """Start live file watching for real-time updates."""
        for harness, path in self._harness_paths.items():
            if path.exists():
                try:
                    watcher = FastFileWatcher(path, recursive=True)
                    watcher.start()
                    self._watchers[harness] = watcher
                    _log.info(f"Watching {harness.value} at {path}")
                except Exception as e:
                    _log.warning(f"Failed to watch {harness.value}: {e}")

    def watch_stop(self) -> None:
        """Stop all watchers."""
        for watcher in self._watchers.values():
            watcher.stop()
        self._watchers.clear()

    def on_session_change(self, callback: Callable[..., Any]) -> None:
        """Register callback for session changes."""
        self._live_callbacks.append(callback)

    # === Traversal ===

    def traverse_by_project(self, project_path: str) -> list[AgentSession]:
        """Get all sessions for a project across harnesses."""
        return self.search(project=project_path, limit=1000)

    def traverse_by_date(self, start: datetime, end: datetime) -> list[AgentSession]:
        """Get sessions in date range."""
        with self._lock:
            with sqlite3.connect(self.db_path) as conn:
                conn.row_factory = sqlite3.Row
                rows = conn.execute(
                    "SELECT * FROM sessions WHERE started_at BETWEEN ? AND ? ORDER BY started_at",
                    (start.isoformat(), end.isoformat()),
                ).fetchall()
                return [self._row_to_session(r) for r in rows]

    def export_for_rag(self, output_path: Path) -> int:
        """Export indexed data for RAG (vector DB ingestion)."""
        sessions = self.search(limit=10000)
        export_data = []
        for s in sessions:
            # Flatten session into a searchable text chunk
            content = f"Harness: {s.harness.value}\n"
            content += f"Project: {s.project_path or 'N/A'}\n"
            content += f"Date: {s.started_at.isoformat()}\n"
            if s.summary:
                content += f"Summary: {s.summary}\n"

            messages_text = ""
            for msg in s.messages:
                role = msg.get("role", "unknown")
                text = msg.get("content", "")
                if text:
                    messages_text += f"{role}: {text}\n"

            content += f"Messages:\n{messages_text}"

            export_data.append(
                {
                    "id": s.session_id,
                    "text": content,
                    "metadata": {
                        "harness": s.harness.value,
                        "project": s.project_path,
                        "date": s.started_at.isoformat(),
                        **s.metadata,
                    },
                }
            )

        with open(output_path, "w") as f:
            json.dump(export_data, f, indent=2)
        return len(export_data)

    def attach_session(self, session_id: str) -> bool:
        """Dynamically attach to an active session (using zmx backend)."""
        from thegent.session.zmx_backend import ZmxBackend

        backend = ZmxBackend()
        if backend.available:
            return backend.attach(session_id)
        return False

    def send_signal(self, session_id: str, sig: int = 2) -> bool:
        """Send a signal (default SIGINT/Ctrl+C) to an active session."""
        import os

        sessions = self.search(limit=1000)
        for s in sessions:
            if s.session_id == session_id and s.metadata.get("pid"):
                try:
                    os.kill(s.metadata["pid"], sig)
                    return True
                except OSError:
                    return False
        return False


# === CLI ===


def main() -> None:
    """CLI for unified session index."""
    import argparse

    parser = argparse.ArgumentParser(description="Unified Agent Session Index")
    parser.add_argument("--db", type=Path, help="SQLite DB path")
    parser.add_argument("--index", action="store_true", help="Full index all harnesses")
    parser.add_argument("--search", type=str, help="Search query")
    parser.add_argument("--harness", type=str, choices=["cursor", "codex", "ante"], help="Filter by harness")
    parser.add_argument("--project", type=str, help="Filter by project path")
    parser.add_argument("--watch", action="store_true", help="Start live watching")
    parser.add_argument("--limit", type=int, default=20, help="Result limit")

    args = parser.parse_args()
    index = UnifiedSessionIndex(args.db)

    if args.index:
        count = index.index_all()
        _log.info("Indexed %s sessions", count)

    if args.watch:
        index.watch_start()
        _log.info("Watching for changes... (Ctrl-C to stop)")
        try:
            while True:
                time.sleep(1)
        except KeyboardInterrupt:
            index.watch_stop()

    sessions = index.search(
        query=args.search,
        harness=HarnessType(args.harness) if args.harness else None,
        project=args.project,
        limit=args.limit,
    )

    for s in sessions:
        _log.info(
            "[%s] %s... | %s | %s",
            s.harness.value,
            s.session_id[:16],
            s.project_path,
            s.started_at.date(),
        )


if __name__ == "__main__":
    main()


# ---------------------------------------------------------------------------
# Harness TUI Mapper - Unified action interface for all agent harnesses
# ---------------------------------------------------------------------------


import subprocess
from thegent.infra.shim_subprocess import run as shim_run
import shlex


class HarnessActionError(Exception):
    """Raised when a harness action fails."""


class HarnessTUIMapper:
    """Maps abstract actions to harness-specific TUI/CLI commands.

    Provides a unified interface to interact with different agent harnesses
    (Cursor, Codex, Claude, Ante, Droid) using abstract action names.

    Usage:
        mapper = HarnessTUIMapper()
        result = mapper.execute(HarnessType.CODEX, "send_message", prompt="Fix the bug")
        result = mapper.execute(HarnessType.CURSOR, "attach_terminal", session_id="abc123")
    """

    # Abstract actions mapped to harness-specific commands
    HARNESS_ACTIONS: ClassVar["dict[str, dict[str, str | dict[str, str]]]"] = {
        "send_message": {
            "codex": "codex -p {prompt}",
            "claude": "claude --print {prompt}",
            "cursor": "cursor --command {prompt}",
            "ante": "ante run {prompt}",
            "droid": "droid chat {prompt}",
        },
        "attach_terminal": {
            "codex": "zmx attach codex-{session_id}",
            "claude": "zmx attach claude-{session_id}",
            "cursor": "cursor --attach {session_id}",
            "ante": "ante attach {session_id}",
            "droid": "droid attach {session_id}",
        },
        "view_history": {
            "codex": "codex history --session {session_id}",
            "claude": "claude sessions list",
            "cursor": "cursor chats list",
            "ante": "ante history",
            "droid": "droid sessions",
        },
        "get_status": {
            "codex": "codex status",
            "claude": "claude status",
            "cursor": "cursor status",
            "ante": "ante status",
            "droid": "droid status",
        },
        "list_sessions": {
            "codex": "codex sessions",
            "claude": "claude sessions list",
            "cursor": "cursor chats list",
            "ante": "ante sessions",
            "droid": "droid sessions",
        },
        "stop_session": {
            "codex": "codex stop {session_id}",
            "claude": "claude sessions stop {session_id}",
            "cursor": "cursor chats stop {session_id}",
            "ante": "ante stop {session_id}",
            "droid": "droid stop {session_id}",
        },
        "pause_session": {
            "codex": "codex pause {session_id}",
            "claude": "claude sessions pause {session_id}",
            "cursor": "cursor chats pause {session_id}",
            "ante": "ante pause {session_id}",
            "droid": "droid pause {session_id}",
        },
        "resume_session": {
            "codex": "codex resume {session_id}",
            "claude": "claude sessions resume {session_id}",
            "cursor": "cursor chats resume {session_id}",
            "ante": "ante resume {session_id}",
            "droid": "droid resume {session_id}",
        },
        "view_logs": {
            "codex": "codex logs {session_id}",
            "claude": "claude sessions logs {session_id}",
            "cursor": "cursor chats logs {session_id}",
            "ante": "ante logs {session_id}",
            "droid": "droid logs {session_id}",
        },
    }

    def __init__(self, custom_actions: dict[str, dict[str, str]] | None = None) -> None:
        """Initialize mapper with optional custom action overrides.

        Args:
            custom_actions: Override or add harness-specific commands.
                           Format: {action: {harness: command_template}}
        """
        self._actions: dict[str, dict[str, str | dict[str, str]]] = dict(self.HARNESS_ACTIONS)
        if custom_actions:
            for k, v in custom_actions.items():
                self._actions[k] = cast("dict[str, str | dict[str, str]]", v)

    def register_host(
        self,
        host_id: str,
        harness: HarnessType,
        command_prefix: str = "",
        custom_actions: dict[str, str] | None = None,
    ) -> None:
        """Register a new host device with custom command mappings.

        Args:
            host_id: Unique identifier for the host
            harness: The harness type this host uses
            command_prefix: Prefix to add to all commands (e.g., "ssh user@host")
            custom_actions: Override commands for this specific host
        """
        host_key = f"{harness.value}@{host_id}"
        for action, harness_cmd in self.HARNESS_ACTIONS.items():
            if harness.value in harness_cmd:
                base_cmd = harness_cmd[harness.value]
                if command_prefix:
                    full_cmd = f"{command_prefix} {base_cmd}"
                else:
                    full_cmd = base_cmd
                if action not in self._actions:
                    self._actions[action] = {}
                self._actions[action][host_key] = full_cmd

        if custom_actions:
            if not hasattr(self, "_host_overrides"):
                self._host_overrides = {}
            self._host_overrides[host_key] = custom_actions
            for action, cmd in custom_actions.items():
                if action not in self._actions:
                    self._actions[action] = {}
                self._actions[action][host_key] = cmd

    def execute(
        self,
        harness: HarnessType,
        action: str,
        host_id: str | None = None,
        **kwargs: Any,
    ) -> dict[str, Any]:
        """Execute an abstract action on a specific harness.

        Args:
            harness: The target harness type
            action: The abstract action to perform
            host_id: Optional host identifier for remote execution
            **kwargs: Template variables for the command (e.g., prompt, session_id)

        Returns:
            Dict with success status, output, and any errors

        Raises:
            HarnessActionError: If the action or harness is not supported
        """
        if action not in self._actions:
            raise HarnessActionError(f"Unknown action: {action}")

        harness_key = f"{harness.value}@{host_id}" if host_id else harness.value

        # Try host-specific command first, then fall back to default
        cmd_template = None
        if host_id and hasattr(self, "_host_overrides"):
            host_overrides = self._host_overrides.get(harness_key, {})
            if action in host_overrides:
                cmd_template = host_overrides[action]
        if not cmd_template and harness_key in self._actions.get(action, {}):
            cmd_template = self._actions[action][harness_key]
        if not cmd_template and harness.value in self._actions.get(action, {}):
            cmd_template = self._actions[action][harness.value]

        if not cmd_template:
            raise HarnessActionError(f"No command mapping for action={action} harness={harness.value}")

        if not isinstance(cmd_template, str):
            raise HarnessActionError(f"Command mapping for action={action} harness={harness.value} is not a string")

        # Substitute template variables
        try:
            cmd = cmd_template.format(**kwargs)
        except KeyError as e:
            raise HarnessActionError(f"Missing required parameter: {e}")

        try:
            result = shim_run(
                shlex.split(cmd),
                capture_output=True,
                text=True,
                timeout=30,
            )
            return {
                "success": result.returncode == 0,
                "stdout": result.stdout,
                "stderr": result.stderr,
                "returncode": result.returncode,
                "command": cmd,
            }
        except subprocess.TimeoutExpired:
            return {
                "success": False,
                "stdout": "",
                "stderr": "Command timed out after 30 seconds",
                "returncode": -1,
                "command": cmd,
            }
        except FileNotFoundError:
            raise HarnessActionError(f"Harness '{harness.value}' CLI not found. Is it installed?")

    def list_actions(self) -> list[str]:
        """List all available abstract actions."""
        return list(self._actions.keys())

    def list_harnesses(self, action: str) -> list[str]:
        """List harnesses that support a given action."""
        if action not in self._actions:
            return []
        return [k for k in self._actions[action] if not k.startswith("@")]

    def get_command(self, harness: HarnessType, action: str) -> str | None:
        """Get the command template for a harness-action pair."""
        value = self._actions.get(action, {}).get(harness.value)
        if isinstance(value, str):
            return value
        return None


__all__ = [
    "AgentSession",
    "HarnessActionError",
    "HarnessTUIMapper",
    "HarnessType",
    "UnifiedSessionIndex",
]
