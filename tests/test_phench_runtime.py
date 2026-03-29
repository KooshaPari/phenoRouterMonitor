from __future__ import annotations

import importlib
import json
import subprocess
from pathlib import Path

import pytest

from phench.service import (
    add_repo,
    audit_shared_modules,
    add_module_to_target,
    build_module_manifest_payload,
    build_scan_candidates,
    scan_shared_modules_across_repos,
    materialize_module_candidate_manifest,
    get_env_profile,
    discover_repos,
    init_target,
    import_repos,
    list_targets,
    list_target_snapshots,
    lock_target,
    materialize_target,
    show_target_snapshot,
    build_project_execution_matrix,
    set_repo_ref,
    run_env_doctor_for_target,
    run_target,
    target_timeline,
    target_status,
    set_env_profile,
    sync_target,
    sync_project_modules_from_repos,
    load_module_manifest,
    list_modules,
    audit_shared_modules_across_repos,
    create_target_snapshot,
)
from phench.models import RepoSelection, RunnerCatalog, RunnerCommand, TargetLock
from phench.store import read_dual


def _run(cmd: list[str], cwd: Path) -> None:
    proc = subprocess.run(cmd, cwd=str(cwd), capture_output=True, text=True, check=False)
    if proc.returncode != 0:
        raise RuntimeError(f"command failed: {' '.join(cmd)}\n{proc.stderr}")


def _init_git_repo(path: Path) -> str:
    path.mkdir(parents=True, exist_ok=True)
    _run(["git", "init"], cwd=path)
    _run(["git", "config", "user.email", "test@example.com"], cwd=path)
    _run(["git", "config", "user.name", "Test User"], cwd=path)
    (path / "Taskfile.yml").write_text(
        "version: '3'\ntasks:\n  hello:\n    cmds:\n      - echo hello\n", encoding="utf-8"
    )
    (path / "README.md").write_text("hello\n", encoding="utf-8")
    _run(["git", "add", "."], cwd=path)
    _run(["git", "commit", "-m", "init"], cwd=path)
    proc = subprocess.run(["git", "rev-parse", "HEAD"], cwd=str(path), capture_output=True, text=True, check=False)
    assert proc.returncode == 0
    return proc.stdout.strip()


def _init_git_repo_with_pkg(path: Path, pkg_name: str) -> None:
    path.mkdir(parents=True, exist_ok=True)
    _run(["git", "init"], cwd=path)
    _run(["git", "config", "user.email", "test@example.com"], cwd=path)
    _run(["git", "config", "user.name", "Test User"], cwd=path)
    pkg_dir = path / "src" / pkg_name
    pkg_dir.mkdir(parents=True, exist_ok=True)
    (pkg_dir / "__init__.py").write_text("", encoding="utf-8")
    (path / "README.md").write_text("hello\n", encoding="utf-8")
    _run(["git", "add", "."], cwd=path)
    _run(["git", "commit", "-m", "init"], cwd=path)


def _init_fake_repo(path: Path) -> None:
    path.mkdir(parents=True, exist_ok=True)
    (path / ".git").mkdir()


