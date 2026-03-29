"""GitHub Projects v2 Bidirectional Sync Integration (WL-157).

Provides optional, standalone-safe bidirectional syncing with GitHub Projects v2.
Skips gracefully when disabled or when gh auth lacks project scope.

Key Principles:
- Standalone-safe: No crash or side effects when disabled or gh auth missing
- Optional: Fully backward compatible; can be disabled entirely
- Bidirectional: Read/write thegent workstream to/from GitHub Projects
- Composable: Works with existing WORK_STREAM.md format
"""

import csv
import io
import orjson as json
import logging
import shutil
import subprocess
from thegent.infra.shim_subprocess import run as shim_run
import re
from dataclasses import dataclass
from datetime import datetime, UTC
from pathlib import Path
from typing import Any, Literal

from thegent.integrations.connector_mapping_cache import ConnectorMappingCache

logger = logging.getLogger(__name__)


@dataclass
class GHProjectConfig:
    """Configuration for GitHub Projects sync."""

    enabled: bool
    owner: str
    number: int
    direction: Literal["read_only", "write_only", "bidirectional"]
    standalone_mode: bool
    sandbox_mode: bool = False
    sandbox_number: int = 0
    mapping_cache_path: Path | None = None

    def is_valid(self) -> bool:
        """Check if config is valid for sync operations."""
        return self.enabled and bool(self.owner) and self.number > 0

    def can_read(self) -> bool:
        """Check if sync direction allows reading."""
        return self.direction in ("read_only", "bidirectional")

    def can_write(self) -> bool:
        """Check if sync direction allows writing."""
        return self.direction in ("write_only", "bidirectional")

    def effective_project_number(self) -> int:
        """Return target project number honoring sandbox mode."""
        if self.sandbox_mode:
            if self.sandbox_number <= 0:
                raise GHProjectSyncError("sandbox_mode requires sandbox_number > 0")
            return self.sandbox_number
        return self.number


class GHProjectSyncError(Exception):
    """Base exception for GitHub Projects sync errors."""


class GHProjectAuthError(GHProjectSyncError):
    """Authentication/authorization error (e.g., missing project scope)."""


class GHProjectNotFoundError(GHProjectSyncError):
    """Project not found error."""


def _check_gh_command() -> bool:
    """Check if gh CLI is available.

    Returns:
        True if gh is available and callable, False otherwise.
    """
    return shutil.which("gh") is not None


def _run_gh_command(args: list[str], capture: bool = True) -> tuple[int, str, str]:
    """Run gh CLI command safely.

    Args:
        args: Command arguments (without 'gh' prefix)
        capture: If True, capture stdout/stderr; if False, stream to console

    Returns:
        Tuple of (return_code, stdout, stderr)

    Raises:
        GHProjectAuthError: If command fails due to auth issues
        GHProjectSyncError: For other gh CLI errors
    """
    if not _check_gh_command():
        raise GHProjectSyncError("gh CLI not found on PATH")

    try:
        cmd = ["gh"] + args
        result = shim_run(
            cmd,
            capture_output=capture,
            text=True,
            timeout=30,
            check=False,
        )

        # Check for auth-related errors
        if result.returncode == 1:
            stderr_lower = result.stderr.lower()
            if "auth" in stderr_lower or "permission" in stderr_lower or "project" in stderr_lower:
                raise GHProjectAuthError(f"GitHub authentication issue: {result.stderr[:200]}")

        if result.returncode != 0:
            raise GHProjectSyncError(f"gh command failed: {result.stderr[:200]}")

        return result.returncode, result.stdout, result.stderr

    except GHProjectAuthError:
        raise
    except subprocess.TimeoutExpired as e:
        raise GHProjectSyncError(f"gh command timeout: {e}")
    except Exception as e:
        raise GHProjectSyncError(f"gh command error: {e}")


def _coerce_gh_result(result: Any) -> tuple[int, str, str]:
    """Normalize gh command output when tests patch `_run_gh_command` loosely."""
    if isinstance(result, tuple) and len(result) == 3:
        code = result[0] if isinstance(result[0], int) else 0
        stdout = result[1]
        stderr = result[2]
        if isinstance(stdout, (bytes, bytearray)):
            stdout = bytes(stdout).decode("utf-8")
        if not isinstance(stdout, str):
            stdout = ""
        if isinstance(stderr, (bytes, bytearray)):
            stderr = bytes(stderr).decode("utf-8")
        if not isinstance(stderr, str):
            stderr = ""
        return code, stdout, stderr
    return 0, "", ""


