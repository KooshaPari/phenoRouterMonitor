"""Project tenancy and AG-DD template orchestration."""

from __future__ import annotations

import json
import shutil
import uuid
from dataclasses import dataclass, field
from datetime import UTC, datetime
from pathlib import Path
from typing import Literal

from pydantic import BaseModel, ConfigDict, Field

from thegent.registry.project_registry import ProjectRegistry

TemplateMode = Literal["smart", "overwrite", "skip"]

_DEFAULT_REGISTRY_PATH = Path.home() / ".thegent" / "projects" / "registry.json"
_DEFAULT_TEMPLATE = "ag-dd"
_DEFAULT_TEMPLATE_VERSION = "1.0.0"


def _utc_now() -> str:
    return datetime.now(UTC).isoformat()


def _new_project_id() -> str:
    return uuid.uuid4().hex[:12]


class TenancyProject(BaseModel):
    """Strict persisted model for project tenancy records."""

    model_config = ConfigDict(extra="forbid", frozen=True)

    project_id: str = Field(min_length=1)
    name: str = Field(min_length=1)
    tenant_id: str = Field(min_length=1)
    path: str = Field(min_length=1)
    product_id: str | None = None
    template: str = Field(min_length=1)
    template_version: str = Field(min_length=1)
    created_at: str = Field(min_length=1)
    updated_at: str = Field(min_length=1)


class TenancyRegistryPayload(BaseModel):
    """On-disk registry payload schema."""

    model_config = ConfigDict(extra="forbid")

    projects: list[TenancyProject] = Field(default_factory=list)


@dataclass(slots=True)
class AssetInstallResult:
    """Deterministic template install result."""

    installed: list[str] = field(default_factory=list)
    overwritten: list[str] = field(default_factory=list)
    skipped: list[str] = field(default_factory=list)
    conflicts: list[str] = field(default_factory=list)
    conflict_files: list[str] = field(default_factory=list)
    unchanged: list[str] = field(default_factory=list)


