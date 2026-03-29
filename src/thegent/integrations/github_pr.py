"""GitHub Pull Request integration helpers for thegent.

Provides helper methods for listing, inspecting, creating, and merging GitHub
pull requests via the ``gh`` CLI subprocess. The ``gh`` CLI must be
authenticated and available on PATH.

All public functions raise ``RuntimeError`` on failure — no silent degradation.
"""

from __future__ import annotations

import json
import logging
import subprocess
from dataclasses import dataclass, field
from typing import Any, Literal

_log = logging.getLogger(__name__)

MergeMethod = Literal["merge", "squash", "rebase"]

# ---------------------------------------------------------------------------
# Internal helpers
# ---------------------------------------------------------------------------


def _gh_api(path: str, method: str = "GET", body: dict[str, Any] | None = None) -> Any:
    """Call ``gh api`` and return parsed JSON.

    Parameters
    ----------
    path:
        GitHub API path, e.g. ``/repos/owner/repo/pulls``.
    method:
        HTTP method.
    body:
        Optional fields forwarded as ``-f key=value`` pairs.

    Raises
    ------
    RuntimeError
        On non-zero exit or JSON parse error.
    """
    cmd = ["gh", "api", "--method", method, path]
    if body:
        for key, value in body.items():
            if isinstance(value, (dict, list)):
                cmd += ["-f", f"{key}={json.dumps(value)}"]
            elif isinstance(value, bool):
                cmd += ["-f", f"{key}={'true' if value else 'false'}"]
            else:
                cmd += ["-f", f"{key}={value}"]

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
class CheckRun:
    """Summary of a single CI check run."""

    name: str
    status: str
    conclusion: str | None
    url: str


@dataclass
class PRStatus:
    """Mergeable state and CI check summary for a pull request."""

    number: int
    title: str
    state: str
    mergeable: bool | None
    mergeable_state: str
    draft: bool
    checks: list[CheckRun] = field(default_factory=list)
    raw: dict[str, Any] = field(default_factory=dict, repr=False)

    @property
    def all_checks_passed(self) -> bool:
        """Return True if every check concluded with 'success'."""
        if not self.checks:
            return False
        return all(c.conclusion == "success" for c in self.checks)

    @property
    def ready_to_merge(self) -> bool:
        """Return True if PR is mergeable and all checks passed."""
        return (
            self.mergeable is True
            and self.mergeable_state == "clean"
            and not self.draft
            and self.all_checks_passed
        )


@dataclass
class PullRequest:
    """Represents a GitHub Pull Request."""

    number: int
    title: str
    body: str
    state: str
    head: str
    base: str
    html_url: str
    draft: bool
    author: str
    created_at: str
    updated_at: str
    labels: list[str] = field(default_factory=list)
    raw: dict[str, Any] = field(default_factory=dict, repr=False)

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> PullRequest:
        """Construct from a GitHub API response dict."""
        return cls(
            number=data["number"],
            title=data.get("title", ""),
            body=data.get("body", "") or "",
            state=data.get("state", ""),
            head=data.get("head", {}).get("ref", ""),
            base=data.get("base", {}).get("ref", ""),
            html_url=data.get("html_url", ""),
            draft=data.get("draft", False),
            author=data.get("user", {}).get("login", ""),
            created_at=data.get("created_at", ""),
            updated_at=data.get("updated_at", ""),
            labels=[lb["name"] for lb in data.get("labels", [])],
            raw=data,
        )


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------


def list_open_prs(
    repo: str,
    *,
    base: str | None = None,
    author: str | None = None,
    label: str | None = None,
    limit: int = 30,
) -> list[PullRequest]:
    """List open pull requests for a repository.

    Parameters
    ----------
    repo:
        Repository in ``owner/repo`` format.
    base:
        Optional base branch filter (e.g. ``main``).
    author:
        Optional GitHub login to filter by PR author.
    label:
        Optional label name to filter by.
    limit:
        Maximum number of PRs to return (default 30).

    Returns
    -------
    list[PullRequest]
        Open PRs, newest first.

    Raises
    ------
    RuntimeError
        On API failure.
    """
    params = f"state=open&per_page={min(limit, 100)}"
    if base:
        params += f"&base={base}"
    if label:
        params += f"&labels={label}"

    data: list[dict[str, Any]] = _gh_api(f"/repos/{repo}/pulls?{params}") or []

    prs = [PullRequest.from_dict(pr) for pr in data[:limit]]
    if author:
        prs = [pr for pr in prs if pr.author == author]
    return prs


def get_pr_status(repo: str, pr_number: int) -> PRStatus:
    """Get the current mergeable state and CI checks for a pull request.

    Parameters
    ----------
    repo:
        Repository in ``owner/repo`` format.
    pr_number:
        The PR number.

    Returns
    -------
    PRStatus
        Mergeable state and check summary.

    Raises
    ------
    RuntimeError
        On API failure.
    """
    pr_data: dict[str, Any] = _gh_api(f"/repos/{repo}/pulls/{pr_number}")

    # Fetch check runs for the HEAD commit.
    head_sha: str = pr_data.get("head", {}).get("sha", "")
    checks: list[CheckRun] = []

    if head_sha:
        checks_data = _gh_api(f"/repos/{repo}/commits/{head_sha}/check-runs") or {}
        for cr in checks_data.get("check_runs", []):
            checks.append(
                CheckRun(
                    name=cr.get("name", ""),
                    status=cr.get("status", ""),
                    conclusion=cr.get("conclusion"),
                    url=cr.get("html_url", ""),
                )
            )

    return PRStatus(
        number=pr_data["number"],
        title=pr_data.get("title", ""),
        state=pr_data.get("state", ""),
        mergeable=pr_data.get("mergeable"),
        mergeable_state=pr_data.get("mergeable_state", "unknown"),
        draft=pr_data.get("draft", False),
        checks=checks,
        raw=pr_data,
    )


