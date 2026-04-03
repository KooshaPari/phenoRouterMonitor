# PhenoSDK npm Publishing Implementation Checklist

**Project**: @phenotype/pheno-* package extraction and publishing
**Date**: 2026-03-29
**Phases**: 4 (Setup, Core, Resilience, LLM, Publishing)
**Estimated Duration**: 80 minutes (agent-driven)

---

## Pre-Implementation

### [ ] Prerequisites & Planning

- [ ] Review PHENOSDK_EXTRACTION_STRATEGY.md document
- [ ] Review PHENOSDK_PACKAGE_AUDIT.md (interface specifications)
- [ ] Confirm GitHub Packages access (test token)
- [ ] Confirm pnpm workspace configuration
- [ ] Create feature branch: `feat/phenosdk-npm-publishing`
- [ ] Create AgilePlus spec and work packages

**Owner**: [TBD]
**Status**: PENDING

---

## Phase 0: Setup (5 minutes)

### [ ] Configure npm Publishing Infrastructure

**WP0.1: Root-level npm configuration**

- [ ] Update `.npmrc` with @phenotype scope
  ```ini
  @phenotype:registry=https://npm.pkg.github.com
  //npm.pkg.github.com/:_authToken=${GITHUB_TOKEN}
  always-auth=true
  ```
- [ ] Update `pnpm-workspace.yaml` to include `packages/*`
- [ ] Verify workspace is recognized: `pnpm list -r --depth=0`

**WP0.2: Directory structure**

- [ ] Create `packages/` directory at repo root
- [ ] Create subdirectories:
  - [ ] `packages/pheno-core/`
  - [ ] `packages/pheno-resilience/`
  - [ ] `packages/pheno-llm/`
- [ ] Verify directory structure with `ls -la packages/`

**WP0.3: GitHub Packages workflow**

- [ ] Create `.github/workflows/publish.yml`
- [ ] Test workflow validation: `act -j publish --dry-run`
- [ ] Commit workflow file

**Checklist Owner**: [TBD]
**Status**: PENDING

---

## Phase 1: @phenotype/pheno-core (20 minutes)

### [ ] Port Hexagonal Architecture Contracts

**WP1.1: Core package structure**

- [ ] Create `packages/pheno-core/package.json`
  - [ ] Verify name: `@phenotype/pheno-core`
  - [ ] Verify exports: ports, models, errors
  - [ ] Verify publishConfig correct
- [ ] Create `packages/pheno-core/tsconfig.json`
  - [ ] Set strict mode: `"strict": true`
  - [ ] Enable declaration: `"declaration": true`
- [ ] Create `packages/pheno-core/vitest.config.ts`
- [ ] Create directory structure:
  - [ ] `src/ports/inbound.ts`
  - [ ] `src/ports/outbound.ts`
  - [ ] `src/models/index.ts`
  - [ ] `src/errors/index.ts`
  - [ ] `src/index.ts` (re-exports all)

**WP1.2: Implement inbound ports**

```typescript
// src/ports/inbound.ts
export interface UseCase<Input, Output> {
  execute(input: Input): Promise<Output>;
}

export interface CommandHandler<Command> {
  handle(command: Command): Promise<void>;
}

export interface QueryHandler<Query, Result> {
  handle(query: Query): Promise<Result>;
}

export interface EventHandler<Event> {
  handle(event: Event): Promise<void>;
}
```

- [ ] Implement UseCase interface
- [ ] Implement CommandHandler interface
- [ ] Implement QueryHandler interface
- [ ] Implement EventHandler interface
- [ ] Add JSDoc comments to all exports
- [ ] Verify TypeScript compilation: `pnpm --filter @phenotype/pheno-core build`

**WP1.3: Implement outbound ports**

```typescript
// src/ports/outbound.ts
export interface Repository<T extends Entity> {
  save(entity: T): Promise<void>;
  findById(id: string): Promise<T | null>;
  delete(id: string): Promise<void>;
}
```

