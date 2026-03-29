"""Linear GraphQL adapter for WL-160 workstream autosync."""

from __future__ import annotations

import logging
import re
from dataclasses import dataclass
from datetime import datetime, UTC
from pathlib import Path
from typing import Any

import httpx

from thegent.integrations.connector_mapping_cache import ConnectorMappingCache

logger = logging.getLogger(__name__)

LINEAR_GRAPHQL_URL = "https://api.linear.app/graphql"
WORKSTREAM_ID_PATTERN = re.compile(r"\[(WL-\d+)\]")
REQUIRED_LINEAR_STATE_TYPES = {"unstarted", "started", "completed"}

LOCAL_STATUS_TO_LINEAR_STATE_TYPE = {
    "BACKLOG": "unstarted",
    "TODO": "unstarted",
    "OPEN": "unstarted",
    "IN PROGRESS": "started",
    "CLAIMED": "started",
    "REVIEW": "started",
    "COMPLETED": "completed",
    "DONE": "completed",
    "CLOSED": "completed",
}


class LinearGraphQLError(Exception):
    """Base Linear GraphQL integration error."""


class LinearGraphQLAuthError(LinearGraphQLError):
    """Authentication/authorization error for Linear."""


@dataclass(frozen=True)
class LinearGraphQLConfig:
    """Connection config for Linear GraphQL operations."""

    api_key: str
    team_key: str
    timeout_seconds: float = 30.0
    mapping_cache_path: Path | None = None


def _mapping_cache(config: LinearGraphQLConfig) -> ConnectorMappingCache:
    return ConnectorMappingCache(cache_file=config.mapping_cache_path)


def _linear_state_cache_key(state_type: str) -> str:
    return f"state:{state_type.strip().lower()}"


def _resolve_linear_state_mapping(
    config: LinearGraphQLConfig,
    states_nodes: list[dict[str, Any]],
) -> dict[str, str]:
    cache = _mapping_cache(config)
    states_by_type = build_linear_state_mapping(states_nodes)
    cached_mappings: dict[str, str] = {}

    for state_type in sorted(REQUIRED_LINEAR_STATE_TYPES):
        state_key = _linear_state_cache_key(state_type)
        cached_state_id = cache.get("linear", state_key)
        current_state_id = states_by_type[state_type]
        if cached_state_id is not None and cached_state_id != current_state_id:
            raise LinearGraphQLError(
                f"Linear schema drift: state '{state_type}' id changed from {cached_state_id} to {current_state_id}"
            )
        cached_mappings[state_key] = current_state_id

    cache.bootstrap("linear", cached_mappings)
    return states_by_type


def _graphql_request(
    config: LinearGraphQLConfig,
    query: str,
    variables: dict[str, Any],
) -> dict[str, Any]:
    headers = {
        "Authorization": config.api_key,
        "Content-Type": "application/json",
    }
    payload = {"query": query, "variables": variables}
    try:
        response = httpx.post(
            LINEAR_GRAPHQL_URL,
            headers=headers,
            json=payload,
            timeout=config.timeout_seconds,
        )
    except httpx.HTTPError as exc:
        raise LinearGraphQLError(f"Linear request failed: {exc}") from exc

    if response.status_code in {401, 403}:
        raise LinearGraphQLAuthError(f"Linear auth failed ({response.status_code})")
    if response.status_code >= 400:
        raise LinearGraphQLError(f"Linear request failed ({response.status_code}): {response.text[:200]}")

    body = response.json()
    errors = body.get("errors") or []
    if errors:
        message = "; ".join(str(error.get("message", "unknown error")) for error in errors if isinstance(error, dict))
        lower_message = message.lower()
        if "auth" in lower_message or "token" in lower_message or "permission" in lower_message:
            raise LinearGraphQLAuthError(message)
        raise LinearGraphQLError(message or "Linear GraphQL returned errors")
    data = body.get("data")
    if not isinstance(data, dict):
        raise LinearGraphQLError("Linear GraphQL response missing data payload")
    return data


