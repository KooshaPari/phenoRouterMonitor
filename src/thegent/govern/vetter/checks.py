"""Concrete VetterCheck implementations for the Vetter governance layer.

Six checks:
  1. SchemaCheck      -- jsonschema validation against a JSON Schema dict
  2. DiffSizeCheck    -- reject outputs where diff exceeds max_lines
  3. SafetyCheck      -- delegates to SemanticFirewall.inspect_output()
  4. LLMJudgeCheck    -- G-Eval-style LLM judge via CLIProxy (bifrost)
  5. TestPassCheck    -- runs pytest via asyncio.create_subprocess_exec
  6. RuffCheck        -- runs ruff check via asyncio.create_subprocess_exec

All implement VetterCheck (Protocol). Fail fast, fail loudly.
No silent fallbacks, no legacy shims.

# @trace WL-090
"""

from __future__ import annotations

import asyncio
import logging
import os
import orjson as json
import re
import subprocess
from thegent.infra.shim_subprocess import run as shim_run
from dataclasses import dataclass, field
from typing import Any, Callable, Literal

import httpx
import jsonschema
from pydantic import BaseModel, StrictInt, ValidationError

from thegent.govern.vetter.models import VetterCheckResult, VetterConfigError  # noqa: TC001

logger = logging.getLogger(__name__)

# CLIProxy configuration
_CLIPROXY_URL = os.environ.get("CLIPROXY_URL", "http://localhost:8317")


async def _call_llm_via_cliproxy(
    model: str,
    messages: list[dict[str, str]],
    temperature: float = 0.0,
    timeout: float = 120.0,
) -> str:
    """Call LLM via CLIProxy /v1/chat/completions."""
    async with httpx.AsyncClient(timeout=timeout) as client:
        resp = await client.post(
            f"{_CLIPROXY_URL}/v1/chat/completions",
            json={
                "model": model,
                "messages": messages,
                "temperature": temperature,
            },
        )
        resp.raise_for_status()
        data = resp.json()
        choices = data.get("choices", [])
        if choices:
            return choices[0].get("message", {}).get("content", "")
        return ""


@dataclass
class SchemaCheck:
    """Validate agent output (parsed as JSON) against a JSON Schema dict.

    Raises VetterConfigError if schema is not provided.
    Fails (passed=False) if JSON parsing or schema validation fails.
    # @trace WL-090
    """

    name: str = "schema"
    schema: dict[str, Any] = field(default_factory=dict)

    async def check(
        self,
        run_id: str,
        output: str,
        context: dict[str, Any],
    ) -> VetterCheckResult:
        """Run schema validation check. # @trace WL-090"""
        if not self.schema:
            raise VetterConfigError(f"SchemaCheck for run {run_id}: schema must not be empty")

        try:
            parsed = json.loads(output)
        except json.JSONDecodeError as exc:
            return VetterCheckResult(
                check_name=self.name,
                passed=False,
                message=f"JSON parse error: {exc}",
            )

        try:
            jsonschema.validate(instance=parsed, schema=self.schema)
        except jsonschema.ValidationError as exc:
            return VetterCheckResult(
                check_name=self.name,
                passed=False,
                message=f"Schema validation failed: {exc.message}",
                metadata={"schema_path": list(exc.absolute_path)},
            )

        return VetterCheckResult(check_name=self.name, passed=True)


@dataclass
class DiffSizeCheck:
    """Reject outputs where unified-diff line count exceeds max_lines.

    Parses the output as a unified diff -- no subprocess.
    # @trace WL-090
    """

    name: str = "diff_size"
    max_lines: int = 500

    async def check(
        self,
        run_id: str,
        output: str,
        context: dict[str, Any],
    ) -> VetterCheckResult:
        """Run diff size check. # @trace WL-090"""
        added = sum(
            1 for line in output.splitlines() if line.startswith(("+", "-")) and not line.startswith(("+++", "---"))
        )

        if added > self.max_lines:
            return VetterCheckResult(
                check_name=self.name,
                passed=False,
                message=f"Diff size {added} lines exceeds max {self.max_lines}",
                metadata={"diff_lines": added, "max_lines": self.max_lines},
            )

        return VetterCheckResult(
            check_name=self.name,
            passed=True,
            metadata={"diff_lines": added},
        )


