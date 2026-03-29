"""Sync adapter for workstream autosync.

Handles GitHub and Linear sync operations.
"""

import hashlib
from typing import Any

from thegent.integrations.workstream_autosync_shared import WorkstreamItem


class SyncAdapter:
    """Adapter for sync operations."""

    def __init__(self, config: Any):
        self.config = config

    # Operation ID helpers
    def build_operation_id(self, platform: str, direction: str, items: list[WorkstreamItem]) -> str:
        """Build replay-safe deterministic operation IDs for sync batches."""
        item_key = ",".join(sorted(item.item_id for item in items))
        digest = hashlib.sha1(f"{platform}:{direction}:{item_key}".encode()).hexdigest()[:12]
        return f"{platform}-{direction}-{digest}"

    def build_mutation_id(self, platform: str, item: WorkstreamItem) -> str:
        """Build a deterministic mutation identifier for one item write."""
        payload = f"{platform}:{item.item_id}:{item.status}:{item.priority}:{item.area}"
        digest = hashlib.sha1(payload.encode("utf-8")).hexdigest()[:16]
        return f"{platform}-mutation-{item.item_id}-{digest}"

    # Checksum helpers
    def normalize_for_checksum(self, payload: list[dict[str, Any]]) -> list[dict[str, Any]]:
        """Return a deterministic remote payload representation for checksum verification."""
        normalized = []
        for item in payload:
            normalized_item = {
                "item_id": item.get("item_id"),
                "status": item.get("status"),
                "priority": item.get("priority"),
                "area": item.get("area"),
            }
            if "blocked_by" in item:
                normalized_item["blocked_by"] = sorted(item.get("blocked_by") or [])
            normalized.append(normalized_item)
        return sorted(normalized, key=lambda x: x.get("item_id", ""))

    # Cycle fingerprint
    def compute_cycle_fingerprint(self, items: list[WorkstreamItem]) -> str:
        """Compute a fingerprint for the current cycle."""
        canonical = ",".join(
            sorted(
                f"{item.item_id}:{item.status}:{item.priority}:{item.area}:{item.blocked_by or ''}" for item in items
            )
        )
        return hashlib.sha1(canonical.encode("utf-8")).hexdigest()


__all__ = ["SyncAdapter"]


# Register with unified adapter registry
from thegent.adapters.ports import AdapterRegistry


class SyncAdapterWrapper:
    """Sync adapter wrapper for registry"""

    def __init__(self):
        self._adapter = SyncAdapter(config=None)

    def call(self, **kwargs) -> dict:
        return {"status": "sync_adapter_ready"}


AdapterRegistry.register("sync", SyncAdapterWrapper())
