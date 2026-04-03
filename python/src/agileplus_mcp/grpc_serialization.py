"""Serialization helpers for the AgilePlus gRPC client."""

from __future__ import annotations

from typing import Any


class AgilePlusGrpcSerializationMixin:
    """Convert generated gRPC messages into plain dictionaries."""

    @staticmethod
    def _feature_to_dict(f: Any) -> dict[str, Any]:
        return {
            "id": f.id,
            "slug": f.slug,
            "friendly_name": f.friendly_name,
            "state": f.state,
            "target_branch": f.target_branch,
            "created_at": f.created_at,
            "updated_at": f.updated_at,
            "wp_count": f.wp_count,
            "wp_done": f.wp_done,
        }

    @staticmethod
    def _wp_to_dict(wp: Any) -> dict[str, Any]:
        return {
            "id": wp.id,
            "title": wp.title,
            "state": wp.state,
            "sequence": wp.sequence,
            "agent_id": wp.agent_id,
            "pr_url": wp.pr_url,
            "pr_state": wp.pr_state,
            "depends_on": list(wp.depends_on),
            "file_scope": list(wp.file_scope),
        }

    @staticmethod
    def _audit_entry_to_dict(e: Any) -> dict[str, Any]:
        return {
            "id": e.id,
            "feature_slug": e.feature_slug,
            "wp_sequence": e.wp_sequence,
            "timestamp": e.timestamp,
            "actor": e.actor,
            "transition": e.transition,
            "evidence_refs": list(e.evidence_refs),
            "prev_hash": bytes(e.prev_hash).hex(),
            "hash": bytes(e.hash).hex(),
        }