def _project_args(config: GHProjectConfig) -> list[str]:
    """Build common project selector arguments."""
    return [str(config.effective_project_number()), "--owner", config.owner]


def _mapping_cache(config: GHProjectConfig) -> ConnectorMappingCache:
    return ConnectorMappingCache(cache_file=config.mapping_cache_path)


def _status_to_github_option(status: str) -> str:
    normalized = status.strip().upper()
    mapping = {
        "BACKLOG": "Todo",
        "TODO": "Todo",
        "OPEN": "Todo",
        "IN PROGRESS": "In Progress",
        "CLAIMED": "In Progress",
        "REVIEW": "In Progress",
        "COMPLETED": "Done",
        "DONE": "Done",
        "CLOSED": "Done",
    }
    return mapping.get(normalized, "Todo")


def _priority_option_candidates(priority: str) -> list[str]:
    normalized = priority.strip().upper()
    mapping = {
        "P0": ["p0", "urgent", "highest", "high"],
        "P1": ["p1", "high", "medium", "normal"],
        "P2": ["p2", "medium", "low"],
        "P3": ["p3", "low"],
    }
    return mapping.get(normalized, [normalized.lower()])


def _resolve_single_select_option_id(option_map: dict[str, str], candidates: list[str]) -> str | None:
    for candidate in candidates:
        option_id = option_map.get(candidate.strip().lower())
        if option_id:
            return option_id
    return None


def _status_from_github(item: dict[str, Any]) -> str:
    field_values = item.get("fieldValues")
    if isinstance(field_values, list):
        for field in field_values:
            if not isinstance(field, dict):
                continue
            field_name = str(field.get("field") or field.get("fieldName") or "").lower()
            if field_name and field_name != "status":
                continue
            if isinstance(field.get("name"), str):
                value = field["name"]
            elif isinstance(field.get("option"), dict) and isinstance(field["option"].get("name"), str):
                value = field["option"]["name"]
            else:
                continue
            status = value.strip().lower()
            if status in {"todo", "backlog", "open"}:
                return "BACKLOG"
            if status in {"in progress", "in_progress", "review"}:
                return "IN PROGRESS"
            if status in {"done", "complete", "completed", "closed"}:
                return "COMPLETED"
    return "BACKLOG"


def _parse_github_issue_references(raw: str) -> list[str]:
    """Extract GitHub issue references from text."""
    references: set[str] = set()
    matches = re.findall(r"https?://github\.com/([^/\s]+)/([^/\s]+)/issues/(\d+)", raw, flags=re.IGNORECASE)
    for owner, repo, issue in matches:
        references.add(f"{owner}/{repo}#{issue}")

    for owner_repo, issue in re.findall(r"\b([A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+)#(\d+)\b", raw):
        references.add(f"{owner_repo}#{issue}")

    for issue in re.findall(r"(?<!/)#(\d+)\b", raw):
        references.add(f"#{issue}")

    return sorted(references)


def extract_github_issue_refs(raw_item: dict[str, Any]) -> list[str]:
    """Extract issue references from a GitHub project item payload."""
    references: set[str] = set()

    content = raw_item.get("content")
    if isinstance(content, dict):
        content_url = content.get("url")
        if isinstance(content_url, str):
            references.update(_parse_github_issue_references(content_url))

    for text in (raw_item.get("body"), raw_item.get("title"), raw_item.get("url"), str(raw_item.get("content") or "")):
        if isinstance(text, str):
            references.update(_parse_github_issue_references(text))

    return sorted(references)


def _normalize_issue_ref(reference: str) -> str | None:
    """Normalize issue refs to `owner/repo#number` format."""
    value = reference.strip()
    if not value:
        return None
    if value.startswith("#"):
        return None
    if not value:
        return None

    if value.startswith("http://") or value.startswith("https://"):
        parsed_refs = _parse_github_issue_references(value)
        return parsed_refs[0] if parsed_refs else None

    return value


