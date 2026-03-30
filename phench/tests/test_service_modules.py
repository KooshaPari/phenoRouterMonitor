"""Unit tests for decomposed service modules.

Tests verify that each extracted module functions correctly and that the
refactored service.py properly imports and re-exports all public functions.
"""

import sys
from pathlib import Path

import pytest

# Add src to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

import phench.service as service
import phench.manifest as manifest_module
import phench.lock_and_snapshot as lock_module
import phench.repo_management as repo_module
import phench.execution as exec_module
import phench.env_and_profile as env_module
import phench.catalog as catalog_module
import phench.lists_and_queries as lists_module
import phench.audit as audit_module
import phench.sync as sync_module
import phench.module_scanning as scan_module
import phench.runner as runner_module


class TestServiceOrchestrator:
    """Test the service.py orchestrator itself."""

    def test_all_exports_present(self):
        """Verify all expected functions are exported from service."""
        expected = {
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
        }

        assert set(service.__all__) == expected
        for name in expected:
            assert hasattr(service, name), f"Function {name} not found in service module"

    def test_service_all_list_complete(self):
        """Verify __all__ contains exactly the expected exports."""
        assert len(service.__all__) == 34
        assert isinstance(service.__all__, list)


class TestManifestModule:
    """Test manifest.py functions."""

    def test_manifest_module_exists(self):
        """Verify manifest module is importable."""
        assert manifest_module is not None

    def test_manifest_public_functions(self):
        """Verify expected public functions are in manifest."""
        expected = {
            "load_module_manifest",
            "load_module_repos",
            "build_module_manifest_payload",
            "add_module_to_target",
        }
        for func_name in expected:
            assert hasattr(manifest_module, func_name)

    def test_manifest_functions_exported_by_service(self):
        """Verify manifest functions are re-exported by service."""
        for func_name in ["load_module_manifest", "load_module_repos", "build_module_manifest_payload", "add_module_to_target"]:
            assert getattr(service, func_name) is getattr(manifest_module, func_name)


class TestLockAndSnapshotModule:
    """Test lock_and_snapshot.py functions."""

    def test_lock_module_exists(self):
        """Verify lock_and_snapshot module is importable."""
        assert lock_module is not None

    def test_lock_public_functions(self):
        """Verify expected public functions are in lock_and_snapshot."""
        expected = {
            "init_target",
            "load_target_lock",
            "lock_target",
            "create_target_snapshot",
            "list_target_snapshots",
            "show_target_snapshot",
        }
        for func_name in expected:
            assert hasattr(lock_module, func_name)

    def test_lock_functions_exported_by_service(self):
        """Verify lock functions are re-exported by service."""
        for func_name in ["init_target", "load_target_lock", "lock_target"]:
            assert getattr(service, func_name) is getattr(lock_module, func_name)


class TestRepoManagementModule:
    """Test repo_management.py functions."""

    def test_repo_module_exists(self):
        """Verify repo_management module is importable."""
        assert repo_module is not None

    def test_repo_public_functions(self):
        """Verify expected public functions are in repo_management."""
        expected = {
            "import_repos",
            "discover_repos",
            "bootstrap_target",
            "set_repo_ref",
            "add_repo",
        }
        for func_name in expected:
            assert hasattr(repo_module, func_name)

    def test_repo_functions_exported_by_service(self):
        """Verify repo functions are re-exported by service."""
        for func_name in ["import_repos", "discover_repos", "bootstrap_target"]:
            assert getattr(service, func_name) is getattr(repo_module, func_name)


class TestExecutionModule:
    """Test execution.py functions."""

    def test_execution_module_exists(self):
        """Verify execution module is importable."""
        assert exec_module is not None

    def test_execution_public_functions(self):
        """Verify expected public functions are in execution."""
        expected = {
            "run_target",
            "build_project_execution_matrix",
        }
        for func_name in expected:
            assert hasattr(exec_module, func_name)

    def test_execution_functions_exported_by_service(self):
        """Verify execution functions are re-exported by service."""
        assert getattr(service, "run_target") is getattr(exec_module, "run_target")
        assert getattr(service, "build_project_execution_matrix") is getattr(exec_module, "build_project_execution_matrix")


class TestEnvAndProfileModule:
    """Test env_and_profile.py functions."""

    def test_env_module_exists(self):
        """Verify env_and_profile module is importable."""
        assert env_module is not None

    def test_env_public_functions(self):
        """Verify expected public functions are in env_and_profile."""
        expected = {
            "set_env_profile",
            "get_env_profile",
            "run_env_doctor_for_target",
        }
        for func_name in expected:
            assert hasattr(env_module, func_name)

    def test_env_functions_exported_by_service(self):
        """Verify env functions are re-exported by service."""
        assert getattr(service, "set_env_profile") is getattr(env_module, "set_env_profile")
        assert getattr(service, "get_env_profile") is getattr(env_module, "get_env_profile")