@dataclass
class SafetyCheck:
    """Delegate content safety evaluation to SemanticFirewall.

    If firewall is None, raises VetterConfigError on construction.
    block action -> passed=False (hard reject).
    warn / redact action -> passed=False (revision candidate).
    # @trace WL-090
    """

    name: str = "safety"
    firewall: Any = None  # SemanticFirewall; Any to avoid circular import at runtime

    def __post_init__(self) -> None:
        if self.firewall is None:
            raise VetterConfigError(
                "SafetyCheck requires a SemanticFirewall instance. Pass firewall=SemanticFirewall() to SafetyCheck."
            )

    async def check(
        self,
        run_id: str,
        output: str,
        context: dict[str, Any],
    ) -> VetterCheckResult:
        """Run safety check via SemanticFirewall. # @trace WL-090"""
        _output, violations = self.firewall.inspect_output(output)

        if not violations:
            return VetterCheckResult(check_name=self.name, passed=True)

        is_block = any("CRITICAL" in v for v in violations)
        return VetterCheckResult(
            check_name=self.name,
            passed=False,
            message=f"Safety violations: {'; '.join(violations)}",
            metadata={"violations": violations, "blocked": is_block},
        )


_JUDGE_SYSTEM = (
    "You are a strict code-output quality judge. "
    "Given an agent task and its output, score each criterion 1-5 and reply ONLY with valid JSON: "
    '{"scores": {"criterion_name": <int>, ...}, "pass_verdict": <bool>, "critique": "<str>"}'
)


class _QualityJudgePayload(BaseModel):
    scores: dict[str, StrictInt]
    pass_verdict: bool
    critique: str


@dataclass
class LLMJudgeCheck:
    """G-Eval-style LLM-as-judge scoring via LiteLLM router.

    judge_model: any LiteLLM-compatible model string.
    criteria: list of criterion names to score (1-5 Likert).
    pass_threshold: weighted mean score / 5 must exceed this (0.0-1.0).
    # @trace WL-090
    """

    name: str = "llm_judge"
    judge_model: str = "gpt-4o-mini"
    criteria: list[str] = field(default_factory=lambda: ["correctness", "completeness", "safety"])
    pass_threshold: float = 0.75

    async def check(
        self,
        run_id: str,
        output: str,
        context: dict[str, Any],
    ) -> VetterCheckResult:
        """Run LLM judge check. # @trace WL-090"""
        task_description = context.get("task", "unknown task")
        criteria_list = ", ".join(self.criteria)

        user_msg = (
            f"Task: {task_description}\n\n"
            f"Agent output:\n{output}\n\n"
            f"Score each criterion ({criteria_list}) from 1 (worst) to 5 (best)."
        )

        messages = [
            {"role": "system", "content": _JUDGE_SYSTEM},
            {"role": "user", "content": user_msg},
        ]

        raw = await _call_llm_via_cliproxy(
            model=self.judge_model,
            messages=messages,
            temperature=0.0,
        )

        judge_data: dict[str, Any] = json.loads(raw)
        scores: dict[str, int] = judge_data.get("scores", {})
        pass_verdict: bool = judge_data.get("pass_verdict", False)
        critique: str = judge_data.get("critique", "")

        mean_score = (sum(scores.values()) / (5.0 * len(scores))) if scores else 0.0
        passed = pass_verdict and mean_score >= self.pass_threshold

        return VetterCheckResult(
            check_name=self.name,
            passed=passed,
            score=round(mean_score, 4),
            message=critique if not passed else "",
            metadata={"scores": scores, "mean_score": mean_score, "judge_model": self.judge_model},
        )


