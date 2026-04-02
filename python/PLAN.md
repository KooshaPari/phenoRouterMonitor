# python Plan

## Overview
Python workspace with SDK packages for Phenotype and AgilePlus MCP.

## Phases

### Phase 1: Core Packages (3 weeks)
- pheno-core (config, errors, logging)
- pheno-atoms (types and validation)
- phenosdk (infrastructure operations)

### Phase 2: Agent Features (3 weeks)
- pheno-agents (orchestration)
- pheno-llm (LLM integration)
- pheno-mcp (MCP tooling)

### Phase 3: Integration (2 weeks)
- agileplus_mcp service
- agileplus_proto gRPC stubs
- Integration tests

## Deliverables
- 7 working Python packages
- Full test coverage
- PyPI publish ready
- MCP service with gRPC backend

## Resources
- 2 Python developers
- Dependencies: pydantic, structlog, httpx