- [ ] Implement Repository interface
- [ ] Implement CachePort interface
- [ ] Implement SecretStore interface
- [ ] Implement EventBus interface
- [ ] Add JSDoc with usage examples
- [ ] Verify TypeScript compilation

**WP1.4: Implement domain models**

```typescript
// src/models/index.ts
export abstract class Entity {
  abstract get id(): string;
}

export abstract class AggregateRoot extends Entity {
  protected uncommittedEvents: DomainEvent[] = [];
}
```

- [ ] Implement Entity abstract class
- [ ] Implement ValueObject abstract class
- [ ] Implement AggregateRoot abstract class
- [ ] Implement DomainEvent interface
- [ ] Add event tracking: raiseEvent(), getUncommittedEvents(), clearUncommittedEvents()

**WP1.5: Implement error types**

```typescript
// src/errors/index.ts
export enum ErrorKind {
  NotFound = 'NotFound',
  Validation = 'Validation',
  // ... etc
}

export class PhenotypeError extends Error {
  constructor(
    public readonly kind: ErrorKind,
    message: string,
    public readonly context?: Record<string, unknown>,
  ) {
    super(message);
  }
}
```

- [ ] Implement ErrorKind enum (all 11 variants)
- [ ] Implement PhenotypeError class
- [ ] Add error predicates: isNotFound(), isValidation(), etc.

**WP1.6: Index and re-exports**

- [ ] Create `src/index.ts` re-exporting all modules:
  ```typescript
  export * from './ports/inbound';
  export * from './ports/outbound';
  export * from './models/index';
  export * from './errors/index';
  ```
- [ ] Verify exports with: `pnpm --filter @phenotype/pheno-core run build`

**WP1.7: Unit tests**

- [ ] Create `tests/ports.test.ts`
  - [ ] Test UseCase contract
  - [ ] Test CommandHandler contract
  - [ ] Test Repository contract
  - [ ] Test CachePort contract
- [ ] Create `tests/models.test.ts`
  - [ ] Test Entity behavior
  - [ ] Test AggregateRoot event tracking
- [ ] Create `tests/errors.test.ts`
  - [ ] Test error creation
  - [ ] Test error predicates
- [ ] Run tests: `pnpm --filter @phenotype/pheno-core run test`
- [ ] Verify coverage ≥80%: `pnpm --filter @phenotype/pheno-core run test:coverage`

**WP1.8: Documentation**

- [ ] Create `packages/pheno-core/README.md`
  - [ ] Overview section
  - [ ] Quick start example
  - [ ] Ports documentation with examples
  - [ ] Models documentation
  - [ ] Error handling guide
- [ ] Create JSDoc for all public exports
- [ ] Add usage examples in code comments

**WP1.9: Pre-publish validation**

- [ ] Lint check: `pnpm --filter @phenotype/pheno-core run lint`
- [ ] Type check: `pnpm --filter @phenotype/pheno-core build --noEmit`
- [ ] Test all pass: `pnpm --filter @phenotype/pheno-core run test`
- [ ] Dist folder created: `ls -la packages/pheno-core/dist/`
- [ ] package.json correct: `cat packages/pheno-core/package.json | grep -A5 publishConfig`

**Checklist Owner**: [TBD]
**Status**: PENDING
**Completion Deadline**: +20 minutes from start

---

## Phase 2: @phenotype/pheno-resilience (30 minutes)

### [ ] Port Event Sourcing, State Machines, Policies, Caching

**WP2.1: Event sourcing module**

- [ ] Create `packages/pheno-resilience/src/event-sourcing/` directory
- [ ] Create `event-envelope.ts`
  - [ ] Implement EventEnvelope<T> interface
  - [ ] Include: id (UUID), timestamp, sequence, actor, eventType, payload, hash, prevHash
- [ ] Create `event-store.ts`
  - [ ] Implement EventStore interface with 6 methods
  - [ ] Document each method: append, getEvents, getEventsSince, getEventsByRange, getLatestSequence, verifyChain
