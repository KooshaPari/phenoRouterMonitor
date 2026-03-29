"""Loader for governance task classifier schema."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from thegent.infra.fast_yaml_parser import yaml_load


_FIELD_REQUIRED_KEYS = ("type", "required")
_OUTPUT_REQUIRED_KEYS = ("type",)


def _require_dict(value: Any, path: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise ValueError(f"{path} must be a mapping, got {type(value).__name__}")
    return value


def _require_str(value: Any, path: str) -> str:
    if not isinstance(value, str):
        raise ValueError(f"{path} must be a string, got {type(value).__name__}")
    return value


def _require_int(value: Any, path: str) -> int:
    if not isinstance(value, int):
        raise ValueError(f"{path} must be an integer, got {type(value).__name__}")
    return value


def _require_bool(value: Any, path: str) -> bool:
    if not isinstance(value, bool):
        raise ValueError(f"{path} must be a boolean, got {type(value).__name__}")
    return value


def _validate_field_contract(fields: dict[str, Any], *, path: str, required_keys: tuple[str, ...]) -> None:
    for field_name, spec in fields.items():
        if not isinstance(field_name, str):
            raise ValueError(f"{path} keys must be strings, got {type(field_name).__name__}")
        spec_dict = _require_dict(spec, f"{path}.{field_name}")
        missing = [k for k in required_keys if k not in spec_dict]
        if missing:
            raise ValueError(f"{path}.{field_name} missing required keys: {', '.join(missing)}")

        field_type = _require_str(spec_dict["type"], f"{path}.{field_name}.type")
        if field_type not in {"string", "enum", "integer", "list"}:
            raise ValueError(f"{path}.{field_name}.type has unsupported value '{field_type}'")

        if "required" in required_keys:
            _require_bool(spec_dict["required"], f"{path}.{field_name}.required")

        if field_type == "enum":
            values = spec_dict.get("values")
            if not isinstance(values, list) or not values:
                raise ValueError(f"{path}.{field_name} enum type must include non-empty 'values' list")

        if field_type == "list":
            items = spec_dict.get("items")
            if not isinstance(items, dict):
                raise ValueError(f"{path}.{field_name}.items must be a mapping")
            item_type = _require_str(items.get("type"), f"{path}.{field_name}.items.type")
            if item_type not in {"string", "integer", "enum", "list", "map"}:
                raise ValueError(f"{path}.{field_name}.items.type has unsupported value '{item_type}'")

        if field_type == "integer" and "range" in spec_dict:
            rng = spec_dict["range"]
            if isinstance(rng, dict):
                low = _require_int(rng.get("min"), f"{path}.{field_name}.range.min")
                high = _require_int(rng.get("max"), f"{path}.{field_name}.range.max")
            elif isinstance(rng, (list, tuple)):
                if len(rng) != 2:
                    raise ValueError(f"{path}.{field_name}.range must have exactly 2 entries")
                low = _require_int(rng[0], f"{path}.{field_name}.range[0]")
                high = _require_int(rng[1], f"{path}.{field_name}.range[1]")
            else:
                raise ValueError(f"{path}.{field_name}.range must be a mapping or 2-tuple list")
            if low > high:
                raise ValueError(f"{path}.{field_name}.range.min must be <= range.max")


def _validate_policy_defaults(policy_defaults: Any, path: str = "policy_defaults") -> None:
    if policy_defaults is None:
        return

    defaults = _require_dict(policy_defaults, path)
    for scale in ("XS", "S", "M", "L", "XL"):
        if scale not in defaults:
            raise ValueError(f"{path} missing required key '{scale}'")

        bucket = _require_dict(defaults[scale], f"{path}.{scale}")
        if "worktree_mode" not in bucket or "commit_mode" not in bucket:
            raise ValueError(f"{path}.{scale} must include worktree_mode and commit_mode")


def _validate_escalation_rules(rules: Any, path: str = "escalation_rules") -> None:
    if rules is None:
        return
    if not isinstance(rules, list):
        raise ValueError(f"{path} must be a list")
    rule_list = rules
    for idx, rule in enumerate(rule_list):
        rule_obj = _require_dict(rule, f"{path}[{idx}]")
        if "if" not in rule_obj or "then" not in rule_obj:
            raise ValueError(f"{path}[{idx}] must include 'if' and 'then'")
        _require_str(rule_obj["if"], f"{path}[{idx}].if")
        _require_dict(rule_obj["then"], f"{path}[{idx}].then")


def load_task_classifier_schema(schema_path: Path | None = None) -> dict[str, Any]:
    """Load and validate the governance task classifier schema."""
    path = schema_path or Path("docs/governance/TASK_CLASSIFIER_SCHEMA.yaml")
    raw = yaml_load(path) or {}
    if not isinstance(raw, dict):
        msg = f"Invalid classifier schema payload type at {path}"
        raise ValueError(msg)
    required_keys = ("version", "fields", "outputs")
    missing = [k for k in required_keys if k not in raw]
    if missing:
        msg = f"Classifier schema missing keys: {', '.join(missing)}"
        raise ValueError(msg)
    version = _require_int(raw["version"], "version")
    if version < 1:
        raise ValueError("Classifier schema version must be >= 1")
    fields = _require_dict(raw["fields"], "fields")
    outputs = _require_dict(raw["outputs"], "outputs")
    _validate_field_contract(fields, path="fields", required_keys=_FIELD_REQUIRED_KEYS)
    _validate_field_contract(outputs, path="outputs", required_keys=_OUTPUT_REQUIRED_KEYS)
    _validate_policy_defaults(raw.get("policy_defaults"))
    _validate_escalation_rules(raw.get("escalation_rules"), "escalation_rules")
    return raw
