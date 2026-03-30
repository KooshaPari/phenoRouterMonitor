# AgilePlus Work Packages — Quick Reference

**Date**: 2026-03-30 | **Status**: Ready for Execution | **Total**: 25 WPs

---

## Directory Structure

```
repos/
├── forgecode-fork/.agileplus/specs/              (8 WPs)
├── phenotype-router-monitor/.agileplus/specs/   (9 WPs)
├── bifrost-routing/.agileplus/specs/            (8 WPs)
└── docs/reports/
    └── AGILEPLUS_WP_CREATION_SUMMARY_2026-03-30.md  (comprehensive)
```

---

## Repo Quick Links

| Repo | WPs | Status | Phase 1 | Phase 2 | Est. Calls |
|------|-----|--------|---------|---------|-----------|
| forgecode-fork | 8 | Ready | 6 | 2 | 85-99 |
| phenotype-router-monitor | 9 | Ready | 7 | 2 | 87-105 |
| bifrost-routing | 8 | Ready | 6 | 2 | 83-100 |
| **TOTAL** | **25** | **Ready** | **19** | **6** | **255-304** |

---

## WP List by Fork

### forgecode-fork (eco-fork-001)

| ID | Title | Phase | Calls | Status | Depends |
|----|-------|-------|-------|--------|---------|
| WP-001 | Provider Trait & Registry | 1 | 12-15 | ⏳ Pending | - |
| WP-002 | Claude Provider | 1 | 10-12 | ⏳ Pending | WP-001 |
| WP-003 | Local Provider + Ollama | 1 | 8-10 | ⏳ Pending | WP-001 |
| WP-004 | Subagent Spawning | 1 | 14-16 | ⏳ Pending | WP-002,003 |
| WP-005 | Capability Discovery | 2 | 11-13 | ⏳ Pending | WP-002,003,004 |
| WP-006 | Performance Metrics | 2 | 10-12 | ⏳ Pending | WP-004,005 |
| WP-007 | CI/CD Integration | 1 | 8-10 | ⏳ Pending | - |
| WP-008 | Documentation & Release | 1 | 12-15 | ⏳ Pending | WP-001-007 |

### phenotype-router-monitor (eco-fork-002)

| ID | Title | Phase | Calls | Status | Depends |
|----|-------|-------|-------|--------|---------|
| WP-001 | Router Core | 1 | 13-15 | ⏳ Pending | - |
| WP-002 | Health Checking | 1 | 11-13 | ⏳ Pending | WP-001 |
| WP-003 | Rate Limiting | 1 | 10-12 | ⏳ Pending | WP-001 |
| WP-004 | Metrics Collection | 1 | 10-12 | ⏳ Pending | WP-001 |
| WP-005 | CLI & Dashboard | 2 | 9-11 | ⏳ Pending | WP-001-004 |
| WP-006 | Integration Testing | 1 | 11-13 | ⏳ Pending | WP-001-005 |
| WP-007 | Documentation | 1 | 10-12 | ⏳ Pending | WP-001-006 |
| WP-008 | Performance Optimization | 2 | 8-10 | ⏳ Pending | WP-001-006 |
| WP-009 | Release v0.1.0 | 1 | 5-7 | ⏳ Pending | WP-001-008 |

### bifrost-routing (eco-fork-003)

| ID | Title | Phase | Calls | Status | Depends |
|----|-------|-------|-------|--------|---------|
| WP-001 | Request Classifier | 1 | 11-13 | ⏳ Pending | - |
| WP-002 | Model Registry | 1 | 8-10 | ⏳ Pending | - |
| WP-003 | Token-Aware Selection | 1 | 12-14 | ⏳ Pending | WP-001,002 |
| WP-004 | SLA Enforcement | 1 | 9-11 | ⏳ Pending | WP-003 |
| WP-005 | Cost Tracking | 1 | 10-12 | ⏳ Pending | WP-002,003,004 |
| WP-006 | A/B Testing & Shadow | 2 | 10-12 | ⏳ Pending | WP-003,005 |
| WP-007 | Integration Testing | 1 | 11-13 | ⏳ Pending | WP-001-006 |
| WP-008 | Documentation & Release | 2 | 12-15 | ⏳ Pending | WP-001-007 |

---

## Execution Timeline

