import logging
from typing import TYPE_CHECKING, Any

from thegent.utils.routing_impl.cliproxy_client import CLIProxyRoutingClient
from thegent.utils.routing_impl.models import TaskCategory, TaskMetadata

if TYPE_CHECKING:
    from thegent.config import ThegentSettings
    from thegent.execution import RunRegistry

logger = logging.getLogger(__name__)


# Role-based routing (subscription-optimized, Terminal Bench 2.0)
# Roles map to optimal models based on task characteristics

_ROLE_TO_MODEL = {
    # WORKHORSE: Bulk tasks, default fallback (minimax quota: 300/5hrs)
    "workhorse": "minimax",
    # RESEARCHER: Fast lookups, exploration (free tier)
    "researcher": "gemini",  # gemini-3-flash primary, haiku-4.5 fallback
    # WRITER: Code implementation, medium-high quality
    "writer_fast": "codex",  # codex-spark for NORMAL
    "writer_standard": "codex",  # codex-5.3-med for NORMAL (better quality)
    "writer_high": "codex",  # codex-5.3-high for COMPLEX (SLOW, high quality)
    # PLANNER: Architecture, design, reasoning (SLOW)
    "planner": "claude",  # opus-4.6
    # LARGE_CONTEXT: >100K tokens, cross-file analysis
    "large_context": "claude",  # sonnet-1m (1M context)
    # EXPERT: Never auto-routed (glm-5 too slow, only explicit)
    "expert": "glm",  # Never used in auto-routing
}

# Keyword-based role detection
_PLANNER_KEYWORDS = {
    "design",
    "architecture",
    "plan",
    "strategy",
    "approach",
    "tradeoffs",
    "pros and cons",
    "which approach",
    "should we",
    "system design",
    "schema design",
    "api design",
    "evaluate options",
    "decide between",
}

_WRITER_KEYWORDS = {
    "implement",
    "write",
    "create",
    "build",
    "add",
    "fix",
    "refactor",
    "optimize",
    "code",
    "function",
    "class",
    "module",
    "test",
    "debug",
}

_RESEARCHER_KEYWORDS = {
    "what is",
    "how does",
    "explain",
    "find",
    "search",
    "list",
    "show me",
    "where is",
    "analyze",
    "investigate",
    "read",
    "understand",
    "learn",
    "explore",
    "research",
}

_MISSION_CRITICAL_KEYWORDS = {
    "security",
    "authentication",
    "authorization",
    "payment",
    "encryption",
    "production",
    "critical",
    "zero-downtime",
    "data integrity",
    "compliance",
    "audit",
}

_LARGE_CONTEXT_INDICATORS = {
    "across all files",
    "entire codebase",
    "all modules",
    "cross-file",
    "multi-file",
    "refactor all",
}


