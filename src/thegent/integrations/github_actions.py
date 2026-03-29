"""GitHub Actions integration helpers for thegent.

Provides helper methods for interacting with GitHub Actions workflows and runs
via the ``gh`` CLI subprocess. The ``gh`` CLI must be authenticated and
available on PATH.

All public functions raise ``RuntimeError`` on failure — no silent degradation.
"""

from __future__ import annotations

import json
import logging
import subprocess
from dataclasses import dataclass, field
from typing import Any

_log = logging.getLogger(__name__)


# ---------------------------------------------------------------------------
# Internal helpers
# ---------------------------------------------------------------------------


def _gh_api(path: str, method: str = "GET", body: dict[str, Any] | None = None) -> Any:
    """Call ``gh api`` and return parsed JSON.

    Parameters
    ----------
    path:
        GitHub API path, e.g. ``/repos/owner/repo/actions/runs``.
    method:
        HTTP method (GET, POST, etc.).
    body:
        Optional request body serialised to JSON.

    Raises
    ------
    RuntimeError
        On non-zero exit or JSON parse error.
    """
    cmd = ["gh", "api", "--method", method, path]
    if body:
        for key, value in body.items():
            cmd += ["-f", f"{key}={json.dumps(value)}"]

    _log.debug("gh api %s %s", method, path)
    result = subprocess.run(cmd, capture_output=True, text=True)

    if result.returncode != 0:
        raise RuntimeError(
            f"gh api {method} {path} failed (exit {result.returncode}): {result.stderr.strip()}"
        )

    if not result.stdout.strip():
        return None

    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError as exc:
        raise RuntimeError(
            f"gh api {method} {path} returned non-JSON output: {result.stdout[:200]}"
        ) from exc


# ---------------------------------------------------------------------------
# Data models
# ---------------------------------------------------------------------------


@dataclass
class WorkflowRun:
    """Represents a single GitHub Actions workflow run."""

    id: int
    name: str
    status: str
    conclusion: str | None
    workflow_id: int
    head_branch: str
    head_sha: str
    html_url: str
    created_at: str
    updated_at: str
    run_number: int
    raw: dict[str, Any] = field(default_factory=dict, repr=False)

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> WorkflowRun:
        """Construct from a GitHub API response dict."""
        return cls(
            id=data["id"],
            name=data.get("name", ""),
            status=data.get("status", ""),
            conclusion=data.get("conclusion"),
            workflow_id=data.get("workflow_id", 0),
            head_branch=data.get("head_branch", ""),
            head_sha=data.get("head_sha", ""),
            html_url=data.get("html_url", ""),
            created_at=data.get("created_at", ""),
            updated_at=data.get("updated_at", ""),
            run_number=data.get("run_number", 0),
            raw=data,
        )


@dataclass
class WorkflowRunStatus:
    """Simplified status for a single workflow run."""

    run_id: int
    status: str
    conclusion: str | None
    in_progress: bool
    completed: bool
    successful: bool

    @classmethod
    def from_run(cls, run: WorkflowRun) -> WorkflowRunStatus:
        """Derive status from a WorkflowRun."""
        completed = run.status == "completed"
        successful = completed and run.conclusion == "success"
        return cls(
            run_id=run.id,
            status=run.status,
            conclusion=run.conclusion,
            in_progress=run.status == "in_progress",
            completed=completed,
            successful=successful,
        )


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------


def get_workflow_runs(
    repo: str,
    workflow_id: str | int,
    *,
    branch: str | None = None,
    limit: int = 20,
) -> list[WorkflowRun]:
    """List recent workflow runs for a given workflow.

    Parameters
    ----------
    repo:
        Repository in ``owner/repo`` format.
    workflow_id:
        The workflow file name (e.g. ``ci.yml``) or numeric workflow ID.
    branch:
        Optional branch filter.
    limit:
        Maximum number of runs to return (default 20, GitHub max 100).

    Returns
    -------
    list[WorkflowRun]
        Most recent runs, newest first.

    Raises
    ------
    RuntimeError
        On API failure.
    """
    path = f"/repos/{repo}/actions/workflows/{workflow_id}/runs"
    params: dict[str, Any] = {"per_page": min(limit, 100)}
    if branch:
        params["branch"] = branch

    query = "&".join(f"{k}={v}" for k, v in params.items())
    data = _gh_api(f"{path}?{query}")

    runs_raw: list[dict[str, Any]] = data.get("workflow_runs", [])
    return [WorkflowRun.from_dict(r) for r in runs_raw[:limit]]


