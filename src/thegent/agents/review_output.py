"""WL-107: strict parser/validator for structured review command output."""

from __future__ import annotations

import orjson as json
from typing import Any

_SEVERITIES = frozenset({"critical", "high", "medium", "low"})
_ISSUE_FIELDS = frozenset({"file", "line", "severity", "message", "suggestion"})
_TOP_LEVEL_FIELDS = frozenset({"summary", "overall_rating", "issues"})


def _require_non_empty_string(payload: dict[str, Any], key: str) -> str:
    value = payload.get(key)
    if not isinstance(value, str) or not value.strip():
        raise ValueError(f"'{key}' must be a non-empty string.")
    return value.strip()


def _validate_issue(issue: dict[str, Any], index: int) -> dict[str, Any]:
    missing = sorted(_ISSUE_FIELDS - issue.keys())
    if missing:
        raise ValueError(f"issues[{index}] is missing required keys: {', '.join(missing)}.")
    extra = sorted(issue.keys() - _ISSUE_FIELDS)
    if extra:
        raise ValueError(f"issues[{index}] has unsupported keys: {', '.join(extra)}.")

    file_path = _require_non_empty_string(issue, "file")

    line = issue.get("line")
    if isinstance(line, bool) or not isinstance(line, int) or line < 1:
        raise ValueError(f"issues[{index}].line must be an integer >= 1.")

    severity = issue.get("severity")
    if not isinstance(severity, str) or severity not in _SEVERITIES:
        allowed = ", ".join(sorted(_SEVERITIES))
        raise ValueError(f"issues[{index}].severity must be one of: {allowed}.")

    message = _require_non_empty_string(issue, "message")
    suggestion = _require_non_empty_string(issue, "suggestion")
    return {
        "file": file_path,
        "line": line,
        "severity": severity,
        "message": message,
        "suggestion": suggestion,
    }


def validate_review_output(payload: dict[str, Any]) -> dict[str, Any]:
    """Validate and normalize a structured review payload."""
    if not isinstance(payload, dict):
        raise ValueError("Review output must be a JSON object.")

    missing = sorted(_TOP_LEVEL_FIELDS - payload.keys())
    if missing:
        raise ValueError(f"Review output is missing required keys: {', '.join(missing)}.")
    extra = sorted(payload.keys() - _TOP_LEVEL_FIELDS)
    if extra:
        raise ValueError(f"Review output has unsupported keys: {', '.join(extra)}.")

    summary = _require_non_empty_string(payload, "summary")
    overall_rating = payload.get("overall_rating")
    if (
        isinstance(overall_rating, bool)
        or not isinstance(overall_rating, int)
        or overall_rating < 0
        or overall_rating > 100
    ):
        raise ValueError("'overall_rating' must be an integer in [0, 100].")

    issues = payload.get("issues")
    if not isinstance(issues, list):
        raise ValueError("'issues' must be a list.")

    normalized_issues: list[dict[str, Any]] = []
    for index, raw_issue in enumerate(issues):
        if not isinstance(raw_issue, dict):
            raise ValueError(f"issues[{index}] must be an object.")
        normalized_issues.append(_validate_issue(raw_issue, index))

    return {
        "summary": summary,
        "overall_rating": overall_rating,
        "issues": normalized_issues,
    }


def parse_review_output(raw_output: str) -> dict[str, Any]:
    """Parse and validate review output from raw JSON text."""
    try:
        parsed = json.loads(raw_output)
    except json.JSONDecodeError as exc:
        raise ValueError(f"Review output is not valid JSON: {exc}") from exc
    return validate_review_output(parsed)
