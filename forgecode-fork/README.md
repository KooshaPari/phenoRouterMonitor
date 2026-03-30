# forgecode-fork

Custom Phenotype fork of forgecode with planned extensions for subagent integration and custom provider infrastructure.

## Purpose

This fork extends forgecode with:
- **Custom providers** for Phenotype-specific code generation and analysis
- **Subagent extensions** for distributed agent execution workflows
- **Rich UI components** for AgilePlus dashboard integration
- **xDD methodology compliance** (Test-Driven, Behavior-Driven, Domain-Driven Development)

## Repository Structure

```
forgecode-fork/
├── README.md                     # This file
├── .gitignore                    # Node/build exclusions
├── docs/
│   ├── FORK_INTENT.md           # Custom providers & subagent extensions
│   ├── ARCHITECTURE.md          # Planned modifications & integration points
│   └── FORK_INTENT.md           # Technical specification for fork customizations
├── .agileplus/                  # AgilePlus specifications (local or linked)
└── [cloned forgecode structure] # Upstream repository structure (after git clone)
```

## Status

- **Repository**: Prepared (not yet cloned)
- **AgilePlus specs**: Pending initialization
- **Custom providers**: Planned (Phase 2)
- **Subagent extensions**: Planned (Phase 3)

## Next Steps

1. **Initialize git clone**: `git clone https://github.com/KooshaPari/forgecode forgecode-fork`
2. **Link or initialize AgilePlus**: Configure spec location (kitty-specs/ or local)
3. **Document custom providers**: Update `docs/FORK_INTENT.md` with implementation plan
4. **Implement subagent framework**: Create extension points in `docs/ARCHITECTURE.md`

## Development

### Local Quality Checks

Pending forgecode clone completion. Expected tools:
- TypeScript/JavaScript linting
- Component testing
- Integration testing

### Documentation

- **FORK_INTENT.md**: Technical vision for custom extensions
- **ARCHITECTURE.md**: Integration points and modification strategy

## Contributing

All work must be tracked in AgilePlus. See `.agileplus/` for specifications.

## Related Projects

- **AgilePlus**: `/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus` (dashboard integration target)
- **phenotype-infrakit**: Core Rust infrastructure crates
- **thegent**: Dotfiles and governance base