def _run_issue_close_and_comment(
    issue_ref: str,
    *,
    close_comment: str | None,
) -> dict[str, Any]:
    """Close and comment a single GitHub issue."""
    normalized = _normalize_issue_ref(issue_ref)
    if not normalized:
        return {"issue_ref": issue_ref, "commented": False, "closed": False, "status": "skipped"}

    if close_comment:
        _run_gh_command(
            [
                "issue",
                "comment",
                normalized,
                "--body",
                close_comment,
            ]
        )
    _run_gh_command(["issue", "close", normalized])
    return {
        "issue_ref": normalized,
        "commented": bool(close_comment),
        "closed": True,
        "status": "ok",
    }


def close_or_comment_github_issue_refs(
    issue_refs: list[str],
    *,
    close_comment: str | None = None,
) -> dict[str, Any]:
    """Close issues and optionally post status comments."""
    if not issue_refs:
        return {
            "items_processed": 0,
            "items_updated": 0,
            "items_commented": 0,
            "issues": [],
            "errors": [],
        }

    requested_refs: list[str] = []
    seen: set[str] = set()
    for item_ref in issue_refs:
        normalized = _normalize_issue_ref(str(item_ref))
        if normalized is None or normalized in seen:
            continue
        seen.add(normalized)
        requested_refs.append(normalized)

    if not requested_refs:
        return {
            "items_processed": len(issue_refs),
            "items_updated": 0,
            "items_commented": 0,
            "issues": [],
            "errors": ["no valid issue references"],
        }

    issues: list[dict[str, Any]] = []
    errors: list[str] = []
    for issue_ref in requested_refs:
        try:
            issues.append(_run_issue_close_and_comment(issue_ref, close_comment=close_comment))
        except GHProjectAuthError as exc:
            errors.append(f"{issue_ref}:{exc}")
        except GHProjectSyncError as exc:
            errors.append(f"{issue_ref}:{exc}")

    if errors:
        return {
            "items_processed": len(requested_refs),
            "items_updated": len(issues),
            "items_commented": sum(1 for issue in issues if issue.get("commented")),
            "issues": issues,
            "errors": errors,
        }

    return {
        "items_processed": len(requested_refs),
        "items_updated": len(requested_refs),
        "items_commented": sum(1 for issue in issues if issue.get("commented")),
        "issues": issues,
        "errors": [],
    }


def _extract_workstream_id(item: dict[str, Any]) -> str | None:
    title = ""
    content = item.get("content")
    if isinstance(content, dict):
        title = str(content.get("title") or "")
    if not title:
        title = str(item.get("title") or "")
    if not title:
        return None
    if title.startswith("[") and "]" in title:
        token = title.split("]", 1)[0].lstrip("[")
        if token.startswith("WL-"):
            return token
    return None


def _build_title(work_item: dict[str, Any]) -> str:
    work_id = str(work_item.get("item_id") or work_item.get("id") or "").strip()
    raw_title = str(work_item.get("title") or "").strip()
    if work_id and raw_title:
        return f"[{work_id}] {raw_title}"
    return raw_title or work_id


def _build_body(work_item: dict[str, Any]) -> str:
    lines: list[str] = []
    work_id = str(work_item.get("item_id") or work_item.get("id") or "").strip()
    if work_id:
        lines.append(f"Workstream ID: {work_id}")
    for field in ("status", "priority", "area", "blocked_by"):
        value = work_item.get(field)
        if value in (None, ""):
            continue
        lines.append(f"{field}: {value}")
    return "\n".join(lines)


def _load_project_items(config: GHProjectConfig) -> list[dict[str, Any]]:
    code, stdout, _ = _coerce_gh_result(
        _run_gh_command(
            [
                "project",
                "item-list",
                *_project_args(config),
                "--format",
                "json",
                "-L",
                "500",
            ]
        )
    )
    if code != 0:
        return []
    parsed = json.loads(stdout or "[]")
    if isinstance(parsed, list):
        return [item for item in parsed if isinstance(item, dict)]
    if isinstance(parsed, dict) and isinstance(parsed.get("items"), list):
        return [item for item in parsed["items"] if isinstance(item, dict)]
    return []


