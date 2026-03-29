from __future__ import annotations

import hashlib
import json
from dataclasses import asdict, is_dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

from .paths import mirror_target_state_root, target_state_root


def utc_now_iso() -> str:
    return datetime.now(UTC).isoformat()


def _stable_dump(payload: dict[str, Any]) -> str:
    return json.dumps(payload, indent=2, sort_keys=True)


def _content_hash(payload: dict[str, Any]) -> str:
    raw = json.dumps(payload, sort_keys=True, separators=(",", ":"))
    return hashlib.sha256(raw.encode("utf-8")).hexdigest()


def _normalize(obj: Any) -> dict[str, Any]:
    if is_dataclass(obj):
        data = asdict(obj)
    elif isinstance(obj, dict):
        data = obj
    else:
        raise TypeError(f"Unsupported payload type: {type(obj)}")
    return data


def _write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp_path = path.with_suffix(path.suffix + ".tmp")
    tmp_path.write_text(_stable_dump(payload) + "\n", encoding="utf-8")
    tmp_path.replace(path)


def dual_write(
    target: str,
    filename: str,
    payload_obj: Any,
    family: str | None = None,
) -> dict[str, str]:
    payload = _normalize(payload_obj)
    sync_id = utc_now_iso()
    payload_hash = _content_hash(payload)
    wrapped = {
        "sync_id": sync_id,
        "content_hash": payload_hash,
        "payload": payload,
    }
    project_path = target_state_root(target, family=family) / filename
    mirror_path = mirror_target_state_root(target, family=family) / filename
    _write_json(project_path, wrapped)
    _write_json(mirror_path, wrapped)
    return {
        "project_path": str(project_path),
        "mirror_path": str(mirror_path),
        "sync_id": sync_id,
        "content_hash": payload_hash,
    }


def read_dual(target: str, filename: str, family: str | None = None) -> dict[str, Any]:
    candidates = [
        target_state_root(target, family=family) / filename,
        mirror_target_state_root(target, family=family) / filename,
    ]
    errors: list[str] = []
    for path in candidates:
        if not path.exists():
            errors.append(f"missing:{path}")
            continue
        try:
            content = json.loads(path.read_text(encoding="utf-8"))
            if not isinstance(content, dict):
                errors.append(f"invalid-shape:{path}")
                continue
            payload = content.get("payload")
            if not isinstance(payload, dict):
                errors.append(f"missing-payload:{path}")
                continue
            stored_hash = content.get("content_hash")
            if not isinstance(stored_hash, str):
                errors.append(f"missing-hash:{path}")
                continue
            computed_hash = _content_hash(payload)
            if computed_hash != stored_hash:
                errors.append(f"hash-mismatch:{path}")
                continue
            return payload
        except json.JSONDecodeError:
            errors.append(f"json-error:{path}")
    raise FileNotFoundError(f"Unable to load {filename} for {target}; {', '.join(errors)}")


def sync_dual(
    target: str,
    filename: str,
    prefer: str | None = None,
    family: str | None = None,
) -> dict[str, Any]:
    project_path = target_state_root(target, family=family) / filename
    mirror_path = mirror_target_state_root(target, family=family) / filename

    if not project_path.exists() and not mirror_path.exists():
        raise FileNotFoundError(f"No state file exists for sync: {filename}")

    def _copy_wrapped_json(source: Path, destination: Path) -> None:
        raw = source.read_text(encoding="utf-8")
        wrapped = json.loads(raw)
        if not isinstance(wrapped, dict):
            raise ValueError(f"state file must be a JSON object: {source}")
        _write_json(destination, wrapped)

    if project_path.exists() and not mirror_path.exists():
        _copy_wrapped_json(project_path, mirror_path)
        return {"source": str(project_path), "synced": str(mirror_path), "status": "repaired"}

    if mirror_path.exists() and not project_path.exists():
        _copy_wrapped_json(mirror_path, project_path)
        return {"source": str(mirror_path), "synced": str(project_path), "status": "repaired"}

    project_raw = project_path.read_text(encoding="utf-8")
    mirror_raw = mirror_path.read_text(encoding="utf-8")
    if project_raw == mirror_raw:
        return {"status": "in-sync", "project_path": str(project_path), "mirror_path": str(mirror_path)}

    if prefer not in {None, "projects", "home"}:
        raise ValueError("prefer must be one of: projects, home")

    if prefer == "projects":
        source, dest = project_path, mirror_path
    elif prefer == "home":
        source, dest = mirror_path, project_path
    else:
        project_mtime = project_path.stat().st_mtime
        mirror_mtime = mirror_path.stat().st_mtime
        if project_mtime == mirror_mtime:
            raise ValueError("Dual state drift with equal mtime; rerun with --prefer projects|home")
        source, dest = (project_path, mirror_path) if project_mtime > mirror_mtime else (mirror_path, project_path)

    _copy_wrapped_json(source, dest)
    return {"status": "repaired", "source": str(source), "synced": str(dest)}