class ProjectTenancy:
    """Project tenancy manager backed by a strict JSON registry."""

    def __init__(self, registry_path: Path | str = _DEFAULT_REGISTRY_PATH) -> None:
        self._registry_path = Path(registry_path).expanduser()

    def init_project(
        self,
        *,
        name: str,
        tenant_id: str,
        path: Path | str,
        product_id: str | None = None,
        template: str = _DEFAULT_TEMPLATE,
        template_version: str = _DEFAULT_TEMPLATE_VERSION,
        project_id: str | None = None,
    ) -> TenancyProject:
        normalized_path = self._normalize_project_path(path)
        payload = self._load_registry()

        effective_project_id = project_id or _new_project_id()
        conflict = self._find_conflict(
            payload.projects,
            project_id=effective_project_id,
            name=name,
            tenant_id=tenant_id,
            path=normalized_path,
        )
        if conflict is not None:
            raise ValueError(conflict)

        now = _utc_now()
        record = TenancyProject(
            project_id=effective_project_id,
            name=name.strip(),
            tenant_id=tenant_id.strip(),
            path=normalized_path,
            product_id=product_id.strip() if product_id else None,
            template=template.strip(),
            template_version=template_version.strip(),
            created_at=now,
            updated_at=now,
        )

        self._link_with_project_registry(record)
        payload.projects.append(record)
        self._save_registry(payload)
        return record

    def sync_project(
        self,
        *,
        path: Path | str,
        name: str | None = None,
        tenant_id: str | None = None,
        template: str | None = None,
        template_version: str | None = None,
    ) -> TenancyProject:
        """Update selected tenancy fields for an existing project record."""
        normalized_path = self._normalize_project_path(path)
        payload = self._load_registry()

        target_index = None
        for index, project in enumerate(payload.projects):
            if project.path == normalized_path:
                target_index = index
                break

        if target_index is None:
            raise KeyError(f"Project is not registered for tenancy: {normalized_path}")

        current = payload.projects[target_index]
        updated_data = current.model_dump()
        if name is not None:
            updated_data["name"] = name
        if tenant_id is not None:
            updated_data["tenant_id"] = tenant_id
        if template is not None:
            updated_data["template"] = template
        if template_version is not None:
            updated_data["template_version"] = template_version

        updates = {
            "name": updated_data["name"].strip(),
            "tenant_id": updated_data["tenant_id"].strip(),
            "template": updated_data["template"].strip(),
            "template_version": updated_data["template_version"].strip(),
        }
        if (
            not updates["name"]
            or not updates["tenant_id"]
            or not updates["template"]
            or not updates["template_version"]
        ):
            raise ValueError("Cannot sync project with blank name/tenant/template/template_version.")

        conflict = self._find_conflict(
            payload.projects,
            project_id=current.project_id,
            name=updates["name"],
            tenant_id=updates["tenant_id"],
            path=normalized_path,
            ignore_project_id=current.project_id,
        )
        if conflict is not None:
            raise ValueError(conflict)

        updated_record = TenancyProject(
            project_id=current.project_id,
            name=updates["name"],
            tenant_id=updates["tenant_id"],
            path=normalized_path,
            product_id=current.product_id,
            template=updates["template"],
            template_version=updates["template_version"],
            created_at=current.created_at,
            updated_at=_utc_now(),
        )

        payload.projects[target_index] = updated_record
        self._save_registry(payload)
        return updated_record

    def list_projects(self) -> list[TenancyProject]:
        payload = self._load_registry()
        return sorted(payload.projects, key=lambda record: record.created_at)

    def get_project(
        self,
        *,
        project_id: str | None = None,
        name: str | None = None,
        tenant_id: str | None = None,
        path: Path | str | None = None,
    ) -> TenancyProject | None:
        selectors = {
            "project_id": project_id,
            "name": name,
            "tenant_id": tenant_id,
            "path": str(path) if path is not None else None,
        }
        selected = [key for key, value in selectors.items() if value is not None]
        if not selected:
            raise ValueError("At least one selector is required: project_id, name, tenant_id, or path.")

        projects = self._load_registry().projects
        matches = projects

        if project_id is not None:
            matches = [record for record in matches if record.project_id == project_id]
        if name is not None:
            matches = [record for record in matches if record.name == name]
        if tenant_id is not None:
            matches = [record for record in matches if record.tenant_id == tenant_id]
        if path is not None:
            normalized_path = self._normalize_project_path(path)
            matches = [record for record in matches if record.path == normalized_path]

        if not matches:
            return None
        if len(matches) > 1:
            raise ValueError("Project selector is ambiguous; refine query with additional selectors.")
        return matches[0]

    def install_project_assets(self, path: Path | str) -> AssetInstallResult:
        normalized_path = self._normalize_project_path(path)
        if self.get_project(path=normalized_path) is None:
            raise KeyError(f"Project is not registered for tenancy: {normalized_path}")
        return self.spawn_template_agdd(normalized_path, mode="smart")

    def spawn_template_agdd(self, path: Path | str, mode: TemplateMode = "smart") -> AssetInstallResult:
        if mode not in {"smart", "overwrite", "skip"}:
            raise ValueError("mode must be one of: smart, overwrite, skip")

        target_root = Path(path).expanduser().resolve()
        if not target_root.exists():
            raise FileNotFoundError(f"Project path not found: {target_root}")
        if not target_root.is_dir():
            raise NotADirectoryError(f"Project path must be a directory: {target_root}")

        template_root = self._template_root()
        result = AssetInstallResult()

        for source_path in sorted(template_root.rglob("*")):
            if source_path.is_dir():
                continue
            relative = source_path.relative_to(template_root)
            destination = target_root / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            relative_str = relative.as_posix()

            if not destination.exists():
                shutil.copy2(source_path, destination)
                result.installed.append(relative_str)
                continue

            source_bytes = source_path.read_bytes()
            destination_bytes = destination.read_bytes()
            if source_bytes == destination_bytes:
                result.unchanged.append(relative_str)
                continue

            if mode == "overwrite":
                destination.write_bytes(source_bytes)
                result.overwritten.append(relative_str)
                continue

            if mode == "skip":
                result.skipped.append(relative_str)
                result.conflicts.append(relative_str)
                continue

            conflict_copy = destination.with_name(f"{destination.name}.thegent.new")
            if conflict_copy.exists() and conflict_copy.read_bytes() != source_bytes:
                raise FileExistsError(f"Conflict sidecar already exists with different content: {conflict_copy}")
            if not conflict_copy.exists():
                conflict_copy.write_bytes(source_bytes)
            result.conflicts.append(relative_str)
            result.conflict_files.append(str(conflict_copy))

        return result

    def _normalize_project_path(self, path: Path | str) -> str:
        normalized = Path(path).expanduser().resolve()
        if not normalized.exists():
            raise FileNotFoundError(f"Project path not found: {normalized}")
        if not normalized.is_dir():
            raise NotADirectoryError(f"Project path must be a directory: {normalized}")
        return str(normalized)

    def _template_root(self) -> Path:
        root = Path(__file__).resolve().parents[3] / "templates" / "projects" / "ag-dd"
        if not root.exists():
            raise FileNotFoundError(f"AG-DD template directory not found: {root}")
        return root

    def _find_conflict(
        self,
        projects: list[TenancyProject],
        *,
        project_id: str,
        name: str,
        tenant_id: str,
        path: str,
        ignore_project_id: str | None = None,
    ) -> str | None:
        for record in projects:
            if ignore_project_id is not None and record.project_id == ignore_project_id:
                continue
            if record.project_id == project_id:
                return f"project_id conflict: {project_id}"
            if record.path == path:
                return f"path conflict: {path}"
            if record.tenant_id == tenant_id and record.name == name:
                return f"tenant/name conflict: tenant_id={tenant_id}, name={name}"
        return None

    def _load_registry(self) -> TenancyRegistryPayload:
        if not self._registry_path.exists():
            return TenancyRegistryPayload()

        payload = json.loads(self._registry_path.read_text(encoding="utf-8"))
        return TenancyRegistryPayload.model_validate(payload)

    def _save_registry(self, payload: TenancyRegistryPayload) -> None:
        self._registry_path.parent.mkdir(parents=True, exist_ok=True)
        temp_path = self._registry_path.with_suffix(".json.tmp")
        content = json.dumps(payload.model_dump(mode="json"), indent=2, sort_keys=True)
        temp_path.write_text(f"{content}\n", encoding="utf-8")
        temp_path.replace(self._registry_path)

    def _link_with_project_registry(self, project: TenancyProject) -> None:
        registry = ProjectRegistry()
        metadata = {
            "tenant_id": project.tenant_id,
            "tenancy_project_id": project.project_id,
            "product_id": project.product_id,
            "template": project.template,
            "template_version": project.template_version,
            "tenancy_path": project.path,
        }

        matches = [
            core_project
            for core_project in registry.list_projects()
            if core_project.path == project.path and core_project.name == project.name
        ]
        if len(matches) > 1:
            raise ValueError(f"Ambiguous ProjectRegistry linkage for path={project.path} name={project.name}")
        if not matches:
            registry.register_project(name=project.name, path=project.path, metadata=metadata)
            return
        registry.update_project_metadata(matches[0].id, metadata=metadata)