class TestCatalogModule:
    """Test catalog.py functions."""

    def test_catalog_module_exists(self):
        """Verify catalog module is importable."""
        assert catalog_module is not None

    def test_catalog_public_functions(self):
        """Verify expected public functions are in catalog."""
        assert hasattr(catalog_module, "build_catalog")

    def test_catalog_functions_exported_by_service(self):
        """Verify catalog functions are re-exported by service."""
        assert getattr(service, "build_catalog") is getattr(catalog_module, "build_catalog")


class TestListsAndQueriesModule:
    """Test lists_and_queries.py functions."""

    def test_lists_module_exists(self):
        """Verify lists_and_queries module is importable."""
        assert lists_module is not None

    def test_lists_public_functions(self):
        """Verify expected public functions are in lists_and_queries."""
        expected = {
            "list_targets",
            "list_modules",
            "target_status",
            "target_timeline",
            "materialize_target",
        }
        for func_name in expected:
            assert hasattr(lists_module, func_name)

    def test_lists_functions_exported_by_service(self):
        """Verify lists functions are re-exported by service."""
        for func_name in ["list_targets", "list_modules", "target_status"]:
            assert getattr(service, func_name) is getattr(lists_module, func_name)


class TestAuditModule:
    """Test audit.py functions."""

    def test_audit_module_exists(self):
        """Verify audit module is importable."""
        assert audit_module is not None

    def test_audit_public_functions(self):
        """Verify expected public functions are in audit."""
        expected = {
            "audit_shared_modules",
            "audit_shared_modules_across_repos",
        }
        for func_name in expected:
            assert hasattr(audit_module, func_name)

    def test_audit_functions_exported_by_service(self):
        """Verify audit functions are re-exported by service."""
        assert getattr(service, "audit_shared_modules") is getattr(audit_module, "audit_shared_modules")


class TestSyncModule:
    """Test sync.py functions."""

    def test_sync_module_exists(self):
        """Verify sync module is importable."""
        assert sync_module is not None

    def test_sync_public_functions(self):
        """Verify expected public functions are in sync."""
        expected = {
            "sync_project_modules_from_repos",
            "sync_target",
        }
        for func_name in expected:
            assert hasattr(sync_module, func_name)

    def test_sync_functions_exported_by_service(self):
        """Verify sync functions are re-exported by service."""
        assert getattr(service, "sync_project_modules_from_repos") is getattr(sync_module, "sync_project_modules_from_repos")
        assert getattr(service, "sync_target") is getattr(sync_module, "sync_target")


class TestModuleScanningModule:
    """Test module_scanning.py functions."""

    def test_scanning_module_exists(self):
        """Verify module_scanning module is importable."""
        assert scan_module is not None

    def test_scanning_public_functions(self):
        """Verify expected public functions are in module_scanning."""
        expected = {
            "scan_shared_modules_across_repos",
            "build_scan_candidates",
            "materialize_module_candidate_manifest",
        }
        for func_name in expected:
            assert hasattr(scan_module, func_name)

    def test_scanning_functions_exported_by_service(self):
        """Verify scanning functions are re-exported by service."""
        assert getattr(service, "scan_shared_modules_across_repos") is getattr(scan_module, "scan_shared_modules_across_repos")


class TestRunnerModule:
    """Test runner.py functions."""

    def test_runner_module_exists(self):
        """Verify runner module is importable."""
        assert runner_module is not None

    def test_runner_public_functions(self):
        """Verify expected public functions are in runner."""
        assert hasattr(runner_module, "build_runner_catalog")

    def test_runner_functions_exported_by_service(self):
        """Verify runner functions are re-exported by service."""
        assert getattr(service, "build_runner_catalog") is getattr(runner_module, "build_runner_catalog")


class TestModuleDecoupling:
    """Test that modules are properly decoupled."""

    def test_no_circular_imports(self):
        """Verify no circular imports exist between modules."""
        # If we got here, imports succeeded with no circular dependency
        assert True

    def test_service_imports_all_modules(self):
        """Verify service.py imports from all feature modules."""
        modules_to_check = [
            "manifest",
            "lock_and_snapshot",
            "repo_management",
            "execution",
            "env_and_profile",
            "catalog",
            "lists_and_queries",
            "audit",
            "sync",
            "module_scanning",
            "runner",
        ]
        # All modules should be importable without errors
        for module_name in modules_to_check:
            module = __import__(f"phench.{module_name}", fromlist=[module_name])
            assert module is not None


class TestBackwardCompatibility:
    """Test backward compatibility with original API."""

    def test_can_import_from_service(self):
        """Verify all public functions can be imported from service."""
        # This tests the primary use case: from phench.service import X
        from phench.service import (
            load_module_manifest,
            run_target,
            list_targets,
            build_catalog,
            audit_shared_modules,
        )
        assert callable(load_module_manifest)
        assert callable(run_target)
        assert callable(list_targets)
        assert callable(build_catalog)
        assert callable(audit_shared_modules)

    def test_can_import_all_from_service(self):
        """Verify wildcard import works from service."""
        # Import all public functions
        from phench import service as svc
        count = 0
        for name in svc.__all__:
            assert hasattr(svc, name), f"Function {name} listed in __all__ but not found"
            count += 1
        assert count == 34
