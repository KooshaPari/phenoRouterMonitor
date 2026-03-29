"""Agent capability indexing and recommendation engine (WL-034).

Scans ~/.claude/agents/*.md and .claude/agents/*.md files, parses YAML
frontmatter, builds an in-memory capability index, and ranks agents for
a given task description using keyword/TF-IDF overlap scoring.
"""

from __future__ import annotations

import logging
import math
import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from cachetools import TTLCache
from thegent.integrations.base import SerializableMixin
from thegent.infra.fast_yaml_parser import yaml_load

logger = logging.getLogger(__name__)

# TTL of 60 seconds for the capability index cache
_INDEX_CACHE: TTLCache[str, CapabilityIndex] = TTLCache(maxsize=1, ttl=60)  # type: ignore[name-defined]
_CACHE_KEY = "singleton"


@dataclass
class AgentRecord:
    """Parsed agent definition from a frontmatter .md file."""

    name: str
    path: Path
    description: str
    capabilities: list[str]
    model: str | None
    runner: str | None
    raw_frontmatter: dict[str, Any]
    body: str


@dataclass
class AgentRecommendation(SerializableMixin):
    """A recommended agent with its relevance score."""

    name: str
    path: Path
    score: float
    description: str
    capabilities: list[str]
    runner: str | None = None


@dataclass
class DoctorResult:
    """Health check result for a single agent."""

    name: str
    path: Path
    valid_frontmatter: bool
    has_runner_config: bool
    issues: list[str] = field(default_factory=list)

    @property
    def healthy(self) -> bool:
        return self.valid_frontmatter and self.has_runner_config and not self.issues


def _parse_frontmatter(content: str) -> tuple[dict[str, Any], str]:
    """Parse YAML frontmatter from markdown content.

    Returns (frontmatter_dict, body_text). Raises ValueError if frontmatter is
    present but not valid YAML.
    """
    if not content.startswith("---"):
        return {}, content

    # Find closing ---
    end = content.find("\n---", 3)
    if end == -1:
        # Frontmatter not closed — treat whole content as unparseable
        raise ValueError("Frontmatter block not closed (missing closing '---')")

    fm_text = content[3:end].strip()
    body = content[end + 4 :].strip()

    parsed = yaml_load(fm_text)
    if parsed is None:
        return {}, body
    if not isinstance(parsed, dict):
        raise ValueError(f"Frontmatter must be a YAML mapping, got {type(parsed).__name__}")
    return parsed, body


def _coerce_list(value: Any) -> list[str]:
    """Coerce a YAML value to a list of strings."""
    if value is None:
        return []
    if isinstance(value, list):
        return [str(v) for v in value]
    if isinstance(value, str):
        return [v.strip() for v in value.split(",") if v.strip()]
    return [str(value)]


def _load_agent_file(path: Path) -> AgentRecord | None:
    """Load and parse a single agent .md file.

    Returns None if the file cannot be read. Raises ValueError for malformed
    frontmatter so callers can distinguish missing-file from parse failure.
    """
    content = path.read_text(encoding="utf-8")
    fm, body = _parse_frontmatter(content)

    name = str(fm.get("name") or path.stem)
    description = str(fm.get("description") or "")
    capabilities = _coerce_list(fm.get("capabilities"))
    model = fm.get("model")
    runner = fm.get("runner")

    return AgentRecord(
        name=name,
        path=path,
        description=description,
        capabilities=capabilities,
        model=str(model) if model is not None else None,
        runner=str(runner) if runner is not None else None,
        raw_frontmatter=fm,
        body=body,
    )


def _try_load_agent_file(path: Path) -> AgentRecord | None:
    """Load an agent file, returning None and logging on any error.

    Separates exception handling from the scanning loop to satisfy PERF203.
    """
    try:  # noqa: SIM105 -- intentional: log-and-skip on any parse error
        return _load_agent_file(path)
    except Exception as exc:
        logger.warning("Skipping %s: %s", path, exc)
        return None


def _glob_agent_dirs() -> list[Path]:
    """Return a list of directories to scan for agent .md files."""
    dirs: list[Path] = []

    global_dir = Path("~/.claude/agents").expanduser()
    if global_dir.exists() and global_dir.is_dir():
        dirs.append(global_dir)

    # Walk up from cwd looking for .claude/agents
    cwd = Path.cwd()
    for candidate in [cwd, *cwd.parents]:
        local_dir = candidate / ".claude" / "agents"
        if local_dir.exists() and local_dir.is_dir():
            dirs.append(local_dir)
            break  # Only the nearest project root

    return dirs


def _tokenize(text: str) -> list[str]:
    """Tokenize text into lower-cased words, stripping punctuation."""
    return re.findall(r"[a-z0-9]+", text.lower())


def _tf_idf_score(query_tokens: list[str], agent: AgentRecord, idf: dict[str, float]) -> float:
    """Compute a TF-IDF-based overlap score between query and agent text corpus.

    The agent corpus is: description tokens + capability tokens.
    Score = sum of idf(token) for each query token present in agent corpus.
    Normalised by the maximum possible score (sum of all query token idfs).
    """
    agent_text = " ".join([agent.description, *agent.capabilities, agent.name])
    agent_tokens = set(_tokenize(agent_text))

    total_idf = sum(idf.get(t, 0.0) for t in set(query_tokens))
    if total_idf == 0.0:
        return 0.0

    matched_idf = sum(idf.get(t, 0.0) for t in set(query_tokens) if t in agent_tokens)
    return matched_idf / total_idf