class TaskClassifier:
    """Categorizes tasks based on prompt analysis and heuristics."""

    def __init__(self, config: "ThegentSettings") -> None:
        self.config = config

    def detect_role(self, prompt: str, agent_role: str | None = None) -> str:
        """Detect task role from agent metadata or prompt keywords.

        Priority:
        1. Agent-specified role (from agent frontmatter)
        2. Auto-detect from prompt keywords
        3. Default to "workhorse"

        Args:
            prompt: User prompt text
            agent_role: Role from agent metadata (e.g., "planner", "writer", "researcher")

        Returns:
            Role string (workhorse/researcher/writer_fast/writer_high/planner/large_context)
        """
        # Priority 1: Agent specifies role
        if agent_role:
            return agent_role.lower()

        # Priority 2: Auto-detect from prompt
        prompt_lower = prompt.lower()

        # Check for large context indicators first
        if any(kw in prompt_lower for kw in _LARGE_CONTEXT_INDICATORS):
            return "large_context"

        # Check for mission-critical indicators (writer_high tier)
        if any(kw in prompt_lower for kw in _MISSION_CRITICAL_KEYWORDS):
            return "writer_high"  # Use Codex XHigh for security/payment/etc.

        # Check for planning keywords
        if any(kw in prompt_lower for kw in _PLANNER_KEYWORDS):
            return "planner"

        # Check for implementation keywords
        if any(kw in prompt_lower for kw in _WRITER_KEYWORDS):
            # Default to fast writer for NORMAL, will upgrade to high if COMPLEX category
            return "writer_fast"

        # Check for research keywords
        if any(kw in prompt_lower for kw in _RESEARCHER_KEYWORDS):
            return "researcher"

        # Default: workhorse (minimax)
        return "workhorse"

    def classify(self, prompt: str, agent_role: str | None = None) -> TaskMetadata:
        """
        Classify task complexity based on prompt content.
        Heuristics:
        - Word count (token estimate proxy)
        - Keywords (architecture, design -> HIGH_COMPLEX)
        - Structure (bullets, code blocks -> COMPLEX)
        """
        prompt_lower = prompt.lower()
        word_count = len(prompt.split())
        estimated_tokens = int(word_count * 1.3)

        # Simple keyword-based complexity scoring
        high_complex_keywords = ["architecture", "design", "refactor", "optimize", "security", "database"]
        complex_keywords = ["implement", "test", "debug", "fix", "improve", "handle"]

        complexity_score = 0.0
        if any(kw in prompt_lower for kw in high_complex_keywords):
            complexity_score += 0.5
        if any(kw in prompt_lower for kw in complex_keywords):
            complexity_score += 0.2

        # Token count weight
        complexity_score += min(0.3, estimated_tokens / 5000)

        # Categorization
        if complexity_score >= 0.7 or estimated_tokens > 5000:
            category = TaskCategory.HIGH_COMPLEX
        elif complexity_score >= 0.4 or estimated_tokens > 1500:
            category = TaskCategory.COMPLEX
        elif complexity_score >= 0.15 or estimated_tokens > 300:
            category = TaskCategory.NORMAL
        else:
            category = TaskCategory.FAST

        # Estimate duration and cost (placeholders based on category)
        estimates = {
            TaskCategory.FAST: (1.0, 0.002),
            TaskCategory.NORMAL: (5.0, 0.03),
            TaskCategory.COMPLEX: (20.0, 0.15),
            TaskCategory.HIGH_COMPLEX: (60.0, 0.85),
        }
        duration, cost = estimates[category]

        return TaskMetadata(
            category=category,
            complexity_score=complexity_score,
            estimated_tokens=estimated_tokens,
            estimated_cost=cost,
            estimated_duration_s=duration,
            reasoning=f"Classified as {category} based on complexity score {complexity_score:.2f}",
            signals={
                "word_count": word_count,
                "token_estimate": estimated_tokens,
                "complexity_score": complexity_score,
            },
        )


class ConstraintValidator:
    """Validates task metadata against configured constraints."""

    def __init__(self, config: "ThegentSettings") -> None:
        self.config = config

    def validate(
        self,
        task_metadata: TaskMetadata,
        registry: "RunRegistry | None" = None,
        model: str | None = None,
    ) -> list[str]:
        """
        Validate task against hard constraints:
        - Instantaneous cost (per-call)
        - Cumulative cost (MTD per category)
        - Speed (SLA)
        """
        violations = []
        category = task_metadata.category

        # 1. Instantaneous cost check
        from thegent.cost.aggregator import CostEstimator

        estimator = CostEstimator()
        actual_est_cost = estimator.estimate(
            model=model,
            prompt_length=task_metadata.signals.get("word_count", 0) * 5,  # proxy for chars
        )

        max_cost = {
            TaskCategory.FAST: 0.01,
            TaskCategory.NORMAL: 0.20,
            TaskCategory.COMPLEX: 1.00,
            TaskCategory.HIGH_COMPLEX: 5.00,
        }.get(category, 1.0)

        if actual_est_cost > max_cost:
            violations.append(f"Cost: Estimated ${actual_est_cost:.3f} exceeds max ${max_cost:.3f} for {category}")

        # 2. Cumulative budget check (if registry provided)
        if registry:
            from thegent.cost.aggregator import CostAggregator

            agg = CostAggregator(registry.session_dir)
            mtd_total = agg.get_mtd_total()
            cost_budget = float(getattr(self.config, "cost_budget_mtd", 100.0))
            if mtd_total >= cost_budget:
                violations.append(f"Budget: Monthly total ${mtd_total:.2f} exceeds budget ${cost_budget:.2f}")

        # 3. Speed SLA check
        sla = {
            TaskCategory.FAST: 5.0,
            TaskCategory.NORMAL: 15.0,
            TaskCategory.COMPLEX: 45.0,
            TaskCategory.HIGH_COMPLEX: 180.0,
        }.get(category, 60.0)

        if task_metadata.estimated_duration_s > sla:
            violations.append(
                f"Speed: Estimated {task_metadata.estimated_duration_s}s exceeds {sla}s SLA for {category}"
            )

        return violations