def _status_to_linear_type(status: str) -> str:
    normalized = status.strip().upper()
    mapped = LOCAL_STATUS_TO_LINEAR_STATE_TYPE.get(normalized)
    if mapped is None:
        raise LinearGraphQLError(f"Unsupported local status for Linear mapping: {status}")
    return mapped


def _status_from_linear(state: dict[str, Any]) -> str:
    state_type = str(state.get("type") or "").lower()
    state_name = str(state.get("name") or "").lower()
    if state_type in {"completed", "canceled"} or state_name in {"done", "complete", "completed", "closed"}:
        return "COMPLETED"
    if state_type == "started" or state_name in {"in progress", "review"}:
        return "IN PROGRESS"
    return "BACKLOG"


def _extract_workstream_id(title: str) -> str | None:
    match = WORKSTREAM_ID_PATTERN.search(title)
    if match is None:
        return None
    return match.group(1)


def _build_issue_title(item: dict[str, Any]) -> str:
    item_id = str(item.get("item_id") or item.get("id") or "").strip()
    title = str(item.get("title") or "").strip()
    if item_id and title:
        return f"[{item_id}] {title}"
    return title or item_id


def _build_issue_description(item: dict[str, Any]) -> str:
    lines: list[str] = []
    for field in ("status", "priority", "area", "blocked_by"):
        value = item.get(field)
        if value in (None, ""):
            continue
        lines.append(f"{field}: {value}")
    return "\n".join(lines)


def build_linear_state_mapping(states_nodes: list[dict[str, Any]]) -> dict[str, str]:
    """Build explicit Linear state ID mapping table with fail-fast validation."""
    states_by_type: dict[str, str] = {}
    for state in states_nodes:
        state_type = str(state.get("type") or "").strip().lower()
        state_id = str(state.get("id") or "").strip()
        if state_type in REQUIRED_LINEAR_STATE_TYPES and state_id and state_type not in states_by_type:
            states_by_type[state_type] = state_id

    missing = sorted(REQUIRED_LINEAR_STATE_TYPES - set(states_by_type.keys()))
    if missing:
        raise LinearGraphQLError("Linear workflow is missing required state type mappings: " + ", ".join(missing))
    return states_by_type


def _load_team_bundle(config: LinearGraphQLConfig, issue_limit: int = 250) -> dict[str, Any]:
    query = """
    query TeamBundle($teamKey: String!, $issueLimit: Int!) {
      teams(filter: { key: { eq: $teamKey } }, first: 1) {
        nodes {
          id
          key
          states {
            nodes {
              id
              name
              type
            }
          }
          issues(first: $issueLimit) {
            nodes {
              id
              identifier
              title
              priority
              state {
                id
                name
                type
              }
            }
          }
        }
      }
    }
    """
    data = _graphql_request(
        config,
        query,
        {"teamKey": config.team_key, "issueLimit": issue_limit},
    )
    teams_data = data.get("teams", {})
    nodes = teams_data.get("nodes", []) if isinstance(teams_data, dict) else []
    if not nodes:
        raise LinearGraphQLError(f"Linear team not found for key: {config.team_key}")
    team = nodes[0]
    return team if isinstance(team, dict) else {}


