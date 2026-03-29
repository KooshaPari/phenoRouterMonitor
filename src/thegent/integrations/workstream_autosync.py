"""Public autosync façade."""

from __future__ import annotations

import asyncio

from thegent.infra.identity_proxy import SSHIdentityProxy
from thegent.integrations.gh_project_sync import (
    close_or_comment_github_issue_refs,
    extract_github_issue_refs,
    sync_from_github as gh_sync_from_github,
    sync_to_github as gh_sync_to_github,
)
from thegent.integrations.linear_graphql import (
    sync_from_linear as linear_sync_from,
    sync_to_linear as linear_sync_to,
)
from thegent.integrations.workstream_autosync_shared import (
    ConnectorSLAThresholds,
    MaintenanceWindow,
    RemoteMissingItemPolicy,
    RetryClass,
    SyncDirection,
    SyncOperation,
    WorkstreamAutosyncConfig,
    WorkstreamAutosyncConfigError,
    WorkstreamAutosyncError,
    WorkstreamItem,
    WorkstreamParser,
    load_autosync_config_from_env,
)
from thegent.autosync.runner import WorkstreamAutosyncRunner

__all__ = [
    "ConnectorSLAThresholds",
    "MaintenanceWindow",
    "RemoteMissingItemPolicy",
    "RetryClass",
    "SSHIdentityProxy",
    "SyncDirection",
    "SyncOperation",
    "WorkstreamAutosyncConfig",
    "WorkstreamAutosyncConfigError",
    "WorkstreamAutosyncError",
    "WorkstreamAutosyncRunner",
    "WorkstreamItem",
    "WorkstreamParser",
    "asyncio",
    "close_or_comment_github_issue_refs",
    "extract_github_issue_refs",
    "gh_sync_from_github",
    "gh_sync_to_github",
    "linear_sync_from",
    "linear_sync_to",
    "load_autosync_config_from_env",
]
