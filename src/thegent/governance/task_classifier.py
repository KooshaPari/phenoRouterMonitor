"""Governance task classifier for delegation placement and gate selection (W78-B01).

This module implements a schema-first loader and a deterministic rule engine for
computing governance placement recommendations from
`docs/governance/TASK_CLASSIFIER_SCHEMA.yaml`.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
import re
from typing import Any

from thegent.infra.fast_yaml_parser import yaml_load


_SCHEMA_PATH = Path(__file__).resolve().parents[3] / "docs" / "governance" / "TASK_CLASSIFIER_SCHEMA.yaml"


class TaskClassifierError(ValueError):
    """Raised for schema, payload, or classification failures."""


@dataclass(frozen=True)
class TaskMetadata:
    task_id: str
    title: str
    domain: str
    scale: str
    risk: str
    coupling: str
    runtime_profile: str
    validation_depth: list[str]
    overlap_risk: int


@dataclass(frozen=True)
class TaskClassification:
    delegation_tier: str
    worker_count: int
    worktree_mode: str
    commit_mode: str
    required_gates: list[str]

    def as_payload(self) -> dict[str, Any]:
        return {
            "delegation_tier": self.delegation_tier,
            "worker_count": self.worker_count,
            "worktree_mode": self.worktree_mode,
            "commit_mode": self.commit_mode,
            "required_gates": self.required_gates,
        }


@dataclass(frozen=True)
class SchemaSpec:
    payload: dict[str, Any]
    fields: dict[str, Any]
    outputs: dict[str, Any]
    policy_defaults: dict[str, Any]
    escalation_rules: list[dict[str, Any]]


def _require(v: object, *, name: str) -> object:
    if v is None:
        raise TaskClassifierError(f"missing required schema field: {name}")
    return v


def _coerce_int_range(raw: object) -> tuple[int, int]:
    if not isinstance(raw, list) or len(raw) != 2:
        raise TaskClassifierError("schema range must be a two-value list")
    min_v, max_v = raw
    if not isinstance(min_v, (int, float)) or not isinstance(max_v, (int, float)):
        raise TaskClassifierError("schema range values must be numeric")
    return int(min_v), int(max_v)


def _as_list(payload: object) -> list[dict[str, Any]]:
    if isinstance(payload, list):
        return payload
    raise TaskClassifierError("schema escalation_rules must be a list")


def _normalize_validation_depth(values: object) -> list[str]:
    if not isinstance(values, list):
        raise TaskClassifierError("validation_depth must be a list")
    cleaned: list[str] = []
    for item in values:
        if not isinstance(item, str):
            raise TaskClassifierError("validation_depth items must be strings")
        cleaned.append(item.strip().lower())
    if not cleaned:
        raise TaskClassifierError("validation_depth must include at least one value")
    return cleaned


def _load_yaml(path: Path) -> dict[str, Any]:
    try:
        payload = yaml_load(path)
    except Exception as exc:  # explicit error is required behavior
        raise TaskClassifierError(f"failed to parse YAML from {path}: {exc}") from exc

    if not isinstance(payload, dict):
        raise TaskClassifierError("schema file must contain a YAML mapping")
    return payload


def load_schema(*, schema_path: Path | None = None) -> SchemaSpec:
    schema_path = schema_path or _SCHEMA_PATH
    root = _load_yaml(schema_path)

    fields = root.get("fields")
    outputs = root.get("outputs")
    policy_defaults = root.get("policy_defaults")
    escalation_rules = _as_list(_require(root.get("escalation_rules"), name="escalation_rules"))
    payload = root

    if not isinstance(fields, dict):
        raise TaskClassifierError("schema field `fields` must be an object")
    if not isinstance(outputs, dict):
        raise TaskClassifierError("schema field `outputs` must be an object")
    if not isinstance(policy_defaults, dict):
        raise TaskClassifierError("schema field `policy_defaults` must be an object")
    for required_key in [
        "task_id",
        "title",
        "domain",
        "scale",
        "risk",
        "coupling",
        "runtime_profile",
        "validation_depth",
        "overlap_risk",
    ]:
        if required_key not in fields:
            raise TaskClassifierError(f"missing required input field definition: {required_key}")
    if not root.get("name"):
        raise TaskClassifierError("schema field `name` is required")
    if not root.get("version"):
        raise TaskClassifierError("schema field `version` is required")

    return SchemaSpec(
        payload=payload,
        fields=fields,
        outputs=outputs,
        policy_defaults=policy_defaults,
        escalation_rules=escalation_rules,
    )


def _validate_list_or_scalar(
    *,
    value: object,
    definition: dict[str, Any],
    field_name: str,
) -> None:
    definition_type = definition.get("type")
    if definition_type == "enum":
        if not isinstance(value, str):
            raise TaskClassifierError(f"{field_name} must be a string")
        allowed = definition.get("values", [])
        if not isinstance(allowed, list) or value not in allowed:
            raise TaskClassifierError(f"{field_name} has invalid value: {value}")
        return

    if definition_type == "integer":
        if not isinstance(value, int):
            raise TaskClassifierError(f"{field_name} must be an integer")
        value_range = definition.get("range")
        if isinstance(value_range, list) and len(value_range) == 2:
            min_v, max_v = _coerce_int_range(value_range)
            if not (min_v <= value <= max_v):
                raise TaskClassifierError(f"{field_name} out of range: {value}")
        return

    if definition_type == "string":
        if not isinstance(value, str):
            raise TaskClassifierError(f"{field_name} must be a string")
        return

    if definition_type == "list":
        if not isinstance(value, list):
            raise TaskClassifierError(f"{field_name} must be a list")
        item_type = definition.get("items", {})
        if item_type.get("type") == "enum":
            values = item_type.get("values", [])
            if not isinstance(values, list):
                raise TaskClassifierError(f"{field_name} item enum must be a list")
            for item in value:
                if not isinstance(item, str):
                    raise TaskClassifierError(f"{field_name} items must be strings")
                if item not in values:
                    raise TaskClassifierError(f"{field_name} has invalid item: {item}")
        elif item_type.get("type") == "string":
            for item in value:
                if not isinstance(item, str):
                    raise TaskClassifierError(f"{field_name} items must be strings")
        return


def validate_classification_payload(payload: dict[str, Any], schema: SchemaSpec) -> None:
    if not isinstance(payload, dict):
        raise TaskClassifierError("payload must be a mapping")

    for field_name, field_definition in schema.fields.items():
        required = bool(field_definition.get("required", False))
        value = payload.get(field_name)
        if required and value is None:
            raise TaskClassifierError(f"missing required payload field: {field_name}")
        if value is None:
            continue
        _validate_list_or_scalar(value=value, definition=field_definition, field_name=field_name)


def _pick_default_tier_and_workers(task: TaskMetadata) -> tuple[str, int]:
    if task.risk in {"high", "critical"}:
        return "L3_specialist", 3

    scale_default = {
        "XS": ("Ln_worker", 1),
        "S": ("Ln_worker", 2),
        "M": ("L2_managed", 4),
        "L": ("L2_managed", 6),
        "XL": ("L3_specialist", 12),
    }
    if task.scale not in scale_default:
        raise TaskClassifierError(f"unknown scale: {task.scale}")
    return scale_default[task.scale]


def _parse_rule_condition(condition: str) -> tuple[str, str, list[str] | int | str]:
    normalized = condition.strip()
    set_match = re.fullmatch(r"^(\w+)\s+in\s*\[(.*)\]$", normalized)
    if set_match:
        field_name, raw_values = set_match.groups()
        values = [value.strip() for value in raw_values.split(",") if value.strip()]
        return field_name, "in", values

    compare_match = re.fullmatch(r"^(\w+)\s*(==|!=|>=|<=|>|<)\s*(.+)$", normalized)
    if not compare_match:
        raise TaskClassifierError(f"unsupported escalation rule condition: {condition!r}")
    field_name, op, rhs = compare_match.groups()
    rhs_norm = rhs.strip()
    if len(rhs_norm) >= 2 and rhs_norm[0] == rhs_norm[-1] and rhs_norm[0] in {'"', "'"}:
        rhs_norm = rhs_norm[1:-1]
    if re.fullmatch(r"^-?\d+$", rhs_norm):
        return field_name, op, int(rhs_norm)
    return field_name, op, rhs_norm


def _eval_rule_condition(task: TaskMetadata, condition: str) -> bool:
    field_name, op, rhs = _parse_rule_condition(condition)
    if not hasattr(task, field_name):
        raise TaskClassifierError(f"unknown condition field: {field_name}")
    value = getattr(task, field_name)

    if op == "in":
        if not isinstance(rhs, list):
            raise TaskClassifierError(f"invalid in-condition rhs: {rhs!r}")
        return str(value) in [str(item) for item in rhs]

    if isinstance(rhs, int):
        if not isinstance(value, int):
            raise TaskClassifierError(f"condition requires numeric lhs for field: {field_name}")
        if op == "==":
            return value == rhs
        if op == "!=":
            return value != rhs
        if op == ">=":
            return value >= rhs
        if op == "<=":
            return value <= rhs
        if op == ">":
            return value > rhs
        if op == "<":
            return value < rhs
        raise TaskClassifierError(f"unsupported comparison operator: {op}")

    if op == "==":
        return str(value) == str(rhs)
    if op == "!=":
        return str(value) != str(rhs)
    raise TaskClassifierError(f"non-numeric condition with operator {op!r} not supported")


def _coerce_gate_values(raw_values: object) -> list[str]:
    if not isinstance(raw_values, list):
        raise TaskClassifierError("escalation required_gates must be a list")
    values: list[str] = []
    for item in raw_values:
        if not isinstance(item, str):
            raise TaskClassifierError("escalation required_gates entries must be strings")
        gate = item.strip()
        if gate:
            values.append(gate)
    return values


def _default_required_gates(task: TaskMetadata) -> list[str]:
    gates = {"lint", "unit"}
    if task.risk in {"high", "critical"}:
        gates |= {"integration", "security"}
    if task.scale in {"L", "XL"}:
        gates |= {"perf", "chaos"}
    return sorted(gates)


def _apply_escalation_rules(
    task: TaskMetadata,
    classification: TaskClassification,
    schema: SchemaSpec,
) -> TaskClassification:
    output_keys = set(schema.outputs.keys())
    rules = []
    for rule in schema.escalation_rules:
        if not isinstance(rule, dict):
            raise TaskClassifierError(f"invalid escalation rule entry: {rule!r}")
        raw_if = rule.get("if")
        raw_then = rule.get("then")
        if not isinstance(raw_if, str) or raw_if.strip() == "":
            raise TaskClassifierError("escalation rule missing if condition")
        if not isinstance(raw_then, dict):
            raise TaskClassifierError(f"escalation rule missing then block: {rule!r}")
        rules.append((raw_if, raw_then))

    gates = set(classification.required_gates)
    tier = classification.delegation_tier
    workers = classification.worker_count
    worktree_mode = classification.worktree_mode
    commit_mode = classification.commit_mode

    for rule_if, then_block in rules:
        if not _eval_rule_condition(task, rule_if):
            continue
        for key, value in then_block.items():
            if key not in output_keys:
                raise TaskClassifierError(f"escalation rule output key unknown: {key}")
            if key == "delegation_tier":
                tier = str(value)
            elif key == "worker_count":
                if not isinstance(value, int):
                    raise TaskClassifierError("escalation worker_count must be an integer")
                workers = value
            elif key == "worktree_mode":
                worktree_mode = str(value)
            elif key == "commit_mode":
                commit_mode = str(value)
            elif key == "required_gates":
                gates.update(_coerce_gate_values(value))
            else:
                raise TaskClassifierError(f"unsupported escalation output key: {key}")

    workers = max(workers, 1)

    final_gates = sorted(gates)
    if not final_gates:
        final_gates = ["lint", "unit"]

    return TaskClassification(
        delegation_tier=tier,
        worker_count=workers,
        worktree_mode=worktree_mode,
        commit_mode=commit_mode,
        required_gates=final_gates,
    )


def _apply_policy_defaults(task: TaskMetadata, schema: SchemaSpec) -> tuple[str, str]:
    scale_defaults = schema.policy_defaults.get(task.scale, {})
    if not isinstance(scale_defaults, dict):
        return "Ln_worker", "micro"
    worktree_mode = scale_defaults.get("worktree_mode") or "shared_lane"
    commit_mode = scale_defaults.get("commit_mode") or "micro"
    return str(worktree_mode), str(commit_mode)


def classify(payload: dict[str, Any], *, schema_path: Path | None = None) -> tuple[TaskMetadata, TaskClassification]:
    schema = load_schema(schema_path=schema_path)
    validate_classification_payload(payload, schema)

    metadata = TaskMetadata(
        task_id=str(payload["task_id"]),
        title=str(payload["title"]),
        domain=str(payload["domain"]),
        scale=str(payload["scale"]),
        risk=str(payload["risk"]),
        coupling=str(payload["coupling"]),
        runtime_profile=str(payload["runtime_profile"]),
        validation_depth=_normalize_validation_depth(payload.get("validation_depth")),
        overlap_risk=int(payload["overlap_risk"]),
    )

    tier, workers = _pick_default_tier_and_workers(metadata)
    worktree_mode, commit_mode = _apply_policy_defaults(metadata, schema)

    base_classification = TaskClassification(
        delegation_tier=tier,
        worker_count=workers,
        worktree_mode=worktree_mode,
        commit_mode=commit_mode,
        required_gates=_default_required_gates(metadata),
    )

    return metadata, _apply_escalation_rules(metadata, base_classification, schema)


__all__ = [
    "_SCHEMA_PATH",
    "SchemaSpec",
    "TaskClassification",
    "TaskClassifierError",
    "TaskMetadata",
    "classify",
    "load_schema",
    "validate_classification_payload",
]
