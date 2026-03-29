"""TheGent integrations package."""

# Base integration components
from .base import (
    IntegrationStatus,
    IntegrationInfo,
    DataclassConfig,
    BaseIntegrationConfig,
    BaseIntegration,
)

# GitHub Actions helpers
from .github_actions import (
    WorkflowRun,
    WorkflowRunStatus,
    cancel_workflow_run,
    get_workflow_run_status,
    get_workflow_runs,
    list_workflows,
    rerun_workflow,
    trigger_workflow,
)

# GitHub PR helpers
from .github_pr import (
    CheckRun,
    MergeMethod,
    PRStatus,
    PullRequest,
    add_pr_labels,
    close_pr,
    create_pr,
    get_pr_status,
    list_open_prs,
    merge_pr,
    request_pr_review,
)

__all__ = [
    # Base
    "BaseIntegration",
    "BaseIntegrationConfig",
    "DataclassConfig",
    "IntegrationInfo",
    "IntegrationStatus",
    # GitHub Actions
    "WorkflowRun",
    "WorkflowRunStatus",
    "cancel_workflow_run",
    "get_workflow_run_status",
    "get_workflow_runs",
    "list_workflows",
    "rerun_workflow",
    "trigger_workflow",
    # GitHub PR
    "CheckRun",
    "MergeMethod",
    "PRStatus",
    "PullRequest",
    "add_pr_labels",
    "close_pr",
    "create_pr",
    "get_pr_status",
    "list_open_prs",
    "merge_pr",
    "request_pr_review",
]
