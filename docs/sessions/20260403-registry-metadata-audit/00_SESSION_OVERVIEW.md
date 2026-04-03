# Registry Metadata Audit (phenotype-registry + template indexes)

Date: 2026-04-03

## Goal

Audit metadata consistency across:

- `phenotype-registry/registry.json`
- `phenotype-registry/projects/*.json` (including `phenoTemplates.json`, `hexagon.json`)
- Template index files (notably `phenoTemplates/registry/index.json`)

Focus: identify stale language/framework metadata after the consolidated `templates/<language>/...` structure.

## Success Criteria

- Clear list of concrete metadata drift cases (language + framework)
- Evidence (file paths) for why metadata is stale
- Expected updates (what needs to change in detection / indexes)

