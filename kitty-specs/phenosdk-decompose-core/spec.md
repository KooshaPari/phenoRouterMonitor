# phenoSDK: Extract pheno-core Package

## Problem
phenoSDK mixes 10+ domains in one monolith. The foundation (config 100%, errors 100%, logging 100%, observability ports 100%) should be extracted as pheno-core — the minimal dependency that all other pheno-* packages depend on.

## Acceptance Criteria
- [ ] New package: pheno-core with modules: config, errors, logging, observability
- [ ] All existing phenoSDK consumers updated to import from pheno-core
- [ ] pheno-core has its own pyproject.toml, tests, and CI entry
- [ ] phenoSDK depends on pheno-core (not vice versa)
- [ ] Existing functionality unchanged (backward compat or migration guide)
- [ ] Published to Phenotype GitHub Packages

## Package Contents
- pheno.config.core (Config.from_env, from_file, load)
- pheno.errors (ZenMCPError hierarchy + retry/circuit breaker)
- pheno.logging (Console, File, JSON, Syslog, Structlog)
- pheno.ports.observability (Logger, Tracer, Meter, HealthChecker, Alerter)
- pheno.ports.registry (Registry, SearchableRegistry, ObservableRegistry)
