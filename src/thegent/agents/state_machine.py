"""Fallback State Machine for agent orchestration.

Manages the lifecycle of a task run across multiple providers and retry attempts,
enforcing fallback policies and semantic validation gates.
"""

import hashlib
import json
import logging
import time
import uuid
from dataclasses import dataclass, field
from pathlib import Path
from types import MappingProxyType
from typing import Any, ClassVar

from thegent.agents.base import RunResult
from thegent.agents.resilience import FailureKind, TransientAgentError, classify_failure, with_retry
from thegent.contracts.adapters import AdapterResult, normalize_output
from thegent.contracts.policy import FallbackPolicy, evaluate_fallback
from thegent.contracts.telemetry import (
    EVENT_NORMALIZATION,
    EVENT_SCHEMA_DRIFT_SEMANTIC,
    EVENT_SCHEMA_DRIFT_STRUCTURAL,
    ContractTelemetry,
)
from thegent.contracts.validation import validate_csm

_log = logging.getLogger(__name__)


class PromotionGate:
    """WP-1005: Evidence capture and validation before state promotion."""

    def __init__(self, session_dir: Path) -> None:
        self.session_dir = session_dir

    def capture_evidence(self, run_id: str, csm: Any) -> str:
        """Capture and hash CSM state as evidence."""
        evidence_data = json.dumps(csm.to_dict().decode(), sort_keys=True)
        evidence_hash = hashlib.sha256(evidence_data.encode()).hexdigest()

        # Store evidence snapshot
        evidence_dir = self.session_dir / "evidence"
        evidence_dir.mkdir(parents=True, exist_ok=True)
        (evidence_dir / f"{run_id}_{csm.phase.value}.json").write_text(evidence_data)

        return evidence_hash

    def validate_promotion(self, csm: Any, policy: Any) -> list[str]:
        """Validate if CSM is ready for promotion based on policy."""
        issues = []
        if csm.confidence_level < policy.min_confidence_threshold:
            issues.append(f"Confidence {csm.confidence_level} below threshold {policy.min_confidence_threshold}")
        if csm.blockers:
            issues.append(f"Active blockers present: {csm.blockers}")
        return issues


@dataclass
class OrchestrationState:
    """State of an orchestration attempt."""

    agent: str
    run_id: str
    model: str | None = None
    attempt: int = 0
    provider_index: int = 0
    providers_tried: list[str] = field(default_factory=list)
    errors: list[str] = field(default_factory=list)
    start_time: float = field(default_factory=time.time)
    status: str = "pending"  # pending, running, success, failed, fallback
    last_result: RunResult | None = None
    last_normalization: AdapterResult | None = None
    policy_issues: list[str] = field(default_factory=list)
    semantic_issues: list[str] = field(default_factory=list)


