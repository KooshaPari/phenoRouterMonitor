---
work_package_id: "WP02"
subtasks:
  - "T007"
  - "T008"
  - "T009"
  - "T010"
  - "T011"
title: "Python MCP Service Scaffold"
phase: "Phase 0 - Foundation"
lane: "planned"
dependencies:
  - "WP01"
assignee: ""
agent: ""
shell_pid: ""
review_status: ""
reviewed_by: ""
history:
  - timestamp: "2026-02-27T00:00:00Z"
    lane: "planned"
    agent: "system"
    shell_pid: ""
    action: "Prompt generated via /spec-kitty.tasks"
---

# Work Package Prompt: WP02 -- Python MCP Service Scaffold

## IMPORTANT: Review Feedback Status

**Read this first if you are implementing this task!**

- **Has review feedback?**: Check the `review_status` field above. If it says `has_feedback`, scroll to the **Review Feedback** section immediately.
- **You must address all feedback** before your work is complete.
- **Mark as acknowledged**: Update `review_status: acknowledged` when you begin addressing feedback.
- **Report progress**: Update the Activity Log as you address each item.

---

## Review Feedback

> **Populated by `/spec-kitty.review`** -- Reviewers add detailed feedback here when work needs changes.

*[This section is empty initially.]*

---

## Markdown Formatting
Wrap HTML/XML tags in backticks: `` `<div>` ``, `` `<script>` ``
Use language identifiers in code blocks: ````python`, ````bash`

---

## Implementation Command

```bash
spec-kitty implement WP02 --base WP01
```

---

## Objectives & Success Criteria

1. **MCP server starts**: `uv run python -m agileplus_mcp` launches without error.
2. **Health check responds**: MCP server responds to a basic health/ping tool call.
3. **gRPC client stub connects**: gRPC client can be instantiated targeting `localhost:50051` (connection will fail without Rust server, but client creation succeeds).
4. **Tool stubs registered**: All MCP tools from `contracts/mcp-tools.json` appear in the server's tool registry.
5. **Test structure exists**: `mcp/tests/unit/`, `mcp/tests/bdd/`, `mcp/tests/contract/` directories created with `__init__.py` and conftest.py files.
6. **uv manages deps**: `uv sync` installs all dependencies; `uv run pytest` runs (0 tests, 0 errors).

---

## Context & Constraints

### Reference Documents
- **Spec**: `kitty-specs/001-spec-driven-development-engine/spec.md` -- MCP integration requirements
- **Plan**: `kitty-specs/001-spec-driven-development-engine/plan.md` -- Python MCP project structure (lines 135-152), technical context
- **Contracts**: `kitty-specs/001-spec-driven-development-engine/contracts/mcp-tools.json` -- MCP tool definitions
- **Research**: `kitty-specs/001-spec-driven-development-engine/research.md` -- FastMCP 3.0 evaluation

### Architectural Constraints
- **FastMCP 3.0**: Use the latest FastMCP release (3.x) for server creation. The server is the Python-side entry point for LLM tool calls.
- **gRPC thin client**: Python MCP service does NO domain logic. Every tool call forwards to the Rust core via gRPC. Python is a translation layer only.
- **uv package manager**: Use `uv` (not pip, poetry, or pipenv) for all Python dependency management.
- **Python 3.13+**: Target free-threaded Python 3.13. Fall back to 3.12 if FastMCP compatibility issues arise.
- **Separate process**: Python MCP runs as its own process, communicating with Rust via gRPC on localhost:50051.

### Dependency on WP01
- WP01 must complete first because:
  - Proto stubs generated in WP01/T006 are needed for the gRPC client.
  - Docker Compose from WP01/T005 defines the Python MCP container.
  - The `proto/agileplus.proto` file is the source of truth for the gRPC interface.

---

## Subtasks & Detailed Guidance

### Subtask T007 -- Create `mcp/pyproject.toml` with FastMCP 3.0, grpcio, opentelemetry-sdk deps

