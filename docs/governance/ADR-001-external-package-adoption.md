# ADR-001: External Package Adoption to Replace Hand-Rolled Utilities

- **Date**: 2026-03-29
- **Status**: Accepted
- **Deciders**: Phenotype Platform Team

---

## Context

A cross-ecosystem audit of the Phenotype monorepo identified substantial volumes of hand-rolled
utility code duplicated across multiple services. The primary hotspots are:

- `tooling/trace/backend/internal/middleware/` — 8 hand-rolled middleware files covering rate
  limiting, CORS, JWT auth, and request logging
- `tooling/bifrost-extensions/internal/middleware/` — parallel middleware stack with overlapping
  patterns
- Various Go services using Viper for configuration, each with boilerplate wrappers
- Multiple Go services with custom HTTP retry and circuit-breaker logic
- Multiple Go services with hand-written mock structs not generated from interfaces
- TypeScript frontends with hand-rolled fetch clients and no type-safe OpenAPI contract enforcement

Estimated hand-rolled LOC subject to replacement: **2,050–4,600 lines** across the ecosystem.

Maintaining divergent implementations of the same cross-cutting concerns raises defect risk,
slows onboarding, and prevents unified policy enforcement (e.g., consistent rate-limit headers,
identical JWT validation logic across services).

---

## Decision

The following external packages are adopted as the Phenotype ecosystem standard for their
respective concerns. Hand-rolled equivalents MUST be migrated and removed on a per-service
basis as services are touched.

### Go Packages

#### 1. Configuration — `knadh/koanf` v2 replaces Viper

| Attribute | Viper | koanf v2 |
|-----------|-------|----------|
| Binary size contribution | Baseline | ~313% smaller |
| Key case sensitivity | Case-insensitive (footgun) | Case-sensitive by default |
| Composability | Monolithic | Provider/parser/merger pipeline |
| Active maintenance | Stalled | Active |

**Rationale**: Viper's case-insensitive key handling has caused silent misconfiguration bugs
in Phenotype services. `koanf` exposes an explicit provider/parser composition model that
maps cleanly to the layered config strategy (defaults -> file -> env -> flags). The binary
size reduction is meaningful for containerised services.

**Import path**: `github.com/knadh/koanf/v2`

---

#### 2. HTTP Middleware — `go-chi` ecosystem replaces hand-rolled middleware

Three packages from the `go-chi` ecosystem replace the 8 hand-rolled middleware files:

| Package | Replaces |
|---------|----------|
| `github.com/go-chi/httprate` | Custom rate-limiter in `middleware/rate_limit.go` |
| `github.com/go-chi/cors` | Custom CORS handler in `middleware/cors.go` |
| `github.com/go-chi/jwtauth/v5` | Custom JWT extractor/validator in `middleware/auth.go` |

**Rationale**: The hand-rolled middleware implementations lack test coverage and diverge in
header naming and error response shapes between `trace` and `bifrost-extensions`. The `go-chi`
ecosystem packages are battle-tested, Chi-router-compatible, and interoperate cleanly with
the standard `net/http` handler chain used across Phenotype services. Adopting them enforces
uniform policy at the package boundary level.

Shared middleware will be extracted into a new shared package: **`phenotype-go-middleware`**.

---

#### 3. HTTP Client Resilience — `hashicorp/go-retryablehttp` + `sony/gobreaker` v2

| Package | Concern |
|---------|---------|
| `github.com/hashicorp/go-retryablehttp` | Automatic retry with exponential backoff, jitter, and hook points |
| `github.com/sony/gobreaker/v2` | Circuit breaker with configurable thresholds and state callbacks |

**Rationale**: Several Phenotype services make outbound HTTP calls to external APIs (LLM
providers, GitHub, CI systems) with ad-hoc retry loops that lack jitter, do not respect
`Retry-After` headers, and have no circuit-breaker protection. `go-retryablehttp` provides
a drop-in `*http.Client`-compatible interface with correct exponential-backoff-with-jitter
and response inspection hooks. `gobreaker` v2 adds the half-open/open/closed state machine
required to prevent thundering-herd reconnect storms.

---

#### 4. Mock Generation — `vektra/mockery` v3 replaces hand-written mocks

**Rationale**: Existing hand-written mocks in several packages are out of sync with their
source interfaces, causing test failures that are only discovered at runtime. `mockery` v3
generates type-safe mocks directly from Go interfaces and can be wired into `go generate`,
ensuring mocks stay current as interfaces evolve. The `--with-expecter` flag produces
ergonomic `.EXPECT()` chains compatible with `testify/mock`.

**Config file**: `.mockery.yaml` at repo root specifying `with-expecter: true`,
`mock-build-tags`, and per-package output directories.

---

### TypeScript Packages

#### 5. API Client Generation — `orval` replaces hand-maintained fetch wrappers

**Rationale**: Frontend packages currently maintain hand-written typed fetch wrappers against
backend OpenAPI specs. These wrappers drift from the actual schema and require manual updates
on every API change. `orval` reads the OpenAPI spec at build time and generates fully-typed
TanStack Query hooks (queries and mutations) with zero boilerplate. This eliminates an entire
category of schema-drift bugs and removes ~400–800 LOC of wrapper code per frontend.

**Config file**: `orval.config.ts` at each frontend package root.

---

#### 6. HTTP Client — `openapi-fetch` replaces hand-rolled fetch

**Package**: `openapi-fetch` from the `openapi-typescript` project family.

**Rationale**: Where direct fetch is required outside of React Query (e.g., server-side
scripts, CLI tools, non-React contexts), `openapi-fetch` provides a fetch wrapper that is
parameterised on the generated `paths` type from `openapi-typescript`. Every request and
response is statically typed against the OpenAPI schema — path parameters, query parameters,
request bodies, and response shapes are all checked at compile time. This replaces the
current pattern of `as unknown as ExpectedType` casts in several utilities.

