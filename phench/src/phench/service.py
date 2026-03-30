"""Service orchestrator - imports and re-exports from feature modules.

This module serves as the public API facade for phench. It consolidates
imports from all feature-specific modules and re-exports them for backward
compatibility and ease of use.

Module Architecture:
- manifest: Module manifest loading, validation, and building
- lock_and_snapshot: Target locks and audit snapshots
- repo_management: Repository discovery, selection, and management
- execution: Target execution and command dispatch
- env_and_profile: Environment profiles and diagnostics
- catalog: Runner catalog building
- lists_and_queries: Queries and listing operations
- audit: Module state auditing and cross-repo consistency checks
- sync: Module and repository synchronization
- module_scanning: Shared module scanning and candidate discovery
"""

from __future__ import annotations

# Import all public functions from feature modules
from .manifest import (
    load_module_manifest,
    load_module_repos,
    build_module_manifest_payload,
    add_module_to_target,
)

from .lock_and_snapshot import (
    init_target,
    load_target_lock,
    lock_target,
    create_target_snapshot,
    list_target_snapshots,
    show_target_snapshot,
)

from .repo_management import (
    import_repos,
    discover_repos,
    bootstrap_target,
    set_repo_ref,
    add_repo,
)

from .execution import (
    run_target,
    build_project_execution_matrix,
)

from .env_and_profile import (
    set_env_profile,
    get_env_profile,
    run_env_doctor_for_target,
)

from .catalog import (
    build_catalog,
)

from .lists_and_queries import (
    list_targets,
    list_modules,
    target_status,
    target_timeline,
    materialize_target,
)

from .audit import (
    audit_shared_modules,
    audit_shared_modules_across_repos,
)

from .sync import (
    sync_project_modules_from_repos,
    sync_target,
)

from .module_scanning import (
    scan_shared_modules_across_repos,
    build_scan_candidates,
    materialize_module_candidate_manifest,
)

from .runner import (
    build_runner_catalog,
)

# Re-export all public functions for backward compatibility
__all__ = [
    # manifest
    "load_module_manifest",
    "load_module_repos",
    "build_module_manifest_payload",
    "add_module_to_target",
    # lock_and_snapshot
    "init_target",
    "load_target_lock",
    "lock_target",
    "create_target_snapshot",
    "list_target_snapshots",
    "show_target_snapshot",
    # repo_management
    "import_repos",
    "discover_repos",
    "bootstrap_target",
    "set_repo_ref",
    "add_repo",
    # execution
    "run_target",
    "build_project_execution_matrix",
    # env_and_profile
    "set_env_profile",
    "get_env_profile",
    "run_env_doctor_for_target",
    # catalog
    "build_catalog",
    # lists_and_queries
    "list_targets",
    "list_modules",
    "target_status",
    "target_timeline",
    "materialize_target",
    # audit
    "audit_shared_modules",
    "audit_shared_modules_across_repos",
    # sync
    "sync_project_modules_from_repos",
    "sync_target",
    # module_scanning
    "scan_shared_modules_across_repos",
    "build_scan_candidates",
    "materialize_module_candidate_manifest",
    # runner
    "build_runner_catalog",
]
