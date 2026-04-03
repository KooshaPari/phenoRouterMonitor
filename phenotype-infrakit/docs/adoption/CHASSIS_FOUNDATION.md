# Phenotype Chassis Foundation — Phase 1

## Overview

The **Chassis Foundation** establishes a unified design system and shared governance layer for the Phenotype ecosystem. This phase creates core infrastructure enabling federation and multi-repo extensibility.

**Phase Status**: Complete (2026-03-29)
**Target**: @phenotype/docs Chassis, spec-driven framework, 10-minute adoption

---

## Core Components

### 1. @phenotype/docs Chassis

Unified design system and documentation shell:
- Design System (Radix + shadcn/ui)
- VitePress Configuration  
- Component Library
- Navigation Templates
- Governance Models

### 2. AgilePlus Governance Chassis

Spec-driven delivery framework with:
- Unified spec registry
- Automated PR generation
- Work progress tracking
- Evidence-based delivery
- Cross-repo dependency tracking

### 3. Adoption Pattern

Three-step integration for new repos:

1. Add submodule: `git submodule add https://github.com/KooshaPari/phenotype-docs.git docs/.phenotype-chassis`
2. Configure VitePress with Chassis config
3. Create `.agileplus/config.json` with governance settings

---

## Governance Framework

### Spec-Driven Delivery (SDD)

All work follows: **spec → PR → review → evidence → merge**

### Evidence-Based Delivery

Every PR requires:
- ✅ All tests passing
- ✅ Lint/format clean  
- ✅ Build succeeds
- ✅ Docs generated
- ✅ Security scans pass

---

## Benefits

| Benefit | Impact |
|---------|--------|
| Design Consistency | All UIs follow unified system |
| Documentation Parity | Seamless navigation between projects |
| Governance Alignment | Predictable, repeatable workflows |
| Code Reuse | 40-50% LOC reduction |
| Dependency Clarity | Safe refactoring and updates |
| Evidence Trail | Audit trail for compliance |

---

## Quick Checklist

- [ ] Clone Chassis repo, review config
- [ ] Add submodule, extend VitePress
- [ ] Create `.agileplus/config.json`
- [ ] Create first spec
- [ ] Generate PR from spec
- [ ] Merge PR with evidence

---

## Next: Phase 2 Federation

Once Chassis is in place:
1. Module Federation — Dynamic module loading
2. Micro-Frontends — Composed UIs
3. Intent-Driven Orchestration — Agent-driven features
4. Feedback Loops — Auto-healing

See FEDERATED_HYBRID_ARCHITECTURE.md for Phase 2.