_DEFAULT_TENANCY = ProjectTenancy()


def init_project(
    *,
    name: str,
    tenant_id: str,
    path: Path | str,
    product_id: str | None = None,
    template: str = _DEFAULT_TEMPLATE,
    template_version: str = _DEFAULT_TEMPLATE_VERSION,
    project_id: str | None = None,
) -> TenancyProject:
    return _DEFAULT_TENANCY.init_project(
        name=name,
        tenant_id=tenant_id,
        path=path,
        product_id=product_id,
        template=template,
        template_version=template_version,
        project_id=project_id,
    )


def sync_project(
    *,
    path: Path | str,
    name: str | None = None,
    tenant_id: str | None = None,
    template: str | None = None,
    template_version: str | None = None,
) -> TenancyProject:
    """Update an existing project record."""
    return _DEFAULT_TENANCY.sync_project(
        path=path,
        name=name,
        tenant_id=tenant_id,
        template=template,
        template_version=template_version,
    )


def list_projects() -> list[TenancyProject]:
    return _DEFAULT_TENANCY.list_projects()


def get_project(
    *,
    project_id: str | None = None,
    name: str | None = None,
    tenant_id: str | None = None,
    path: Path | str | None = None,
) -> TenancyProject | None:
    return _DEFAULT_TENANCY.get_project(
        project_id=project_id,
        name=name,
        tenant_id=tenant_id,
        path=path,
    )


def install_project_assets(path: Path | str) -> AssetInstallResult:
    return _DEFAULT_TENANCY.install_project_assets(path)


def spawn_template_agdd(path: Path | str, mode: TemplateMode = "smart") -> AssetInstallResult:
    return _DEFAULT_TENANCY.spawn_template_agdd(path=path, mode=mode)