class TaskRouter:
    """Orchestrates task classification and constraint validation."""

    def __init__(self, config: "ThegentSettings") -> None:
        self.config = config
        self.classifier = TaskClassifier(config)
        self.validator = ConstraintValidator(config)

    def classify(self, prompt: str) -> TaskMetadata:
        """Classify task."""
        return self.classifier.classify(prompt)

    def validate(
        self,
        task_metadata: TaskMetadata,
        registry: "RunRegistry | None" = None,
        model: str | None = None,
    ) -> list[str]:
        """Validate task against constraints."""
        return self.validator.validate(task_metadata, registry, model)

    def route(
        self,
        prompt: str,
        registry: "RunRegistry | None" = None,
        model: str | None = None,
    ) -> tuple[TaskMetadata, list[str]]:
        """
        Full routing: classify + validate.
        Returns (TaskMetadata, violations).
        """
        task = self.classify(prompt)
        violations = self.validate(task, registry, model)
        return task, violations

    def route_via_cliproxy(
        self,
        prompt: str,
        max_cost_per_call: float = 0.01,
        max_latency_ms: int = 5000,
        min_quality_score: float = 0.7,
    ) -> tuple[str, str, float]:
        """Route using CLIProxy Pareto router endpoint.

        Returns:
            tuple: (model_id, provider, estimated_cost)

        @trace FR-CLIPROXY-INTEGRATION-002
        """
        task = self.classify(prompt)
        category = task.category.value.upper()

        try:
            with CLIProxyRoutingClient() as client:
                result = client.select_model(
                    task_complexity=category,
                    max_cost_per_call=max_cost_per_call,
                    max_latency_ms=max_latency_ms,
                    min_quality_score=min_quality_score,
                )
                logger.debug(
                    f"CLIProxy routing: {result.model_id} (provider={result.provider}, "
                    f"cost=${result.estimated_cost:.4f})"
                )
                return result.model_id, result.provider, result.estimated_cost
        except Exception as e:
            logger.warning(f"CLIProxy routing failed, falling back to local: {e}")
            # Fall back to local routing
            violations = self.validate(task, None, None)
            if not violations:
                # Return a default model based on category
                defaults = {
                    "FAST": ("gemini-3-flash", "gemini", 0.0001),
                    "NORMAL": ("claude-sonnet-4.5", "claude", 0.003),
                    "COMPLEX": ("claude-opus-4.6", "claude", 0.015),
                }
                return defaults.get(category, ("claude-opus-4.6-1m", "claude", 0.015))
            raise

    def get_fallback_chain(self, category: TaskCategory) -> list[str]:
        """Get LiteLLM-style fallback chain for task category (WP-1001)."""
        from thegent.execution import LoadClassifier, ProviderScorer

        lc = LoadClassifier(self.config.session_dir)
        load_level = lc.get_load_level()

        # WP-5008: Dynamic tuning based on load
        if load_level == "burst":
            # In burst mode, prefer faster/cheaper models for all but critical
            if category != TaskCategory.HIGH_COMPLEX:
                return ["gemini-3-flash", "claude-haiku-4.5"]

        # WP-Y8: Provider scoring and learning
        scorer = ProviderScorer(self.config.session_dir)
        scores = scorer.get_scores()

        # Map category to characteristic
        characteristic = "coding"
        if category == TaskCategory.FAST:
            characteristic = "research"
        elif category == TaskCategory.HIGH_COMPLEX:
            characteristic = "orchestration"

        char_scores = scores.get(characteristic, {})
        if char_scores:
            # Sort providers by score desc
            sorted_providers = sorted(char_scores.items(), key=lambda x: x[1], reverse=True)
            # Map top providers back to known model IDs (simplified)
            mapping = {"codex": "gpt-5.3-codex", "claude": "claude-sonnet-4.5", "gemini": "gemini-3.1-pro"}
            return [mapping[p] for p, _s in sorted_providers if p in mapping]

        if category == TaskCategory.FAST:
            return ["gemini-3-flash", "claude-haiku-4.5"]
        if category == TaskCategory.NORMAL:
            return ["gpt-5.3-codex-spark", "claude-haiku-4.5"]
        if category == TaskCategory.COMPLEX:
            return ["gpt-5.3-codex", "gemini-3.1-pro"]
        return ["claude-opus-4.6", "gpt-5.3-codex-max"]

    def route_dag_tasks(self, dag: Any) -> dict[str, list[str]]:
        """Route multiple tasks from a DAG, considering dependencies (WP-1001)."""
        routing_table = {}
        for task_id, task in getattr(dag, "tasks", {}).items():
            meta = self.classify(task.get("prompt", ""))
            routing_table[task_id] = self.get_fallback_chain(meta.category)
        return routing_table

    def route_by_capability(self, task_type: str) -> str:
        """Route to an agent based on task capability (WP-1007)."""
        mapping = {
            "coding": "codex",
            "research": "gemini",
            "orchestration": "claude",
            "review": "cursor-agent",
            "fast-fix": "copilot",
        }
        return mapping.get(task_type, "gemini")

    def should_delegate_to_reviewer(self, confidence: float) -> bool:
        """Determine if a task should be delegated to a reviewer based on confidence (WP-1007)."""
        # Threshold G-CA-02 B1: confidence < 0.7 triggers reviewer delegation
        return confidence < 0.7

    def shape_task(self, prompt: str, category: TaskCategory) -> dict[str, Any]:
        """WP-11006: Adaptive task shaping (split/merge engine)."""
        # Simplified shaping logic
        word_count = len(prompt.split())

        if category == TaskCategory.HIGH_COMPLEX and word_count > 500:
            return {
                "action": "split",
                "reason": "Task exceeds complexity and size threshold for single run.",
                "sub_tasks": ["Phase 1: Discovery", "Phase 2: Implementation", "Phase 3: Validation"],
                "rationale": "Large complex tasks are more reliable when decomposed.",
            }

        if category == TaskCategory.FAST and word_count < 10:
            return {
                "action": "merge",
                "reason": "Task is trivial; candidate for batching.",
                "rationale": "Reducing overhead for micro-tasks.",
            }

        return {"action": "none", "reason": "Task size and complexity optimal."}

    def find_active_terminal_for_path(self, path: str) -> str | None:
        """
        Find an active tmux pane matching the given project path.
        Returns pane_id if found.
        """
        from pathlib import Path

        from thegent.skills.terminal import is_claude_code_pane, list_tmux_panes

        target_path = Path(path).resolve()
        panes = list_tmux_panes()
        # 1. Exact match
        for p in panes:
            if Path(p.path).resolve() == target_path and is_claude_code_pane(p):
                return p.pane_id

        # 2. Subpath/Parent match (deepest first)
        best_pane = None
        best_depth = -1
        for p in panes:
            if not is_claude_code_pane(p):
                continue
            pane_path = Path(p.path).resolve()
            if target_path in pane_path.parents or pane_path in target_path.parents:
                depth = len(pane_path.parts)
                if depth > best_depth:
                    best_depth = depth
                    best_pane = p.pane_id

        return best_pane