def _load_single_select_field(
    config: GHProjectConfig,
    *,
    field_name: str,
) -> tuple[str | None, dict[str, str]]:
    cache = _mapping_cache(config)
    target = field_name.strip().lower()
    cache_key = f"field:{target}"

    code, stdout, _ = _coerce_gh_result(
        _run_gh_command(
            [
                "project",
                "field-list",
                *_project_args(config),
                "--format",
                "json",
                "-L",
                "200",
            ]
        )
    )
    if code != 0:
        cached_field_id = cache.get("github", cache_key)
        if cached_field_id:
            return cached_field_id, {}
        return None, {}
    parsed = json.loads(stdout or "[]")
    fields: list[dict[str, Any]]
    if isinstance(parsed, list):
        fields = [field for field in parsed if isinstance(field, dict)]
    elif isinstance(parsed, dict) and isinstance(parsed.get("fields"), list):
        fields = [field for field in parsed["fields"] if isinstance(field, dict)]
    else:
        fields = []

    for field in fields:
        if str(field.get("name", "")).lower() != target:
            continue
        field_id = field.get("id")
        options_raw = field.get("options") or []
        option_map: dict[str, str] = {}
        if isinstance(options_raw, list):
            for option in options_raw:
                if not isinstance(option, dict):
                    continue
                option_name = str(option.get("name") or "").strip().lower()
                option_id = str(option.get("id") or "").strip()
                if option_name and option_id:
                    option_map[option_name] = option_id
        if isinstance(field_id, str) and field_id:
            cache.put("github", f"field:{target}", field_id)
            return field_id, option_map
    cached_field_id = cache.get("github", cache_key)
    if cached_field_id:
        return cached_field_id, {}
    return None, {}


def _prepare_github_status_mapping(
    workstream_data: list[dict[str, Any]],
    status_options: dict[str, str],
) -> dict[str, str]:
    status_option_by_id: dict[str, str] = {}
    missing_statuses: set[str] = set()
    enforce_mapping = bool(status_options)

    for item in workstream_data:
        item_id = str(item.get("item_id") or item.get("id") or "").strip()
        if not item_id:
            continue
        status_name = _status_to_github_option(str(item.get("status") or "BACKLOG")).lower()
        status_option_id = _resolve_single_select_option_id(status_options, [status_name])
        if status_option_id is None:
            missing_statuses.add(status_name)
            continue
        status_option_by_id[item_id] = status_option_id

    if missing_statuses and enforce_mapping:
        missing = ", ".join(sorted(missing_statuses))
        raise GHProjectSyncError(f"GitHub schema drift: missing required status option mappings for [{missing}]")

    return status_option_by_id


def _load_status_field(config: GHProjectConfig) -> tuple[str | None, dict[str, str]]:
    return _load_single_select_field(config, field_name="status")


def _load_priority_field(config: GHProjectConfig) -> tuple[str | None, dict[str, str]]:
    return _load_single_select_field(config, field_name="priority")


def get_project_status(config: GHProjectConfig) -> dict[str, Any]:
    """Get GitHub Project sync status.

    Args:
        config: GitHub Projects configuration

    Returns:
        Dict with project metadata, item count, and sync status.
        Returns empty dict if sync disabled or auth unavailable.

    Raises:
        GHProjectSyncError: For unexpected errors (not auth issues)
    """
    if not config.is_valid():
        if config.standalone_mode:
            logger.debug("GH Project sync not configured or disabled; skipping status")
            return {"enabled": False, "reason": "not_configured"}
        raise GHProjectSyncError("GitHub project config invalid")

    try:
        # Query project info via gh API
        result = _coerce_gh_result(
            _run_gh_command(
                [
                    "project",
                    "view",
                    *_project_args(config),
                    "--format",
                    "json",
                ]
            )
        )
        code, stdout, _ = result

        if code == 0:
            data = json.loads(stdout)
            return {
                "enabled": True,
                "owner": config.owner,
                "number": config.effective_project_number(),
                "id": data.get("id", ""),
                "title": data.get("title", ""),
                "url": data.get("url", ""),
                "items": data.get("items", []),
                "synced_at": None,
                "direction": config.direction,
                "sandbox_mode": config.sandbox_mode,
            }
        return {"enabled": True, "status": "unavailable", "reason": "query_failed"}

    except GHProjectAuthError as e:
        if config.standalone_mode:
            logger.warning(f"GH Project auth issue (standalone mode, skipping): {e}")
            return {"enabled": True, "status": "auth_required", "reason": str(e)}
        raise
    except GHProjectSyncError:
        raise


