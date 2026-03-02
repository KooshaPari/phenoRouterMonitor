"""AgilePlus MCP server — FastMCP entry point.

Registers all MCP tools, implements the Roots and Elicitation primitives,
and connects to the Rust gRPC backend.

Traceability: FR-010, FR-049 / WP14-T082, T084b, T084c, T084d

Usage::

    uv run python -m agileplus_mcp
    # or after installation:
    agileplus-mcp
"""

from __future__ import annotations

import logging
import os
from typing import Any

from fastmcp import FastMCP

from agileplus_mcp.grpc_client import AgilePlusCoreClient, GrpcConnectionError
from agileplus_mcp.sampling import SamplingHandler
from agileplus_mcp.tools import features as features_module
from agileplus_mcp.tools import governance as governance_module
from agileplus_mcp.tools import status as status_module

logger = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Global state: one client and one FastMCP app shared across the process.
# ---------------------------------------------------------------------------

GRPC_ADDRESS = os.environ.get("AGILEPLUS_GRPC_ADDRESS", "localhost:50051")

mcp: FastMCP = FastMCP("AgilePlus")
_client: AgilePlusCoreClient | None = None
_sampling: SamplingHandler | None = None


def _get_client() -> AgilePlusCoreClient:
    if _client is None:
        raise RuntimeError("gRPC client not initialised — call startup() first")
    return _client


# ---------------------------------------------------------------------------
# T084c: MCP Roots primitive — declare workspace boundaries.
# ---------------------------------------------------------------------------

@mcp.resource("roots://workspace")
async def get_workspace_roots() -> dict[str, Any]:
    """Declare workspace roots for the MCP client.

    Returns a list of filesystem roots the server works within, allowing
    the MCP client to scope file operations correctly.

    Roots update dynamically as features are created.
    """
    client = _get_client()
    features = await client.list_features()

    roots = [
        {"uri": "file:///", "name": "project-root"},
        {"uri": "file://.agileplus/", "name": "agileplus-data"},
    ]

    for feature in features:
        slug = feature["slug"]
        roots.append(
            {
                "uri": f"file://kitty-specs/{slug}/",
                "name": f"feature-spec-{slug}",
            }
        )
        roots.append(
            {
                "uri": f"file://.worktrees/{slug}/",
                "name": f"feature-worktree-{slug}",
            }
        )

    return {"roots": roots}


# ---------------------------------------------------------------------------
# T084d: MCP Elicitation primitive — structured discovery interviews.
# ---------------------------------------------------------------------------

@mcp.tool(name="agileplus_elicit_feature")
async def elicit_feature(
    feature_name: str,
    target_branch: str = "main",
) -> dict[str, Any]:
    """Begin an elicitation interview to specify a new feature.

    Sends structured questions to the MCP client and gathers answers to
    build a complete feature specification.

    Args:
        feature_name: Human-readable feature name (used to derive the slug).
        target_branch: Target branch for the eventual merge.

    Returns:
        dict with ``questions`` for the caller to answer, plus a ``session_id``
        to pass back with answers.
    """
    import hashlib
    import time

    session_id = hashlib.sha1(
        f"{feature_name}{time.time()}".encode()
    ).hexdigest()[:8]

    return {
        "session_id": session_id,
        "feature_name": feature_name,
        "target_branch": target_branch,
        "questions": [
            {
                "id": "problem_statement",
                "question": "What problem does this feature solve?",
                "type": "text",
                "required": True,
            },
            {
                "id": "acceptance_criteria",
                "question": "What are the acceptance criteria? (one per line)",
                "type": "multiline",
                "required": True,
            },
            {
                "id": "scope",
                "question": "Which files or modules are in scope? (comma-separated paths)",
                "type": "text",
                "required": False,
            },
            {
                "id": "out_of_scope",
                "question": "What is explicitly out of scope?",
                "type": "text",
                "required": False,
            },
            {
                "id": "risks",
                "question": "What are the main risks or open questions?",
                "type": "text",
                "required": False,
            },
        ],
    }