class FallbackStateMachine:
    """State machine for managing orchestration fallbacks."""

    def __init__(
        self,
        providers: list[str],
        run_id: str | None = None,
        policy: FallbackPolicy | None = None,
        telemetry: ContractTelemetry | None = None,
        max_retries_per_provider: int = 3,
        retry_delay_base: float = 2.0,
    ) -> None:
        self.providers = providers
        self.run_id = run_id or f"run_{uuid.uuid4().hex[:8]}"
        self.policy = policy or FallbackPolicy()
        self.telemetry = telemetry
        self.max_retries = max_retries_per_provider
        self.retry_delay = retry_delay_base
        self.state = OrchestrationState(
            agent=providers[0] if providers else "unknown",
            run_id=self.run_id,
        )

    _PHASE_TRANSITIONS: ClassVar[Any] = MappingProxyType(
        {
            "pending": ["running", "failed"],
            "running": ["success", "failed", "fallback", "paused"],
            "fallback": ["running", "failed"],
            "paused": ["running", "failed"],
            "success": ["completed"],
            "failed": ["rolled_back"],
        }
    )

    def validate_transition(self, from_state: str, to_state: str) -> bool:
        """Validate if a state transition is allowed (WP-1004)."""
        allowed = self._PHASE_TRANSITIONS.get(from_state, [])
        return to_state in allowed

    def suggest_fallbacks(self) -> list[dict[str, Any]]:
        """Suggest safe fallback options for the current failed/blocked state (WP-4003)."""
        suggestions = []

        # 1. Next provider in chain
        if self.state.provider_index < len(self.providers) - 1:
            next_agent = self.providers[self.state.provider_index + 1]
            suggestions.append(
                {
                    "type": "provider_fallback",
                    "agent": next_agent,
                    "reason": "Next provider in pre-configured fallback chain",
                    "command": f'thegent run --agent {next_agent} --prompt "{self.state.run_id}" --retry',
                }
            )

        # 2. Strict policy relaxation
        if self.state.policy_issues:
            suggestions.append(
                {
                    "type": "policy_override",
                    "reason": "Relax strict policy constraints (requires audit)",
                    "command": f'thegent run --override "Force retry after policy block" --run-id {self.state.run_id}',
                }
            )

        # 3. Model-first alternative
        if self.state.model:
            suggestions.append(
                {
                    "type": "model_fallback",
                    "reason": "Try a different provider for the same model",
                    "command": f"thegent run --model {self.state.model} --failover --run-id {self.state.run_id}",
                }
            )

        return suggestions

    def _run_with_retry(
        self,
        runner: Any,
        prompt: str,
        run_kwargs: dict[str, Any],
        current_agent: str,
        span: Any,
    ) -> RunResult:
        """Run agent with tenacity retry on RATE_LIMIT/TRANSIENT failures."""

        @with_retry(
            max_attempts=self.max_retries,
            min_wait=self.retry_delay,
            max_wait=60.0,
        )
        def _run() -> RunResult:
            self.state.attempt += 1
            span.add_event(
                "agent_attempt",
                attributes={"agent": current_agent, "attempt": self.state.attempt},
            )
            _log.info("Attempting %s (attempt %d/%d)", current_agent, self.state.attempt, self.max_retries)
            result = runner.run(prompt=prompt, **run_kwargs)
            if result.exit_code != 0:
                failure_kind = classify_failure(result)
                if failure_kind in (FailureKind.RATE_LIMIT, FailureKind.TRANSIENT):
                    raise TransientAgentError(result)
            return result

        return _run()

    def run(
        self,
        runner_factory: Any,  # Callable[[str], Optional[AgentRunner]]
        prompt: str,
        model: str | None = None,
        **run_kwargs: Any,
    ) -> tuple[RunResult, AdapterResult | None]:
        """Execute the orchestration loop."""
        from opentelemetry import trace

        tracer = trace.get_tracer("thegent.orchestrator")

        if not self.providers:
            raise ValueError("No providers specified for orchestration.")

        self.state.model = model
        self.state.status = "running"

        # Get stats for global fallback rate check
        stats = self.telemetry.get_stats(limit=100) if self.telemetry else None

        with tracer.start_as_current_span(
            "orchestration.run",
            attributes={
                "thegent.run_id": self.run_id,
                "thegent.providers": self.providers,
                "thegent.policy": str(self.policy),
            },
        ) as span:
            while self.state.provider_index < len(self.providers):
                current_agent = self.providers[self.state.provider_index]
                self.state.agent = current_agent
                if current_agent not in self.state.providers_tried:
                    self.state.providers_tried.append(current_agent)

                self.state.attempt = 0

                # SLO Latency Check (WP-X6)
                elapsed_ms = (time.time() - self.state.start_time) * 1000
                if elapsed_ms > self.policy.max_latency_ms:
                    _log.warning("SLO: max_latency_ms exceeded (%dms > %dms)", elapsed_ms, self.policy.max_latency_ms)
                    self.state.errors.append(f"SLO Timeout ({current_agent})")
                    break

                # 1. Resolve runner
                runner = runner_factory(current_agent)
                if runner is None:
                    _log.error("No runner found for %s", current_agent)
                    self.state.errors.append(f"No runner for {current_agent}")
                    break

                # 2. Execution (with tenacity retry for RATE_LIMIT/TRANSIENT)
                try:
                    result = self._run_with_retry(runner, prompt, run_kwargs, current_agent, span)
                except TransientAgentError as e:
                    self.state.last_result = e.result
                    _log.warning(
                        "Run failed for %s after %d retries (code %d)",
                        current_agent,
                        self.max_retries,
                        e.result.exit_code,
                    )
                    self.state.errors.append(f"Run failed ({current_agent}, code {e.result.exit_code})")
                    break
                except Exception as e:
                    import traceback

                    _log.error("Execution error for %s: %s\n%s", current_agent, e, traceback.format_exc())
                    self.state.errors.append(f"Execution error ({current_agent}): {e}")
                    break

                self.state.last_result = result

                # 3. Failure Classification (non-retryable)
                failure_kind = classify_failure(result)
                if failure_kind == FailureKind.USAGE_LIMIT:
                    _log.warning("Usage limit reached for %s. Falling back.", current_agent)
                    self.state.errors.append(f"Usage limit ({current_agent})")
                    break

                if result.exit_code != 0:
                    _log.warning("Run failed for %s (code %d)", current_agent, result.exit_code)
                    self.state.errors.append(f"Run failed ({current_agent}, code {result.exit_code})")
                    break

                # 4. Normalization and Semantic Validation
                norm_res = normalize_output(
                    current_agent,
                    {"stdout": result.stdout, "stderr": result.stderr, "exit_code": result.exit_code},
                    context={"run_id": self.run_id},
                )
                self.state.last_normalization = norm_res

                semantic_issues = validate_csm(norm_res.csm)
                self.state.semantic_issues = semantic_issues
                if semantic_issues:
                    _log.warning("Semantic validation failed for %s: %s", current_agent, semantic_issues)

                # 5. Fallback Policy Evaluation
                is_fallback = norm_res.csm.source_contract == "fallback-plain"
                policy_violations = evaluate_fallback(
                    provider=current_agent,
                    confidence=norm_res.confidence,
                    is_fallback=is_fallback,
                    policy=self.policy,
                    stats=stats,
                )
                self.state.policy_issues = policy_violations

                # 6. Record Telemetry (G-RV-07: drift event types)
                if self.telemetry:
                    success = not norm_res.parse_errors and not policy_violations and not semantic_issues
                    errors = (norm_res.parse_errors or []) + policy_violations + semantic_issues
                    if norm_res.parse_errors:
                        event_type = EVENT_SCHEMA_DRIFT_STRUCTURAL
                        self.telemetry.emit_drift_event(
                            self.run_id,
                            current_agent,
                            norm_res.csm.source_contract,
                            "structural",
                            {"parse_errors": norm_res.parse_errors},
                        )
                    elif semantic_issues:
                        event_type = EVENT_SCHEMA_DRIFT_SEMANTIC
                        self.telemetry.emit_drift_event(
                            self.run_id,
                            current_agent,
                            norm_res.csm.source_contract,
                            "semantic",
                            {"semantic_issues": semantic_issues},
                        )
                    else:
                        event_type = EVENT_NORMALIZATION
                    self.telemetry.record_normalization(
                        run_id=self.run_id,
                        provider=current_agent,
                        contract=norm_res.csm.source_contract,
                        confidence=norm_res.confidence,
                        success=success,
                        errors=errors,
                        event_type=event_type,
                    )

                if not policy_violations and not semantic_issues:
                    # WP-1005: Promotion Gate
                    if self.telemetry:
                        gate = PromotionGate(self.telemetry.session_dir)
                        gate_issues = gate.validate_promotion(norm_res.csm, self.policy)
                        if gate_issues:
                            _log.warning("Promotion gate failed for %s: %s", current_agent, gate_issues)
                            self.state.errors.append(f"Promotion gate failure ({current_agent})")
                            break

                        evidence_hash = gate.capture_evidence(self.run_id, norm_res.csm)
                        # store promotion evidence in the CSM raw_payload for compatibility
                        norm_res.csm.raw_payload["promotion_evidence_hash"] = evidence_hash

                    if self.validate_transition(self.state.status, "success"):
                        self.state.status = "success"
                    return result, norm_res

                # If we have other providers, move to next provider
                if self.state.provider_index < len(self.providers) - 1:
                    _log.info("Violations found and fallbacks available. Moving to next provider.")
                    self.state.errors.append(f"Policy/Semantic violation ({current_agent})")
                    break
                # No more providers. Accept if not a hard block.
                hard_block = any("disabled" in v or "strict" in v for v in policy_violations)
                if not hard_block:
                    _log.info("No more providers. Accepting output despite violations.")
                    self.state.status = "success"
                    return result, norm_res
                self.state.status = "failed"
                self.state.errors.append("Policy blocked all available providers.")
                return result, norm_res

            self.state.provider_index += 1
            self.state.status = "fallback"

        self.state.status = "failed"
        return (
            self.state.last_result
            or RunResult(exit_code=1, stdout="", stderr="Orchestration failed: no results available")
        ), self.state.last_normalization