def test_lock_and_materialize_with_dual_write(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    projects_root = phenotype_root / "projects"
    mirror_root = tmp_path / "home-phench"
    source_repo = tmp_path / "source-repo"

    sha = _init_git_repo(source_repo)

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    monkeypatch.setenv("THGENT_PHENCH_HOME_ROOT", str(mirror_root))

    lock = init_target("alpha", mode="repo")
    assert lock.target_name == "alpha"

    lock = add_repo("alpha", repo_path=str(source_repo), selected_ref="HEAD")
    assert len(lock.repos) == 1

    lock = lock_target("alpha")
    assert lock.repos[0].resolved_sha == sha

    runtime = materialize_target("alpha")
    assert runtime.repo_materializations
    checkout = Path(runtime.repo_materializations[0].checkout_path)
    assert (checkout / "README.md").exists()

    project_lock = projects_root / "alpha" / ".phench" / "target.lock.json"
    mirror_lock = mirror_root / "alpha" / ".phench" / "target.lock.json"
    assert project_lock.exists()
    assert mirror_lock.exists()


def test_env_doctor_reports_missing_runner_binary(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    mirror_root = tmp_path / "home-phench"
    source_repo = tmp_path / "source-repo"

    _init_git_repo(source_repo)

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    monkeypatch.setenv("THGENT_PHENCH_HOME_ROOT", str(mirror_root))

    init_target("beta", mode="repo")
    add_repo("beta", repo_path=str(source_repo), selected_ref="HEAD")
    lock_target("beta")
    materialize_target("beta")

    monkeypatch.setattr("shutil.which", lambda name: None)
    report = run_env_doctor_for_target("beta")
    assert report["doctor_status"] == "fail"
    assert "task" in report["missing_requirements"]


def test_sync_repairs_dual_state(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    projects_root = phenotype_root / "projects"
    mirror_root = tmp_path / "home-phench"
    source_repo = tmp_path / "source-repo"

    _init_git_repo(source_repo)

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    monkeypatch.setenv("THGENT_PHENCH_HOME_ROOT", str(mirror_root))

    init_target("gamma", mode="repo")
    add_repo("gamma", repo_path=str(source_repo), selected_ref="HEAD")
    lock_target("gamma")

    project_lock = projects_root / "gamma" / ".phench" / "target.lock.json"
    payload = json.loads(project_lock.read_text(encoding="utf-8"))
    payload["payload"]["lock_hash"] = "corrupt"
    project_lock.write_text(json.dumps(payload), encoding="utf-8")

    result = sync_target("gamma", prefer="home")
    assert "target.lock.json" in result
    repaired = json.loads(project_lock.read_text(encoding="utf-8"))
    assert repaired["payload"]["lock_hash"] != "corrupt"


def test_list_targets_discovers_initialized_targets(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    mirror_root = tmp_path / "home-phench"
    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    monkeypatch.setenv("THGENT_PHENCH_HOME_ROOT", str(mirror_root))

    init_target("one", mode="repo")
    init_target("two", mode="stack")
    assert list_targets() == ["one", "two"]


def test_list_targets_supports_family_filtering(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    mirror_root = tmp_path / "home-phench"
    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    monkeypatch.setenv("THGENT_PHENCH_HOME_ROOT", str(mirror_root))

    init_target("one", mode="repo")
    init_target("two", mode="repo", family="acme")

    assert list_targets() == ["one", "acme/two"]
    assert list_targets(family="acme") == ["two"]


def test_list_modules_lists_directory_basenames_with_manifests(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    modules_root = phenotype_root / "projects" / "modules"
    (modules_root / "thegent-app").mkdir(parents=True, exist_ok=True)
    (modules_root / "platform-core").mkdir(parents=True, exist_ok=True)
    (modules_root / "legacy").mkdir(parents=True, exist_ok=True)
    (modules_root / "legacy" / "manifest.json").write_text("{}", encoding="utf-8")
    (modules_root / "thegent-app" / "manifest.json").write_text("{}", encoding="utf-8")
    (modules_root / "platform-core" / "manifest.json").write_text("{}", encoding="utf-8")

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))

    assert list_modules() == ["legacy", "platform-core", "thegent-app"]


def test_list_modules_missing_root_returns_empty(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    assert list_modules() == []


def test_audit_shared_modules_across_repos_excludes_default_repos_and_reports_shared_candidates(tmp_path: Path) -> None:
    repos_root = tmp_path / "repos"
    alpha = repos_root / "repo-alpha"
    beta = repos_root / "repo-beta"
    excluded = repos_root / "4sgm"
    legacy = repos_root / "repo-legacy"
    for candidate in [alpha, beta, excluded, legacy]:
        _init_fake_repo(candidate)

    (alpha / "src" / "thegent-app").mkdir(parents=True, exist_ok=True)
    (alpha / "src" / "thegent-app" / "__init__.py").write_text("", encoding="utf-8")
    (alpha / "src" / "thegent-mcp").mkdir(parents=True, exist_ok=True)
    (alpha / "src" / "thegent-mcp" / "__init__.py").write_text("", encoding="utf-8")
    (alpha / "modules" / "platform-core").mkdir(parents=True, exist_ok=True)
    (alpha / "modules" / "platform-core" / "manifest.json").write_text(
        json.dumps({"schema_version": 1}),
        encoding="utf-8",
    )

    (beta / "src" / "thegent-app").mkdir(parents=True, exist_ok=True)
    (beta / "src" / "thegent-app" / "__init__.py").write_text("", encoding="utf-8")
    (beta / "modules" / "platform-core").mkdir(parents=True, exist_ok=True)
    (beta / "modules" / "platform-core" / "manifest.json").write_text(
        json.dumps({"schema_version": 1}),
        encoding="utf-8",
    )

    (excluded / "src" / "platform-core").mkdir(parents=True, exist_ok=True)
    (excluded / "src" / "platform-core" / "__init__.py").write_text("", encoding="utf-8")
    (excluded / "modules" / "platform-core").mkdir(parents=True, exist_ok=True)
    (excluded / "modules" / "platform-core" / "manifest.json").write_text(
        json.dumps({"schema_version": 1}),
        encoding="utf-8",
    )

    (legacy / "src" / "thegent-mcp").mkdir(parents=True, exist_ok=True)
    (legacy / "src" / "thegent-mcp" / "__init__.py").write_text("", encoding="utf-8")

    result = audit_shared_modules_across_repos(
        source_root=repos_root,
        include_repo_modules_root=True,
    )
    assert result["shared_module_count"] == 1
    assert result["module_count"] == 3
    assert result["shared_modules"] == {"platform-core": ["repo-alpha", "repo-beta"]}
    assert "4sgm" in result["excluded_repos"]


def test_audit_shared_modules_across_repos_respects_filters_and_min_repo_count(tmp_path: Path) -> None:
    repos_root = tmp_path / "repos"
    alpha = repos_root / "repo-alpha"
    beta = repos_root / "repo-beta"
    gamma = repos_root / "repo-gamma"
    for candidate in [alpha, beta, gamma]:
        _init_fake_repo(candidate)

    for repo in [alpha, beta]:
        (repo / "src" / "module-a").mkdir(parents=True, exist_ok=True)
        (repo / "src" / "module-a" / "__init__.py").write_text("", encoding="utf-8")

    (gamma / "src" / "module-a").mkdir(parents=True, exist_ok=True)
    (gamma / "src" / "module-a" / "__init__.py").write_text("", encoding="utf-8")
    (gamma / "src" / "module-b").mkdir(parents=True, exist_ok=True)
    (gamma / "src" / "module-b" / "__init__.py").write_text("", encoding="utf-8")

    shared_two = audit_shared_modules_across_repos(
        source_root=repos_root,
        include_modules=["module-a"],
        min_repo_count=2,
        include_repo_modules_root=False,
    )
    assert shared_two["shared_modules"] == {"module-a": ["repo-alpha", "repo-beta", "repo-gamma"]}
    assert shared_two["moduleization_candidates"] == []

    filtered = audit_shared_modules_across_repos(
        source_root=repos_root,
        include_modules=["module-a"],
        exclude_modules=["module-a"],
        min_repo_count=2,
        include_repo_modules_root=False,
    )
    assert filtered["shared_modules"] == {}


def test_audit_shared_modules_across_repos_moduleization_candidates_skip_existing_modules(
    tmp_path: Path,
) -> None:
    repos_root = tmp_path / "repos"
    phenotype_root = tmp_path / "Phenotype"
    existing_modules = phenotype_root / "projects" / "modules"
    alpha = repos_root / "repo-alpha"
    beta = repos_root / "repo-beta"
    for candidate in [alpha, beta]:
        _init_fake_repo(candidate)

    (alpha / "modules" / "thegent-app").mkdir(parents=True, exist_ok=True)
    (alpha / "modules" / "thegent-app" / "manifest.json").write_text(
        json.dumps({"schema_version": 1, "repo_ids": ["repo-alpha"]}),
        encoding="utf-8",
    )
    (alpha / "modules" / "thegent-governance").mkdir(parents=True, exist_ok=True)
    (alpha / "modules" / "thegent-governance" / "manifest.json").write_text(
        json.dumps({"schema_version": 1, "repo_ids": ["repo-alpha"]}),
        encoding="utf-8",
    )
    (beta / "modules" / "thegent-app").mkdir(parents=True, exist_ok=True)
    (beta / "modules" / "thegent-app" / "manifest.json").write_text(
        json.dumps({"schema_version": 1, "repo_ids": ["repo-beta"]}),
        encoding="utf-8",
    )

    (existing_modules / "thegent-app").mkdir(parents=True, exist_ok=True)
    (existing_modules / "thegent-app" / "manifest.json").write_text(
        json.dumps({"schema_version": 1, "repo_ids": ["existing"]}),
        encoding="utf-8",
    )

    result = audit_shared_modules_across_repos(
        source_root=repos_root,
        include_repo_modules_root=True,
        min_repo_count=2,
    )

    assert "thegent-app" in result["shared_modules"]
    assert "thegent-governance" not in result["shared_modules"]
    assert result["moduleization_candidates"] == []


def test_sync_project_modules_from_repos_invalid_manifest_raises_for_non_object_payload(tmp_path: Path) -> None:
    repos_root = tmp_path / "repos"
    destination_root = tmp_path / "Phenotype" / "projects" / "modules"
    alpha = repos_root / "repo-alpha"
    for candidate in [alpha]:
        _init_fake_repo(candidate)

    (alpha / "modules" / "thegent-app").mkdir(parents=True, exist_ok=True)
    (alpha / "modules" / "thegent-app" / "manifest.json").write_text("[]", encoding="utf-8")
    with pytest.raises(ValueError, match="invalid module manifest in repo-alpha"):
        sync_project_modules_from_repos(
            source_root=repos_root,
            destination_root=destination_root,
            include_modules=["thegent-app"],
        )


def test_sync_project_modules_from_repos_invalid_manifest_raises_for_invalid_json(tmp_path: Path) -> None:
    repos_root = tmp_path / "repos"
    destination_root = tmp_path / "Phenotype" / "projects" / "modules"
    beta = repos_root / "repo-beta"
    _init_fake_repo(beta)

    (beta / "modules" / "thegent-app").mkdir(parents=True, exist_ok=True)
    (beta / "modules" / "thegent-app" / "manifest.json").write_text("", encoding="utf-8")
    with pytest.raises(ValueError, match="invalid module manifest in repo-beta"):
        sync_project_modules_from_repos(
            source_root=repos_root,
            destination_root=destination_root,
            include_modules=["thegent-app"],
        )


def test_sync_project_modules_from_repos_dry_run_respects_default_excludes_and_module_filter(tmp_path: Path) -> None:
    repos_root = tmp_path / "repos"
    alpha = repos_root / "repo-alpha"
    beta = repos_root / "repo-beta"
    excluded = repos_root / "trace"
    for candidate in [alpha, beta, excluded]:
        _init_fake_repo(candidate)

    (alpha / "modules" / "thegent-app").mkdir(parents=True, exist_ok=True)
    (alpha / "modules" / "thegent-app" / "manifest.json").write_text(
        json.dumps({"repo_ids": ["repo-alpha"]}),
        encoding="utf-8",
    )
    (alpha / "modules" / "thegent-core").mkdir(parents=True, exist_ok=True)
    (alpha / "modules" / "thegent-core" / "manifest.json").write_text(
        json.dumps({"repo_ids": ["repo-alpha"]}),
        encoding="utf-8",
    )
    (beta / "modules" / "thegent-app").mkdir(parents=True, exist_ok=True)
    (beta / "modules" / "thegent-app" / "manifest.json").write_text(
        json.dumps({"repo_ids": ["repo-beta"]}),
        encoding="utf-8",
    )
    (excluded / "modules" / "trace-module").mkdir(parents=True, exist_ok=True)
    (excluded / "modules" / "trace-module" / "manifest.json").write_text(
        json.dumps({"repo_ids": ["trace"]}),
        encoding="utf-8",
    )

    result = sync_project_modules_from_repos(
        source_root=repos_root,
        include_modules=["thegent-app", "thegent-core"],
        dry_run=True,
    )
    assert result["dry_run"] is True
    assert result["created"] == ["thegent-app", "thegent-core"]
    assert result["updated"] == []
    assert result["skipped"] == []
    assert "trace-module" not in result["discovered_modules"]


def test_sync_project_modules_from_repos_conflict_and_overwrite_controls_updates(tmp_path: Path) -> None:
    repos_root = tmp_path / "repos"
    destination_root = tmp_path / "Phenotype" / "projects" / "modules"
    alpha = repos_root / "repo-alpha"
    beta = repos_root / "repo-beta"
    for candidate in [alpha, beta]:
        _init_fake_repo(candidate)

    (alpha / "modules" / "thegent-app").mkdir(parents=True, exist_ok=True)
    (alpha / "modules" / "thegent-app" / "manifest.json").write_text(
        json.dumps({"repo_ids": ["repo-alpha"]}),
        encoding="utf-8",
    )
    (beta / "modules" / "thegent-app").mkdir(parents=True, exist_ok=True)
    (beta / "modules" / "thegent-app" / "manifest.json").write_text(
        json.dumps({"repo_ids": ["repo-beta"]}),
        encoding="utf-8",
    )

    with pytest.raises(ValueError, match="conflicting manifests for module 'thegent-app'"):
        sync_project_modules_from_repos(
            source_root=repos_root,
            destination_root=destination_root,
            include_modules=["thegent-app"],
        )

    (beta / "modules" / "thegent-app" / "manifest.json").write_text(
        json.dumps({"repo_ids": ["repo-alpha", "repo-beta"]}),
        encoding="utf-8",
    )
    sync_result = sync_project_modules_from_repos(
        source_root=repos_root,
        destination_root=destination_root,
        include_modules=["thegent-app"],
    )
    assert sync_result["created"] == ["thegent-app"]
    assert sync_result["updated"] == []
    assert sync_result["skipped"] == []

    overwrite_result = sync_project_modules_from_repos(
        source_root=repos_root,
        destination_root=destination_root,
        include_modules=["thegent-app"],
        overwrite=True,
    )
    assert overwrite_result["updated"] == ["thegent-app"]
    assert overwrite_result["created"] == []


def test_invalid_target_name_rejected(tmp_path: Path, monkeypatch) -> None:
    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(tmp_path / "Phenotype"))
    monkeypatch.setenv("THGENT_PHENCH_HOME_ROOT", str(tmp_path / "home-phench"))
    with pytest.raises(ValueError):
        init_target("../bad", mode="repo")


def test_run_target_all_repos_serial_and_parallel(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    mirror_root = tmp_path / "home-phench"
    repo_a = tmp_path / "repo-a"
    repo_b = tmp_path / "repo-b"
    _init_git_repo(repo_a)
    _init_git_repo(repo_b)

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    monkeypatch.setenv("THGENT_PHENCH_HOME_ROOT", str(mirror_root))

    init_target("stacky", mode="stack")
    add_repo("stacky", repo_path=str(repo_a), selected_ref="HEAD", repo_id="a")
    add_repo("stacky", repo_path=str(repo_b), selected_ref="HEAD", repo_id="b")
    lock_target("stacky")
    materialize_target("stacky")
    monkeypatch.setattr(
        "thegent.phench.service.run_env_doctor_for_target",
        lambda target, family=None: {"doctor_status": "pass", "missing_requirements": []},
    )

    monkeypatch.setattr(
        "thegent.phench.service.build_runner_catalog",
        lambda target, repo_checkout: RunnerCatalog(
            target_name=target,
            runners_detected=["task"],
            commands=[RunnerCommand("task", "hello", "task hello", str(repo_checkout / "Taskfile.yml"))],
            default_command="task hello",
        ),
    )

    calls: list[str] = []

    def _fake_run_command(
        checkout: Path,
        runner: str,
        command_name: str,
        env_overrides: dict[str, str] | None = None,
    ) -> int:
        calls.append(f"{checkout.name}:{runner}:{command_name}")
        return 0

    monkeypatch.setattr("thegent.phench.service.run_command", _fake_run_command)

    serial_code = run_target("stacky", runner="task", command_name="hello", all_repos=True, execution_mode="serial")
    assert serial_code == 0
    assert len(calls) == 2

    calls.clear()
    parallel_code = run_target(
        "stacky",
        runner="task",
        command_name="hello",
        all_repos=True,
        execution_mode="parallel",
    )
    assert parallel_code == 0
    assert len(calls) == 2


def test_run_target_all_repos_requires_explicit_runner_and_command(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    mirror_root = tmp_path / "home-phench"
    repo_a = tmp_path / "repo-a"
    _init_git_repo(repo_a)

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    monkeypatch.setenv("THGENT_PHENCH_HOME_ROOT", str(mirror_root))

    init_target("stacky2", mode="stack")
    add_repo("stacky2", repo_path=str(repo_a), selected_ref="HEAD", repo_id="a")
    lock_target("stacky2")
    materialize_target("stacky2")
    monkeypatch.setattr(
        "thegent.phench.service.run_env_doctor_for_target",
        lambda target, family=None: {"doctor_status": "pass", "missing_requirements": []},
    )

    with pytest.raises(ValueError):
        run_target("stacky2", all_repos=True)


def test_run_target_with_snapshot_id_runs_from_snapshot_state(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    mirror_root = tmp_path / "home-phench"
    source_repo = tmp_path / "repo"
    _init_git_repo(source_repo)

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    monkeypatch.setenv("THGENT_PHENCH_HOME_ROOT", str(mirror_root))

    init_target("snaprun", mode="repo")
    add_repo("snaprun", repo_path=str(source_repo), selected_ref="HEAD", repo_id="repo")
    lock_target("snaprun")
    materialize_target("snaprun")
    snapshot = create_target_snapshot("snaprun")

    monkeypatch.setattr(
        "thegent.phench.service.run_env_doctor_for_target",
        lambda target, family=None: (_ for _ in ()).throw(RuntimeError("unexpected live env doctor call")),
    )
    monkeypatch.setattr(
        "thegent.phench.service._run_env_doctor_for_materializations",
        lambda target, materializations, family=None: {"doctor_status": "pass", "missing_requirements": []},
    )
    calls: list[str] = []
    monkeypatch.setattr(
        "thegent.phench.service.build_runner_catalog",
        lambda target, repo_checkout: RunnerCatalog(
            target_name=target,
            runners_detected=["task"],
            commands=[RunnerCommand("task", "hello", "task hello", str(repo_checkout / "Taskfile.yml"))],
            default_command="task hello",
        ),
    )

    def _fake_run_command(
        checkout: Path,
        runner: str,
        command_name: str,
        env_overrides: dict[str, str] | None = None,
    ) -> int:
        calls.append(str(checkout))
        return 0

    monkeypatch.setattr("thegent.phench.service.run_command", _fake_run_command)

    exit_code = run_target(
        "snaprun",
        snapshot_id=str(snapshot["snapshot_id"]),
        runner="task",
        command_name="hello",
    )
    assert exit_code == 0
    expected_checkout = str((phenotype_root / "projects" / "snaprun" / "repos" / "repo").resolve())
    assert calls == [expected_checkout]


def test_run_target_with_invalid_snapshot_runtime_fails(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    mirror_root = tmp_path / "home-phench"
    source_repo = tmp_path / "repo"
    _init_git_repo(source_repo)

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    monkeypatch.setenv("THGENT_PHENCH_HOME_ROOT", str(mirror_root))

    init_target("snapbroken", mode="repo")
    add_repo("snapbroken", repo_path=str(source_repo), selected_ref="HEAD", repo_id="repo")
    lock_target("snapbroken")
    materialize_target("snapbroken")
    snapshot = create_target_snapshot("snapbroken")
    monkeypatch.setattr(
        "thegent.phench.service.show_target_snapshot",
        lambda target, snapshot_id, family=None: {"runtime": "bad-runtime", "lock": snapshot.get("lock", {})},
    )

    with pytest.raises(ValueError, match=r"snapshot '.*' has no runtime payload"):
        run_target("snapbroken", snapshot_id=str(snapshot["snapshot_id"]))


def test_run_target_with_invalid_snapshot_lock_fails(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    mirror_root = tmp_path / "home-phench"
    source_repo = tmp_path / "repo"
    _init_git_repo(source_repo)

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    monkeypatch.setenv("THGENT_PHENCH_HOME_ROOT", str(mirror_root))

    init_target("snaplocked", mode="repo")
    add_repo("snaplocked", repo_path=str(source_repo), selected_ref="HEAD", repo_id="repo")
    lock_target("snaplocked")
    materialize_target("snaplocked")
    snapshot = create_target_snapshot("snaplocked")
    monkeypatch.setattr(
        "thegent.phench.service.show_target_snapshot",
        lambda target, snapshot_id, family=None: {
            "runtime": {"repo_materializations": [{"repo_id": "repo", "checkout_path": "repo"}]},
            "lock": "invalid-lock",
        },
    )

    with pytest.raises(ValueError, match=r"snapshot '.*' has invalid lock payload"):
        run_target("snaplocked", snapshot_id=str(snapshot["snapshot_id"]))


def test_run_target_ref_override_rematerializes_runtime_checkout(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    mirror_root = tmp_path / "home-phench"
    repo = tmp_path / "repo"
    _init_git_repo(repo)

    _run(["git", "-C", str(repo), "checkout", "-b", "feature"], cwd=repo)
    _run(["bash", "-lc", "printf '\\nfeature\\n' >> README.md"], cwd=repo)
    _run(["git", "-C", str(repo), "add", "README.md"], cwd=repo)
    _run(["git", "-C", str(repo), "commit", "-m", "feature"], cwd=repo)
    _run(["git", "-C", str(repo), "checkout", "main"], cwd=repo)

    feature_sha_proc = subprocess.run(
        ["git", "-C", str(repo), "rev-parse", "feature^{commit}"],
        capture_output=True,
        text=True,
        check=False,
    )
    assert feature_sha_proc.returncode == 0
    feature_sha = feature_sha_proc.stdout.strip()

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    monkeypatch.setenv("THGENT_PHENCH_HOME_ROOT", str(mirror_root))

    init_target("runref", mode="repo")
    add_repo("runref", repo_path=str(repo), selected_ref="main", repo_id="repo")
    lock_target("runref")
    materialize_target("runref")
    monkeypatch.setattr(
        "thegent.phench.service.run_env_doctor_for_target",
        lambda target, family=None: {"doctor_status": "pass", "missing_requirements": []},
    )

    resolved_refs: list[str] = []
    materialize_calls: list[tuple[str, str, str]] = []

    def _recorded_resolve_ref(repo_path: Path, ref: str) -> str:
        resolved_refs.append(ref)
        proc = subprocess.run(
            ["git", "-C", str(repo_path), "rev-parse", f"{ref}^{{commit}}"],
            capture_output=True,
            text=True,
            check=False,
        )
        assert proc.returncode == 0
        return proc.stdout.strip()

    monkeypatch.setattr("thegent.phench.service.resolve_ref_to_sha", _recorded_resolve_ref)
    monkeypatch.setattr(
        "thegent.phench.service.materialize_repo_checkout",
        lambda source_repo, checkout_path, resolved_sha: materialize_calls.append(
            (str(source_repo), str(checkout_path), resolved_sha),
        ),
    )
    monkeypatch.setattr(
        "thegent.phench.service.build_runner_catalog",
        lambda target, repo_checkout: RunnerCatalog(
            target_name=target,
            runners_detected=["task"],
            commands=[RunnerCommand("task", "hello", "task hello", str(repo_checkout / "Taskfile.yml"))],
            default_command="task hello",
        ),
    )

    calls: list[str] = []

    def _fake_run_command(
        checkout: Path,
        runner: str,
        command_name: str,
        env_overrides: dict[str, str] | None = None,
    ) -> int:
        calls.append(f"{checkout.name}:{command_name}:{(checkout / '.git').exists()}")
        return 0

    monkeypatch.setattr("thegent.phench.service.run_command", _fake_run_command)

    exit_code = run_target("runref", runner="task", command_name="hello", selected_ref="feature")
    assert exit_code == 0
    assert resolved_refs == ["feature"]
    assert len(materialize_calls) == 1
    assert len(calls) == 1
    assert materialize_calls == [
        (
            str(repo),
            str((phenotype_root / "projects" / "runref" / "repos" / "repo").resolve()),
            feature_sha,
        )
    ]


def test_run_target_non_interactive_requires_explicit_runner_and_command(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    mirror_root = tmp_path / "home-phench"
    repo = tmp_path / "repo"
    _init_git_repo(repo)

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    monkeypatch.setenv("THGENT_PHENCH_HOME_ROOT", str(mirror_root))

    init_target("runnon", mode="repo")
    add_repo("runnon", repo_path=str(repo), selected_ref="HEAD", repo_id="repo")
    lock_target("runnon")
    materialize_target("runnon")
    monkeypatch.setattr(
        "thegent.phench.service.run_env_doctor_for_target",
        lambda target, family=None: {"doctor_status": "pass", "missing_requirements": []},
    )

    with pytest.raises(ValueError, match="requires runner policy"):
        run_target("runnon", non_interactive=True)


def test_run_target_uses_repo_policy_runner_command_and_ref(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    mirror_root = tmp_path / "home-phench"
    repo = tmp_path / "repo-policy"
    _init_git_repo(repo)

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    monkeypatch.setenv("THGENT_PHENCH_HOME_ROOT", str(mirror_root))

    init_target("policy", mode="repo")
    add_repo(
        "policy",
        repo_path=str(repo),
        selected_ref="HEAD",
        repo_id="repo",
        preferred_runner="task",
        preferred_command="hello",
        preferred_ref="feature",
    )

    _run(["git", "-C", str(repo), "checkout", "-b", "feature"], cwd=repo)
    _run(["bash", "-lc", "printf '\\nfeature\\n' >> README.md"], cwd=repo)
    _run(["git", "-C", str(repo), "add", "README.md"], cwd=repo)
    _run(["git", "-C", str(repo), "commit", "-m", "feature"], cwd=repo)
    _run(["git", "-C", str(repo), "checkout", "main"], cwd=repo)

    lock_target("policy")
    materialize_target("policy")

    monkeypatch.setattr(
        "thegent.phench.service.run_env_doctor_for_target",
        lambda target, family=None: {"doctor_status": "pass", "missing_requirements": []},
    )

    observed_ref: list[str] = []

    def _fake_resolve_ref(repo_path: Path, ref: str) -> str:
        observed_ref.append(ref)
        proc = subprocess.run(
            ["git", "-C", str(repo_path), "rev-parse", f"{ref}^{{commit}}"],
            capture_output=True,
            text=True,
            check=False,
        )
        assert proc.returncode == 0
        return proc.stdout.strip()

    monkeypatch.setattr("thegent.phench.service.resolve_ref_to_sha", _fake_resolve_ref)
    monkeypatch.setattr(
        "thegent.phench.service.build_runner_catalog",
        lambda target, repo_checkout: RunnerCatalog(
            target_name=target,
            runners_detected=["task"],
            commands=[RunnerCommand("task", "hello", "task hello", str(repo_checkout / "Taskfile.yml"))],
            default_command="task hello",
        ),
    )

    calls: list[str] = []

    def _fake_run_command(
        checkout: Path,
        runner: str,
        command_name: str,
        env_overrides: dict[str, str] | None = None,
    ) -> int:
        calls.append(f"{runner}:{command_name}")
        return 0

    monkeypatch.setattr("thegent.phench.service.run_command", _fake_run_command)

    exit_code = run_target("policy")
    assert exit_code == 0
    assert observed_ref == ["feature"]
    assert calls == ["task:hello"]


def test_run_target_all_repos_uses_policy_runner_when_allows_single_command_mode(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    mirror_root = tmp_path / "home-phench"
    repo_a = tmp_path / "repo-a"
    repo_b = tmp_path / "repo-b"
    _init_git_repo(repo_a)
    _init_git_repo(repo_b)

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    monkeypatch.setenv("THGENT_PHENCH_HOME_ROOT", str(mirror_root))

    init_target("policy-stack", mode="stack")
    add_repo(
        "policy-stack",
        repo_path=str(repo_a),
        selected_ref="HEAD",
        repo_id="a",
        preferred_runner="task",
        preferred_command="hello",
    )
    add_repo(
        "policy-stack",
        repo_path=str(repo_b),
        selected_ref="HEAD",
        repo_id="b",
        preferred_runner="task",
        preferred_command="hello",
    )
    lock_target("policy-stack")
    materialize_target("policy-stack")

    monkeypatch.setattr(
        "thegent.phench.service.run_env_doctor_for_target",
        lambda target, family=None: {"doctor_status": "pass", "missing_requirements": []},
    )
    monkeypatch.setattr(
        "thegent.phench.service.build_runner_catalog",
        lambda target, repo_checkout: RunnerCatalog(
            target_name=target,
            runners_detected=["task"],
            commands=[RunnerCommand("task", "hello", "task hello", str(repo_checkout / "Taskfile.yml"))],
            default_command="task hello",
        ),
    )
    calls: list[str] = []

    def _fake_run_command(
        checkout: Path,
        runner: str,
        command_name: str,
        env_overrides: dict[str, str] | None = None,
    ) -> int:
        calls.append(f"{checkout.name}:{runner}:{command_name}")
        return 0

    monkeypatch.setattr("thegent.phench.service.run_command", _fake_run_command)
    exit_code = run_target("policy-stack", all_repos=True, execution_mode="serial")
    assert exit_code == 0
    assert sorted(calls) == ["a:task:hello", "b:task:hello"]


def test_run_target_rejects_runner_flag_like_command_name(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    mirror_root = tmp_path / "home-phench"
    repo = tmp_path / "repo"
    _init_git_repo(repo)

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    monkeypatch.setenv("THGENT_PHENCH_HOME_ROOT", str(mirror_root))

    init_target("delta", mode="repo")
    add_repo("delta", repo_path=str(repo), selected_ref="HEAD", repo_id="repo")
    lock_target("delta")
    materialize_target("delta")
    monkeypatch.setattr(
        "thegent.phench.service.run_env_doctor_for_target",
        lambda target, family=None: {"doctor_status": "pass", "missing_requirements": []},
    )

    with pytest.raises(ValueError):
        run_target("delta", runner="task", command_name="--help")


def test_load_module_manifest_parses_patterns_and_overrides(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    modules_dir = phenotype_root / "projects" / "modules" / "thegent-app"
    modules_dir.mkdir(parents=True, exist_ok=True)
    manifest = {
        "schema_version": 1,
        "repo_patterns": ["thegent-*", "platform-*"],
        "owners": ["OwnerA", "ownerB", "OWNERA"],
        "refresh_cadence": "daily",
        "repo_ref_overrides": {"thegent-api": "main"},
        "repo_runner_overrides": {"thegent-api": "task"},
        "repo_command_overrides": {"thegent-api": "hello"},
        "repo_env_profile_overrides": {"platform-core": "ci"},
    }
    (modules_dir / "manifest.json").write_text(json.dumps(manifest), encoding="utf-8")

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    loaded = load_module_manifest(
        "thegent-app",
        available_repo_ids=["thegent-api", "platform-core", "other"],
    )
    assert loaded["repo_ids"] == ["platform-core", "thegent-api"]
    assert loaded["owners"] == ["ownera", "ownerb"]
    assert loaded["refresh_cadence"] == "daily"
    assert loaded["repo_ref_overrides"] == {"thegent-api": "main"}
    assert loaded["repo_runner_overrides"] == {"thegent-api": "task"}
    assert loaded["repo_command_overrides"] == {"thegent-api": "hello"}
    assert loaded["repo_env_profile_overrides"] == {"platform-core": "ci"}


@pytest.mark.parametrize(
    "module_input",
    [
        "{module_dir}",
        "{module_dir}/manifest.json",
        "{legacy_module_dir}",
        "{relative_legacy_module_dir}",
    ],
)
def test_load_module_manifest_accepts_legacy_module_paths(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    module_input: str,
) -> None:
    phenotype_root = tmp_path / "Phenotype"
    legacy_modules_dir = phenotype_root / "projects" / "modules" / "thegent-app"
    modules_dir = phenotype_root / "projects" / "modules" / "legacy-layout"
    relative_modules_dir = Path("projects") / "modules" / "legacy-layout"

    modules_dir.mkdir(parents=True, exist_ok=True)
    legacy_modules_dir.mkdir(parents=True, exist_ok=True)
    manifest = {"schema_version": 1, "repo_ids": ["thegent-api"]}
    (legacy_modules_dir / "manifest.json").write_text(json.dumps(manifest), encoding="utf-8")
    (modules_dir / "manifest.json").write_text(json.dumps(manifest), encoding="utf-8")
    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))

    candidate = module_input.format(
        module_dir=modules_dir,
        legacy_module_dir=legacy_modules_dir,
        relative_legacy_module_dir=relative_modules_dir,
    )
    loaded = load_module_manifest(candidate, available_repo_ids=["thegent-api"])
    assert loaded["repo_ids"] == ["thegent-api"]


def test_load_module_manifest_rejects_unknown_repo_override(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    modules_dir = phenotype_root / "projects" / "modules" / "thegent-app"
    modules_dir.mkdir(parents=True, exist_ok=True)
    manifest = {
        "schema_version": 1,
        "repo_ids": ["thegent-api"],
        "repo_ref_overrides": {"missing-repo": "main"},
    }
    (modules_dir / "manifest.json").write_text(json.dumps(manifest), encoding="utf-8")

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    with pytest.raises(ValueError, match="unknown repo_ref_overrides key"):
        load_module_manifest("thegent-app", available_repo_ids=["thegent-api"])


def test_load_module_manifest_defaults_owners_and_refresh_cadence(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    modules_dir = phenotype_root / "projects" / "modules" / "thegent-app"
    modules_dir.mkdir(parents=True, exist_ok=True)
    manifest = {"repo_ids": ["thegent-api"]}
    (modules_dir / "manifest.json").write_text(json.dumps(manifest), encoding="utf-8")

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    loaded = load_module_manifest("thegent-app", available_repo_ids=["thegent-api"])
    assert loaded["owners"] == []
    assert loaded["refresh_cadence"] == "never"


def test_load_module_manifest_rejects_unsupported_schema_version(
    tmp_path: Path,
    monkeypatch,
) -> None:
    phenotype_root = tmp_path / "Phenotype"
    modules_dir = phenotype_root / "projects" / "modules" / "thegent-app"
    modules_dir.mkdir(parents=True, exist_ok=True)
    manifest = {"schema_version": 99, "repo_ids": ["thegent-api"]}
    (modules_dir / "manifest.json").write_text(json.dumps(manifest), encoding="utf-8")

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    with pytest.raises(ValueError, match="unsupported schema_version"):
        load_module_manifest("thegent-app", available_repo_ids=["thegent-api"])


def test_load_module_manifest_defaults_schema_version_when_missing(
    tmp_path: Path,
    monkeypatch,
) -> None:
    phenotype_root = tmp_path / "Phenotype"
    modules_dir = phenotype_root / "projects" / "modules" / "thegent-app"
    modules_dir.mkdir(parents=True, exist_ok=True)
    manifest = {"repo_ids": ["thegent-api"]}
    (modules_dir / "manifest.json").write_text(json.dumps(manifest), encoding="utf-8")

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    loaded = load_module_manifest("thegent-app", available_repo_ids=["thegent-api"])
    assert loaded["schema_version"] == 1


def test_load_module_manifest_rejects_invalid_refresh_cadence(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    modules_dir = phenotype_root / "projects" / "modules" / "thegent-app"
    modules_dir.mkdir(parents=True, exist_ok=True)
    manifest = {"repo_ids": ["thegent-api"], "refresh_cadence": "biweekly"}
    (modules_dir / "manifest.json").write_text(json.dumps(manifest), encoding="utf-8")

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    with pytest.raises(ValueError, match="must be one of"):
        load_module_manifest("thegent-app", available_repo_ids=["thegent-api"])


def test_load_module_manifest_rejects_invalid_owners_payload(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    modules_dir = phenotype_root / "projects" / "modules" / "thegent-app"
    modules_dir.mkdir(parents=True, exist_ok=True)
    manifest = {"repo_ids": ["thegent-api"], "owners": [""]}
    (modules_dir / "manifest.json").write_text(json.dumps(manifest), encoding="utf-8")

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    loaded = load_module_manifest("thegent-app", available_repo_ids=["thegent-api"])
    assert loaded["owners"] == []


def test_load_module_manifest_rejects_non_list_owners(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    modules_dir = phenotype_root / "projects" / "modules" / "thegent-app"
    modules_dir.mkdir(parents=True, exist_ok=True)
    manifest = {"repo_ids": ["thegent-api"], "owners": "thegent"}
    (modules_dir / "manifest.json").write_text(json.dumps(manifest), encoding="utf-8")

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    with pytest.raises(ValueError, match="must be a list"):
        load_module_manifest("thegent-app", available_repo_ids=["thegent-api"])


def test_run_target_respects_per_repo_env_profile_override(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    mirror_root = tmp_path / "home-phench"
    repo_a = tmp_path / "repo-a"
    repo_b = tmp_path / "repo-b"
    _init_git_repo(repo_a)
    _init_git_repo(repo_b)

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    monkeypatch.setenv("THGENT_PHENCH_HOME_ROOT", str(mirror_root))

    init_target("envprofile", mode="stack")
    add_repo(
        "envprofile",
        repo_path=str(repo_a),
        repo_id="repo-a",
        selected_ref="HEAD",
        preferred_runner="task",
        preferred_command="hello",
    )
    add_repo(
        "envprofile",
        repo_path=str(repo_b),
        repo_id="repo-b",
        selected_ref="HEAD",
        preferred_runner="task",
        preferred_command="hello",
    )
    lock_target("envprofile")
    materialize_target("envprofile")
    monkeypatch.setattr(
        "thegent.phench.service.run_env_doctor_for_target",
        lambda target, family=None: {"doctor_status": "pass", "missing_requirements": []},
    )

    set_env_profile("envprofile", "ci-a", {"ENV": "A"})
    set_env_profile("envprofile", "ci-b", {"ENV": "B"})

    observed: dict[str, dict[str, str]] = {}

    def _fake_run_command(
        checkout: Path,
        runner: str,
        command_name: str,
        env_overrides: dict[str, str] | None = None,
    ) -> int:
        observed[checkout.name] = dict(env_overrides or {})
        return 0

    monkeypatch.setattr("thegent.phench.service.run_command", _fake_run_command)
    monkeypatch.setattr(
        "thegent.phench.service.build_runner_catalog",
        lambda target, repo_checkout: RunnerCatalog(
            target_name=target,
            runners_detected=["task"],
            commands=[RunnerCommand("task", "hello", "task hello", str(repo_checkout / "Taskfile.yml"))],
            default_command="task hello",
        ),
    )

    exit_code = run_target(
        "envprofile",
        repo_ids=["repo-a", "repo-b"],
        command_name="hello",
        repo_env_profile_overrides={"repo-a": "ci-a", "repo-b": "ci-b"},
    )
    assert exit_code == 0
    assert observed["repo-a"]["ENV"] == "A"
    assert observed["repo-b"]["ENV"] == "B"


def test_read_dual_rejects_hash_mismatch(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    projects_root = phenotype_root / "projects"
    mirror_root = tmp_path / "home-phench"
    source_repo = tmp_path / "source-repo"
    _init_git_repo(source_repo)

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    monkeypatch.setenv("THGENT_PHENCH_HOME_ROOT", str(mirror_root))

    init_target("epsilon", mode="repo")
    add_repo("epsilon", repo_path=str(source_repo), selected_ref="HEAD")
    lock_target("epsilon")

    project_lock = projects_root / "epsilon" / ".phench" / "target.lock.json"
    mirror_lock = mirror_root / "epsilon" / ".phench" / "target.lock.json"
    payload = json.loads(project_lock.read_text(encoding="utf-8"))
    payload["payload"]["lock_hash"] = "corrupt"
    project_lock.write_text(json.dumps(payload), encoding="utf-8")
    mirror_lock.write_text(json.dumps(payload), encoding="utf-8")

    with pytest.raises(FileNotFoundError):
        read_dual("epsilon", "target.lock.json")


def test_env_profile_applies_to_run(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    mirror_root = tmp_path / "home-phench"
    repo = tmp_path / "repo-env"
    _init_git_repo(repo)

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    monkeypatch.setenv("THGENT_PHENCH_HOME_ROOT", str(mirror_root))

    init_target("zeta", mode="repo")
    add_repo("zeta", repo_path=str(repo), selected_ref="HEAD", repo_id="repo")
    lock_target("zeta")
    materialize_target("zeta")
    monkeypatch.setattr(
        "thegent.phench.service.run_env_doctor_for_target",
        lambda target, family=None: {"doctor_status": "pass", "missing_requirements": []},
    )

    set_env_profile("zeta", "ci", {"FOO": "BAR"})
    assert get_env_profile("zeta", "ci") == {"FOO": "BAR"}

    observed: dict[str, str] = {}

    def _fake_run_command(
        checkout: Path,
        runner: str,
        command_name: str,
        env_overrides: dict[str, str] | None = None,
    ) -> int:
        if env_overrides:
            observed.update(env_overrides)
        return 0

    monkeypatch.setattr("thegent.phench.service.run_command", _fake_run_command)
    run_target("zeta", runner="task", command_name="hello", env_profile="ci")
    assert observed == {"FOO": "BAR"}


def test_add_module_to_target_appends_matching_repos(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    mirror_root = tmp_path / "home-phench"
    repo_a = phenotype_root / "repos" / "repo-a"
    repo_b = phenotype_root / "repos" / "repo-b"
    repo_x = phenotype_root / "repos" / "other-repo"
    _init_git_repo(repo_a)
    _init_git_repo(repo_b)
    _init_git_repo(repo_x)

    module_root = phenotype_root / "projects" / "modules" / "thegent-test-module"
    module_root.mkdir(parents=True, exist_ok=True)
    (module_root / "manifest.json").write_text(
        json.dumps(
            {
                "schema_version": 1,
                "repo_patterns": ["repo-*"],
                "default_ref": "HEAD",
                "repo_ref_overrides": {"repo-a": "HEAD"},
                "repo_runner_overrides": {"repo-a": "task"},
                "repo_command_overrides": {"repo-a": "hello"},
                "repo_env_profile_overrides": {"repo-b": "ci"},
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    monkeypatch.setenv("THGENT_PHENCH_HOME_ROOT", str(mirror_root))

    init_target("module-target", mode="stack")
    lock = add_module_to_target("module-target", "thegent-test-module")
    repos = {repo.repo_id: repo for repo in lock.repos}
    assert set(repos.keys()) == {"repo-a", "repo-b"}
    assert repos["repo-a"].selected_ref == "HEAD"
    assert repos["repo-a"].selected_runner == "task"
    assert repos["repo-a"].selected_command == "hello"
    assert repos["repo-b"].selected_env_profile == "ci"


def test_add_module_to_target_fails_when_manifest_missing(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    mirror_root = tmp_path / "home-phench"

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    monkeypatch.setenv("THGENT_PHENCH_HOME_ROOT", str(mirror_root))

    init_target("module-missing", mode="stack")
    with pytest.raises(ValueError, match="manifest not found"):
        add_module_to_target("module-missing", "no-such-module")


def test_add_module_to_target_fails_when_no_repos_match_patterns(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    mirror_root = tmp_path / "home-phench"
    repo_a = phenotype_root / "repos" / "repo-a"
    _init_git_repo(repo_a)

    module_root = phenotype_root / "projects" / "modules" / "thegent-empty-module"
    module_root.mkdir(parents=True, exist_ok=True)
    (module_root / "manifest.json").write_text(
        json.dumps(
            {
                "schema_version": 1,
                "repo_patterns": ["does-not-match-*"],
                "default_ref": "HEAD",
                "repo_ref_overrides": {},
                "repo_runner_overrides": {},
                "repo_command_overrides": {},
                "repo_env_profile_overrides": {},
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    monkeypatch.setenv("THGENT_PHENCH_HOME_ROOT", str(mirror_root))

    init_target("module-empty", mode="stack")
    with pytest.raises(ValueError, match="selected no matching repos"):
        add_module_to_target("module-empty", "thegent-empty-module")


def test_add_module_to_target_uses_default_excludes(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    mirror_root = tmp_path / "home-phench"
    repo_a = phenotype_root / "repos" / "repo-a"
    excluded = [
        phenotype_root / "repos" / "4sgm",
        phenotype_root / "repos" / "parpour",
        phenotype_root / "repos" / "civ",
        phenotype_root / "repos" / "trace",
    ]
    _init_git_repo(repo_a)
    for repo in excluded:
        _init_git_repo(repo)

    module_root = phenotype_root / "projects" / "modules" / "thegent-default-excludes"
    module_root.mkdir(parents=True, exist_ok=True)
    (module_root / "manifest.json").write_text(
        json.dumps(
            {
                "schema_version": 1,
                "repo_patterns": ["*"],
                "default_ref": "HEAD",
                "repo_ref_overrides": {},
                "repo_runner_overrides": {},
                "repo_command_overrides": {},
                "repo_env_profile_overrides": {},
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    monkeypatch.setenv("THGENT_PHENCH_HOME_ROOT", str(mirror_root))

    init_target("module-excludes", mode="stack")
    lock = add_module_to_target("module-excludes", "thegent-default-excludes")
    assert [entry.repo_id for entry in lock.repos] == ["repo-a"]


def test_add_module_respects_repo_and_global_excludes(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    mirror_root = tmp_path / "home-phench"
    repo_a = phenotype_root / "repos" / "repo-a"
    repo_b = phenotype_root / "repos" / "repo-b"
    excluded = phenotype_root / "repos" / "repo-ex"
    _init_git_repo(repo_a)
    _init_git_repo(repo_b)
    _init_git_repo(excluded)

    module_root = phenotype_root / "projects" / "modules" / "thegent-test-module-ex"
    module_root.mkdir(parents=True, exist_ok=True)
    (module_root / "manifest.json").write_text(
        json.dumps(
            {
                "schema_version": 1,
                "repo_patterns": ["repo-*"],
                "default_ref": "HEAD",
                "repo_ref_overrides": {},
                "repo_runner_overrides": {},
                "repo_command_overrides": {},
                "repo_env_profile_overrides": {},
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    monkeypatch.setenv("THGENT_PHENCH_HOME_ROOT", str(mirror_root))

    init_target("module-target-2", mode="stack")
    lock = add_module_to_target("module-target-2", "thegent-test-module-ex", exclude_repos={"repo-b"})
    assert {entry.repo_id for entry in lock.repos} == {"repo-a", "repo-ex"}


def test_run_target_uses_module_overrides(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    mirror_root = tmp_path / "home-phench"
    repo = phenotype_root / "repos" / "repo-run"
    _init_git_repo(repo)

    module_root = phenotype_root / "projects" / "modules" / "thegent-run-module"
    module_root.mkdir(parents=True, exist_ok=True)
    (module_root / "manifest.json").write_text(
        json.dumps(
            {
                "schema_version": 1,
                "repo_patterns": ["repo-*"],
                "default_ref": "HEAD",
                "repo_ref_overrides": {},
                "repo_runner_overrides": {"repo-run": "task"},
                "repo_command_overrides": {"repo-run": "hello"},
                "repo_env_profile_overrides": {"repo-run": "ci"},
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    monkeypatch.setenv("THGENT_PHENCH_HOME_ROOT", str(mirror_root))

    init_target("module-run", mode="repo")
    add_module_to_target("module-run", "thegent-run-module")
    lock_target("module-run")
    materialize_target("module-run")
    set_env_profile("module-run", "ci", {"FROM_PROFILE": "1"})
    monkeypatch.setattr(
        "thegent.phench.service.run_env_doctor_for_target",
        lambda target: {"doctor_status": "pass", "missing_requirements": []},
    )
    monkeypatch.setattr(
        "thegent.phench.service.build_catalog",
        lambda target, repo_id=None: RunnerCatalog(
            target_name=target,
            runners_detected=["task"],
            commands=[RunnerCommand("task", "hello", "task hello", "Taskfile.yml")],
            default_command="task hello",
        ),
    )

    observed: dict[str, str] = {}

    def _fake_run_command(
        checkout: Path,
        runner: str,
        command_name: str,
        env_overrides: dict[str, str] | None = None,
    ) -> int:
        observed["runner"] = runner
        observed["command_name"] = command_name
        if env_overrides:
            observed.update(env_overrides)
        return 0

    monkeypatch.setattr("thegent.phench.service.run_command", _fake_run_command)
    exit_code = run_target("module-run")
    assert exit_code == 0
    assert observed["runner"] == "task"
    assert observed["command_name"] == "hello"
    assert observed["FROM_PROFILE"] == "1"


def test_audit_shared_modules(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    mirror_root = tmp_path / "home-phench"
    repo_a = tmp_path / "repo-a"
    repo_b = tmp_path / "repo-b"
    _init_git_repo_with_pkg(repo_a, "sharedpkg")
    _init_git_repo_with_pkg(repo_b, "sharedpkg")

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    monkeypatch.setenv("THGENT_PHENCH_HOME_ROOT", str(mirror_root))
    init_target("audity", mode="stack")
    add_repo("audity", repo_path=str(repo_a), selected_ref="HEAD", repo_id="a")
    add_repo("audity", repo_path=str(repo_b), selected_ref="HEAD", repo_id="b")

    result = audit_shared_modules("audity")
    assert result["shared_modules"]["sharedpkg"] == ["a", "b"]


def test_scan_shared_modules_across_repos_respects_excludes_and_minimum_repo_count(tmp_path: Path) -> None:
    phenotype_root = tmp_path / "Phenotype"
    repos_root = phenotype_root / "repos"
    repo_a = repos_root / "repo-a"
    repo_b = repos_root / "repo-b"
    repo_c = repos_root / "repo-c"
    repo_d = phenotype_root / "repos" / "4sgm"

    _init_git_repo_with_pkg(repo_a, "sharedpkg")
    _init_git_repo_with_pkg(repo_a, "exclusive-a")
    _init_git_repo_with_pkg(repo_b, "sharedpkg")
    _init_git_repo_with_pkg(repo_b, "bothrepos")
    _init_git_repo_with_pkg(repo_c, "sharedpkg")
    _init_git_repo_with_pkg(repo_c, "bothrepos")
    _init_git_repo_with_pkg(repo_d, "sharedpkg")

    result = scan_shared_modules_across_repos(
        repos_root=repos_root,
        exclude_repos={"repo-a"},
        min_repo_count=2,
    )
    assert result["shared_modules"] == {"bothrepos": ["repo-b", "repo-c"], "sharedpkg": ["repo-b", "repo-c"]}

    min3 = scan_shared_modules_across_repos(repos_root=repos_root, min_repo_count=3)
    assert min3["shared_modules"] == {"sharedpkg": ["repo-a", "repo-b", "repo-c"]}

    result_with_default = scan_shared_modules_across_repos(repos_root=repos_root, exclude_repos=set())
    assert "sharedpkg" in result_with_default["shared_modules"]
    assert "4sgm" not in result_with_default["examined_repos"]
    assert result_with_default["scan_schema_version"] == 1


def test_scan_shared_modules_across_repos_validates_exclude_ids(tmp_path: Path) -> None:
    phenotype_root = tmp_path / "Phenotype"
    repos_root = phenotype_root / "repos"
    repo_a = repos_root / "repo-a"

    _init_git_repo_with_pkg(repo_a, "sharedpkg")

    with pytest.raises(ValueError, match="exclude repo id cannot be blank"):
        scan_shared_modules_across_repos(
            repos_root=repos_root,
            exclude_repos={"   "},
        )

    with pytest.raises(ValueError, match="invalid repo id: repo/a"):
        scan_shared_modules_across_repos(
            repos_root=repos_root,
            exclude_repos={"repo/a"},
        )


def test_scan_shared_modules_across_repos_supports_worktree_root_mode(tmp_path: Path) -> None:
    phenotype_root = tmp_path / "Phenotype"
    worktrees_root = phenotype_root / "thegent-wtrees"
    repo_a = worktrees_root / "worktree-a"
    repo_b = worktrees_root / "worktree-b"
    _init_git_repo_with_pkg(repo_a, "sharedpkg")
    _init_git_repo_with_pkg(repo_b, "sharedpkg")

    result = scan_shared_modules_across_repos(
        repos_root=worktrees_root,
        min_repo_count=2,
        repos_root_mode="worktrees",
    )
    assert result["repos_root_mode"] == "worktrees"
    assert result["shared_modules"] == {"sharedpkg": ["worktree-a", "worktree-b"]}


def test_build_scan_candidates_generates_sorted_overlapping_candidates() -> None:
    candidates = build_scan_candidates(
        {
            "alpha": ["repo-c", "repo-a", "repo-b"],
            "zeta": ["repo-a", "repo-b"],
            "beta": ["repo-b", "repo-a", "repo-c"],
        },
        module_prefix="scanmod",
    )
    assert [item["module_name"] for item in candidates] == [
        "scanmod-alpha-3",
        "scanmod-beta-3",
        "scanmod-zeta-2",
    ]
    assert candidates[0]["manifest_template"]["repo_patterns"] == ["repo-a", "repo-b", "repo-c"]
    assert candidates[0]["repo_count"] == 3
    assert len(candidates[0]["module_name"]) <= 60


def test_build_module_manifest_payload_has_sorted_repo_patterns() -> None:
    payload = build_module_manifest_payload("alpha", ["z", "a", "m"])
    assert payload["repo_patterns"] == ["a", "m", "z"]
    assert payload["matched_repos"] == ["a", "m", "z"]


def test_materialize_module_candidate_manifest_honors_repo_pinning_and_dry_run(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    repos_root = phenotype_root / "repos"
    repo_a = repos_root / "repo-a"
    repo_b = repos_root / "repo-b"
    repo_c = repos_root / "repo-c"
    _init_git_repo_with_pkg(repo_a, "sharedpkg")
    _init_git_repo_with_pkg(repo_b, "sharedpkg")
    _init_git_repo_with_pkg(repo_c, "sharedpkg")

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    result = materialize_module_candidate_manifest(
        "sharedpkg",
        repos_root=repos_root,
        repos=["repo-a", "repo-c"],
        min_repo_count=2,
        output_dir=tmp_path / "modules-out",
        dry_run=True,
    )
    assert result["dry_run"] is True
    assert result["repos"] == ["repo-a", "repo-c"]
    assert result["manifest_payload"]["repo_patterns"] == ["repo-a", "repo-c"]
    assert result["manifest_path"].endswith("shared-module-sharedpkg-2/manifest.json")


def test_materialize_module_candidate_manifest_respects_existing_manifest_idempotent(
    tmp_path: Path, monkeypatch
) -> None:
    phenotype_root = tmp_path / "Phenotype"
    repos_root = phenotype_root / "repos"
    repo_a = repos_root / "repo-a"
    repo_b = repos_root / "repo-b"
    _init_git_repo_with_pkg(repo_a, "sharedpkg")
    _init_git_repo_with_pkg(repo_b, "sharedpkg")

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    output_dir = tmp_path / "modules-out"
    first = materialize_module_candidate_manifest(
        "sharedpkg",
        repos_root=repos_root,
        output_dir=output_dir,
        min_repo_count=2,
    )
    second = materialize_module_candidate_manifest(
        "sharedpkg",
        repos_root=repos_root,
        output_dir=output_dir,
        min_repo_count=2,
    )
    assert first["manifest_path"] == second["manifest_path"]
    assert second["manifest_path"] == str(output_dir / "shared-module-sharedpkg-2" / "manifest.json")
    assert not second["dry_run"]


def test_materialize_module_candidate_manifest_filters_explicit_pins_for_default_excludes(
    tmp_path: Path, monkeypatch
) -> None:
    phenotype_root = tmp_path / "Phenotype"
    repos_root = phenotype_root / "repos"
    repo_a = repos_root / "repo-a"
    repo_b = repos_root / "repo-b"
    repo_excluded = repos_root / "4sgm"
    _init_git_repo_with_pkg(repo_a, "sharedpkg")
    _init_git_repo_with_pkg(repo_b, "sharedpkg")
    _init_git_repo_with_pkg(repo_excluded, "sharedpkg")

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    with pytest.raises(ValueError, match="has insufficient pinned repos after filtering"):
        materialize_module_candidate_manifest(
            "sharedpkg",
            repos_root=repos_root,
            repos=["4sgm", "repo-a"],
            min_repo_count=2,
            output_dir=tmp_path / "modules-out",
        )


def test_scan_shared_modules_across_repos_keeps_recommendations_when_omit_candidates_and_applies_regex(
    tmp_path: Path,
) -> None:
    phenotype_root = tmp_path / "Phenotype"
    repos_root = phenotype_root / "repos"
    repo_a = repos_root / "repo-a"
    repo_b = repos_root / "repo-b"
    _init_git_repo_with_pkg(repo_a, "alpha")
    _init_git_repo_with_pkg(repo_a, "beta")
    (repo_a / "src" / "alpha" / "__init__.py").write_text("import beta\n", encoding="utf-8")
    _init_git_repo_with_pkg(repo_b, "alpha")
    _init_git_repo_with_pkg(repo_b, "beta")
    (repo_b / "src" / "alpha" / "__init__.py").write_text("import beta\n", encoding="utf-8")

    full = scan_shared_modules_across_repos(
        repos_root=repos_root,
        min_repo_count=2,
        candidate_name_regex="^alp.*",
        omit_candidates=True,
    )
    assert [item["module"] for item in full["recommended_modules"]] == ["alpha", "beta"]
    assert full["recommended_modules"][0]["depends_on_count"] == 1
    assert full["recommended_modules"][0]["depends_on"] == ["beta"]
    assert full["module_candidates"] == []
    assert any("candidate-name-regex ignored when candidates are omitted" in warning for warning in full["warnings"])

    filtered = scan_shared_modules_across_repos(
        repos_root=repos_root,
        min_repo_count=2,
        candidate_name_regex="^alp.*",
        omit_candidates=False,
    )
    assert [item["module"] for item in filtered["module_candidates"]] == ["alpha"]


def test_scan_shared_modules_across_repos_reports_root_mode_hint_and_repo_paths(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    repos_root = phenotype_root / "repos"
    repo_a = repos_root / "repo-a"
    repo_b = repos_root / "repo-b"
    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    _init_git_repo_with_pkg(repo_a, "sharedpkg")
    _init_git_repo_with_pkg(repo_b, "sharedpkg")
    result = scan_shared_modules_across_repos(
        repos_root=None,
        min_repo_count=2,
    )
    assert "defaulted repos_root" in result["root_mode_hint"]
    assert result["repo_paths"] == {
        "repo-a": str(repo_a),
        "repo-b": str(repo_b),
    }
    assert result["warnings"] == []


def test_scan_shared_modules_handles_nested_src_worktree_root(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    worktree_root = phenotype_root / "thegent-wtrees" / "repo-a"
    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    _init_git_repo_with_pkg(worktree_root, "sharedpkg")

    nested_src = worktree_root / "src"
    result = scan_shared_modules_across_repos(
        repos_root=nested_src,
        repos_root_mode="worktrees",
        min_repo_count=2,
    )
    assert result["shared_modules"] == {}


def test_build_scan_candidates_collision_safe_manifest_names() -> None:
    candidates = build_scan_candidates(
        {
            "x" * 80: ["repo-a", "repo-b"],
            "x" * 80 + "y": ["repo-a", "repo-c"],
        },
        module_prefix="shared-module",
    )
    assert len(candidates) == 2
    names = [candidate["module_name"] for candidate in candidates]
    assert len(set(names)) == len(names)
    assert len(names[0]) <= 60
    assert names[1] != names[0]
    assert len(names[1]) <= 60
    assert names[1].count("-") >= 2


def test_materialize_module_candidate_manifest_includes_index_and_audit_payload(tmp_path: Path, monkeypatch) -> None:
    phenotype_root = tmp_path / "Phenotype"
    repos_root = phenotype_root / "repos"
    repo_a = repos_root / "repo-a"
    repo_b = repos_root / "repo-b"
    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    _init_git_repo_with_pkg(repo_a, "sharedpkg")
    _init_git_repo_with_pkg(repo_b, "sharedpkg")
    output_dir = tmp_path / "modules-out"

    monkeypatch.setenv("THGENT_PHENOTYPE_ROOT", str(phenotype_root))
    dry_run = materialize_module_candidate_manifest(
        "sharedpkg",
        repos_root=repos_root,
        min_repo_count=2,
        output_dir=output_dir,
        dry_run=True,
    )
    assert dry_run["dry_run"] is True
    assert dry_run["manifest_after"] == dry_run["manifest_payload"]
    assert dry_run["index_before"] == []
    assert dry_run["index_after"] == [
        {
            "module_name": "shared-module-sharedpkg-2",
            "manifest_path": str(output_dir / "shared-module-sharedpkg-2" / "manifest.json"),
            "repo_count": 2,
            "generated_at": dry_run["index_after"][0]["generated_at"],
        }
    ]

    committed = materialize_module_candidate_manifest(
        "sharedpkg",
        repos_root=repos_root,
        min_repo_count=2,
        output_dir=output_dir,
        dry_run=False,
    )
    assert committed["index_summary"]["manifest_count"] == 1
    assert (output_dir / "index-summary.json").exists()
    audit = output_dir / "manifest-audit.jsonl"
    assert audit.exists()
    logs = [json.loads(line) for line in audit.read_text(encoding="utf-8").splitlines() if line]
    assert logs and logs[-1]["module"] == "sharedpkg"


def _load_phench_cli_app(path: Path):
    spec = importlib.util.spec_from_file_location(
        f"phench_cli_module_{str(path).replace('/', '_').replace('.', '_')}",
        path,
    )
    if spec is None or spec.loader is None:
        raise RuntimeError("failed to load phench cli module")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def test_scan_shared_modules_across_repos_recommends_sorted_by_overlap(tmp_path: Path) -> None:
    phenotype_root = tmp_path / "Phenotype"
    repos_root = phenotype_root / "repos"
    repo_a = repos_root / "repo-a"
    repo_b = repos_root / "repo-b"
    repo_c = repos_root / "repo-c"
    _init_git_repo_with_pkg(repo_a, "alpha")
    _init_git_repo_with_pkg(repo_a, "beta")
    _init_git_repo_with_pkg(repo_b, "alpha")
    _init_git_repo_with_pkg(repo_b, "beta")
    _init_git_repo_with_pkg(repo_b, "gamma")
    _init_git_repo_with_pkg(repo_b, "shared")
    _init_git_repo_with_pkg(repo_c, "gamma")
    _init_git_repo_with_pkg(repo_c, "alpha")
    _init_git_repo_with_pkg(repo_c, "shared")

    result = scan_shared_modules_across_repos(
        repos_root=repos_root,
        min_repo_count=2,
    )
    assert [item["module"] for item in result["recommended_modules"]] == ["alpha", "beta", "gamma", "shared"]
    assert result["recommended_modules"][0]["repo_count"] == 3


def test_scan_shared_repos_cli_candidates_now_sorted_by_overlap_and_schema(tmp_path: Path, monkeypatch) -> None:
    from typer.testing import CliRunner

    phench_cli = _load_phench_cli_app(Path(__file__).resolve().parents[1] / "src/thegent/cli/apps/phench.py")

    payload = {
        "repos_root": str(tmp_path / "Phenotype" / "repos"),
        "shared_modules": {
            "alpha": ["repo-c", "repo-a"],
            "beta": ["repo-a", "repo-b", "repo-c"],
            "zeta": ["repo-c", "repo-d"],
        },
        "shared_count": 3,
        "module_count": 3,
        "repo_count": 4,
        "excluded_repos": ["4sgm", "parpour"],
        "examined_repos": ["repo-a", "repo-b", "repo-c", "repo-d"],
        "min_repo_count": 2,
    }
    monkeypatch.setattr(
        phench_cli,
        "scan_shared_modules_across_repos",
        lambda **kwargs: payload,
    )
    monkeypatch.setattr(
        phench_cli,
        "build_scan_candidates",
        lambda shared_modules, module_prefix="shared-module": [
            {
                "module": "beta",
                "module_name": "shared-module-beta-3",
                "repo_ids": ["repo-a", "repo-b", "repo-c"],
                "repo_count": 3,
                "manifest_template": {},
            },
            {
                "module": "alpha",
                "module_name": "shared-module-alpha-2",
                "repo_ids": ["repo-a", "repo-c"],
                "repo_count": 2,
                "manifest_template": {},
            },
            {
                "module": "zeta",
                "module_name": "shared-module-zeta-2",
                "repo_ids": ["repo-c", "repo-d"],
                "repo_count": 2,
                "manifest_template": {},
            },
        ],
    )
    result = CliRunner().invoke(
        phench_cli.app,
        ["scan-shared-repos", "--repos-root", str(tmp_path / "Phenotype" / "repos"), "--candidates"],
    )
    assert result.exit_code == 0
    output = json.loads(result.output)
    assert output["module_candidates"][0]["module"] == "beta"
    assert output["module_candidates"][1]["repo_count"] == 2


def test_scan_shared_modules_cli_command(tmp_path: Path, monkeypatch) -> None:
    from typer.testing import CliRunner

    phench_cli = _load_phench_cli_app(Path(__file__).resolve().parents[1] / "src/thegent/cli/apps/phench.py")

    monkeypatch.setattr(
        phench_cli,
        "scan_shared_modules_across_repos",
        lambda **kwargs: {
            "repos_root": str(kwargs.get("repos_root", "/tmp/repos")),
            "shared_modules": {"sharedpkg": ["repo-a", "repo-b"]},
            "shared_count": 1,
            "module_count": 2,
            "repo_count": 2,
            "excluded_repos": ["4sgm", "parpour"],
            "examined_repos": ["repo-a", "repo-b"],
            "min_repo_count": 2,
        },
    )

    result = CliRunner().invoke(
        phench_cli.app,
        [
            "scan-shared-repos",
            "--repos-root",
            str(tmp_path / "Phenotype" / "repos"),
            "--repos-root-mode",
            "repos",
            "--exclude",
            "repo-b",
            "--min-repos",
            "2",
            "--candidates",
        ],
    )
    assert result.exit_code == 0
    payload = json.loads(result.output)
    assert payload["repos_root"] == str(tmp_path / "Phenotype" / "repos")
    assert payload["shared_modules"] == {"sharedpkg": ["repo-a", "repo-b"]}
    assert payload["module_candidates"][0]["module"] == "sharedpkg"


def test_materialize_module_manifest_cli_command(tmp_path: Path, monkeypatch) -> None:
    from typer.testing import CliRunner

    phench_cli = _load_phench_cli_app(Path(__file__).resolve().parents[1] / "src/thegent/cli/apps/phench.py")

    def _fake_materialize_module_candidate_manifest(
        module: str,
        repos_root: Path | None = None,
        repos_root_mode: str | None = None,
        repos: list[str] | None = None,
        min_repo_count: int = 2,
        output_dir: Path | None = None,
        dry_run: bool = False,
    ) -> dict[str, object]:
        return {
            "module": module,
            "module_name": "shared-module-sharedpkg-2",
            "repos": ["repo-a", "repo-b"],
            "manifest_path": "/tmp/modules/shared-module-sharedpkg-2/manifest.json",
            "manifest_payload": {},
            "dry_run": False,
        }

    monkeypatch.setattr(
        phench_cli,
        "materialize_module_candidate_manifest",
        _fake_materialize_module_candidate_manifest,
    )
    result = CliRunner().invoke(
        phench_cli.app,
        [
            "materialize-module-manifest",
            "--module",
            "sharedpkg",
            "--repos-root-mode",
            "repos",
            "--repo",
            "repo-a",
            "--print-snippets",
        ],
    )
    assert result.exit_code == 0
    payload = json.loads(result.output)
    assert payload["module"] == "sharedpkg"
    assert payload["module_name"] == "shared-module-sharedpkg-2"
    assert payload["shell_snippets"][0] == "thegent phench target init shared-module-sharedpkg-2 --mode stack"


@pytest.mark.parametrize(
    "cli_path",
    [
        ("src/thegent/cli/apps/phench.py", "package"),
        ("src/thegent/cli/apps/phench.py", "source"),
    ],
)
def test_phench_cli_scan_shared_repos_accepts_regex_and_omit_candidates(
    tmp_path: Path, monkeypatch, cli_path: tuple[str, str]
) -> None:
    from typer.testing import CliRunner

    path, _label = cli_path
    cli_module_path = Path(__file__).resolve().parents[1] / path
    phench_cli = _load_phench_cli_app(cli_module_path)

    captured: dict[str, object] = {}

    def _fake_scan_shared_modules_across_repos(**kwargs) -> dict[str, object]:
        captured.update(kwargs)
        return {
            "scan_schema_version": 1,
            "repos_root": str(kwargs.get("repos_root")),
            "repos_root_mode": kwargs.get("repos_root_mode", "repos"),
            "root_mode_hint": "captured",
            "warnings": [],
            "repo_paths": {},
            "shared_modules": {"alpha": ["repo-a", "repo-b"]},
            "shared_count": 1,
            "module_count": 2,
            "repo_count": 2,
            "excluded_repos": [],
            "examined_repos": ["repo-a", "repo-b"],
            "min_repo_count": kwargs.get("min_repo_count", 2),
            "module_candidates": [],
            "recommended_modules": [],
        }

    monkeypatch.setattr(phench_cli, "scan_shared_modules_across_repos", _fake_scan_shared_modules_across_repos)
    result = CliRunner().invoke(
        phench_cli.app,
        [
            "scan-shared-repos",
            "--repos-root",
            str(tmp_path / "Phenotype" / "repos"),
            "--omit-candidates",
            "--candidate-name-regex",
            "^alpha$",
            "--candidates",
        ],
    )
    assert result.exit_code == 0
    output = json.loads(result.output)
    assert output["module_candidates"] == []
    assert captured["omit_candidates"] is True
    assert captured["candidate_name_regex"] == "^alpha$"


@pytest.mark.parametrize(
    ("cli_path", "snippet_flag"),
    [
        ("src/thegent/cli/apps/phench.py", "--print-target-snippets"),
    ],
)
def test_materialize_module_manifest_cli_print_target_snippet_alias(
    tmp_path: Path, monkeypatch, cli_path: str, snippet_flag: str
) -> None:
    from typer.testing import CliRunner

    phench_cli = _load_phench_cli_app(Path(__file__).resolve().parents[1] / cli_path)

    def _fake_materialize_module_candidate_manifest(
        module: str,
        repos_root: Path | None = None,
        repos_root_mode: str | None = None,
        repos: list[str] | None = None,
        min_repo_count: int = 2,
        output_dir: Path | None = None,
        dry_run: bool = False,
    ) -> dict[str, object]:
        return {
            "module": module,
            "module_name": "shared-module-sharedpkg-2",
            "repos": ["repo-a", "repo-b"],
            "manifest_path": "/tmp/modules/shared-module-sharedpkg-2/manifest.json",
            "manifest_payload": {},
            "dry_run": False,
        }

    monkeypatch.setattr(
        phench_cli, "materialize_module_candidate_manifest", _fake_materialize_module_candidate_manifest
    )
    result = CliRunner().invoke(
        phench_cli.app,
        [
            "materialize-module-manifest",
            "--module",
            "sharedpkg",
            snippet_flag,
        ],
    )
    assert result.exit_code == 0
    payload = json.loads(result.output)
    assert payload["shell_snippets"][0].startswith("thegent phench target init shared-module-sharedpkg-2")


def test_cli_target_add_module_cmd_invokes_service(monkeypatch) -> None:
    import importlib.util

    from typer.testing import CliRunner

    cli_module_path = Path(__file__).resolve().parents[1] / "src/thegent/cli/apps/phench.py"
    spec = importlib.util.spec_from_file_location("phench_cli_target_module", cli_module_path)
    if spec is None or spec.loader is None:
        raise RuntimeError("failed to load phench cli module")
    phench_cli = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(phench_cli)

    observed: dict[str, str | set[str] | None] = {"name": None, "module": None, "ref": None, "exclude": set()}

    def _fake_add_module_to_target(
        name: str,
        module_name: str,
        selected_ref: str | None = None,
        exclude_repos: set[str] | None = None,
    ) -> TargetLock:
        observed["name"] = name
        observed["module"] = module_name
        observed["ref"] = selected_ref
        observed["exclude"] = exclude_repos or set()
        return TargetLock(
            schema_version=1,
            target_name=name,
            mode="stack",
            repos=[
                RepoSelection(
                    repo_id="repo-a",
                    repo_path="/tmp/repo-a",
                    selected_ref="HEAD",
                    module_name=module_name,
                )
            ],
            lock_hash="abc",
        )

    monkeypatch.setattr(phench_cli, "add_module_to_target", _fake_add_module_to_target)
    result = CliRunner().invoke(
        phench_cli.app,
        ["target", "add-module", "smoke", "--module", "thegent-app", "--ref", "dev", "--exclude", "skip-me"],
    )
    assert result.exit_code == 0
    assert observed["name"] == "smoke"
    assert observed["module"] == "thegent-app"
    assert observed["ref"] == "dev"
    assert observed["exclude"] == {"skip-me"}
    payload = json.loads(result.output)
    assert payload["target"] == "smoke"
    assert payload["module"] == "thegent-app"
    assert payload["repos"] == ["repo-a"]