@dataclass
class QualityScoreVetterCheck:
    """LLM-as-judge quality scoring check using a structured rubric and 1-5 Likert scores.

    Judge response JSON contract:
      {
        "scores": {"criterion": <int 1-5>, ...},
        "pass_verdict": true|false,
        "critique": "..."
      }

    pass_threshold applies to mean(scores) / 5.0 (normalised to 0.0-1.0).
    min_criterion_score is an integer in [1, 5] — each criterion must meet this floor.
    When judge_model="auto", CapabilityIndex.recommend("quality scoring") selects the model.

    # @trace WL-095
    """

    name: str = "quality_score"
    judge_model: str = "auto"
    rubric: list[str] | dict[str, str] = field(default_factory=lambda: ["correctness", "completeness", "safety"])
    pass_threshold: float = 0.75
    min_criterion_score: int = 3
    always_run: bool = False
    model_resolver: Callable[[str, dict[str, Any]], str] | None = None

    def __post_init__(self) -> None:
        if not 0.0 <= self.pass_threshold <= 1.0:
            raise VetterConfigError("QualityScoreVetterCheck pass_threshold must be in range [0.0, 1.0]")
        if not 1 <= self.min_criterion_score <= 5:
            raise VetterConfigError("QualityScoreVetterCheck min_criterion_score must be in range [1, 5]")
        self._rubric_map = self._normalize_rubric(self.rubric)

    def _normalize_rubric(self, rubric: list[str] | dict[str, str]) -> dict[str, str]:
        if isinstance(rubric, list):
            keys = [entry.strip() for entry in rubric if entry.strip()]
            if not keys:
                raise VetterConfigError("QualityScoreVetterCheck rubric list must contain at least one criterion")
            key_counts: dict[str, int] = {}
            for key in keys:
                key_counts[key] = key_counts.get(key, 0) + 1
            duplicate_keys = sorted(key for key, count in key_counts.items() if count > 1)
            if duplicate_keys:
                raise VetterConfigError(
                    "QualityScoreVetterCheck duplicate rubric criterion after normalization: "
                    + ", ".join(duplicate_keys)
                )
            return {key: key for key in keys}
        if isinstance(rubric, dict):
            normalized: dict[str, str] = {}
            for raw_key, raw_description in rubric.items():
                key = str(raw_key).strip()
                if not key:
                    continue
                if key in normalized:
                    raise VetterConfigError(
                        f"QualityScoreVetterCheck duplicate rubric criterion after normalization: {key}"
                    )
                normalized[key] = str(raw_description).strip() or key
            if not normalized:
                raise VetterConfigError("QualityScoreVetterCheck rubric dict must contain at least one criterion")
            return normalized
        raise VetterConfigError("QualityScoreVetterCheck rubric must be list[str] or dict[str, str]")

    def _resolve_model(self, context: dict[str, Any]) -> str:
        if self.judge_model != "auto":
            return self.judge_model
        if self.model_resolver is None:
            model = self._resolve_auto_model(context)
            if not model:
                raise VetterConfigError(
                    "QualityScoreVetterCheck judge_model='auto' returned no model from CapabilityIndex"
                )
            return model
        resolved = self.model_resolver("quality scoring", context)
        if not isinstance(resolved, str):
            raise VetterConfigError("QualityScoreVetterCheck model_resolver must return a non-empty string model name")
        model = resolved.strip()
        if not model:
            raise VetterConfigError("QualityScoreVetterCheck model_resolver returned empty model name")
        return model

    def _resolve_auto_model(self, context: dict[str, Any]) -> str:
        # Local import keeps the check module independent from agent indexing when auto is not used.
        from thegent.agents.capability_index import CapabilityIndex

        index = context.get("capability_index")
        if index is None:
            extra_dirs = context.get("capability_index_extra_dirs")
            index = CapabilityIndex.get(extra_dirs=extra_dirs)

        recommendations = index.recommend("quality scoring", top_n=5)
        if not recommendations:
            raise VetterConfigError(
                "QualityScoreVetterCheck judge_model='auto' found no CapabilityIndex recommendations for quality scoring"
            )

        all_agents = index.all_agents()
        for recommendation in recommendations:
            for agent in all_agents:
                if agent.path == recommendation.path and agent.model and agent.model.strip():
                    return agent.model.strip()
        raise VetterConfigError(
            "QualityScoreVetterCheck judge_model='auto' recommendations did not include a configured model"
        )

    async def check(
        self,
        run_id: str,
        output: str,
        context: dict[str, Any],
    ) -> VetterCheckResult:
        """Run quality-scoring check and enforce aggregate + per-criterion thresholds. # @trace WL-095"""
        rubric_lines = "\n".join(
            f"- {criterion}: {description}" for criterion, description in sorted(self._rubric_map.items())
        )
        task_description = context.get("task", "unknown task")
        resolved_model = self._resolve_model(context)

        user_msg = (
            f"Run ID: {run_id}\n"
            f"Task: {task_description}\n\n"
            f"Rubric criteria (score each criterion from 1 to 5 — integer Likert scale):\n{rubric_lines}\n\n"
            f"Agent output:\n{output}\n\n"
            "Respond with strict JSON only: "
            '{"scores":{"criterion":3},"pass_verdict":true,"critique":"..."}'
        )

        messages = [
            {"role": "system", "content": _JUDGE_SYSTEM},
            {"role": "user", "content": user_msg},
        ]

        raw = await _call_llm_via_cliproxy(
            model=resolved_model,
            messages=messages,
            temperature=0.0,
        )

        try:
            decoded = json.loads(raw)
        except json.JSONDecodeError as exc:
            raise VetterConfigError(
                f"QualityScoreVetterCheck judge response was not valid JSON (model={resolved_model}): {exc}"
            ) from exc
        try:
            parsed = _QualityJudgePayload.model_validate(decoded)
        except ValidationError as exc:
            raise VetterConfigError(
                f"QualityScoreVetterCheck judge response failed schema validation (model={resolved_model}): {exc}"
            ) from exc

        expected = set(self._rubric_map.keys())
        seen = set(parsed.scores.keys())
        missing = sorted(expected - seen)
        if missing:
            raise VetterConfigError(f"QualityScoreVetterCheck missing score(s) for criterion: {', '.join(missing)}")
        unexpected = sorted(seen - expected)
        if unexpected:
            raise VetterConfigError(
                f"QualityScoreVetterCheck unexpected score(s) for criterion: {', '.join(unexpected)}"
            )

        scores = {criterion: int(parsed.scores[criterion]) for criterion in sorted(expected)}
        if any(score < 1 or score > 5 for score in scores.values()):
            raise VetterConfigError("QualityScoreVetterCheck scores must be integers in range [1, 5]")

        # Normalise mean score to [0.0, 1.0] for threshold comparison
        mean_raw = sum(scores.values()) / len(scores)
        aggregate_score = mean_raw / 5.0
        aggregate_ok = aggregate_score >= self.pass_threshold
        per_criterion_ok = all(score >= self.min_criterion_score for score in scores.values())
        passed = bool(parsed.pass_verdict and aggregate_ok and per_criterion_ok)

        message = ""
        if not passed:
            message = parsed.critique.strip()
            if not message:
                failing = [
                    f"{criterion}={score}" for criterion, score in scores.items() if score < self.min_criterion_score
                ]
                details = ", ".join(failing) if failing else "no criterion fell below minimum score"
                message = (
                    "Quality judge rejected output: "
                    f"pass_verdict={parsed.pass_verdict}, "
                    f"aggregate_score={aggregate_score:.3f} (threshold={self.pass_threshold:.3f}), "
                    f"min_criterion_score={self.min_criterion_score}, "
                    f"failing_criteria={details}"
                )

        return VetterCheckResult(
            check_name=self.name,
            passed=passed,
            score=round(aggregate_score, 4),
            message=message,
            metadata={
                "scores": scores,
                "judge_model": resolved_model,
                "thresholds": {
                    "pass_threshold": self.pass_threshold,
                    "min_criterion_score": self.min_criterion_score,
                },
                "rubric": self._rubric_map,
                "pass_verdict": parsed.pass_verdict,
            },
        )