- [ ] Create `hash-chain.ts`
  - [ ] Implement computeHash() using Node.js crypto
  - [ ] Implement verifyChain() function
  - [ ] Add JSDoc with algorithm explanation
- [ ] Create `in-memory-store.ts`
  - [ ] Implement InMemoryEventStore
  - [ ] Use Map<string, Map<string, StoredEvent[]>> for storage
  - [ ] Include clear() and eventCount() methods
- [ ] Create `index.ts` with all exports
- [ ] Verify: `pnpm --filter @phenotype/pheno-resilience build`

**Tests for event-sourcing**:
- [ ] Create `tests/event-sourcing.test.ts`
  - [ ] Test append and retrieve
  - [ ] Test sequence numbering
  - [ ] Test hash chain integrity
  - [ ] Test getEventsSince pagination
  - [ ] Test getEventsByRange time filtering
  - [ ] Test chain verification on broken link
  - [ ] Test InMemoryEventStore

**WP2.2: State machine module**

- [ ] Create `packages/pheno-resilience/src/state-machine/` directory
- [ ] Create `state.ts`
  - [ ] Implement State interface with name, onEnter, onExit
- [ ] Create `transition.ts`
  - [ ] Implement Transition interface with from, to, event, guard, action
- [ ] Create `machine.ts`
  - [ ] Implement StateMachine class
  - [ ] Methods: constructor, getCurrentState, sendEvent, canTransition, getAvailableTransitions, getHistory
  - [ ] Support guard condition evaluation
  - [ ] Execute action on transition
  - [ ] Track state history with timestamps
- [ ] Create `index.ts` with exports
- [ ] Verify compilation

**Tests for state-machine**:
- [ ] Create `tests/state-machine.test.ts`
  - [ ] Test state transitions
  - [ ] Test guard evaluation (permit/deny)
  - [ ] Test action execution
  - [ ] Test history tracking
  - [ ] Test invalid transition rejection
  - [ ] Test canTransition validation
  - [ ] Test multiple transitions available

**WP2.3: Policy engine module**

- [ ] Create `packages/pheno-resilience/src/policy-engine/` directory
- [ ] Create `policy.ts`
  - [ ] Implement PolicyRule interface
  - [ ] Implement PolicySet interface with combinator
  - [ ] Implement PolicyContext interface
  - [ ] Define decision combinator enum
- [ ] Create `engine.ts`
  - [ ] Implement PolicyEngine class
  - [ ] Methods: addPolicies, evaluate
  - [ ] Support first-applicable, deny-overrides, permit-overrides combinators
  - [ ] Regex pattern matching
  - [ ] Condition evaluation
- [ ] Create `index.ts` with exports
- [ ] Verify compilation

**Tests for policy-engine**:
- [ ] Create `tests/policy-engine.test.ts`
  - [ ] Test basic rule matching
  - [ ] Test deny-overrides combinator
  - [ ] Test permit-overrides combinator
  - [ ] Test first-applicable combinator
  - [ ] Test priority ordering
  - [ ] Test regex pattern evaluation
  - [ ] Test context conditions

**WP2.4: Cache module**

- [ ] Create `packages/pheno-resilience/src/cache/` directory
- [ ] Create `cache.ts` (interface)
  - [ ] Implement Cache<K, V> interface
  - [ ] Methods: get, set, delete, clear
- [ ] Create `lru-cache.ts`
  - [ ] Implement LRUCache using lru-cache package
  - [ ] Support TTL (optional)
  - [ ] Support eviction policies
- [ ] Create `distributed-cache.ts`
  - [ ] Implement DistributedCache interface (abstract)
  - [ ] Document Redis adapter pattern
- [ ] Create `two-tier-cache.ts`
  - [ ] Implement TwoTierCache combining L1 + L2
  - [ ] L1 (LRU): fast, local, hot data
  - [ ] L2 (Distributed): slower, shared, warm data
  - [ ] Cross-tier write consistency