def create_pr(
    repo: str,
    head: str,
    base: str,
    title: str,
    body: str,
    *,
    draft: bool = False,
) -> PullRequest:
    """Create a new pull request.

    Parameters
    ----------
    repo:
        Repository in ``owner/repo`` format.
    head:
        Source branch (``owner:branch`` or plain branch name for same-repo).
    base:
        Target branch (e.g. ``main``).
    title:
        PR title.
    body:
        PR description/body markdown.
    draft:
        When ``True``, creates the PR in draft state.

    Returns
    -------
    PullRequest
        The newly created pull request.

    Raises
    ------
    RuntimeError
        On API failure (e.g. branch does not exist, PR already exists).
    """
    payload: dict[str, Any] = {
        "title": title,
        "head": head,
        "base": base,
        "body": body,
        "draft": draft,
    }
    _log.info("Creating PR '%s' in %s (%s -> %s)", title, repo, head, base)
    data: dict[str, Any] = _gh_api(f"/repos/{repo}/pulls", method="POST", body=payload)
    return PullRequest.from_dict(data)


def merge_pr(
    repo: str,
    pr_number: int,
    method: MergeMethod = "squash",
    *,
    commit_title: str | None = None,
    commit_message: str | None = None,
    delete_branch: bool = True,
) -> dict[str, Any]:
    """Merge a pull request.

    Parameters
    ----------
    repo:
        Repository in ``owner/repo`` format.
    pr_number:
        The PR number.
    method:
        One of ``"merge"``, ``"squash"``, or ``"rebase"``.
    commit_title:
        Optional override for the merge commit title.
    commit_message:
        Optional override for the merge commit message body.
    delete_branch:
        When ``True``, deletes the head branch after merging (default True).

    Returns
    -------
    dict[str, Any]
        Raw merge response from the GitHub API.

    Raises
    ------
    RuntimeError
        On API failure (e.g. not mergeable, conflicts).
    """
    payload: dict[str, Any] = {"merge_method": method}
    if commit_title:
        payload["commit_title"] = commit_title
    if commit_message:
        payload["commit_message"] = commit_message

    _log.info("Merging PR #%s in %s via %s", pr_number, repo, method)
    result: dict[str, Any] = _gh_api(
        f"/repos/{repo}/pulls/{pr_number}/merge", method="PUT", body=payload
    )

    if delete_branch:
        try:
            # Retrieve head ref to delete.
            pr_data: dict[str, Any] = _gh_api(f"/repos/{repo}/pulls/{pr_number}") or {}
            head_ref: str = pr_data.get("head", {}).get("ref", "")
            if head_ref:
                _gh_cmd("api", "--method", "DELETE", f"/repos/{repo}/git/refs/heads/{head_ref}")
                _log.info("Deleted branch %s in %s", head_ref, repo)
        except RuntimeError as exc:
            _log.warning("Branch deletion failed (non-fatal): %s", exc)

    return result or {}


def request_pr_review(repo: str, pr_number: int, reviewers: list[str]) -> None:
    """Request reviews from specific GitHub users.

    Parameters
    ----------
    repo:
        Repository in ``owner/repo`` format.
    pr_number:
        The PR number.
    reviewers:
        List of GitHub login names to request reviews from.

    Raises
    ------
    RuntimeError
        On API failure.
    """
    _gh_api(
        f"/repos/{repo}/pulls/{pr_number}/requested_reviewers",
        method="POST",
        body={"reviewers": reviewers},
    )
    _log.info("Requested reviews from %s on PR #%s in %s", reviewers, pr_number, repo)


def add_pr_labels(repo: str, pr_number: int, labels: list[str]) -> None:
    """Add labels to a pull request.

    Parameters
    ----------
    repo:
        Repository in ``owner/repo`` format.
    pr_number:
        The PR number.
    labels:
        Label names to add.

    Raises
    ------
    RuntimeError
        On API failure.
    """
    _gh_api(
        f"/repos/{repo}/issues/{pr_number}/labels",
        method="POST",
        body={"labels": labels},
    )
    _log.info("Added labels %s to PR #%s in %s", labels, pr_number, repo)


def close_pr(repo: str, pr_number: int) -> None:
    """Close (without merging) a pull request.

    Parameters
    ----------
    repo:
        Repository in ``owner/repo`` format.
    pr_number:
        The PR number.

    Raises
    ------
    RuntimeError
        On API failure.
    """
    _gh_api(
        f"/repos/{repo}/pulls/{pr_number}",
        method="PATCH",
        body={"state": "closed"},
    )
    _log.info("Closed PR #%s in %s", pr_number, repo)