@dataclass
class TestPassCheck:
    """Run pytest (or configured test command); fail if exit code is non-zero.

    Uses asyncio.create_subprocess_exec -- not shim_run.
    # @trace WL-090
    """

    name: str = "test_pass"
    test_runner: str = "pytest"
    extra_args: list[str] = field(default_factory=lambda: ["--tb=short", "-q"])
    timeout_seconds: int = 120
    cwd: str | None = None

    async def check(
        self,
        run_id: str,
        output: str,
        context: dict[str, Any],
    ) -> VetterCheckResult:
        """Run test suite check. # @trace WL-090"""
        changed_files = _extract_changed_py_files(output)

        cmd_args: list[str] = [self.test_runner, *self.extra_args]
        if changed_files:
            cmd_args.extend(changed_files)

        proc = await asyncio.create_subprocess_exec(
            *cmd_args,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.STDOUT,
            cwd=self.cwd or context.get("cwd"),
        )

        try:
            stdout, _ = await asyncio.wait_for(proc.communicate(), timeout=self.timeout_seconds)
        except TimeoutError:
            proc.kill()  # Non-async; asyncio.subprocess.Process.kill() is synchronous
            await proc.wait()
            return VetterCheckResult(
                check_name=self.name,
                passed=False,
                message=f"Test runner timed out after {self.timeout_seconds}s",
                metadata={"timeout": True},
            )

        test_output = stdout.decode(errors="replace")
        passed = proc.returncode == 0

        return VetterCheckResult(
            check_name=self.name,
            passed=passed,
            message="" if passed else test_output[-2000:],
            metadata={"returncode": proc.returncode, "files_tested": changed_files},
        )