- [ ] Create `index.ts` with exports
- [ ] Verify compilation

**Tests for cache**:
- [ ] Create `tests/cache.test.ts`
  - [ ] Test LRU basic operations
  - [ ] Test LRU eviction on max size
  - [ ] Test TTL expiration
  - [ ] Test TwoTierCache hit patterns
  - [ ] Test cache invalidation
  - [ ] Test concurrent access

**WP2.5: Integration tests**

- [ ] Create `tests/integration.test.ts`
  - [ ] Test event sourcing + state machine workflow
  - [ ] Test policy evaluation with cache
  - [ ] Test end-to-end scenario: event → state change → policy evaluation

**WP2.6: Package configuration**

- [ ] Create `packages/pheno-resilience/package.json`
  - [ ] Name: `@phenotype/pheno-resilience`
  - [ ] Peer dependency: `@phenotype/pheno-core@^1.0.0`
  - [ ] Dependencies: uuid, lru-cache
  - [ ] Optional dependencies: redis
  - [ ] Exports: ., ./event-sourcing, ./state-machine, ./policy-engine, ./cache
- [ ] Create `packages/pheno-resilience/tsconfig.json`
- [ ] Create `packages/pheno-resilience/vitest.config.ts`

**WP2.7: Documentation**

- [ ] Create `packages/pheno-resilience/README.md`
  - [ ] Event sourcing guide with examples
  - [ ] State machine patterns and recipes
  - [ ] Policy authoring guide
  - [ ] Cache strategy selection
- [ ] Add JSDoc comments to all public exports

**WP2.8: Pre-publish validation**

- [ ] All tests pass: `pnpm --filter @phenotype/pheno-resilience run test`
- [ ] Coverage ≥80%: `pnpm --filter @phenotype/pheno-resilience run test:coverage`
- [ ] Build succeeds: `pnpm --filter @phenotype/pheno-resilience run build`
- [ ] No type errors: `pnpm --filter @phenotype/pheno-resilience run build --noEmit`
- [ ] Dist folder created and populated

**Checklist Owner**: [TBD]
**Status**: PENDING
**Completion Deadline**: +30 minutes from start of WP2.1

---

## Phase 3: @phenotype/pheno-llm (15 minutes)

### [ ] Port LLM Integration & Prompt Management

**WP3.1: LLM provider interface**

- [ ] Create `packages/pheno-llm/src/providers/` directory
- [ ] Create `provider.ts`
  - [ ] Implement LLMProvider interface with complete, chat, embed, tokenize, countTokens
  - [ ] Implement LLMOptions interface
  - [ ] Implement LLMMessage interface with role types
  - [ ] Implement LLMResponse interface
  - [ ] Implement TokenUsage interface
  - [ ] Implement ToolDefinition and ToolCall interfaces
- [ ] Create `index.ts` with exports
- [ ] Verify compilation

**WP3.2: Prompt management**

- [ ] Create `packages/pheno-llm/src/prompts/` directory
- [ ] Create `prompt.ts`
  - [ ] Implement Prompt interface
  - [ ] Methods: template, variables, render(), countTokens(), validate()
- [ ] Create `prompt-manager.ts`
  - [ ] Implement PromptManager class
  - [ ] Methods: registerTemplate, getTemplate, compilePrompt, validatePrompt
  - [ ] Support variable substitution
- [ ] Create `prompt-builder.ts`
  - [ ] Implement PromptBuilder class (fluent API)
  - [ ] Methods: system(), user(), assistant(), tool(), build()
- [ ] Create `token-utils.ts`
  - [ ] Implement estimateTokens() function using js-tiktoken
  - [ ] Support multiple models (gpt-4, gpt-3.5-turbo, etc.)
- [ ] Create `index.ts` with exports
- [ ] Verify compilation

**WP3.3: OpenAI adapter (example)**

