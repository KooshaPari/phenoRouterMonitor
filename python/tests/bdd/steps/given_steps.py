"""Given step definitions for AgilePlus BDD tests.

Traceability: WP16-T093
"""
from __future__ import annotations

from typing import Any
from unittest.mock import AsyncMock

from behave import given  # type: ignore[import]


def _make_feature(slug: str, state: str) -> dict[str, Any]:
    return {
        "id": len(slug),  # deterministic synthetic id
        "slug": slug,
        "friendly_name": slug.replace("-", " ").title(),
        "state": state,
        "target_branch": "main",
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-15T00:00:00Z",
        "wp_count": 0,
        "wp_done": 0,
    }


@given("a fresh AgilePlus project with no features")
def fresh_project(context):  # noqa: ANN001
    context.client.list_features = AsyncMock(return_value=[])
    context.features = {}


@given('a feature "{slug}" in state "{state}"')
def feature_in_state(context, slug, state):  # noqa: ANN001
    feature = _make_feature(slug, state)
    context.features[slug] = feature
    context.client.get_feature = AsyncMock(return_value=feature)
    context.client.get_feature_state = AsyncMock(
        return_value={"state": state, "next_command": "", "blockers": []}
    )


@given('a feature "{slug}" in state "{state}" with {count:d} work packages')
def feature_in_state_with_wps(context, slug, state, count):  # noqa: ANN001
    feature = _make_feature(slug, state)
    feature["wp_count"] = count
    context.features[slug] = feature
    context.client.get_feature = AsyncMock(return_value=feature)
    wps = [
        {
            "id": i,
            "title": f"WP{i:02d}",
            "state": "planned",
            "sequence": i,
            "agent_id": "",
            "pr_url": "",
            "pr_state": "",
            "depends_on": [],
            "file_scope": [],
        }
        for i in range(1, count + 1)
    ]
    context.client.list_work_packages = AsyncMock(return_value=wps)


@given('a feature "{slug}" with WP01 in state "{wp_state}"')
def feature_with_wp_in_state(context, slug, wp_state):  # noqa: ANN001
    feature = _make_feature(slug, "implementing")
    context.features[slug] = feature
    context.client.get_feature = AsyncMock(return_value=feature)
    wp = {
        "id": 1,
        "title": "WP01",
        "state": wp_state,
        "sequence": 1,
        "agent_id": "mock-agent",
        "pr_url": "",
        "pr_state": "",
        "depends_on": [],
        "file_scope": ["src/main.rs"],
    }
    context.client.list_work_packages = AsyncMock(return_value=[wp])


@given("the agent has committed code in the WP01 worktree")
def agent_committed_code(context):  # noqa: ANN001
    # Declarative pre-condition only; mock already in place.
    pass


@given('a feature with WP01 file_scope "{scope1}" and WP02 file_scope "{scope2}"')
def feature_with_two_wps(context, scope1, scope2):  # noqa: ANN001
    slug = "parallel-feature"
    feature = _make_feature(slug, "planned")
    feature["wp_count"] = 2
    context.features[slug] = feature
    context.client.get_feature = AsyncMock(return_value=feature)
    wps = [
        {
            "id": 1,
            "title": "WP01",
            "state": "planned",
            "sequence": 1,
            "agent_id": "",
            "pr_url": "",
            "pr_state": "",
            "depends_on": [],
            "file_scope": [scope1],
        },
        {
            "id": 2,
            "title": "WP02",
            "state": "planned",
            "sequence": 2,
            "agent_id": "",
            "pr_url": "",
            "pr_state": "",
            "depends_on": [],
            "file_scope": [scope2],
        },
    ]
    context.client.list_work_packages = AsyncMock(return_value=wps)


@given('a feature "{slug}" with {count:d} audit entries')
def feature_with_audit_entries(context, slug, count):  # noqa: ANN001
    feature = _make_feature(slug, "specified")
    context.features[slug] = feature
    context.client.get_feature = AsyncMock(return_value=feature)

    if count == 0:
        context.client.get_audit_trail = AsyncMock(return_value=[])
        context.client.verify_audit_chain = AsyncMock(
            return_value={
                "valid": False,
                "entries_verified": 0,
                "first_invalid_id": 0,
                "error_message": "chain is empty",
            }
        )
    else:
        entries = [
            {
                "id": i,
                "feature_slug": slug,
                "wp_sequence": 0,
                "timestamp": f"2026-01-0{i}T00:00:00Z",
                "actor": "user",
                "transition": f"transition-{i}",
                "evidence_refs": [],
                "prev_hash": "0" * 64,
                "hash": "a" * 64,
            }
            for i in range(1, count + 1)
        ]
        context.audit_entries = entries
        context.client.get_audit_trail = AsyncMock(return_value=entries)
        context.client.verify_audit_chain = AsyncMock(
            return_value={
                "valid": True,
                "entries_verified": count,
                "first_invalid_id": 0,
                "error_message": "",
            }
        )


@given("audit entry {index:d} has been tampered with")
def audit_entry_tampered(context, index):  # noqa: ANN001
    # Simulate tampered chain: verification will fail at given index
    context.client.verify_audit_chain = AsyncMock(
        return_value={
            "valid": False,
            "entries_verified": index - 1,
            "first_invalid_id": index,
            "error_message": f"hash mismatch at entry {index - 1}",
        }
    )


@given("the governance contract requires test_result evidence for {fr_id}")
def governance_requires_test_result(context, fr_id):  # noqa: ANN001
    _add_governance_requirement(context, fr_id, "test_result")


@given("the governance contract requires review_approval evidence for {fr_id}")
def governance_requires_review_approval(context, fr_id):  # noqa: ANN001
    _add_governance_requirement(context, fr_id, "review_approval")


def _add_governance_requirement(context, fr_id: str, ev_type: str) -> None:
    feature_slug = next(
        (s for s, f in context.features.items() if f["state"] == "implementing"),
        None,
    )
    if feature_slug not in context.governance_contracts:
        context.governance_contracts[feature_slug] = {
            "version": 1,
            "rules": [],
        }
    context.governance_contracts[feature_slug]["rules"].append(
        {
            "transition": "implementing -> validated",
            "fr_id": fr_id,
            "evidence_type": ev_type,
        }
    )


@given('evidence exists for {fr_id} with type "{ev_type}"')
def evidence_exists(context, fr_id, ev_type):  # noqa: ANN001
    context.evidence.append({"fr_id": fr_id, "evidence_type": ev_type})


@given("no evidence exists for {fr_id}")
def no_evidence_exists(context, fr_id):  # noqa: ANN001
    # Ensure no evidence for this FR
    context.evidence = [e for e in context.evidence if e.get("fr_id") != fr_id]