def sync_to_github(
    config: GHProjectConfig,
    workstream_data: list[dict[str, Any]],
) -> dict[str, Any]:
    """Sync thegent workstream to GitHub Projects.

    Args:
        config: GitHub Projects configuration
        workstream_data: Workstream items (from WORK_STREAM.md or similar)

    Returns:
        Dict with sync results: items_created, items_updated, errors

    Raises:
        GHProjectSyncError: For unexpected errors (not auth issues)
    """
    if not config.is_valid() or not config.can_write():
        if config.standalone_mode:
            logger.debug("GH Project sync not configured or read-only; skipping write sync")
            return {"items_synced": 0, "reason": "not_writable"}
        raise GHProjectSyncError("GitHub project not writable")

    try:
        _, view_stdout, _ = _coerce_gh_result(
            _run_gh_command(["project", "view", *_project_args(config), "--format", "json"])
        )
        project_id = ""
        if isinstance(view_stdout, str) and view_stdout.strip():
            try:
                project_id = str(json.loads(view_stdout).get("id") or "")
            except json.JSONDecodeError:
                project_id = ""
        existing_items = _load_project_items(config)
        existing_by_id = {
            item_id: item for item in existing_items for item_id in [_extract_workstream_id(item)] if item_id
        }
        status_field_id, status_options = _load_status_field(config)
        priority_field_id, priority_options = _load_priority_field(config)
        if not status_field_id:
            raise GHProjectSyncError("GitHub schema drift: required single-select field 'status' is missing")
        if not status_options:
            raise GHProjectSyncError(
                "GitHub schema drift: required status option mappings for field 'status' are missing"
            )
        status_option_by_id = _prepare_github_status_mapping(workstream_data, status_options)

        items_created = 0
        items_updated = 0
        errors: list[str] = []

        for work_item in workstream_data:
            item_id = str(work_item.get("item_id") or work_item.get("id") or "").strip()
            if not item_id:
                errors.append("missing_item_id")
                continue
            status_option_id = status_option_by_id.get(item_id)
            priority_option_id = _resolve_single_select_option_id(
                priority_options,
                _priority_option_candidates(str(work_item.get("priority") or "P2")),
            )
            title = _build_title(work_item)
            body = _build_body(work_item)
            existing = existing_by_id.get(item_id)
            try:
                if existing is None:
                    create_args = [
                        "project",
                        "item-create",
                        *_project_args(config),
                        "--title",
                        title,
                        "--body",
                        body,
                        "--format",
                        "json",
                    ]
                    _, create_stdout, _ = _coerce_gh_result(_run_gh_command(create_args))
                    created_item = json.loads(create_stdout or "{}")
                    created_item_id = str(created_item.get("id") or "")
                    if status_field_id and status_option_id and project_id and created_item_id:
                        _run_gh_command(
                            [
                                "project",
                                "item-edit",
                                "--id",
                                created_item_id,
                                "--project-id",
                                project_id,
                                "--field-id",
                                status_field_id,
                                "--single-select-option-id",
                                status_option_id,
                            ]
                        )
                    if priority_field_id and priority_option_id and project_id and created_item_id:
                        _run_gh_command(
                            [
                                "project",
                                "item-edit",
                                "--id",
                                created_item_id,
                                "--project-id",
                                project_id,
                                "--field-id",
                                priority_field_id,
                                "--single-select-option-id",
                                priority_option_id,
                            ]
                        )
                    items_created += 1
                    continue

                existing_item_id = str(existing.get("id") or "")
                if existing_item_id:
                    _run_gh_command(
                        [
                            "project",
                            "item-edit",
                            "--id",
                            existing_item_id,
                            "--title",
                            title,
                            "--body",
                            body,
                        ]
                    )
                    if status_field_id and status_option_id and project_id:
                        _run_gh_command(
                            [
                                "project",
                                "item-edit",
                                "--id",
                                existing_item_id,
                                "--project-id",
                                project_id,
                                "--field-id",
                                status_field_id,
                                "--single-select-option-id",
                                status_option_id,
                            ]
                        )
                    if priority_field_id and priority_option_id and project_id:
                        _run_gh_command(
                            [
                                "project",
                                "item-edit",
                                "--id",
                                existing_item_id,
                                "--project-id",
                                project_id,
                                "--field-id",
                                priority_field_id,
                                "--single-select-option-id",
                                priority_option_id,
                            ]
                        )
                    items_updated += 1
                else:
                    errors.append(f"{item_id}:missing_existing_item_id")
            except GHProjectSyncError as exc:
                errors.append(f"{item_id}:{exc}")

        if errors and not config.standalone_mode:
            raise GHProjectSyncError("; ".join(errors))

        return {
            "items_created": items_created,
            "items_updated": items_updated,
            "items_synced": items_created + items_updated,
            "errors": errors,
            "synced_at": datetime.now(UTC).isoformat(),
        }

    except GHProjectAuthError as e:
        if config.standalone_mode:
            logger.warning(f"GH Project auth issue (standalone mode, skipping): {e}")
            return {
                "items_created": 0,
                "items_updated": 0,
                "errors": [str(e)],
                "status": "auth_required",
            }
        raise
    except GHProjectSyncError:
        raise


