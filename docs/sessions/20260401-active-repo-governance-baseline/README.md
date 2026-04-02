# Active Repo Governance Baseline

**Date:** 2026-04-01
**Scope:** resume active-repo governance hardening across AgilePlus, heliosCLI, heliosApp, and thegent

## Goal

Land a repo-native governance baseline that can be enforced both in git and in GitHub:

- no force pushes to protected branches
- PR-required merge flow
- at least one approval on protected branches
- resolved review threads before merge
- explicit stacked-PR policy
- checked-in ruleset manifests instead of undocumented GitHub-only state

## Deliverables

- checked-in ruleset manifests
- reusable ruleset apply script
- missing repo-native governance surfaces in active repos
- workflow fixes where governance and CI had obviously drifted