@dataclass
class RuffCheck:
    """Run ruff check on Python files touched in the diff.

    Uses asyncio.create_subprocess_exec -- not shim_run.
    Fails fast: non-zero ruff exit code = passed=False.
    # @trace WL-090
    """

    name: str = "ruff"
    select_rules: list[str] = field(default_factory=list)
    cwd: str | None = None

    async def check(
        self,
        run_id: str,
        output: str,
        context: dict[str, Any],
    ) -> VetterCheckResult:
        """Run ruff linter check. # @trace WL-090"""
        changed_files = _extract_changed_py_files(output)

        if not changed_files:
            return VetterCheckResult(
                check_name=self.name,
                passed=True,
                message="No Python files in diff -- ruff check skipped",
            )

        cmd_args: list[str] = ["ruff", "check"]
        if self.select_rules:
            cmd_args.extend(["--select", ",".join(self.select_rules)])
        cmd_args.extend(changed_files)

        proc = await asyncio.create_subprocess_exec(
            *cmd_args,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.STDOUT,
            cwd=self.cwd or context.get("cwd"),
        )

        stdout, _ = await proc.communicate()
        ruff_output = stdout.decode(errors="replace")
        passed = proc.returncode == 0

        return VetterCheckResult(
            check_name=self.name,
            passed=passed,
            message="" if passed else ruff_output,
            metadata={"returncode": proc.returncode, "files_checked": changed_files},
        )


# ---------------------------------------------------------------------------
# Internal helpers
# ---------------------------------------------------------------------------

_PY_FILE_RE = re.compile(r"^(?:\+\+\+|---)\s+(?:a/|b/)?(\S+\.py)", re.MULTILINE)


def _extract_changed_py_files(diff_text: str) -> list[str]:
    """Extract unique .py filenames from a unified diff header. # @trace WL-090"""
    return list(dict.fromkeys(_PY_FILE_RE.findall(diff_text)))


# ---------------------------------------------------------------------------
# WL-091: Phase-1 Vetter Checks
# ---------------------------------------------------------------------------

# Secret patterns: bearer tokens, AWS access keys, GitHub PATs, sk-... API keys
_SECRET_PATTERNS: list[re.Pattern[str]] = [
    re.compile(r"Bearer\s+[A-Za-z0-9\-_\.]{20,}", re.IGNORECASE),
    re.compile(r"AKIA[0-9A-Z]{16}"),
    re.compile(r"ghp_[A-Za-z0-9]{30,}"),
    re.compile(r"sk-[A-Za-z0-9]{20,}"),
]

# PII patterns: email addresses, SSN (NNN-NN-NNNN)
_PII_PATTERNS: list[re.Pattern[str]] = [
    re.compile(r"[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}"),
    re.compile(r"\b\d{3}-\d{2}-\d{4}\b"),
]


