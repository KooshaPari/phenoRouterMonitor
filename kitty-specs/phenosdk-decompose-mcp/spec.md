# phenoSDK: Extract pheno-mcp Package

## Problem
pheno/mcp (104 files) is the MCP tooling layer — FastMCP wrappers, tool registry, agent orchestration. Currently atoms-specific and embedded in the monolith. Should be a standalone pheno-mcp package usable by any MCP server, not just atoms.

## Acceptance Criteria
- [ ] pheno-mcp: new package with mcp/, tools/, agents/ modules
- [ ] No atoms-specific references in pheno-mcp
- [ ] FastMCP integration properly abstracted via port
- [ ] CrewAI adapter generalized (not atoms-specific)
- [ ] pheno-mcp depends on pheno-core only
- [ ] Integration tests with mock MCP server
- [ ] Published to Phenotype GitHub Packages

## Key extractions
- pheno/mcp/tools/decorators.py (FastMCP decorators, 349 LOC)
- pheno/mcp/agents/orchestration.py (CrewAI adapter, 372 LOC)
- pheno/mcp/entry_points.py (generalized, atoms refs removed)