@mcp.tool(name="agileplus_elicit_clarify")
async def elicit_clarify(feature_slug: str) -> dict[str, Any]:
    """Generate clarifying questions for an existing feature spec.

    Analyses the current spec and returns targeted questions to resolve
    ambiguities before planning.

    Args:
        feature_slug: Kebab-case feature identifier.

    Returns:
        dict with ``questions`` and current ``feature`` snapshot.
    """
    client = _get_client()
    feature = await client.get_feature(feature_slug)
    state = await client.get_feature_state(feature_slug)

    return {
        "feature": feature,
        "current_state": state["state"],
        "questions": [
            {
                "id": "blockers",
                "question": f"Are there any blockers preventing moving from {state['state']} to {state.get('next_command', 'next state')}?",
                "type": "text",
                "required": False,
            },
            {
                "id": "dependencies",
                "question": "Does this feature depend on any other features or external systems?",
                "type": "text",
                "required": False,
            },
            {
                "id": "timeline",
                "question": "Is there a target completion date?",
                "type": "text",
                "required": False,
            },
        ],
    }


# ---------------------------------------------------------------------------
# T084b: Sampling tool — server-initiated analysis
# ---------------------------------------------------------------------------

@mcp.tool(name="agileplus_sample_triage")
async def sample_triage(feature_slug: str, agent_output: str) -> dict[str, Any]:
    """Server-initiated triage of agent output.

    Classifies errors/warnings in agent output and suggests remediation.

    Args:
        feature_slug: Feature the agent was working on.
        agent_output: Raw output from the agent run.

    Returns:
        Triage result with ``severity``, ``category``, and ``remediation``.
    """
    sampling = _sampling
    if sampling is None:
        raise RuntimeError("Sampling handler not initialised")
    return await sampling.auto_triage(feature_slug, agent_output)


@mcp.tool(name="agileplus_sample_governance_check")
async def sample_governance_check(
    feature_slug: str, planned_transition: str
) -> dict[str, Any]:
    """Server-initiated governance pre-check before a state transition.

    Args:
        feature_slug: Feature about to transition.
        planned_transition: Transition string (e.g. implementing->validated).

    Returns:
        dict with ``ready`` bool and ``blockers`` list.
    """
    sampling = _sampling
    if sampling is None:
        raise RuntimeError("Sampling handler not initialised")
    return await sampling.governance_pre_check(feature_slug, planned_transition)


@mcp.tool(name="agileplus_sample_retrospective")
async def sample_retrospective(feature_slug: str) -> dict[str, Any]:
    """Server-initiated retrospective analysis of a shipped feature.

    Args:
        feature_slug: Shipped feature to analyse.

    Returns:
        Retrospective summary with highlights, issues, and metrics.
    """
    sampling = _sampling
    if sampling is None:
        raise RuntimeError("Sampling handler not initialised")
    return await sampling.generate_retrospective(feature_slug)


# ---------------------------------------------------------------------------
# Startup / shutdown lifecycle
# ---------------------------------------------------------------------------

async def startup(grpc_address: str = GRPC_ADDRESS) -> None:
    """Initialise the gRPC client and register all tools."""
    global _client, _sampling  # noqa: PLW0603

    client = AgilePlusCoreClient(grpc_address)
    try:
        await client.connect()
    except GrpcConnectionError as exc:
        logger.warning(
            "Could not connect to gRPC server at %s: %s — tools will fail until server is up",
            grpc_address,
            exc,
        )

    _client = client
    _sampling = SamplingHandler(client)

    # Register domain tool modules
    features_module.register_tools(mcp, client)
    governance_module.register_tools(mcp, client)
    status_module.register_tools(mcp, client)

    logger.info("AgilePlus MCP server ready (gRPC: %s)", grpc_address)


async def shutdown() -> None:
    """Close the gRPC connection."""
    global _client  # noqa: PLW0603
    if _client is not None:
        await _client.close()
        _client = None


def main() -> None:
    """Entry point for `agileplus-mcp` command."""
    import asyncio

    logging.basicConfig(level=logging.INFO)

    async def _run() -> None:
        await startup()
        await mcp.run_async()
        await shutdown()

    asyncio.run(_run())


if __name__ == "__main__":
    main()
