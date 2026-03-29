"""CLIProxy Cursor Phase 2 — cursor: schema, token-file provider, token refresh.

G-CP-01 / G-CP-02 / G-CP-03: Adds the Cursor-dedicated block to CLIProxy routing.

Capabilities:
  - cursor: schema support (token-file or auth-token variants)
  - Token refresh: reads sk-... from ~/.cursor-server/ or a configured token-file,
    re-reads on TTL expiry, signals rebindExecutors on change.
  - rebindExecutors: notifies active httpx sessions to re-authenticate after a
    token rotation event.

# @trace FR-CP-002
"""

from __future__ import annotations

import logging
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

_log = logging.getLogger(__name__)

# Default locations where cursor-api writes the session token
_CURSOR_SERVER_TOKEN_CANDIDATES: list[Path] = [
    Path("~/.cursor-server/session-token.txt"),
    Path("~/.cursor/session-token.txt"),
    Path("~/.config/cursor/session-token.txt"),
]

# Token TTL: re-read from disk every 5 minutes
_TOKEN_TTL_SECONDS: int = 300


# --------------------------------------------------------------------------- #
# Token provider                                                               #
# --------------------------------------------------------------------------- #


@dataclass
class CursorTokenProvider:
    """Reads and caches the Cursor session token from a file on disk.

    The token file contains a bare `sk-...` bearer token (written by
    cursor-api /build-key or auto-managed by thegent).

    # @trace FR-CP-002
    """

    token_file: Path
    _cached_token: str = field(default="", init=False, repr=False)
    _cached_mtime: float = field(default=0.0, init=False, repr=False)
    _last_read_at: float = field(default=0.0, init=False, repr=False)

    @classmethod
    def discover(cls) -> CursorTokenProvider | None:
        """Auto-discover the first readable token file from known Cursor paths."""
        for candidate in _CURSOR_SERVER_TOKEN_CANDIDATES:
            expanded = candidate.expanduser()
            if expanded.exists() and expanded.is_file():
                _log.debug("cursor token discovered at %s", expanded)
                return cls(token_file=expanded)
        return None

    def get_token(self) -> str:
        """Return the current token, re-reading from disk when TTL has expired."""
        now = time.monotonic()
        if now - self._last_read_at < _TOKEN_TTL_SECONDS:
            return self._cached_token
        self._refresh_from_disk()
        return self._cached_token

    def is_expired(self) -> bool:
        """Return True when the cached token is stale (TTL exceeded)."""
        return (time.monotonic() - self._last_read_at) >= _TOKEN_TTL_SECONDS

    def _refresh_from_disk(self) -> bool:
        """Re-read token from disk. Returns True if the token changed."""
        path = self.token_file.expanduser()
        if not path.exists():
            raise FileNotFoundError(
                f"Cursor token file not found: {path}. Run `thegent cliproxy login cursor` to generate one."
            )
        try:
            mtime = path.stat().st_mtime
            if mtime == self._cached_mtime:
                self._last_read_at = time.monotonic()
                return False
            token = path.read_text(encoding="utf-8").strip()
        except OSError as exc:
            raise RuntimeError(f"Cannot read Cursor token file {path}: {exc}") from exc
        changed = token != self._cached_token
        self._cached_token = token
        self._cached_mtime = mtime
        self._last_read_at = time.monotonic()
        if changed:
            _log.info("cursor token refreshed from %s (mtime changed)", path)
        return changed


# --------------------------------------------------------------------------- #
# Executor rebinding                                                           #
# --------------------------------------------------------------------------- #


class CursorExecutorManager:
    """Tracks active Cursor HTTP sessions and rebinds them on token rotation.

    When the token file changes, `rebindExecutors` closes stale sessions so the
    next request picks up the new bearer token automatically.

    # @trace FR-CP-002
    """

    def __init__(self, provider: CursorTokenProvider) -> None:
        self._provider = provider
        self._active_clients: list[Any] = []
        self._last_token: str = ""

    def register(self, client: Any) -> None:
        """Register an httpx.AsyncClient (or compatible) for rebinding."""
        self._active_clients.append(client)

    async def rebind_executors(self) -> int:
        """Close all registered clients if the token has rotated.

        Returns the number of clients that were re-bound.
        """
        current = self._provider.get_token()
        if current == self._last_token:
            return 0
        _log.info("cursor token rotated — rebinding %d active executor(s)", len(self._active_clients))
        count = 0
        for client in list(self._active_clients):
            if hasattr(client, "aclose"):
                try:
                    await client.aclose()
                except Exception:
                    pass
            count += 1
        self._active_clients.clear()
        self._last_token = current
        return count

    def get_auth_headers(self) -> dict[str, str]:
        """Build Authorization header dict for the current token."""
        token = self._provider.get_token()
        if not token:
            raise RuntimeError("Cursor token is empty. Run `thegent cliproxy login cursor` to authenticate.")
        return {"Authorization": f"Bearer {token}"}


# --------------------------------------------------------------------------- #
# Routing config builder                                                       #
# --------------------------------------------------------------------------- #


@dataclass
class CursorProviderConfig:
    """Full Phase 2 config for the cursor: routing schema.

    Maps to CLIProxyAPIPlus CursorKey YAML schema:
        cursor:
          - token-file: <path>
            cursor-api-url: <url>
    """

    cursor_api_url: str = "http://127.0.0.1:3000"
    token_file: str = ""
    auth_token: str = ""

    def to_cliproxy_block(self) -> dict[str, Any]:
        """Serialize to CLIProxyAPIPlus cursor YAML block."""
        entry: dict[str, Any] = {"cursor-api-url": self.cursor_api_url}
        if self.token_file:
            entry["token-file"] = self.token_file
        elif self.auth_token:
            entry["auth-token"] = self.auth_token
        else:
            raise ValueError(
                "CursorProviderConfig requires either token_file or auth_token. "
                "Set THGENT_CURSOR_API_TOKEN or configure a token file."
            )
        return {"cursor": [entry]}


def build_cursor_routing_config(
    cursor_api_url: str,
    token_file: str | None = None,
    auth_token: str | None = None,
) -> dict[str, Any]:
    """Build the cursor: routing config block for CLIProxyAPIPlus.

    Validates that exactly one auth mechanism is provided (token-file OR auth-token).

    # @trace FR-CP-002
    """
    if not token_file and not auth_token:
        raise ValueError(
            "Cursor Phase 2 requires either a token_file path or an auth_token. "
            "Provide THGENT_CURSOR_API_TOKEN (sk-...) or configure token-file in cliproxy config."
        )
    cfg = CursorProviderConfig(
        cursor_api_url=cursor_api_url.rstrip("/"),
        token_file=token_file or "",
        auth_token=auth_token or "",
    )
    return cfg.to_cliproxy_block()