def sync_from_github(config: GHProjectConfig) -> dict[str, Any]:
    """Sync GitHub Projects to thegent workstream.

    Args:
        config: GitHub Projects configuration

    Returns:
        Dict with sync results: items_imported, errors

    Raises:
        GHProjectSyncError: For unexpected errors (not auth issues)
    """
    if not config.is_valid() or not config.can_read():
        if config.standalone_mode:
            logger.debug("GH Project sync not configured or write-only; skipping read sync")
            return {"items_imported": 0, "reason": "not_readable"}
        raise GHProjectSyncError("GitHub project not readable")

    try:
        items = _load_project_items(config)
        normalized_items: list[dict[str, Any]] = []
        for item in items:
            normalized_items.append(
                {
                    "id": item.get("id"),
                    "item_id": _extract_workstream_id(item),
                    "title": (
                        item.get("content", {}).get("title")
                        if isinstance(item.get("content"), dict)
                        else item.get("title", "")
                    ),
                    "status": _status_from_github(item),
                    "raw": item,
                }
            )
        if normalized_items:
            return {
                "items_imported": len(normalized_items),
                "items": normalized_items,
                "errors": [],
                "synced_at": datetime.now(UTC).isoformat(),
            }
        return {"items_imported": 0, "status": "failed"}

    except GHProjectAuthError as e:
        if config.standalone_mode:
            logger.warning(f"GH Project auth issue (standalone mode, skipping): {e}")
            return {"items_imported": 0, "status": "auth_required", "reason": str(e)}
        raise
    except GHProjectSyncError:
        raise


def export_to_csv(
    config: GHProjectConfig,
    output_path: Path,
) -> dict[str, Any]:
    """Export GitHub Project to CSV.

    Args:
        config: GitHub Projects configuration
        output_path: Path to write CSV export

    Returns:
        Dict with export results: items_exported, file_path
    """
    if not config.is_valid() or not config.can_read():
        if config.standalone_mode:
            logger.debug("GH Project sync not configured or write-only; skipping export")
            return {"items_exported": 0, "reason": "not_readable"}
        raise GHProjectSyncError("GitHub project not readable")

    try:
        code, stdout, _ = _coerce_gh_result(
            _run_gh_command(
                [
                    "project",
                    "item-list",
                    *_project_args(config),
                    "--format",
                    "csv",
                    "-L",
                    "500",
                ]
            )
        )

        if code == 0:
            output_path.parent.mkdir(parents=True, exist_ok=True)
            output_path.write_text(stdout)
            return {
                "items_exported": len(stdout.splitlines()),
                "file_path": str(output_path),
            }
        return {"items_exported": 0, "status": "failed"}

    except GHProjectAuthError as e:
        if config.standalone_mode:
            logger.warning(f"GH Project auth issue (standalone mode, skipping): {e}")
            return {"items_exported": 0, "status": "auth_required", "reason": str(e)}
        raise
    except GHProjectSyncError:
        raise


