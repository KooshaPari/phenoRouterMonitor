# Dependency Migration Guide

Step-by-step migration instructions for standardizing dependencies across Phenotype projects.

## Overview

This guide provides a structured approach to migrating projects from current dependencies to standardized ones defined in DEPENDENCIES.md.

## Migration Priority

1. **P0 (Critical)**: Security vulnerabilities, EOL dependencies
2. **P1 (High)**: Major version differences, significant maintenance burden
3. **P2 (Medium)**: Standardization benefits, DX improvements
4. **P3 (Low)**: Nice-to-have consistency

## Migration Template

For each project, create an entry following this structure:

```markdown
### [Project Name]

**Path**: `relative/path/to/project`
**Language**: [Rust|Python|TypeScript|Go]
**Priority**: [P0|P1|P2|P3]
**Effort**: [Small|Medium|Large]
**Risk**: [Low|Medium|High]

#### Current Dependencies

| Function | Current | Status |
|----------|---------|--------|
| Function name | `crate-name` | Redundant/Non-standard |

#### Recommended Changes

| Priority | From | To | Rationale |
|----------|------|-----|-------------|
| P1 | `old-crate` | `new-crate` | Reason for change |

#### Migration Steps

1. Step description
2. Step description
3. Step description

#### Compatibility Notes

- Breaking changes to document
- Migration shims needed
- Deprecation timeline
```

---

## Active Migration Projects

### [Template] Example Project

**Path**: `crates/example-project`
**Language**: Rust
**Priority**: P2
**Effort**: Small
**Risk**: Low

#### Current Dependencies

| Function | Current | Status |
|----------|---------|--------|
| Web Framework | `actix-web` | Non-standard |
| Database | `diesel` | Non-standard |
| CLI | `structopt` | Deprecated |

#### Recommended Changes

| Priority | From | To | Rationale |
|----------|------|-----|-------------|
| P1 | `structopt` | `clap` v4 | structopt is deprecated, merged into clap |
| P2 | `actix-web` | `axum` | Standardize on axum across ecosystem |
| P2 | `diesel` | `sqlx` | Async-first, compile-time checks |

#### Migration Steps

1. Update `Cargo.toml` dependencies
2. Replace `#[derive(StructOpt)]` with `#[derive(Parser)]`
3. Migrate actix-web extractors to axum extractors
4. Convert Diesel migrations to SQLx migrations
5. Run `cargo test` to verify functionality
6. Update documentation

#### Compatibility Notes

- StructOpt migration is drop-in replacement
- Actix-web to Axum requires handler signature changes
- Diesel to SQLx requires query rewrites

---

## Python Migrations

### Python: Example Service

**Path**: `python/example-service`
**Language**: Python
**Priority**: P1
**Effort**: Medium
**Risk**: Medium

#### Current Dependencies

| Function | Current | Status |
|----------|---------|--------|
| Web Framework | `Flask` | Non-standard |
| Validation | `marshmallow` | Non-standard |
| HTTP Client | `requests` | Synchronous only |
| Testing | `unittest` | Non-standard |

#### Recommended Changes

| Priority | From | To | Rationale |
|----------|------|-----|-------------|
| P1 | `Flask` | `FastAPI` | Performance, async, modern |
| P1 | `marshmallow` | `Pydantic` v2 | Type-safe, faster, standard |
| P1 | `requests` | `httpx` | Async support, compatible API |
| P2 | `unittest` | `pytest` | Richer testing, fixtures |

#### Migration Steps

1. Install FastAPI and dependencies:
   ```bash
   uv add fastapi uvicorn[standard]
   ```

2. Replace Flask app with FastAPI:
   ```python
   # Before
   from flask import Flask
   app = Flask(__name__)
   
   # After
   from fastapi import FastAPI
   app = FastAPI()
   ```

3. Convert marshmallow schemas to Pydantic models:
   ```python
   # Before
   from marshmallow import Schema, fields
   class UserSchema(Schema):
       name = fields.Str()
   
   # After
   from pydantic import BaseModel
   class User(BaseModel):
       name: str
   ```

4. Replace requests with httpx:
   ```python
   # Before
   import requests
   response = requests.get(url)
   
   # After
   import httpx
   async with httpx.AsyncClient() as client:
       response = await client.get(url)
   ```