- **Purpose**: Establish the Python project with all required dependencies so subsequent subtasks can import and use them. The pyproject.toml is the single source of truth for Python package metadata and deps.
- **Steps**:
  1. Create `mcp/` directory at repository root (if not already present from WP01).
  2. Create `mcp/pyproject.toml`:
     ```toml
     [project]
     name = "agileplus-mcp"
     version = "0.1.0"
     description = "AgilePlus MCP Service - FastMCP 3.0 bridge to Rust core"
     requires-python = ">=3.12"
     dependencies = [
         "fastmcp>=3.0",
         "grpcio>=1.68",
         "grpcio-tools>=1.68",
         "protobuf>=5.29",
         "opentelemetry-sdk>=1.29",
         "opentelemetry-exporter-otlp>=1.29",
         "pydantic>=2.10",
     ]

     [project.optional-dependencies]
     dev = [
         "pytest>=8.0",
         "pytest-asyncio>=0.24",
         "pytest-cov>=6.0",
         "ruff>=0.8",
         "behave>=1.2",
         "pact-python>=2.2",
         "mypy>=1.13",
     ]

     [build-system]
     requires = ["hatchling"]
     build-backend = "hatchling.build"

     [tool.hatch.build.targets.wheel]
     packages = ["src/agileplus_mcp"]

     [tool.ruff]
     target-version = "py312"
     line-length = 100

     [tool.ruff.lint]
     select = ["E", "F", "I", "N", "W", "UP", "ANN", "S", "B"]

     [tool.pytest.ini_options]
     testpaths = ["tests"]
     asyncio_mode = "auto"

     [tool.mypy]
     python_version = "3.12"
     strict = true
     ```
  3. Create `mcp/.python-version` with content `3.13` (uv will use this).
  4. Run `cd mcp && uv sync` to create the virtual environment and lockfile.