@dataclass
class SchemaVetterCheck:
    """Validate JSON agent output against a Pydantic BaseModel schema.

    target selects which stream to parse:
      - "stdout"   (default): uses context["stdout"]
      - "stderr":             uses context["stderr"]
      - "combined":           concatenates context["stdout"] + context["stderr"]

    Fails fast:
      - passed=False, reason="JSON parse failed: {e}" on JSONDecodeError
      - passed=False, reason="Schema validation failed: {e}" on ValidationError

    No silent fallbacks. No external dependencies beyond Pydantic.
    # @trace WL-091
    """

    schema_model: type[BaseModel]
    target: Literal["stdout", "stderr", "combined"] = "stdout"
    name: str = "schema_vetter"

    def _select_text(self, output: str, context: dict[str, Any]) -> str:
        """Select the text to validate based on target. # @trace WL-091"""
        if self.target == "stdout":
            return context.get("stdout", output)
        if self.target == "stderr":
            return context.get("stderr", "")
        # combined
        return context.get("stdout", "") + context.get("stderr", "")

    async def check(
        self,
        run_id: str,
        output: str,
        context: dict[str, Any],
    ) -> VetterCheckResult:
        """Run Pydantic schema validation check. # @trace WL-091"""
        text = self._select_text(output, context)

        try:
            json.loads(text)
        except json.JSONDecodeError as exc:
            return VetterCheckResult(
                check_name=self.name,
                passed=False,
                message=f"JSON parse failed: {exc}",
            )

        try:
            self.schema_model.model_validate_json(text)
        except ValidationError as exc:
            return VetterCheckResult(
                check_name=self.name,
                passed=False,
                message=f"Schema validation failed: {exc}",
            )

        return VetterCheckResult(check_name=self.name, passed=True)


@dataclass
class DiffSizeVetterCheck:
    """Reject outputs where unified-diff line count exceeds max_lines_changed.

    Counts lines starting with '+' or '-', excluding '+++' / '---' headers.
    Fails fast when count > max_lines_changed (strict greater-than).
    No external dependencies.
    # @trace WL-091
    """

    max_lines_changed: int = 500
    name: str = "diff_size_vetter"

    async def check(
        self,
        run_id: str,
        output: str,
        context: dict[str, Any],
    ) -> VetterCheckResult:
        """Run diff size check. # @trace WL-091"""
        lines_changed = sum(
            1 for line in output.splitlines() if line.startswith(("+", "-")) and not line.startswith(("+++", "---"))
        )

        if lines_changed > self.max_lines_changed:
            return VetterCheckResult(
                check_name=self.name,
                passed=False,
                message=(f"Diff size {lines_changed} lines exceeds max {self.max_lines_changed}"),
                metadata={"lines_changed": lines_changed, "max_lines_changed": self.max_lines_changed},
            )

        return VetterCheckResult(
            check_name=self.name,
            passed=True,
            metadata={"lines_changed": lines_changed},
        )


@dataclass
class SafetyVetterCheck:
    """Check output for secrets and PII using pure-regex patterns.

    Secret patterns (checked first):
      - Bearer tokens (Authorization: Bearer ...)
      - AWS access keys (AKIA...)
      - GitHub PATs (ghp_...)
      - OpenAI-style API keys (sk-...)

    PII patterns (checked if no secrets found):
      - Email addresses
      - US SSN pattern (NNN-NN-NNNN)

    If secrets found: passed=False, reason="Secret pattern detected"
    If PII found:     passed=False, reason="PII pattern detected"
    No external dependencies. Fail fast, fail loudly.
    # @trace WL-091
    """

    name: str = "safety_vetter"

    async def check(
        self,
        run_id: str,
        output: str,
        context: dict[str, Any],
    ) -> VetterCheckResult:
        """Run regex-based secret and PII detection. # @trace WL-091"""
        for pattern in _SECRET_PATTERNS:
            if pattern.search(output):
                return VetterCheckResult(
                    check_name=self.name,
                    passed=False,
                    message="Secret pattern detected",
                )

        for pattern in _PII_PATTERNS:
            if pattern.search(output):
                return VetterCheckResult(
                    check_name=self.name,
                    passed=False,
                    message="PII pattern detected",
                )

        return VetterCheckResult(check_name=self.name, passed=True)


