# Implementation Strategy

## Principles

- keep GitHub settings and repo-tracked policy aligned, but do not pretend the settings are
  verified when the detail endpoint is unavailable
- bootstrap weak repos with small, reusable governance surfaces instead of inventing large new CI
  systems
- require server-side policy gates for branch naming, stack shape, and merge-commit rejection
- keep helper automations out of the required-check set unless they directly block merge decisions

## Repo-Level Strategy

### AgilePlus

- use existing `pr-governance-gate`, PR template, and ruleset docs as the strongest reference
- keep required-check discussions centered on real branch jobs, not helper workflows

### heliosApp

- add a minimal but strict PR policy workflow
- add repo-tracked ruleset and review contract files
- make secret scanning a real failure signal

### forgecode

- add the same governance contract shape as heliosApp
- add a basic docs CI workflow because the repo currently lacks a primary CI surface
- make secret scanning a real failure signal

### phenotype-infrakit

- preserve the bootstrap files already staged in this lane as the repo-local contract
- treat live GitHub ruleset enablement as a follow-up after policy names stabilize
