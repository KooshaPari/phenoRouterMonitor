# @phenotype npm Packages

Phenotype SDK for TypeScript/JavaScript — framework-agnostic, typed abstractions for distributed systems.

## Packages

### @phenotype/pheno-core

Core contracts, ports, and domain models for Phenotype hexagonal architecture.

```bash
npm install @phenotype/pheno-core
```

**Features:**
- Hexagonal architecture ports (inbound/outbound)
- Domain model abstractions (Entity, AggregateRoot, ValueObject)
- Railway-oriented Result and Option types
- Canonical error types (PhenotypeError, NotFoundError, ValidationError, ConflictError, UnauthorizedError)
- Event and event factory abstractions

**Exports:**
- `./ports` — Driving and driven port interfaces
- `./models` — Entity, ValueObject, Result, Option types
- (main) — All exports combined

### @phenotype/pheno-resilience

Event sourcing, caching, policies, and state machines for resilient distributed systems.

```bash
npm install @phenotype/pheno-resilience
```

**Features:**
- **Event Sourcing**: EventStore, Snapshot, EventProjection interfaces + InMemoryEventStore
- **Caching**: Two-tier cache (L1 in-memory, L2 optional), LRU eviction, cache statistics
- **Policy Engine**: Rule-based policy evaluation with guards, AND/OR logic, built-in rules
- **State Machine**: Typed state transitions, guards, actions, history, fluent builder API

**Exports:**
- `./event-sourcing` — Event store, snapshots, projections
- `./cache` — Two-tier caching with statistics
- `./policy` — Policy engine and rule abstractions
- `./state-machine` — State machine with builder pattern
- (main) — All exports combined

### @phenotype/pheno-llm

LLM provider abstraction and prompt management (NEW, no Rust equivalent).

```bash
npm install @phenotype/pheno-llm
```

**Features:**
- **Provider Abstraction**: Framework-agnostic LLM interface (OpenAI, Anthropic, local, mock)
- **Streaming**: AsyncIterable streaming with chunk types
- **Tools**: Tool/function definition and registration
- **Prompts**: Template system with variable binding, fluent builder, registry
- **Token Counting**: Estimate tokens without calling provider

**Exports:**
- `./providers` — LLM provider interface, implementations, registry
- `./prompts` — Prompt templates, builders, registry
- (main) — All exports combined

## Architecture

All packages follow **Hexagonal Architecture** principles:

```
                    Inbound Ports
                   /             \
            UseCase         CommandHandler    QueryHandler
                   \             /
                     ↓       ↓
              [DOMAIN CORE]
                     ↑       ↑
                   /         \
          Repository      CachePort    EventBus    PolicyEngine
                    \             /
                   Outbound Ports
```

## Usage Examples

### pheno-core

```typescript
import { UseCase, NotFoundError, some, Ok } from '@phenotype/pheno-core';

class FindUserUseCase implements UseCase<string, User> {
  async execute(userId: string): Promise<User> {
    const user = await this.repository.findById(userId);
    if (!user) {
      throw new NotFoundError('User', userId);
    }
    return user;
  }
}
```

### pheno-resilience

```typescript
import { InMemoryEventStore, StateMachineBuilder } from '@phenotype/pheno-resilience';

// Event sourcing
const eventStore = new InMemoryEventStore();
await eventStore.append('agg-1', { eventId: '1', eventType: 'Created', timestamp: new Date(), payload: {} });

// State machine
const sm = new StateMachineBuilder('idle')
  .addState({ id: 'idle', name: 'Idle' })
  .addState({ id: 'active', name: 'Active' })
  .addTransition({ from: 'idle', to: 'active', event: 'start' })
  .build();

await sm.trigger('start');
console.log(sm.getCurrentState()); // 'active'
```

### pheno-llm

```typescript
import { FluentPromptBuilder, MockLLMProvider } from '@phenotype/pheno-llm';

// Build prompt fluently
const prompt = new FluentPromptBuilder()
  .system('You are a code expert')
  .user('Write a function that...')
  .section('Requirements', 'Fast, readable, well-documented')
  .build();

// Use with provider
const provider = new MockLLMProvider();
const response = await provider.complete({
  model: 'gpt-4-turbo',
  messages: [{ role: 'user', content: prompt.userPrompt }],
});
```

## Development

All packages use:
- **Build Tool**: tsup (ESM, TypeScript definitions)
- **Test Runner**: vitest
- **Linter**: oxlint
- **TypeScript**: v5.4+ with strict mode

```bash
# Build all packages
cd packages/pheno-core && npm run build
cd packages/pheno-resilience && npm run build
cd packages/pheno-llm && npm run build

# Test
npm run test

# Lint
npm run lint

# Watch mode
npm run dev
```

## Publishing

All @phenotype packages publish to GitHub Packages:

```bash
npm publish
```

Set `GITHUB_TOKEN` environment variable or configure `.npmrc`:

```
@phenotype:registry=https://npm.pkg.github.com
//npm.pkg.github.com/:_authToken=ghp_xxxxx
```

## Integration with Phenotype

These packages mirror Rust crates in `crates/`:

| npm Package | Rust Crate | Purpose |
|---|---|---|
| @phenotype/pheno-core | phenotype-contracts | Hexagonal ports & domain models |
| @phenotype/pheno-resilience | phenotype-event-sourcing + cache-adapter + policy-engine + state-machine | Resilience patterns |
| @phenotype/pheno-llm | (NEW) | LLM abstractions |

Cross-language consistency maintained through interface documentation and example implementations.

## License

MIT — See LICENSE file