# ---------------------------------------------------------------------------
# WL-097: Vetter Code Checks — TestPassVetterCheck + RuffVetterCheck
# ---------------------------------------------------------------------------


@dataclass
class TestPassVetterCheck:
    """Run pytest (or configured test runner) on changed Python files from the diff.

    Extracts changed .py files from the unified diff in ``output`` (the agent's
    RunResult.stdout).  If no Python files are found in the diff, runs pytest
    with no file arguments (i.e., the full suite).

    Uses shim_run (not asyncio.create_subprocess_exec) so that tests can
    mock shim_run without an asyncio harness.

    Fail fast: non-zero exit code -> passed=False, revision_hint contains the
    truncated test output.  No silent error handling.

    # @trace WL-097
    """

    test_runner: str = "pytest"
    scope: str = "changed_files"
    timeout_seconds: int = 120
    name: str = "test_pass_vetter"
    extra_args: list[str] = field(default_factory=lambda: ["--tb=short", "-q"])
    cwd: str | None = None

    async def check(
        self,
        run_id: str,
        output: str,
        context: dict[str, Any],
    ) -> VetterCheckResult:
        """Run pytest on changed Python files extracted from the diff.

        # @trace WL-097
        """
        changed_files = _extract_changed_py_files(output)

        cmd: list[str] = [self.test_runner, *self.extra_args]
        if changed_files and self.scope == "changed_files":
            cmd.extend(changed_files)

        try:
            proc = shim_run(
                cmd,
                capture_output=True,
                timeout=self.timeout_seconds,
                cwd=self.cwd or context.get("cwd"),
            )
        except subprocess.TimeoutExpired:
            return VetterCheckResult(
                check_name=self.name,
                passed=False,
                message=f"Test runner timed out after {self.timeout_seconds}s",
                metadata={"timeout": True, "files_tested": changed_files},
            )

        combined_output = (proc.stdout or b"").decode(errors="replace") + (proc.stderr or b"").decode(errors="replace")
        passed = proc.returncode == 0

        return VetterCheckResult(
            check_name=self.name,
            passed=passed,
            message="" if passed else combined_output[-2000:],
            metadata={"returncode": proc.returncode, "files_tested": changed_files},
        )


@dataclass
class RuffVetterCheck:
    """Run ruff check on Python files touched in the diff.

    Extracts changed .py files from the unified diff in ``output`` (the agent's
    RunResult.stdout).  If no Python files are found in the diff, returns
    passed=True (nothing to lint).

    Uses shim_run (not asyncio.create_subprocess_exec) so that tests can
    mock shim_run without an asyncio harness.

    fix_mode=True passes --fix to ruff (auto-fix enabled).
    select_rules limits which rules are evaluated via --select.

    Fail fast: non-zero ruff exit code -> passed=False, revision_hint contains
    the full ruff output.  No silent error handling.

    # @trace WL-097
    """

    fix_mode: bool = False
    select_rules: list[str] = field(default_factory=list)
    name: str = "ruff_vetter"
    cwd: str | None = None

    async def check(
        self,
        run_id: str,
        output: str,
        context: dict[str, Any],
    ) -> VetterCheckResult:
        """Run ruff linter on changed Python files extracted from the diff.

        # @trace WL-097
        """
        changed_files = _extract_changed_py_files(output)

        if not changed_files:
            return VetterCheckResult(
                check_name=self.name,
                passed=True,
                message="No Python files in diff — ruff check skipped",
            )

        cmd: list[str] = ["ruff", "check"]
        if self.fix_mode:
            cmd.append("--fix")
        if self.select_rules:
            cmd.extend(["--select", ",".join(self.select_rules)])
        cmd.extend(changed_files)

        proc = shim_run(
            cmd,
            capture_output=True,
            cwd=self.cwd or context.get("cwd"),
        )

        ruff_output = (proc.stdout or b"").decode(errors="replace") + (proc.stderr or b"").decode(errors="replace")
        passed = proc.returncode == 0

        return VetterCheckResult(
            check_name=self.name,
            passed=passed,
            message="" if passed else ruff_output,
            metadata={"returncode": proc.returncode, "files_checked": changed_files},
        )
