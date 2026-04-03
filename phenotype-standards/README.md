# Phenotype Standards

Dependency standardization and migration documentation for the Phenotype ecosystem.

## Documents

| Document | Purpose |
|----------|---------|
| [DEPENDENCIES.md](./DEPENDENCIES.md) | Approved dependencies by language and function |
| [MIGRATION.md](./MIGRATION.md) | Migration guide for standardizing dependencies |
| [EXCEPTIONS.md](./EXCEPTIONS.md) | Process for requesting standard exceptions |

## Quick Reference

### Rust
- **Web**: axum
- **Database**: sqlx
- **CLI**: clap
- **HTTP**: reqwest
- **Async**: tokio

### Python
- **Web**: FastAPI
- **Validation**: Pydantic v2
- **CLI**: typer
- **HTTP**: httpx
- **Testing**: pytest

### TypeScript
- **Web**: Fastify
- **Validation**: Zod
- **CLI**: commander
- **HTTP**: axios
- **Testing**: Vitest

### Go
- **Web**: gin
- **Database**: sqlx/gorm
- **CLI**: cobra + viper
- **HTTP**: net/http
- **Testing**: testing + testify

## Usage

1. **New Projects**: Start with standards in DEPENDENCIES.md
2. **Existing Projects**: Follow MIGRATION.md for standardization
3. **Exceptions**: Use EXCEPTIONS.md process if standards don't fit