---

#### 7. Error Boundaries — `react-error-boundary` v4

**Rationale**: Several dashboard components lack React error boundaries, causing entire
subtrees to unmount on unhandled promise rejections or render errors. `react-error-boundary`
v4 provides `ErrorBoundary`, `useErrorBoundary`, and `withErrorBoundary` — all with full
TypeScript generics and a `resetKeys` prop for programmatic recovery. Wrapping data-fetching
subtrees with `react-error-boundary` + `Suspense` establishes the standard loading/error UI
pattern across Phenotype dashboards.

---

## Consequences

### Positive

- **LOC reduction**: Estimated elimination of 2,050–4,600 lines of duplicated hand-rolled
  code across the Phenotype ecosystem.
- **Defect reduction**: Removes divergent implementations of security-sensitive concerns
  (JWT validation, CORS headers, rate limiting). A single well-tested package version is
  easier to audit and patch.
- **Schema safety**: OpenAPI contract is enforced at compile time for TypeScript consumers,
  eliminating runtime schema-drift bugs.
- **Onboarding speed**: New contributors encounter standard OSS packages with public
  documentation rather than undocumented internal utilities.
- **Shared package creation**: `phenotype-go-middleware` and `@phenotype/api-client` emerge
  as natural extraction points, enabling future services to adopt the same patterns at
  near-zero cost.
- **Mock correctness**: Generated mocks are always in sync with interfaces; stale-mock
  failures are eliminated from CI.

### Negative

- **Migration effort**: Each service requires a targeted migration. Services that currently
  import hand-rolled middleware or config must be updated. Estimated: 3–5 parallel subagent
  batches across services.
- **Learning curve**: Team members unfamiliar with `koanf`'s provider pipeline or
  `mockery`'s `.EXPECT()` chains will need brief orientation.
- **Transitive dependency additions**: Each adopted package introduces its own dependency
  graph. Dependency audits (`govulncheck`, `npm audit`) must be run after each adoption.
- **orval codegen in CI**: Orval requires the OpenAPI spec to be available at build time.
  Services that generate their spec dynamically will need a static spec export step added
  to their build pipeline.

---

## Alternatives Considered

### Go Config: `spf13/viper` (retain)

Rejected. Viper's case-insensitive key handling is a latent footgun that has caused
production misconfiguration. The project is in maintenance mode with unresolved issues
regarding concurrent map access. `koanf` v2 is architecturally superior and has no
regression risk for new code.

### Go Config: `BurntSushi/toml` + custom loader

Rejected. Hand-rolling a layered config loader (defaults -> file -> env -> flags) duplicates
exactly the problem this ADR aims to solve. `koanf` provides this pipeline out of the box.

### Go HTTP Middleware: retain hand-rolled, extract to shared package

Rejected. Extracting the existing divergent implementations into a shared package merely
consolidates the debt without addressing correctness or coverage gaps. Replacing with
`go-chi` ecosystem packages achieves the same consolidation goal with better-tested
primitives.

### Go HTTP Client Resilience: `failsafe-go`

Considered. `failsafe-go` provides a more general policy-composition model. Rejected in
favour of `go-retryablehttp` + `gobreaker` because the two-library combination maps
directly to the retry and circuit-breaker concerns already identified, has broader ecosystem
adoption in the Go community, and requires no new abstraction layer.

### TypeScript API Client: `swagger-codegen` / `openapi-generator`

Rejected. Both tools generate heavyweight client classes that conflict with the TanStack
Query hook pattern used across Phenotype frontends. `orval` generates idiomatic React Query
hooks directly and is actively maintained against the TanStack Query v5 API surface.

### TypeScript HTTP Client: `axios`

Rejected. `axios` does not provide compile-time schema validation. `openapi-fetch` enforces
the OpenAPI contract at the TypeScript type level, which is the primary requirement for
replacing hand-rolled fetch wrappers.

---

## Implementation Notes

Migration is forward-only. New services MUST adopt these packages from day one. Existing
services SHOULD migrate when the service is next substantially modified. The following
shared packages will be created as part of the initial migration wave:

- **`phenotype-go-middleware`** — Go module wrapping `go-chi/httprate`, `go-chi/cors`, and
  `go-chi/jwtauth/v5` with Phenotype-specific defaults (response shape, logging hooks).
- **`@phenotype/api-client`** — TypeScript package exporting `orval`-generated TanStack
  Query hooks for each Phenotype backend service, with `openapi-fetch` for non-React
  consumers.

Primary migration targets in priority order:

1. `tooling/trace/backend/internal/middleware/` (8 files, highest duplication density)
2. `tooling/bifrost-extensions/internal/middleware/`
3. All Go services currently importing `spf13/viper` (config migration)
4. All Go services with custom retry loops (HTTP client resilience)

---

## References

- `knadh/koanf`: https://github.com/knadh/koanf
- `go-chi/httprate`: https://github.com/go-chi/httprate
- `go-chi/cors`: https://github.com/go-chi/cors
- `go-chi/jwtauth`: https://github.com/go-chi/jwtauth
- `hashicorp/go-retryablehttp`: https://github.com/hashicorp/go-retryablehttp
- `sony/gobreaker`: https://github.com/sony/gobreaker
- `vektra/mockery`: https://github.com/vektra/mockery
- `orval`: https://orval.dev
- `openapi-fetch`: https://openapi-ts.dev/openapi-fetch/
- `react-error-boundary`: https://github.com/bvaughn/react-error-boundary
