---
audience: [developers]
---

# Contributing

How to set up your development environment and contribute to AgilePlus.

## Prerequisites

- Rust 1.75+ (`rustup default stable`)
- Bun (for docs)
- Git
- A GitHub account

## Setup

```bash
git clone https://github.com/KooshaPari/AgilePlus.git
cd AgilePlus
cargo build
```

For docs development:

```bash
bun install
bun run dev
```

## Branch Workflow

1. Create a feature branch from `main`
2. Make your changes
3. Run tests: `cargo test`
4. Push and open a PR

## Commit Convention

```
type(scope): description

feat(dispatch): add retry logic for agent sessions
fix(sync): handle empty response from Plane API
docs(guide): add sync configuration section
```

Types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`

## Code Style

- Follow `rustfmt` defaults
- Run `cargo clippy` before committing
- No `unwrap()` in library code — use `Result` or `?`
- Keep functions under 50 lines where practical

## PR Requirements

- Tests pass (`cargo test`)
- No clippy warnings (`cargo clippy -- -D warnings`)
- Formatted (`cargo fmt --check`)
- PR description explains **why**, not just **what**

## Getting Help

- Open a [GitHub Issue](https://github.com/KooshaPari/AgilePlus/issues) for bugs or feature requests
- Check existing specs in `kitty-specs/` for planned work
