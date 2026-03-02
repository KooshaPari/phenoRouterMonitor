"""Behave environment hooks for AgilePlus BDD tests.

Sets up mock gRPC client state before each scenario and tears it down
after. All BDD tests run against mocks — no real gRPC server required.

Traceability: WP16-T093
"""
from __future__ import annotations

import asyncio
from typing import Any
from unittest.mock import AsyncMock, MagicMock

from agileplus_mcp.grpc_client import AgilePlusCoreClient


def _make_feature(slug: str, state: str) -> dict[str, Any]:
    return {
        "id": 1,
        "slug": slug,
        "friendly_name": slug.replace("-", " ").title(),
        "state": state,
        "target_branch": "main",
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-15T00:00:00Z",
        "wp_count": 0,
        "wp_done": 0,
    }


def before_scenario(context, scenario):  # noqa: ANN001
    """Initialise fresh mock state before each BDD scenario."""
    context.loop = asyncio.new_event_loop()
    context.client = MagicMock(spec=AgilePlusCoreClient)

    # Default mock responses (overridden in Given steps as needed)
    context.client.run_command = AsyncMock(
        return_value={"success": True, "message": "ok", "outputs": {}}
    )
    context.client.get_feature = AsyncMock(
        return_value=_make_feature("default-feature", "created")
    )
    context.client.list_features = AsyncMock(return_value=[])
    context.client.list_work_packages = AsyncMock(return_value=[])
    context.client.get_feature_state = AsyncMock(
        return_value={"state": "created", "next_command": "specify", "blockers": []}
    )
    context.client.get_audit_trail = AsyncMock(return_value=[])
    context.client.verify_audit_chain = AsyncMock(
        return_value={
            "valid": True,
            "entries_verified": 0,
            "first_invalid_id": 0,
            "error_message": "",
        }
    )
    context.client.check_governance_gate = AsyncMock(
        return_value={"passed": True, "violations": []}
    )

    # Scenario-level state
    context.last_result = None
    context.features: dict[str, dict[str, Any]] = {}
    context.audit_entries: list[dict[str, Any]] = []
    context.governance_contracts: dict[str, dict[str, Any]] = {}
    context.evidence: list[dict[str, Any]] = []
    context.validation_report: dict[str, Any] | None = None


def after_scenario(context, scenario):  # noqa: ANN001
    """Clean up the event loop after each scenario."""
    context.loop.close()