def sync_to_linear(config: LinearGraphQLConfig, workstream_data: list[dict[str, Any]]) -> dict[str, Any]:
    """Upsert workstream items into Linear issues."""
    team = _load_team_bundle(config)
    team_id = str(team.get("id") or "")
    if not team_id:
        raise LinearGraphQLError("Linear team id missing from API response")

    states_nodes = team.get("states", {}).get("nodes", []) if isinstance(team.get("states"), dict) else []
    states_by_type = _resolve_linear_state_mapping(config, states_nodes)

    issues_nodes = team.get("issues", {}).get("nodes", []) if isinstance(team.get("issues"), dict) else []
    issues_by_workstream_id: dict[str, dict[str, Any]] = {}
    for issue in issues_nodes:
        if not isinstance(issue, dict):
            continue
        workstream_id = _extract_workstream_id(str(issue.get("title") or ""))
        if workstream_id:
            issues_by_workstream_id[workstream_id] = issue

    items_created = 0
    items_updated = 0
    errors: list[str] = []

    for item in workstream_data:
        item_id = str(item.get("item_id") or item.get("id") or "").strip()
        if not item_id:
            errors.append("missing_item_id")
            continue
        target_state_type = _status_to_linear_type(str(item.get("status") or "BACKLOG"))
        target_state_id = states_by_type.get(target_state_type)
        title = _build_issue_title(item)
        description = _build_issue_description(item)

        try:
            existing_issue = issues_by_workstream_id.get(item_id)
            if existing_issue is None:
                mutation = """
                mutation CreateIssue($input: IssueCreateInput!) {
                  issueCreate(input: $input) {
                    success
                    issue { id identifier title }
                  }
                }
                """
                input_payload: dict[str, Any] = {
                    "teamId": team_id,
                    "title": title,
                    "description": description,
                }
                if target_state_id:
                    input_payload["stateId"] = target_state_id
                result = _graphql_request(config, mutation, {"input": input_payload})
                create_result = result.get("issueCreate", {})
                if not isinstance(create_result, dict) or create_result.get("success") is not True:
                    raise LinearGraphQLError(f"Linear issueCreate failed for {item_id}")
                created_issue = create_result.get("issue")
                if isinstance(created_issue, dict):
                    issues_by_workstream_id[item_id] = created_issue
                items_created += 1
                continue

            issue_uuid = str(existing_issue.get("id") or "")
            if not issue_uuid:
                raise LinearGraphQLError(f"Linear issue missing id for {item_id}")
            mutation = """
            mutation UpdateIssue($id: String!, $input: IssueUpdateInput!) {
              issueUpdate(id: $id, input: $input) {
                success
                issue { id identifier title }
              }
            }
            """
            update_payload: dict[str, Any] = {"title": title, "description": description}
            if target_state_id:
                update_payload["stateId"] = target_state_id
            result = _graphql_request(
                config,
                mutation,
                {"id": issue_uuid, "input": update_payload},
            )
            update_result = result.get("issueUpdate", {})
            if not isinstance(update_result, dict) or update_result.get("success") is not True:
                raise LinearGraphQLError(f"Linear issueUpdate failed for {item_id}")
            items_updated += 1
        except LinearGraphQLError as exc:
            errors.append(f"{item_id}:{exc}")

    return {
        "items_created": items_created,
        "items_updated": items_updated,
        "items_synced": items_created + items_updated,
        "errors": errors,
        "synced_at": datetime.now(UTC).isoformat(),
    }


def sync_from_linear(config: LinearGraphQLConfig) -> dict[str, Any]:
    """Read workstream-related issue status from Linear."""
    team = _load_team_bundle(config)
    issues_nodes = team.get("issues", {}).get("nodes", []) if isinstance(team.get("issues"), dict) else []
    results: list[dict[str, Any]] = []
    for issue in issues_nodes:
        if not isinstance(issue, dict):
            continue
        title = str(issue.get("title") or "")
        item_id = _extract_workstream_id(title)
        if item_id is None:
            continue
        state = issue.get("state")
        if not isinstance(state, dict):
            state = {}
        results.append(
            {
                "id": issue.get("id"),
                "identifier": issue.get("identifier"),
                "item_id": item_id,
                "title": title,
                "status": _status_from_linear(state),
                "raw": issue,
            }
        )
    return {
        "items_imported": len(results),
        "items": results,
        "errors": [],
        "synced_at": datetime.now(UTC).isoformat(),
    }
