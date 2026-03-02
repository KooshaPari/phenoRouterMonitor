"""When step definitions for AgilePlus BDD tests.

Traceability: WP16-T093
"""
from __future__ import annotations

from unittest.mock import AsyncMock

from behave import when  # type: ignore[import]


@when('I run "agileplus specify" with feature slug "{slug}"')
def run_specify(context, slug):  # noqa: ANN001
    feature_state = context.features.get(slug, {}).get("state", "created")
    allowed_states = ("created", "specified")

    if feature_state not in allowed_states:
        context.last_result = {
            "success": False,
            "error": f"InvalidState: cannot specify from state {feature_state!r}",
        }
        return

    context.client.run_command = AsyncMock(
        return_value={"success": True, "message": "Spec created", "outputs": {}}
    )
    result = context.loop.run_until_complete(
        context.client.run_command("specify", feature_slug=slug)
    )
    if result["success"]:
        # Update mock to reflect new state
        updated = dict(context.features.get(slug, {"slug": slug}))
        updated["state"] = "specified"
        context.features[slug] = updated
        context.client.get_feature = AsyncMock(return_value=updated)
    context.last_result = result


@when("I provide specification details via stdin")
def provide_spec_stdin(context):  # noqa: ANN001
    # Stdin interaction simulated in run_specify.
    pass


@when("I provide updated specification details")
def provide_updated_spec(context):  # noqa: ANN001
    pass


@when('I run "agileplus implement" for feature "{slug}"')
def run_implement(context, slug):  # noqa: ANN001
    feature = context.features.get(slug, {})
    if feature.get("state") != "planned":
        context.last_result = {
            "success": False,
            "error": f"InvalidState: expected planned, got {feature.get('state')!r}",
        }
        return

    context.client.run_command = AsyncMock(
        return_value={"success": True, "message": "Agent dispatched", "outputs": {}}
    )
    result = context.loop.run_until_complete(
        context.client.run_command("implement", feature_slug=slug)
    )
    if result["success"]:
        updated = dict(feature)
        updated["state"] = "implementing"
        context.features[slug] = updated
        context.client.get_feature = AsyncMock(return_value=updated)
    context.last_result = result


@when("I run \"agileplus implement\" for the feature")
def run_implement_for_the_feature(context):  # noqa: ANN001
    slug = next(iter(context.features), "parallel-feature")
    run_implement(context, slug)


@when("the agent completes WP01 implementation")
def agent_completes_wp01(context):  # noqa: ANN001
    # Simulate PR creation by updating mock
    context.client.list_work_packages = AsyncMock(
        return_value=[
            {
                "id": 1,
                "title": "WP01",
                "state": "review",
                "sequence": 1,
                "agent_id": "mock-agent",
                "pr_url": "https://github.com/example/repo/pull/1",
                "pr_state": "open",
                "depends_on": [],
                "file_scope": [],
            }
        ]
    )
    context.last_result = {"success": True, "message": "PR created"}


@when('I run "agileplus plan" for feature "{slug}"')
def run_plan(context, slug):  # noqa: ANN001
    feature = context.features.get(slug, {})
    if feature.get("state") != "researched":
        context.last_result = {
            "success": False,
            "error": f"InvalidState: expected researched, got {feature.get('state')!r}",
        }
        return

    context.client.run_command = AsyncMock(
        return_value={"success": True, "message": "Plan generated", "outputs": {}}
    )
    result = context.loop.run_until_complete(
        context.client.run_command("plan", feature_slug=slug)
    )
    if result["success"]:
        updated = dict(feature)
        updated["state"] = "planned"
        context.features[slug] = updated
        context.client.get_feature = AsyncMock(return_value=updated)
        # Add a default governance contract
        context.governance_contracts[slug] = {
            "version": 1,
            "rules": [
                {
                    "transition": "implementing -> validated",
                    "fr_id": "FR-001",
                    "evidence_type": "test_result",
                }
            ],
        }
    context.last_result = result


@when('I run "agileplus validate" for feature "{slug}"')
def run_validate(context, slug):  # noqa: ANN001
    feature = context.features.get(slug, {})
    if feature.get("state") != "implementing":
        context.last_result = {
            "success": False,
            "error": f"InvalidState: expected implementing, got {feature.get('state')!r}",
        }
        return

    # Check governance requirements against collected evidence
    contract = context.governance_contracts.get(slug, {})
    rules = contract.get("rules", [])
    missing = []
    for rule in rules:
        fr_id = rule["fr_id"]
        ev_type = rule["evidence_type"]
        found = any(
            e.get("fr_id") == fr_id and e.get("evidence_type") == ev_type
            for e in context.evidence
        )
        if not found:
            missing.append(fr_id)

    passed = len(missing) == 0
    context.validation_report = {"passed": passed, "missing_evidence": missing}

    if passed:
        updated = dict(feature)
        updated["state"] = "validated"
        context.features[slug] = updated
        context.client.get_feature = AsyncMock(return_value=updated)
        context.client.run_command = AsyncMock(
            return_value={"success": True, "message": "Validated", "outputs": {}}
        )
        result = context.loop.run_until_complete(
            context.client.run_command("validate", feature_slug=slug)
        )
    else:
        context.client.run_command = AsyncMock(
            return_value={
                "success": False,
                "message": f"Missing evidence for: {', '.join(missing)}",
                "outputs": {},
            }
        )
        result = context.loop.run_until_complete(
            context.client.run_command("validate", feature_slug=slug)
        )
    context.last_result = result


@when('I verify the audit chain for "{slug}"')
def verify_audit_chain(context, slug):  # noqa: ANN001
    result = context.loop.run_until_complete(
        context.client.verify_audit_chain(slug)
    )
    context.last_result = result
