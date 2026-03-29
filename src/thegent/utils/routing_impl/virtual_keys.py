"""GW-28: Virtual key system — per-key budget, rate, and model restrictions.

Each virtual key maps to a set of allowed models, rate limits, and a budget.
Keys are resolved from the Authorization header (Bearer sk-tg-...).

# @trace FR-KEYS-028
"""

from __future__ import annotations

import logging
import threading
import time
from dataclasses import dataclass, field

_log = logging.getLogger(__name__)


@dataclass
class VirtualKeyConfig:
    """Configuration for a single virtual key."""

    key_id: str  # e.g., "sk-tg-abc123"
    name: str = ""  # human label
    allowed_models: list[str] = field(default_factory=list)  # empty = all allowed
    rate_limit_rpm: int = 0  # requests/min; 0 = unlimited
    budget_usd: float = 0.0  # USD; 0 = unlimited
    budget_period: str = "monthly"  # "daily" | "weekly" | "monthly"
    metadata: dict[str, str] = field(default_factory=dict)
    created_at: float = field(default_factory=time.time)
    owner_id: str = ""  # user_id or team_id that owns this key


@dataclass
class VirtualKeyValidationResult:
    """Result of validating a virtual key against a request."""

    allowed: bool
    reason: str
    key_config: VirtualKeyConfig | None = None


class VirtualKeyStore:
    """Thread-safe store for VirtualKeyConfig objects, keyed by key_id."""

    def __init__(self) -> None:
        self._keys: dict[str, VirtualKeyConfig] = {}
        self._lock = threading.Lock()

    def register(self, config: VirtualKeyConfig) -> None:
        """Register or replace a virtual key configuration."""
        with self._lock:
            self._keys[config.key_id] = config
            _log.debug("Registered virtual key key_id=%s owner=%s", config.key_id, config.owner_id)

    def get(self, key_id: str) -> VirtualKeyConfig | None:
        """Return the VirtualKeyConfig for key_id, or None if not found."""
        with self._lock:
            return self._keys.get(key_id)

    def delete(self, key_id: str) -> bool:
        """Remove the key with key_id. Returns True if it existed, False otherwise."""
        with self._lock:
            if key_id in self._keys:
                del self._keys[key_id]
                _log.debug("Deleted virtual key key_id=%s", key_id)
                return True
            return False

    def list_keys(self, owner_id: str | None = None) -> list[VirtualKeyConfig]:
        """Return all registered keys, optionally filtered by owner_id."""
        with self._lock:
            if owner_id is None:
                return list(self._keys.values())
            return [cfg for cfg in self._keys.values() if cfg.owner_id == owner_id]


class VirtualKeyValidator:
    """Validates virtual keys against a request's model and store state."""

    def validate_key(
        self,
        key_id: str,
        model: str,
        store: VirtualKeyStore | None = None,
    ) -> VirtualKeyValidationResult:
        """Validate key_id for the requested model.

        Checks:
        1. Key exists in the store.
        2. If allowed_models is non-empty, model must be in the list.

        Args:
            key_id: The virtual key identifier (e.g. "sk-tg-abc123").
            model: The model being requested (e.g. "gpt-4o").
            store: The VirtualKeyStore to look up the key in. Defaults to the
                   global singleton.

        Returns:
            VirtualKeyValidationResult with allowed, reason, and key_config.
        """
        effective_store = store if store is not None else get_key_store()
        config = effective_store.get(key_id)

        if config is None:
            _log.warning("Virtual key not found key_id=%s", key_id)
            return VirtualKeyValidationResult(allowed=False, reason="key_not_found")

        if config.allowed_models and model not in config.allowed_models:
            _log.warning(
                "Model not allowed for virtual key key_id=%s model=%s allowed=%s",
                key_id,
                model,
                config.allowed_models,
            )
            return VirtualKeyValidationResult(
                allowed=False,
                reason="model_not_allowed",
                key_config=config,
            )

        _log.debug("Virtual key valid key_id=%s model=%s", key_id, model)
        return VirtualKeyValidationResult(allowed=True, reason="ok", key_config=config)


def extract_virtual_key_id(authorization: str | None) -> str | None:
    """Extract key_id from 'Bearer sk-tg-...' Authorization header.

    Returns None if not a virtual key (doesn't start with 'sk-tg-').

    Args:
        authorization: The raw Authorization header value, e.g.
            ``"Bearer sk-tg-abc123"``.

    Returns:
        The token string (e.g. ``"sk-tg-abc123"``) when it is a virtual key,
        or ``None`` otherwise.
    """
    if not authorization:
        return None
    if not authorization.startswith("Bearer "):
        return None
    token = authorization[7:]  # strip "Bearer "
    if token.startswith("sk-tg-"):
        return token
    return None


# ---------------------------------------------------------------------------
# Module-level singleton
# ---------------------------------------------------------------------------

_key_store: VirtualKeyStore | None = None
_key_store_lock = threading.Lock()


def get_key_store() -> VirtualKeyStore:
    """Return the process-global VirtualKeyStore singleton."""
    global _key_store
    if _key_store is None:
        with _key_store_lock:
            if _key_store is None:
                _key_store = VirtualKeyStore()
                _log.debug("Created global VirtualKeyStore singleton")
    return _key_store


def reset_key_store() -> None:
    """Reset the singleton (for testing only)."""
    global _key_store
    with _key_store_lock:
        _key_store = None