def get_workflow_run_status(repo: str, run_id: int) -> WorkflowRunStatus:
    """Get the current status of a specific workflow run.

    Parameters
    ----------
    repo:
        Repository in ``owner/repo`` format.
    run_id:
        The numeric run ID.

    Returns
    -------
    WorkflowRunStatus
        Status summary for the run.

    Raises
    ------
    RuntimeError
        On API failure.
    """
    data = _gh_api(f"/repos/{repo}/actions/runs/{run_id}")
    run = WorkflowRun.from_dict(data)
    return WorkflowRunStatus.from_run(run)


def trigger_workflow(
    repo: str,
    workflow_id: str | int,
    ref: str,
    inputs: dict[str, str] | None = None,
) -> None:
    """Dispatch a workflow via ``workflow_dispatch`` event.

    Parameters
    ----------
    repo:
        Repository in ``owner/repo`` format.
    workflow_id:
        Workflow file name or numeric ID.
    ref:
        Branch or tag ref to run the workflow on.
    inputs:
        Optional key/value pairs passed as workflow inputs.

    Raises
    ------
    RuntimeError
        On API failure or non-2xx response.
    """
    path = f"/repos/{repo}/actions/workflows/{workflow_id}/dispatches"
    body: dict[str, Any] = {"ref": ref}
    if inputs:
        body["inputs"] = inputs

    _log.info("Triggering workflow %s on %s@%s", workflow_id, repo, ref)

    # POST to dispatches endpoint; returns 204 No Content on success.
    extra: list[str] = []
    if inputs:
        for k, v in inputs.items():
            extra += ["-f", f"inputs[{k}]={v}"]

    _gh_run_cmd("api", "--method", "POST", path, "-f", f"ref={ref}", *extra)
    _log.info("Workflow dispatch accepted for %s/%s@%s", repo, workflow_id, ref)


def list_workflows(repo: str) -> list[dict[str, Any]]:
    """List all workflows defined in a repository.

    Parameters
    ----------
    repo:
        Repository in ``owner/repo`` format.

    Returns
    -------
    list[dict[str, Any]]
        Raw workflow objects from the GitHub API.

    Raises
    ------
    RuntimeError
        On API failure.
    """
    data = _gh_api(f"/repos/{repo}/actions/workflows")
    return data.get("workflows", [])


def cancel_workflow_run(repo: str, run_id: int) -> None:
    """Cancel an in-progress workflow run.

    Parameters
    ----------
    repo:
        Repository in ``owner/repo`` format.
    run_id:
        The numeric run ID.

    Raises
    ------
    RuntimeError
        On API failure.
    """
    _gh_run_cmd("api", "--method", "POST", f"/repos/{repo}/actions/runs/{run_id}/cancel")
    _log.info("Cancelled workflow run %s in %s", run_id, repo)


def rerun_workflow(repo: str, run_id: int, *, failed_only: bool = False) -> None:
    """Re-run a completed workflow run (all jobs or failed jobs only).

    Parameters
    ----------
    repo:
        Repository in ``owner/repo`` format.
    run_id:
        The numeric run ID.
    failed_only:
        When ``True``, only re-run failed jobs.

    Raises
    ------
    RuntimeError
        On API failure.
    """
    suffix = "/failed-jobs" if failed_only else ""
    _gh_run_cmd("api", "--method", "POST", f"/repos/{repo}/actions/runs/{run_id}/rerun{suffix}")
    _log.info("Re-run triggered for workflow run %s in %s (failed_only=%s)", run_id, repo, failed_only)