- [ ] Create `packages/pheno-llm/src/adapters/` directory
- [ ] Create `openai-adapter.ts`
  - [ ] Implement LLMProvider for OpenAI API
  - [ ] Methods: complete, chat, embed, tokenize, countTokens
  - [ ] Handle tool/function calling
  - [ ] Support streaming (optional)
  - [ ] Map OpenAI response → LLMResponse
- [ ] Create `index.ts` with exports
- [ ] Mark adapter as optional export (not in main index.ts)

**WP3.4: Package configuration**

- [ ] Create `packages/pheno-llm/package.json`
  - [ ] Name: `@phenotype/pheno-llm`
  - [ ] Peer dependency: `@phenotype/pheno-core@^1.0.0`
  - [ ] Dependencies: js-tiktoken
  - [ ] Optional dependencies: openai
  - [ ] Exports: ., ./providers, ./prompts, ./adapters
- [ ] Create `packages/pheno-llm/tsconfig.json`
- [ ] Create `packages/pheno-llm/vitest.config.ts`

**WP3.5: Tests**

- [ ] Create `tests/providers.test.ts`
  - [ ] Test LLMProvider interface contract
  - [ ] Test TokenUsage tracking
  - [ ] Test message role validation
  - [ ] Test tool call structure
- [ ] Create `tests/prompts.test.ts`
  - [ ] Test Prompt rendering with variables
  - [ ] Test PromptManager registration and retrieval
  - [ ] Test PromptBuilder fluent API
  - [ ] Test token counting
- [ ] Create `tests/openai-adapter.test.ts` (optional, mock API)
  - [ ] Test response mapping
  - [ ] Test error handling
  - [ ] Test token usage tracking

**WP3.6: Documentation**

- [ ] Create `packages/pheno-llm/README.md`
  - [ ] LLM provider interface overview
  - [ ] Prompt template syntax guide
  - [ ] Token counting strategy
  - [ ] Adapter implementation guide (using OpenAI as example)
  - [ ] Tool/function calling patterns
- [ ] Add JSDoc comments

**WP3.7: Pre-publish validation**

- [ ] All tests pass: `pnpm --filter @phenotype/pheno-llm run test`
- [ ] Build succeeds: `pnpm --filter @phenotype/pheno-llm run build`
- [ ] No type errors

**Checklist Owner**: [TBD]
**Status**: PENDING
**Completion Deadline**: +15 minutes from start of WP3.1

---

## Phase 4: Publishing & Distribution (10 minutes)

### [ ] Configure Publishing & Release Process

**WP4.1: Validate all packages**

- [ ] Root workspace builds: `pnpm run build`
- [ ] All tests pass: `pnpm run test`
- [ ] No linting errors: `pnpm run lint`
- [ ] Coverage ≥80% across all packages

**WP4.2: Version management**

- [ ] Verify all packages have matching version: `1.0.0`
- [ ] Update package.json version to `1.0.0` for all three packages
- [ ] Create version bump script (optional, for future releases)

**WP4.3: GitHub Actions workflow validation**

- [ ] Workflow file `.github/workflows/publish.yml` present
- [ ] Workflow has correct triggers: push tags matching `v*`
- [ ] Workflow runs build, test, publish in correct order
- [ ] Workflow uses GITHUB_TOKEN for authentication
- [ ] Dry run test (local): `act -j publish --dry-run`

**WP4.4: Create GitHub release**

- [ ] Tag each package:
  ```bash
  git tag -a @phenotype/pheno-core@1.0.0 -m "Release pheno-core 1.0.0"
  git tag -a @phenotype/pheno-resilience@1.0.0 -m "Release pheno-resilience 1.0.0"
  git tag -a @phenotype/pheno-llm@1.0.0 -m "Release pheno-llm 1.0.0"
  ```
- [ ] Push tags: `git push origin --tags`
- [ ] Verify workflow triggered in GitHub Actions
- [ ] Monitor workflow for success

**WP4.5: Verify package publication**