def import_from_csv(
    config: GHProjectConfig,
    csv_path: Path,
) -> dict[str, Any]:
    """Import items to GitHub Project from CSV.

    Args:
        config: GitHub Projects configuration
        csv_path: Path to CSV file to import

    Returns:
        Dict with import results: items_imported, errors
    """
    if not config.is_valid() or not config.can_write():
        if config.standalone_mode:
            logger.debug("GH Project sync not configured or read-only; skipping import")
            return {"items_imported": 0, "reason": "not_writable"}
        raise GHProjectSyncError("GitHub project not writable")

    if not csv_path.exists():
        raise GHProjectSyncError(f"CSV file not found: {csv_path}")

    try:
        _, view_stdout, _ = _coerce_gh_result(
            _run_gh_command(["project", "view", *_project_args(config), "--format", "json"])
        )
        project_id = ""
        if isinstance(view_stdout, str) and view_stdout.strip():
            try:
                project_id = str(json.loads(view_stdout).get("id") or "")
            except json.JSONDecodeError:
                project_id = ""
        status_field_id, status_options = _load_status_field(config)
        priority_field_id, priority_options = _load_priority_field(config)
        if not status_field_id:
            raise GHProjectSyncError("GitHub schema drift: required single-select field 'status' is missing")

        rows = list(csv.DictReader(io.StringIO(csv_path.read_text(encoding="utf-8"))))
        status_option_by_id = _prepare_github_status_mapping(rows, status_options)
        imported = 0
        errors: list[str] = []

        for row in rows:
            item_id = str(row.get("item_id") or row.get("id") or "").strip()
            title_text = str(row.get("title") or "").strip()
            if not item_id and not title_text:
                continue
            title = f"[{item_id}] {title_text}".strip() if item_id else title_text
            status_option_id = status_option_by_id.get(item_id)
            priority_option_id = _resolve_single_select_option_id(
                priority_options,
                _priority_option_candidates(str(row.get("priority") or "P2")),
            )
            try:
                _, create_stdout, _ = _coerce_gh_result(
                    _run_gh_command(
                        [
                            "project",
                            "item-create",
                            *_project_args(config),
                            "--title",
                            title,
                            "--body",
                            "\n".join(
                                [
                                    f"Workstream ID: {item_id}" if item_id else "",
                                    f"priority: {row.get('priority', '')}".strip(),
                                    f"area: {row.get('area', '')}".strip(),
                                ]
                            ).strip(),
                            "--format",
                            "json",
                        ]
                    )
                )
                created_item = json.loads(create_stdout or "{}")
                created_item_id = str(created_item.get("id") or "")
                if status_field_id and status_option_id and project_id and created_item_id:
                    _run_gh_command(
                        [
                            "project",
                            "item-edit",
                            "--id",
                            created_item_id,
                            "--project-id",
                            project_id,
                            "--field-id",
                            status_field_id,
                            "--single-select-option-id",
                            status_option_id,
                        ]
                    )
                if priority_field_id and priority_option_id and project_id and created_item_id:
                    _run_gh_command(
                        [
                            "project",
                            "item-edit",
                            "--id",
                            created_item_id,
                            "--project-id",
                            project_id,
                            "--field-id",
                            priority_field_id,
                            "--single-select-option-id",
                            priority_option_id,
                        ]
                    )
                imported += 1
            except GHProjectSyncError as exc:
                errors.append(f"{item_id or title}:{exc}")

        if errors and not config.standalone_mode:
            raise GHProjectSyncError("; ".join(errors))

        return {
            "items_imported": imported,
            "errors": errors,
        }

    except GHProjectAuthError as e:
        if config.standalone_mode:
            logger.warning(f"GH Project auth issue (standalone mode, skipping): {e}")
            return {
                "items_imported": 0,
                "errors": [str(e)],
                "status": "auth_required",
            }
        raise
    except GHProjectSyncError:
        raise


# End of file