5. Convert unittest to pytest:
   ```python
   # Before
   import unittest
   class TestUser(unittest.TestCase):
       def test_create(self): ...
   
   # After
   def test_create(): ...
   ```

6. Run test suite to verify

#### Compatibility Notes

- Flask blueprints map to FastAPI routers
- Marshmallow validators can be converted to Pydantic validators
- httpx is nearly API-compatible with requests

---

## TypeScript Migrations

### TypeScript: Example API

**Path**: `apps/api`
**Language**: TypeScript
**Priority**: P2
**Effort**: Medium
**Risk**: Low

#### Current Dependencies

| Function | Current | Status |
|----------|---------|--------|
| Web Framework | `Express` | Alternative standard |
| Validation | `Joi` | Non-standard |
| Testing | `Jest` | Standard but slower |
| Build | `webpack` | Complex configuration |

#### Recommended Changes

| Priority | From | To | Rationale |
|----------|------|-----|-------------|
| P2 | `Express` | `Fastify` | Performance, plugins |
| P2 | `Joi` | `Zod` | Type-safe, better DX |
| P3 | `Jest` | `Vitest` | Faster, native TS |
| P2 | `webpack` | `esbuild` + `tsc` | Faster builds |

#### Migration Steps

1. Install Fastify:
   ```bash
   npm install fastify @fastify/cors
   ```

2. Convert Express middleware to Fastify plugins
3. Replace Joi schemas with Zod:
   ```typescript
   // Before
   import Joi from 'joi';
   const schema = Joi.object({ name: Joi.string() });
   
   // After
   import { z } from 'zod';
   const schema = z.object({ name: z.string() });
   ```

4. Update test configuration
5. Update build scripts

#### Compatibility Notes

- Fastify plugin ecosystem differs from Express middleware
- Zod can infer TypeScript types from schemas
- Vitest is Jest-compatible for most features

---

## Go Migrations

### Go: Example Service

**Path**: `services/example`
**Language**: Go
**Priority**: P2
**Effort**: Small
**Risk**: Low

#### Current Dependencies

| Function | Current | Status |
|----------|---------|--------|
| Web Framework | `fiber` | Non-standard |
| Database | raw `database/sql` | Verbose |
| CLI | `urfave/cli` | Non-standard |

#### Recommended Changes

| Priority | From | To | Rationale |
|----------|------|-----|-------------|
| P2 | `fiber` | `gin` or `echo` | Standardize ecosystem |
| P2 | raw SQL | `sqlx` | Cleaner scanning |
| P1 | `urfave/cli` | `cobra` | Industry standard |

#### Migration Steps

1. Replace fiber with gin:
   ```go
   // Before
   import "github.com/gofiber/fiber/v2"
   app := fiber.New()
   
   // After
   import "github.com/gin-gonic/gin"
   r := gin.Default()
   ```

2. Add sqlx for cleaner database code
3. Convert CLI to cobra structure

#### Compatibility Notes

- Gin and Fiber have similar routing
- sqlx extends standard library
- Cobra requires command structure changes

---

## Bulk Migration Commands

### Detect Non-Standard Dependencies

```bash
# Rust
find . -name "Cargo.toml" -exec grep -l "actix-web\|rocket\|warp" {} \;

# Python
find . -name "pyproject.toml" -exec grep -l "Flask\|Django\|marshmallow" {} \;

# TypeScript
find . -name "package.json" -exec grep -l "express\|joi\|mocha" {} \;

# Go
find . -name "go.mod" -exec grep -l "fiber\|beego\|urfave" {} \;
```

### Generate Migration Report

```bash
# Run dependency audit
cargo audit 2>/dev/null || true
pip-audit 2>/dev/null || true
npm audit 2>/dev/null || true
go list -u -m all 2>/dev/null || true
```

---

## Rollback Procedures

If a migration causes issues:

1. **Immediate**: Revert to previous git commit
2. **Short-term**: Feature flag new dependencies
3. **Long-term**: Plan phased re-migration with more testing

## Migration Verification Checklist

- [ ] All tests pass
- [ ] No security vulnerabilities (`cargo audit`, `pip-audit`, `npm audit`)
- [ ] Performance metrics meet or exceed baseline
- [ ] Documentation updated
- [ ] Team notified of changes
- [ ] Monitoring alerts configured