class CapabilityIndex:
    """In-memory index of agent capabilities built from frontmatter .md files.

    Usage:
        index = CapabilityIndex.get()          # uses 60-second TTL singleton
        recs = index.recommend("review python code", top_n=3)
        results = index.doctor()
    """

    def __init__(self, agents: list[AgentRecord]) -> None:
        self._agents = agents
        self._by_capability: dict[str, list[AgentRecord]] = {}
        self._idf: dict[str, float] = {}
        self._build_index()

    def _build_index(self) -> None:
        """Build capability index and IDF table from loaded agents."""
        # Capability → agents
        for agent in self._agents:
            for cap in agent.capabilities:
                key = cap.lower().strip()
                if key not in self._by_capability:
                    self._by_capability[key] = []
                self._by_capability[key].append(agent)

        # Build IDF: over all agent corpora
        N = len(self._agents)
        if N == 0:
            return

        # Document frequency per token
        df: dict[str, int] = {}
        for agent in self._agents:
            corpus = " ".join([agent.description, *agent.capabilities, agent.name])
            for tok in set(_tokenize(corpus)):
                df[tok] = df.get(tok, 0) + 1

        # IDF = log((N + 1) / (df + 1)) + 1  (smoothed)
        self._idf = {tok: math.log((N + 1) / (cnt + 1)) + 1.0 for tok, cnt in df.items()}

    @classmethod
    def build(cls, extra_dirs: list[Path] | None = None) -> CapabilityIndex:
        """Build a fresh CapabilityIndex by scanning agent directories."""
        scan_dirs = _glob_agent_dirs()
        if extra_dirs:
            scan_dirs.extend(extra_dirs)

        agents: list[AgentRecord] = []
        for d in scan_dirs:
            for md_file in sorted(d.glob("*.md")):
                record = _try_load_agent_file(md_file)
                if record is not None:
                    agents.append(record)

        logger.debug("CapabilityIndex: loaded %d agents from %d dirs", len(agents), len(scan_dirs))
        return cls(agents)

    @classmethod
    def get(cls, extra_dirs: list[Path] | None = None) -> CapabilityIndex:
        """Return the cached CapabilityIndex, rebuilding if expired (TTL=60s)."""
        if _CACHE_KEY not in _INDEX_CACHE:
            _INDEX_CACHE[_CACHE_KEY] = cls.build(extra_dirs)
        return _INDEX_CACHE[_CACHE_KEY]

    @classmethod
    def invalidate(cls) -> None:
        """Force cache invalidation (useful for tests)."""
        _INDEX_CACHE.clear()

    def agents_for_capability(self, capability: str) -> list[AgentRecord]:
        """Return agents that declared the given capability."""
        return self._by_capability.get(capability.lower().strip(), [])

    def all_agents(self) -> list[AgentRecord]:
        """Return all indexed agents."""
        return list(self._agents)

    def recommend(self, task_description: str, top_n: int = 3) -> list[AgentRecommendation]:
        """Recommend top-N agents for the given task description.

        Uses TF-IDF keyword overlap between task_description and each agent's
        description + capabilities. No LLM calls.

        Args:
            task_description: Free-text description of the task to perform.
            top_n: Maximum number of recommendations to return.

        Returns:
            List of AgentRecommendation sorted by score descending.
        """
        query_tokens = _tokenize(task_description)
        if not query_tokens:
            return []

        scored: list[tuple[float, AgentRecord]] = []
        for agent in self._agents:
            score = _tf_idf_score(query_tokens, agent, self._idf)
            if score > 0.0:
                scored.append((score, agent))

        scored.sort(key=lambda x: (-x[0], x[1].name))

        return [
            AgentRecommendation(
                name=a.name,
                path=a.path,
                score=round(s, 4),
                description=a.description,
                capabilities=a.capabilities,
                runner=a.runner,
            )
            for s, a in scored[:top_n]
        ]

    def doctor(self) -> list[DoctorResult]:
        """Validate all indexed agents for runner config and frontmatter health.

        Checks:
        - Frontmatter is parseable YAML (validated at load time)
        - Agent has a 'model' or 'runner' field
        - No dangling/invalid references in raw frontmatter

        Returns:
            List of DoctorResult, one per agent.
        """
        results: list[DoctorResult] = []
        for agent in self._agents:
            issues: list[str] = []

            has_runner_config = bool(agent.model or agent.runner)
            if not has_runner_config:
                issues.append("Missing 'model' or 'runner' field in frontmatter")

            if not agent.description:
                issues.append("Missing 'description' field")

            # Check for dangling: if 'runner' is set, it should be a non-empty string
            if agent.runner is not None and not agent.runner.strip():
                issues.append("'runner' field is empty string")

            results.append(
                DoctorResult(
                    name=agent.name,
                    path=agent.path,
                    valid_frontmatter=True,  # If we got here, frontmatter parsed OK
                    has_runner_config=has_runner_config,
                    issues=issues,
                )
            )

        return results