```
Week 1-2 (Phase 1A - Foundational):
  ├─ forgecode-fork: WP-001 (Provider trait)
  ├─ phenotype-router-monitor: WP-001 (Router core)
  └─ bifrost-routing: WP-001-002 (Classifier + registry)

Week 2-3 (Phase 1B - Core Implementation):
  ├─ forgecode-fork: WP-002-004 (Provider impls)
  ├─ phenotype-router-monitor: WP-002-004 (Health, rate limit, metrics)
  └─ bifrost-routing: WP-003-005 (Selection, SLA, cost)

Week 4-6 (Phase 2 - Advanced + Integration):
  ├─ forgecode-fork: WP-005-008 (Routing + feedback loop + release)
  ├─ phenotype-router-monitor: WP-005, 008, 009 (CLI + dashboard + release)
  └─ bifrost-routing: WP-006, 008 (A/B testing + release)
```

---

## Key Metrics

**By Numbers**:
- 25 work packages
- 34 functional requirements
- 160+ subtasks (T-NNN)
- 255-304 tool calls
- 8,600-9,800 LOC
- 75+ success metrics
- 75+ risk assessments

**By Category**:
- Rust Crates: 15+ (across 3 forks)
- CLI Commands: 20+
- HTTP Endpoints: 15+
- Database Tables: 12+
- Tests: 200+

---

## Execution Checklist

**Before Starting Phase 1A**:
- [ ] Read comprehensive summary document
- [ ] Review all foundational WPs (WP-001s)
- [ ] Verify directory structure
- [ ] Confirm config files exist
- [ ] Load specs into AgilePlus dashboard

**Phase 1A Startup**:
- [ ] Activate 12-15 agents per fork
- [ ] Assign WP-001 tasks
- [ ] Set up progress tracking
- [ ] Establish daily standup

**Ongoing**:
- [ ] Track cross-fork dependencies
- [ ] Monitor effort estimates vs actuals
- [ ] Weekly rollup reports
- [ ] Unblock dependencies proactively

---

## Success Criteria (Each Fork)

| Gate | forgecode-fork | phenotype-router | bifrost-routing |
|------|---|---|---|
| **Build** | `cargo build --release` ✅ | `cargo build --release` ✅ | `cargo build --release` ✅ |
| **Test** | ≥85% coverage | ≥85% coverage | ≥85% coverage |
| **Lint** | 0 clippy warnings | 0 clippy warnings | 0 clippy warnings |
| **Docs** | All APIs documented | All APIs documented | All APIs documented |
| **Integration** | All FRs traced | All FRs traced | All FRs traced |

---

## Document Locations

### Summary & Reference
- **Comprehensive**: `/repos/docs/reports/AGILEPLUS_WP_CREATION_SUMMARY_2026-03-30.md` (558 lines)
- **Quick Reference**: `/repos/docs/reference/AGILEPLUS_WP_QUICK_REFERENCE_2026-03-30.md` (this file)

### Work Package Specs
- **forgecode-fork**: `/forgecode-fork/.agileplus/specs/WP-{001..008}_*.md`
- **phenotype-router-monitor**: `/phenotype-router-monitor/.agileplus/specs/WP-{001..009}_*.md`
- **bifrost-routing**: `/bifrost-routing/.agileplus/specs/WP-{001..008}_*.md`

### Configuration
- **forgecode-fork**: `/forgecode-fork/.agileplus/config.toml` (83 lines)
- **phenotype-router-monitor**: `/phenotype-router-monitor/.agileplus/config.toml` (52 lines)
- **bifrost-routing**: `/bifrost-routing/.agileplus/config.toml` (52 lines)

---

## Common Commands

```bash
# List all WPs for a fork
ls /Users/kooshapari/CodeProjects/Phenotype/repos/forgecode-fork/.agileplus/specs/

# Read a specific WP
cat /Users/kooshapari/CodeProjects/Phenotype/repos/forgecode-fork/.agileplus/specs/WP-001_Provider_Trait_Registry.md

# Load specs into AgilePlus
cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus
agileplus import-specs ../forgecode-fork/.agileplus/specs/

# View comprehensive summary
less /Users/kooshapari/CodeProjects/Phenotype/repos/docs/reports/AGILEPLUS_WP_CREATION_SUMMARY_2026-03-30.md
```

---

## Contact & Escalation

**Questions about specs?** → See comprehensive summary
**Questions about a WP?** → Read WP document directly
**Dependency blockers?** → Check dependency graph in summary
**Effort mismatches?** → Review risk assessment section

---

**Last Updated**: 2026-03-30
**Status**: ✅ READY FOR EXECUTION
**Next Action**: Activate Phase 1A agents
