# phenoSDK: Remove atoms.tech Identifiers

## Problem
phenoSDK was extracted from zen-mcp-server (atoms.tech school capstone MCP backend). The code contains atoms-specific identifiers: AtomsMCPEntryPoint, AtomsMCPCLI, ATOMS_MCP_RISK_ASSESSMENT.md, author "ATOMS-PHENO Team", description "ATOMS-PHENO SDK for infrastructure migration and operations". This must be sanitized before any open-source release or sharing.

## Acceptance Criteria
- [ ] All `Atoms` / `ATOMS` / `atoms` identifiers in MCP entry points replaced with generic equivalents
- [ ] pyproject.toml: author changed to Phenotype org, description updated to generic SDK
- [ ] ATOMS_MCP_RISK_ASSESSMENT.md reviewed and removed or sanitized
- [ ] No references to atoms.tech domain or atoms school project in public-facing code
- [ ] Tests still pass after rename

## Scope
File: src/pheno/mcp/entry_points.py + src/pheno/shared/mcp_entry_points.py + pyproject.toml