- **Files**: `mcp/pyproject.toml`, `mcp/.python-version`
- **Parallel?**: No -- must complete before T008-T011 can import anything.
- **Validation**: `cd mcp && uv sync` succeeds; `uv run python -c "import fastmcp; print(fastmcp.__version__)"` prints version.
- **Notes**: The `grpcio-tools` dep is needed for proto generation (used by WP01/T006's Python script). Pin minimum versions, not exact, to allow uv to resolve compatible combinations. If Python 3.13 free-threaded causes issues with grpcio C extensions, fall back to `requires-python = ">=3.12"` and `.python-version = "3.12"`.

### Subtask T008 -- Create `mcp/src/agileplus_mcp/__init__.py` and `server.py` (FastMCP entry)

- **Purpose**: Create the FastMCP server entry point that registers all tools and starts the MCP server. This is the main executable for the Python MCP service.
- **Steps**:
  1. Create directory structure: `mcp/src/agileplus_mcp/`
  2. Create `mcp/src/agileplus_mcp/__init__.py`:
     ```python
     """AgilePlus MCP Service - FastMCP 3.0 bridge to Rust core via gRPC."""

     __version__ = "0.1.0"
     ```
  3. Create `mcp/src/agileplus_mcp/__main__.py` for `python -m` execution:
     ```python
     """Entry point for `python -m agileplus_mcp`."""

     from agileplus_mcp.server import main

     main()
     ```
  4. Create `mcp/src/agileplus_mcp/server.py`:
     ```python
     """FastMCP 3.0 server for AgilePlus."""

     from fastmcp import FastMCP

     from agileplus_mcp.tools import features, governance, status

     mcp = FastMCP(
         name="agileplus",
         description="Spec-driven development engine with governance",
     )

     # Register tool modules
     features.register(mcp)
     governance.register(mcp)
     status.register(mcp)


     def main() -> None:
         """Start the MCP server."""
         mcp.run()
     ```
  5. Ensure the server can start in stdio mode (default FastMCP transport) and SSE mode (for web clients).
- **Files**: `mcp/src/agileplus_mcp/__init__.py`, `mcp/src/agileplus_mcp/__main__.py`, `mcp/src/agileplus_mcp/server.py`
- **Parallel?**: No -- T009 and T010 depend on this for imports.
- **Validation**: `cd mcp && uv run python -c "from agileplus_mcp.server import mcp; print(mcp.name)"` prints "agileplus".
- **Notes**: FastMCP 3.0 uses a decorator-based API for tool registration. The `mcp.run()` call starts the server in stdio mode by default. For SSE mode, use `mcp.run(transport="sse")`. Keep the server.py minimal -- tool logic lives in the tools/ modules.

### Subtask T009 -- Create `mcp/src/agileplus_mcp/grpc_client.py` (stub gRPC connection to Rust core)

- **Purpose**: Provide a typed gRPC client that connects to the Rust core service. All MCP tool handlers call through this client. At scaffold time, the client is a stub that can be instantiated but won't connect until the Rust gRPC server (WP14) is running.
- **Steps**:
  1. Create `mcp/src/agileplus_mcp/grpc_client.py`:
     ```python
     """gRPC client for communication with Rust AgilePlus core."""

     from __future__ import annotations

     import logging
     from dataclasses import dataclass, field

     import grpc

     logger = logging.getLogger(__name__)

     DEFAULT_HOST = "localhost"
     DEFAULT_PORT = 50051


     @dataclass
     class AgilePlusCoreClient:
         """Client for the AgilePlus Rust core gRPC service."""

         host: str = DEFAULT_HOST
         port: int = DEFAULT_PORT
         _channel: grpc.Channel | None = field(default=None, init=False, repr=False)

         @property
         def target(self) -> str:
             return f"{self.host}:{self.port}"

         def connect(self) -> None:
             """Establish gRPC channel to the Rust core."""
             logger.info("Connecting to AgilePlus core at %s", self.target)
             self._channel = grpc.insecure_channel(self.target)

         def close(self) -> None:
             """Close the gRPC channel."""
             if self._channel is not None:
                 self._channel.close()
                 self._channel = None

         async def get_feature(self, slug: str) -> dict:
             """Stub: Get feature by slug via gRPC."""
             raise NotImplementedError("gRPC stubs not yet generated")

         async def list_features(self) -> list[dict]:
             """Stub: List all features via gRPC."""
             raise NotImplementedError("gRPC stubs not yet generated")

         async def check_governance(self, feature_id: int) -> dict:
             """Stub: Check governance status via gRPC."""
             raise NotImplementedError("gRPC stubs not yet generated")

         async def get_audit_trail(self, feature_id: int) -> list[dict]:
             """Stub: Get audit trail via gRPC."""
             raise NotImplementedError("gRPC stubs not yet generated")
     ```
  2. Add type stubs for all gRPC methods matching `proto/agileplus.proto` service definition.
  3. Methods should raise `NotImplementedError` with a descriptive message until proto stubs are wired in WP14.
- **Files**: `mcp/src/agileplus_mcp/grpc_client.py`
- **Parallel?**: Yes -- independent after T008. Can run alongside T010.
- **Validation**: `cd mcp && uv run python -c "from agileplus_mcp.grpc_client import AgilePlusCoreClient; c = AgilePlusCoreClient(); print(c.target)"` prints "localhost:50051".
- **Notes**: Use `grpc.insecure_channel` for development. Production will need TLS, but that is out of scope for this WP. The async methods will be wired to actual gRPC stubs (generated from proto) in WP14. Keep the client dataclass-based for easy testing and mocking.

### Subtask T010 -- Create `mcp/src/agileplus_mcp/tools/` directory with stub tool files

- **Purpose**: Create the MCP tool modules that will be registered with the FastMCP server. Each module corresponds to a domain area and contains tool function stubs. The tool definitions should match `contracts/mcp-tools.json`.
- **Steps**:
  1. Create `mcp/src/agileplus_mcp/tools/__init__.py`:
     ```python
     """MCP tool modules for AgilePlus."""
     ```
  2. Create `mcp/src/agileplus_mcp/tools/features.py`:
     ```python
     """Feature management MCP tools."""

     from __future__ import annotations

     from fastmcp import FastMCP


     def register(mcp: FastMCP) -> None:
         """Register feature tools with the MCP server."""

         @mcp.tool()
         async def get_feature(slug: str) -> dict:
             """Get a feature by its slug identifier.

             Args:
                 slug: The kebab-case feature slug (e.g., '001-spec-engine')

             Returns:
                 Feature details including state, spec hash, and timestamps.
             """
             return {"error": "not_implemented", "message": "Feature lookup not yet wired to gRPC"}

         @mcp.tool()
         async def list_features(state: str | None = None) -> list[dict]:
             """List all features, optionally filtered by state.

             Args:
                 state: Optional state filter (created, specified, planned, etc.)

             Returns:
                 List of feature summaries.
             """
             return [{"error": "not_implemented"}]

         @mcp.tool()
         async def get_work_packages(feature_slug: str) -> list[dict]:
             """Get all work packages for a feature.

             Args:
                 feature_slug: The parent feature's slug

             Returns:
                 List of work packages with states and dependencies.
             """
             return [{"error": "not_implemented"}]
     ```
  3. Create `mcp/src/agileplus_mcp/tools/governance.py`:
     ```python
     """Governance and audit MCP tools."""

     from __future__ import annotations

     from fastmcp import FastMCP


     def register(mcp: FastMCP) -> None:
         """Register governance tools with the MCP server."""

         @mcp.tool()
         async def check_governance(feature_slug: str) -> dict:
             """Check governance contract compliance for a feature.

             Args:
                 feature_slug: Feature to check governance for

             Returns:
                 Governance check result with pass/fail and violations.
             """
             return {"error": "not_implemented"}

         @mcp.tool()
         async def get_audit_trail(feature_slug: str, limit: int = 50) -> list[dict]:
             """Get the hash-chained audit trail for a feature.

             Args:
                 feature_slug: Feature to get audit trail for
                 limit: Maximum number of entries to return

             Returns:
                 List of audit entries with hash chain integrity status.
             """
             return [{"error": "not_implemented"}]

         @mcp.tool()
         async def verify_audit_chain(feature_slug: str) -> dict:
             """Verify integrity of a feature's audit hash chain.

             Args:
                 feature_slug: Feature whose audit chain to verify

             Returns:
                 Verification result: valid/invalid with first broken entry if invalid.
             """
             return {"error": "not_implemented"}
     ```
  4. Create `mcp/src/agileplus_mcp/tools/status.py`:
     ```python
     """Status and dashboard MCP tools."""

     from __future__ import annotations

     from fastmcp import FastMCP


     def register(mcp: FastMCP) -> None:
         """Register status tools with the MCP server."""

         @mcp.tool()
         async def get_dashboard() -> dict:
             """Get a dashboard summary of all active features.

             Returns:
                 Summary with feature counts by state, active WPs, recent audit entries.
             """
             return {"error": "not_implemented"}

         @mcp.tool()
         async def get_metrics(feature_slug: str | None = None) -> dict:
             """Get telemetry metrics, optionally scoped to a feature.

             Args:
                 feature_slug: Optional feature to scope metrics to

             Returns:
                 Metrics including duration, agent runs, review cycles.
             """
             return {"error": "not_implemented"}
     ```
  5. Each tool function should have comprehensive docstrings (these become MCP tool descriptions visible to LLMs).
  6. All tools return stub responses with `{"error": "not_implemented"}` until WP14 wires them to gRPC.
- **Files**: `mcp/src/agileplus_mcp/tools/__init__.py`, `tools/features.py`, `tools/governance.py`, `tools/status.py`
- **Parallel?**: Yes -- independent after T008. Can run alongside T009.
- **Validation**: `cd mcp && uv run python -c "from agileplus_mcp.server import mcp; print([t.name for t in mcp.list_tools()])"` lists all registered tool names.
- **Notes**: Tool docstrings are critical -- they are the LLM-facing documentation. Be descriptive about args and return values. Match the tool names and schemas to `contracts/mcp-tools.json` if that file exists. If not, use the names defined here as the initial contract.

### Subtask T011 -- Create `mcp/tests/` directory structure (unit/, bdd/, contract/)

- **Purpose**: Establish the test directory structure matching the project's test strategy: unit tests (pytest), BDD tests (behave), and contract tests (pact-python). Each directory should have proper Python packaging files and a basic conftest.
- **Steps**:
  1. Create directory tree:
     ```
     mcp/tests/
     ├── __init__.py
     ├── conftest.py
     ├── unit/
     │   ├── __init__.py
     │   └── test_server.py
     ├── bdd/
     │   ├── __init__.py
     │   ├── environment.py
     │   └── features/
     │       └── .gitkeep
     └── contract/
         ├── __init__.py
         └── .gitkeep
     ```
  2. Create `mcp/tests/conftest.py` with shared fixtures:
     ```python
     """Shared test fixtures for AgilePlus MCP tests."""

     import pytest

     from agileplus_mcp.grpc_client import AgilePlusCoreClient


     @pytest.fixture
     def grpc_client() -> AgilePlusCoreClient:
         """Create a gRPC client for testing (not connected)."""
         return AgilePlusCoreClient(host="localhost", port=50051)
     ```
  3. Create `mcp/tests/unit/test_server.py` with a minimal smoke test:
     ```python
     """Smoke tests for MCP server initialization."""

     def test_server_creates() -> None:
         from agileplus_mcp.server import mcp
         assert mcp.name == "agileplus"

     def test_tools_registered() -> None:
         from agileplus_mcp.server import mcp
         tool_names = [t.name for t in mcp.list_tools()]
         assert "get_feature" in tool_names
         assert "check_governance" in tool_names
         assert "get_dashboard" in tool_names
     ```
  4. Create `mcp/tests/bdd/environment.py` with behave setup stub.
- **Files**: `mcp/tests/__init__.py`, `mcp/tests/conftest.py`, `mcp/tests/unit/__init__.py`, `mcp/tests/unit/test_server.py`, `mcp/tests/bdd/`, `mcp/tests/contract/`
- **Parallel?**: No -- depends on T008 and T010 for imports.
- **Validation**: `cd mcp && uv run pytest tests/ -v` passes with 2+ tests.
- **Notes**: The `test_tools_registered` test serves as a contract: if someone removes a tool, this test catches it. BDD and contract directories are empty placeholders for WP16. Use `.gitkeep` files so git tracks empty directories.

---

## Test Strategy

- **Primary validation**: `cd mcp && uv run pytest tests/ -v`
- **Expected**: 2+ passing tests (server smoke tests from T011)
- **Lint**: `cd mcp && uv run ruff check .` must pass
- **Type check**: `cd mcp && uv run mypy src/` should pass (may need type stubs for fastmcp)
- **Server start**: `cd mcp && uv run python -m agileplus_mcp` starts without crash (Ctrl+C to exit)

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Python 3.13 free-threaded incompatible with grpcio C extension | Import errors at runtime | Fall back to Python 3.12; update `.python-version` |
| FastMCP 3.0 API changes (rapid development) | Tool registration breaks | Pin fastmcp version in pyproject.toml once working version confirmed |
| grpcio-tools version mismatch with proto | Generated stubs incompatible | Pin grpcio and grpcio-tools to same version |
| uv lockfile conflicts | Dependency resolution fails | Use `uv lock --upgrade` to refresh; check uv version is latest |
| MCP tool names drift from contracts | LLM tool calls fail | T011 test_tools_registered acts as contract test |

---

## Review Guidance

Reviewers should verify:

1. **pyproject.toml completeness**: All deps present, version constraints reasonable, dev deps separate.
2. **Server starts**: `uv run python -m agileplus_mcp` launches without error.
3. **Tool registration**: All tools from contracts/mcp-tools.json (or defined stubs) are registered.
4. **gRPC client stub**: Client can be instantiated, target address correct, methods raise NotImplementedError.
5. **Docstrings quality**: Tool docstrings are descriptive enough for LLM consumption.
6. **Test structure**: All three test directories exist, smoke tests pass.
7. **No domain logic**: Python side is a pass-through only. No business logic in tool handlers.
8. **Coding standards**: ruff passes, type hints present on all public functions.

---

## Activity Log

> **CRITICAL**: Activity log entries MUST be in chronological order (oldest first, newest last).

### How to Add Activity Log Entries

**When adding an entry**:
1. Scroll to the bottom of this Activity Log section
2. **APPEND the new entry at the END** (do NOT prepend or insert in middle)
3. Use exact format: `- YYYY-MM-DDTHH:MM:SSZ – agent_id – lane=<lane> – <action>`
4. Timestamp MUST be current time in UTC
5. Lane MUST match the frontmatter `lane:` field exactly

**Valid lanes**: `planned`, `doing`, `for_review`, `done`

- 2026-02-27T00:00:00Z – system – lane=planned – Prompt created.
