# thegent: Dotfiles Manager Consolidation

## Problem
thegent was designed as the dotfiles/bootstrap manager for all systems but agents scattered governance, templates, and hooks across dozens of repos. Items in ~/.claude/, CodeProjects/orphans/, and thegent itself need consolidation. Goal: run thegent setup to configure any macOS/Linux/Windows system.

## Acceptance Criteria
- [ ] thegent/templates/ contains ALL project templates (currently in template-* repos)
- [ ] thegent/hooks/ contains ALL Claude hooks (currently in ~/.claude/hooks/)
- [ ] thegent/dotfiles/ manages shell config, brew packages, dev tools
- [ ] thegent setup <profile> command bootstraps a system
- [ ] thegent/crates/ Rust libs extracted to published crates (thegent-cache, etc.)
- [ ] README: single-command system setup documented

## Consolidation sources
- ~/.claude/ hooks → thegent/hooks/
- template-commons, template-lang-* repos → thegent/templates/
- ~/.claude/CLAUDE.md governance → thegent/governance/
- Orphan project configs → thegent/dotfiles/