- [ ] Package available on GitHub Packages:
  ```bash
  npm view @phenotype/pheno-core --registry https://npm.pkg.github.com
  ```
- [ ] Package installable:
  ```bash
  npm install @phenotype/pheno-core --registry https://npm.pkg.github.com
  ```
- [ ] Check all three packages published:
  - [ ] @phenotype/pheno-core
  - [ ] @phenotype/pheno-resilience
  - [ ] @phenotype/pheno-llm

**WP4.6: Root documentation**

- [ ] Create `packages/README.md`
  ```markdown
  # @phenotype Packages

  Shared npm packages for the Phenotype ecosystem.

  ## Packages
  - @phenotype/pheno-core
  - @phenotype/pheno-resilience
  - @phenotype/pheno-llm

  ## Installation

  Configure .npmrc for GitHub Packages, then:
  npm install @phenotype/pheno-core
  ```
- [ ] Create `packages/PUBLISHING.md` with release procedures
- [ ] Create `packages/CONTRIBUTING.md` with dev setup

**WP4.7: PR preparation**

- [ ] All files committed
- [ ] Rebase on main: `git rebase origin/main`
- [ ] Create PR with title: `feat(packages): add @phenotype npm packages`
- [ ] PR description includes:
  - [ ] Summary of 3 packages added
  - [ ] Link to PHENOSDK_EXTRACTION_STRATEGY.md
  - [ ] Link to PHENOSDK_PACKAGE_AUDIT.md
  - [ ] Testing instructions
  - [ ] Publishing instructions

**WP4.8: Final quality checks**

- [ ] All CI checks passing (except GitHub Actions due to billing)
- [ ] No linting warnings
- [ ] All tests green
- [ ] Types correct
- [ ] Documentation complete
- [ ] Coverage meets threshold

**Checklist Owner**: [TBD]
**Status**: PENDING
**Completion Deadline**: +10 minutes from start of WP4.1

---

## Post-Implementation Validation

### [ ] Verify Implementation Completeness

- [ ] All three packages in `packages/` directory: 3/3 ✓
- [ ] All packages have package.json with correct metadata: 3/3 ✓
- [ ] All packages have full TypeScript implementation: 3/3 ✓
- [ ] All packages have unit tests with ≥80% coverage: 3/3 ✓
- [ ] All packages have JSDoc comments: 3/3 ✓
- [ ] All packages have README.md: 3/3 ✓
- [ ] Publishing workflow configured and tested: 1/1 ✓
- [ ] All packages published to GitHub Packages: 3/3 ✓
- [ ] PR created and passing all checks: 1/1 ✓

### [ ] Success Metrics

| Metric | Target | Actual | Status |
|--------|--------|--------|--------|
| Packages Published | 3 | TBD | [ ] |
| Test Coverage | ≥80% | TBD | [ ] |
| Type Safety | 100% | TBD | [ ] |
| Dependencies (pheno-core) | 0 | TBD | [ ] |
| Documentation | 100% | TBD | [ ] |
| Zero Breaking Changes | - | TBD | [ ] |
| GitHub Packages Accessible | - | TBD | [ ] |

### [ ] Final Checklist

- [ ] Code review complete
- [ ] All comments resolved
- [ ] Merge to main
- [ ] Tag release in GitHub
- [ ] Announce in Phenotype org (slack/teams)
- [ ] Update dependent projects with new packages

---

## Sign-Off

| Role | Name | Date | Status |
|------|------|------|--------|
| **Implementation Lead** | [TBD] | [TBD] | [ ] |
| **Code Reviewer** | [TBD] | [TBD] | [ ] |
| **QA/Testing** | [TBD] | [TBD] | [ ] |
| **Project Manager** | [TBD] | [TBD] | [ ] |

---

**Document Status**: COMPLETE & READY FOR EXECUTION
**Total Estimated Duration**: 80 minutes (agent-driven)
**Confidence Level**: HIGH (95%+)

*Use this checklist to track implementation progress. Update status as each work package completes.*
