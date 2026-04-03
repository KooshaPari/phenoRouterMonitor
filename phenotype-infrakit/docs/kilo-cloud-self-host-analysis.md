# Kilo Cloud Self-Host: Full Analysis
> Generated 2026-03-31 | 100+ tools evaluated | 28 PG extensions | 28 DB alternatives | 37 CLI agents

---

## Table of Contents
1. [PostgreSQL Extensions (Modern 2026)](#1-postgresql-extensions-modern-2026)
2. [PostgreSQL Alternatives](#2-postgresql-alternatives)
3. [CLI Coding Agents (37)](#3-cli-coding-agents)
4. [Full Platform Agents (30)](#4-full-platform-agents)
5. [Partial/Niche Tools (118)](#5-partialniche-tools)
6. [Recommended Self-Host Stack](#6-recommended-self-host-stack)

---

## 1. PostgreSQL Extensions (Modern 2026)

### Vector/AI
| Extension | Stars | Description | Relevance |
|-----------|-------|-------------|-----------|
| **pgvector** | 20.5k | HNSW + IVFFlat vector similarity search | CRITICAL — embedding storage |
| **pgai** (TimescaleDB) | 5.8k | RAG pipeline, auto-embedding via SQL, vectorizer workers | HIGH — simplifies RAG |
| **pgvectorscale** (TimescaleDB) | 3.0k | DiskANN for billion-scale vector search | HIGH — scale beyond millions |
| **VectorChord** (TensorChord) | 1.6k | Disk-efficient vector search in Rust (pgvector successor) | HIGH — better disk efficiency |
| **VectorChord-bm25** | 361 | Native BM25 ranking index — hybrid vector+keyword | HIGH — hybrid search |
| **pgvecto.rs** | 2.2k | Rust vector search (superseded by VectorChord) | MEDIUM — migrate to VectorChord |
| **Lantern** | 878 | HNSW with embedding generation routines | MEDIUM |
| **PostgresML** | 6.7k | In-database ML training, inference, embeddings, GPU | HIGH — in-DB inference |

### Search
| Extension | Stars | Description | Relevance |
|-----------|-------|-------------|-----------|
| **ParadeDB pg_search** | 8.6k | Elastic-quality BM25 via Tantivy, hybrid search | CRITICAL — code search |
| **pg_trgm** | built-in | Trigram similarity, LIKE acceleration | HIGH — fuzzy search |
| **RUM** | 826 | Positional inverted index for full-text | MEDIUM — phrase search |
| **ZomboDB** | 4.7k | Elasticsearch-backed Postgres indexes | MEDIUM — powerful but ES dep |

### Queue/Messaging
| Extension | Stars | Description | Relevance |
|-----------|-------|-------------|-----------|
| **PGMQ** | 4.7k | SQS-like message queue in Postgres | HIGH — async task queues |
| **pg-boss** | 3.3k | Node.js job queue backed by Postgres | HIGH — if Node backend |
| **Graphile Worker** | ~2k | Postgres job queue with TypeScript | MEDIUM |
| **pgq** (Skytools) | ~400 | Legacy Postgres queue | LOW — superseded |

### Time-Series
| Extension | Stars | Description | Relevance |
|-----------|-------|-------------|-----------|
| **TimescaleDB** | 22.3k | Time-series hypertables, compression, continuous agg | HIGH — telemetry, usage |
| **TimescaleDB Toolkit** | 464 | Hyperfunctions: percentile, counters, stats | MEDIUM — analytics |

### Graph
| Extension | Stars | Description | Relevance |
|-----------|-------|-------------|-----------|
| **Apache AGE** | 4.4k | Graph DB on Postgres with Cypher queries | HIGH — code dependency graphs |
| **AgensGraph** | ~300 | Enterprise graph (Bitnine) | LOW — AGE is OSS successor |

### JSON/Document
| Extension | Stars | Description | Relevance |
|-----------|-------|-------------|-----------|
| **pg_jsonschema** | 1.2k | JSON Schema validation at DB level | HIGH — validate tool schemas |
| **jsonb** | built-in | Binary JSON with GIN indexing | CRITICAL — flexible agent state |

### Auth/Security
| Extension | Stars | Description | Relevance |
|-----------|-------|-------------|-----------|
| **pgaudit** | 1.6k | Comprehensive audit logging | HIGH — SOC2/HIPAA compliance |
| **pgcrypto** | built-in | Hashing, encryption, HMAC | HIGH — encrypt API keys |
| **pgjwt** | 438 | JWT generation/verification in SQL | MEDIUM — auth tokens from DB |
| **pgsodium** | 597 | libsodium encryption, column-level, key management | HIGH — encrypt at rest |

### Monitoring
| Extension | Stars | Description | Relevance |
|-----------|-------|-------------|-----------|
| **pg_stat_statements** | built-in | Query execution statistics | CRITICAL — query monitoring |
| **pg_stat_monitor** (Percona) | 564 | Enhanced monitoring with histograms | HIGH — deeper analysis |
| **pg_partman** | 2.6k | Automatic time-based partitioning | HIGH — partition logs/events |

### CDC/Event Sourcing
| Extension | Stars | Description | Relevance |
|-----------|-------|-------------|-----------|
| **wal2json** | 1.5k | WAL logical decoding to JSON | HIGH — event sourcing |
| **Debezium** | 12.6k | Enterprise CDC via Kafka Connect | HIGH — stream DB changes |
| **pglogical** | 1.2k | Logical replication for cross-DB sync | MEDIUM — multi-region |
| **pgoutput** | built-in | Native logical replication | MEDIUM — lightweight CDC |

### Sharding/Distribution
| Extension | Stars | Description | Relevance |
|-----------|-------|-------------|-----------|
| **Citus** | 12.4k | Distributed Postgres: sharding, columnar | HIGH — scale vector search |
| **postgres_fdw** | built-in | Foreign data wrapper | MEDIUM — federate queries |
| **Supabase Wrappers** | 833 | FDW framework in Rust (Stripe, S3, etc.) | MEDIUM — external data |

### HTTP/Network
| Extension | Stars | Description | Relevance |
|-----------|-------|-------------|-----------|
| **pg_net** (Supabase) | 348 | Async HTTP requests from SQL | CRITICAL — call LLM APIs from triggers |
| **pgsql-http** | 1.6k | Synchronous HTTP client | HIGH — webhooks |
| **pg_graphql** | 3.3k | Auto-GraphQL from schema | MEDIUM — GraphQL API layer |
| **PostgREST** | 26.7k | Auto-REST from schema | HIGH — instant REST API |

### Other Notable
| Extension | Stars | Description | Relevance |
|-----------|-------|-------------|-----------|
| **pg_cron** | 3.7k | Cron scheduler inside Postgres | HIGH — scheduled tasks |
| **pg_ivm** | 1.4k | Incremental materialized view maintenance | HIGH — real-time dashboards |
| **pgtap** | 1.1k | Unit testing framework for Postgres | HIGH — test DB functions |
| **pg_repack** | 2.2k | Online table reorganization | MEDIUM — maintenance |

### Recommended Core Stack
```sql
CREATE EXTENSION IF NOT EXISTS vector;              -- pgvector
CREATE EXTENSION IF NOT EXISTS pg_trgm;             -- fuzzy search
CREATE EXTENSION IF NOT EXISTS pgcrypto;            -- encryption
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";         -- UUIDs
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;  -- monitoring
CREATE EXTENSION IF NOT EXISTS pg_cron;             -- scheduling
-- Optional:
CREATE EXTENSION IF NOT EXISTS age;                 -- graph (Apache AGE)
CREATE EXTENSION IF NOT EXISTS pgmq;                -- message queue
CREATE EXTENSION IF NOT EXISTS pg_net;              -- HTTP from SQL
CREATE EXTENSION IF NOT EXISTS pg_jsonschema;       -- JSON validation
CREATE EXTENSION IF NOT EXISTS timescaledb;         -- time-series
```

---

## 2. PostgreSQL Alternatives

### Distributed SQL
| DB | Language | License | Stars | Self-host | Best For |
|----|----------|---------|-------|-----------|----------|
| **CockroachDB** | Go | BSL→Apache 2.0 | 32k | Yes | Multi-region, serializable isolation |
| **TiDB** | Go | Apache 2.0 | 39.9k | Yes | HTAP (OLTP+OLAP in one) |
| **Vitess** | Go | Apache 2.0 | 20.9k | Yes | MySQL sharding, online schema changes |
| **YugabyteDB** | C++ | Apache 2.0 | 9k | Yes | PG-compatible distributed |

### Serverless/Edge
| DB | Language | License | Stars | Self-host | Best For |
|----|----------|---------|-------|-----------|----------|
| **Neon** | Rust | Apache 2.0 | 21.4k | No (cloud) | Branching, scale-to-zero |
| **Turso/libSQL** | Rust | MIT | ~12k | Yes | Per-agent embedded DB, edge |
| **Supabase** | TS | Apache 2.0 | 100k | Complex | PG + Auth + Realtime bundle |

### OLAP/Analytics
| DB | Language | License | Stars | Self-host | Best For |
|----|----------|---------|-------|-----------|----------|
| **DuckDB** | C++ | MIT | 37.1k | Embedded | In-process analytics, zero-copy Arrow |
| **ClickHouse** | C++ | Apache 2.0 | 46.6k | Yes | Sub-second queries on billions of rows |
| **Materialize** | Rust | BSL 1.1 | 6.3k | Yes | Streaming materialized views |
| **RisingWave** | Rust | Apache 2.0 | 8.9k | Yes | Streaming platform, PG-compatible |

### Vector Databases
| DB | Language | License | Stars | Self-host | Best For |
|----|----------|---------|-------|-----------|----------|
| **Qdrant** | Rust | Apache 2.0 | 30k | Yes | Purpose-built vector search |
| **Milvus** | Go/C++ | Apache 2.0 | 43.5k | Yes | GPU-accelerated, billion-scale |
| **Weaviate** | Go | BSD-3 | 15.9k | Yes | Built-in vectorizers |
| **Chroma** | Python/Rust | Apache 2.0 | ~22k | Yes | Simplest API, lightweight |
| **LanceDB** | Rust | Apache 2.0 | 9.7k | Embedded | Serverless, zero-copy Lance format |

### Search Engines
| DB | Language | License | Stars | Self-host | Best For |
|----|----------|---------|-------|-----------|----------|
| **Meilisearch** | Rust | MIT | 56.9k | Yes | Typo-tolerant instant search |
| **Typesense** | C++ | GPL-3.0 | 25.5k | Yes | In-memory, curation-focused |
| **Tantivy** | Rust | MIT | 14.8k | Library | Embeddable Lucene-like search |

### Multi-Model
| DB | Language | License | Stars | Self-host | Best For |
|----|----------|---------|-------|-----------|----------|
| **SurrealDB** | Rust | BSL 1.1 | 31.7k | Yes | Doc+graph+rel+vector+KV single binary |
| **ArangoDB** | C++ | Apache 2.0/BSL | 14.1k | Yes | AQL graph traversals |
| **CozoDB** | Rust | MPL 2.0 | 3.9k | Embedded | Datalog queries, graph algorithms |

### Caching/KV
| DB | Language | License | Stars | Self-host | Best For |
|----|----------|---------|-------|-----------|----------|
| **Dragonfly** | C++ | BSL 1.1 | 30.3k | Yes | 25x Redis, fork-less snapshots |
| **FoundationDB** | C++ | Apache 2.0 | 16.2k | Yes | Apple-proven ordered KV |
| **KeyDB** | C++ | BSD-3 | 9.7k | Yes | Multi-threaded Redis fork |
| **Garnet** (Microsoft) | C# | MIT | 16.4k | Yes | .NET Redis-compatible |

### Specialty
| DB | Language | License | Stars | Self-host | Best For |
|----|----------|---------|-------|-----------|----------|
| **TigerBeetle** | Zig | Apache 2.0 | 15.5k | Yes | Deterministic financial ledger |
| **FerretDB** | Go | Apache 2.0 | 10.9k | Yes | MongoDB-compatible on PG |
| **GlueSQL** | Rust | Apache 2.0 | 3k | Library | SQL over JSON/CSV/custom stores |

### Recommended Polyglot Stack
```
Transactional:   Postgres 16 + pgvector + extensions (core)
Cache:           DragonflyDB (Redis replacement)
Search:          Meilisearch or Qdrant (port-based)
Analytics:       DuckDB (in-process, per-session)
Graph:           Apache AGE (PG extension) or SurrealDB
Time-series:     TimescaleDB (PG extension)
Message queue:   PGMQ (PG extension) or NATS
```

---

## 3. CLI Coding Agents

### Tier 1: Major (>30k ★)
| # | Name | Language | License | Stars | Cloud | Self-host | Key Differentiator |
|---|------|----------|---------|-------|-------|-----------|-------------------|
| 1 | **OpenClaw** | TypeScript | MIT | 343k | Yes | Yes | Skills marketplace, 5400+ skills, personal AI |
| 2 | **OpenCode** | TypeScript/Bun | MIT | 134k | No | Yes | Multi-model TUI, MCP, file editing |
| 3 | **Gemini CLI** | TypeScript | Apache-2.0 | 100k | Yes | Yes | Google integration, free tier |
| 4 | **Claude Code** | TypeScript | Proprietary | N/A | Yes | No | Best single-agent, CLAUDE.md context, sub-agents |
| 5 | **Codex CLI** (OpenAI) | **Rust** | Apache-2.0 | 70k | Yes | Yes | Sandboxed by default, Rust speed |
| 6 | **Open Interpreter** | Python | MIT | 63k | No | Yes | Controls entire computer via NL |
| 7 | **Cline** | TypeScript | Apache-2.0 | 60k | Optional | Yes | VS Code autonomous agent, MCP marketplace |
| 8 | **GPT Engineer** | Python | MIT | 55k | No | Yes | Prompt-to-project pioneer |
| 9 | **Aider** | Python | Apache-2.0 | 43k | No | Yes | Best pair-programming CLI, git-aware |
| 10 | **Qwen Code** | TypeScript | Apache-2.0 | 21k | Yes | Yes | Alibaba's agent, multilingual |

### Tier 2: Strong (10k–30k ★)
| # | Name | Language | License | Stars | Cloud | Self-host | Key Differentiator |
|---|------|----------|---------|-------|-------|-----------|-------------------|
| 11 | **Goose** | **Rust** | Apache-2.0 | 34k | No | Yes | Block/Square backed, any LLM |
| 12 | **Continue** | TypeScript | Apache-2.0 | 32k | No | Yes | CI-integrated AI checks |
| 13 | **Tabby** | **Rust** | Apache-2.0 | 33k | No | **Yes** | Best self-hosted Copilot replacement |
| 14 | **Pi** | TypeScript | MIT | 30k | No | Yes | Full-stack toolkit: CLI + API + TUI + Slack |
| 15 | **Roo Code** | TypeScript | Apache-2.0 | 23k | No | Yes | Multi-mode agent team (Cline fork) |
| 16 | **SWE-Agent** | Python | MIT | 19k | No | Yes | Academic, SWE-bench competitive |

### Tier 3: Established (1k–10k ★)
| # | Name | Language | License | Stars | Cloud | Self-host | Key Differentiator |
|---|------|----------|---------|-------|-------|-----------|-------------------|
| 17 | **Kilo CLI** | TypeScript | MIT | 17k | Yes | Yes | OpenCode fork, VS Code + CLI, 500+ models |
| 18 | **Plandex** | **Go** | MIT | 15k | Yes | Yes | Go-based, large real-world codebases |
| 19 | **Smol Developer** | Python | MIT | 12k | No | Yes | Embeddable dev agent library |
| 20 | **Sweep** | Python | AGPL-3.0 | 7.7k | Yes | Yes | Autonomous PR from issues |
| 21 | **Copilot CLI** | Go | Proprietary | 9.7k | Yes | No | GitHub ecosystem integration |
| 22 | **Cody** | TypeScript | Apache-2.0 | 3.8k | Yes | Yes | Sourcegraph code graph context |
| 23 | **Mistral Vibe** | Python | Apache-2.0 | 3.7k | Yes | Yes | Minimal, European AI |
| 24 | **Oh-My-Pi** | TypeScript/Bun | MIT | 2.5k | No | Yes | Hash-anchored edits, subagents |
| 25 | **Auggie** (Augment) | TypeScript | Proprietary | 172 | Yes | No | Enterprise codebase indexing |

### Tier 4: Emerging / Niche (2025-2026)
| # | Name | Language | License | Stars | Cloud | Self-host | Key Differentiator |
|---|------|----------|---------|-------|-------|-----------|-------------------|
| 26 | **VT Code** | **Rust** | MIT | 470 | No | Yes | Semantic Rust-native analysis |
| 27 | **Shai** (OVH) | **Rust** | MIT | 594 | No | Yes | Simple Rust pair-programming |
| 28 | **LLxprt Code** | TypeScript | MIT | 654 | No | Yes | Multi-provider CLI |
| 29 | **Snow CLI** | TypeScript | MIT | 537 | No | Yes | Simultaneous multi-provider |
| 30 | **Wolverine** | Python | MIT | 5.1k | No | Yes | Auto-patches Python runtime errors |

### Orchestration/Meta-Agents
| # | Name | Stars | Purpose |
|---|------|-------|---------|
| 31 | **Claude Squad** | 6.8k | Manage multiple agent sessions |
| 32 | **Superset** | 8.4k | Desktop multi-agent runner |
| 33 | **Agent Deck** | 1.8k | TUI session manager |
| 34 | **Emdash** | 3.4k | Agentic dev environment (YC W26) |
| 35 | **Agent of Empires** | 1.4k | tmux session manager for agents |

### Proprietary/Cloud-Only
| # | Name | Cloud | Key Differentiator |
|---|------|-------|-------------------|
| 36 | **Factory Droid** | Yes | Enterprise, API-first |
| 37 | **Devin** | Yes | Fully autonomous SWE |

---

## 4. Full Platform Agents (30)

| # | Name | Type | OSS? | Cloud Agents | Code Review | Self-host | Pricing |
|---|------|------|------|-------------|-------------|-----------|---------|
| 1 | **Cursor** | VS Code fork | No | Yes | Yes (Bugbot) | No (self-hosted agents Mar 2026) | $20-200/mo |
| 2 | **Kilo Code** | Ext + CLI + Cloud | Yes | Yes | Yes | Yes | Free + credits |
| 3 | **Windsurf** | VS Code fork | No | Yes | Yes | No | $20-200/mo |
| 4 | **GitHub Copilot** | Multi-IDE | No | Yes | Yes | No | $10-19/mo |
| 5 | **Replit Agent** | Web IDE | No | Built-in | No | No | $25-60/mo |
| 6 | **Lovable** | Web builder | No | Deploy | No | No | $25-50/mo |
| 7 | **Bolt.new** | Web builder | No | Cloud | No | No | $25-30/mo |
| 8 | **v0** (Vercel) | UI generator | No | Deploy | No | No | Free + credits |
| 9 | **Augment Code** | Multi-IDE | No | Yes | Yes | No | Enterprise |
| 10 | **Factory** | Multi-surface | No | Yes | Yes | No | Enterprise |
| 11 | **Devin** | Cloud agent | No | Core | Yes | No | $500+/mo |
| 12 | **Qodo** (Codium) | IDE ext | No | Yes | Yes (core) | No | Free + paid |
| 13 | **Tabnine** | Multi-IDE | No | No | Yes | Enterprise | $12-39/mo |
| 14 | **JetBrains AI** | JetBrains | No | No | No | No | Included |
| 15 | **Amazon Q** | Multi-IDE + CLI | No | Yes | Yes | No | $19/mo |
| 16 | **Gemini Code Assist** | Multi-IDE + CLI | Partial | Yes | Yes | No | Free |
| 17 | **Amp** (Sourcegraph) | IDE + CLI | No | Yes | Yes | No | Free + teams |
| 18 | **Cline** | VS Code ext | Yes | Optional | No | Yes | Free (BYOK) |
| 19 | **Roo Code** | VS Code ext | Yes | No | No | Yes | Free (BYOK) |
| 20 | **Firebase Studio** | Web IDE | No | Yes | No | No | Free + GDL |
| 21 | **Claude Code** | CLI + Desktop | No | Yes (Cowork) | Yes | No | $20-200/mo |
| 22 | **Zed AI** | Editor | Yes (editor) | Yes | No | No | Free + credits |
| 23 | **PearAI** | VS Code fork | Yes | No | No | No | Free (BYOK) |
| 24 | **Void Editor** | VS Code fork | Yes | No | No | Yes | Free (BYOK) |
| 25 | **Warp** | AI terminal | No | No | No | No | Free + paid |
| 26 | **Supermaven** | IDE ext | No | No | No | No | Free + $10/mo |
| 27 | **Tabby** | Server + IDE ext | Yes | No | No | **Yes** | Free |
| 28 | **Aider** | CLI | Yes | No | No | Yes | Free (BYOK) |
| 29 | **CodeGeeX** | IDE ext | Yes | No | No | Yes | Free |
| 30 | **Continue** | IDE ext | Yes | No | No | Yes | Free |

---

## 5. Partial/Niche Tools (118 total)

### Code Review (10)
CodeRabbit, PR-Agent (Qodo), SonarQube AI, Kodus AI, Git-LRC, AI-Review, OpenReview (Vercel), Gito, Vet (Imbue), Bottleneck

### App Builders (10)
v0, Bolt.new, Lovable, GPT Engineer, Screenshot-to-Code, Design2Code, CodeImage, React Native IDE, Tempo Labs, Rivet

### Testing (9)
Codium/Qodo, Early AI, Meticulous AI, Korbit AI, Giskard, Promptfoo, Checkie.ai, TestGen AI, AICGSecEval

### Deployment (6)
Railway, Fly.io, Netlify AI, Render, Koyeb, Northflank

### PM/Kanban (5)
VibeKanban, Linear AI, Height AI, Shortcut AI, Plane AI

### Autonomous Agents (10)
Aizen, Sweep AI, SWE-Agent, OpenHands, Mini-SWE-Agent, Refact, Devin, MarsCode, Codex CLI, AgentDeck

### Search/Context (7)
Sourcegraph, Cody, Pieces, Magnet, Greptile, Bloop, Onboard

### Documentation (6)
Mintlify, Swimm, Readme.ai, DocuWriter.ai, Autodoc, Notion AI

### Security (8)
Snyk AI, Semgrep AI, Checkmarx AI, Skylos, Bug-Hunter, Nono, Promptmap, Claude Bug Bounty

### MCP/Tools (6)
Toolhive, Composio, FastMCP, mcp-use, MCP-Agent, Firecrawl MCP

### Code Completion (7)
Supermaven, Tabby, Fauxpilot, Continue, Codeium, Aider, Whisper.cpp pipeline

### Git/Commits (8)
AICommits, OpenCommit, AICommit2, CodeGPT, LazyCommit, GCop, Git-Rewrite, I-Dont-Care-About-Commit-Message

### LLM Ops (5)
Langfuse, Opik, MLflow, LangSmith, Braintrust

### Terminal (5)
YAI, Copilot CLI, ShellGPT, AskAI, Wut

### Slack/Chat (4)
Kilo for Slack, Cursor for Slack, AskCodi, Codegen

### Prompt Management (3)
PromptLayer, Humanloop, Agenta

### DB/Query (3)
Anyquery, Outerbase, Chat2DB

### Code Quality (3)
ai-rules, Bito AI, Sourcery

### Benchmarking (3)
PinchBench, SWE-bench, Aider Polyglot

---

## 6. Recommended Self-Host Stack

### For Kilo Cloud Self-Host (Hybrid OrbStack + Cloudflare Free Tier)

```
┌─────────────────────────────────────────────────────────────┐
│                     ORBSTACK (Local)                         │
│                                                              │
│  ┌──────────┐  ┌───────────────┐  ┌──────────────────────┐  │
│  │ Next.js  │  │  Postgres 16  │  │   DragonflyDB        │  │
│  │ web:3000 │  │  + pgvector   │  │   cache:6379         │  │
│  │          │  │  + pgai       │  │   (25x Redis)        │  │
│  │          │  │  + pg_search  │  │                      │  │
│  │          │  │  + pgmq       │  │                      │  │
│  │          │  │  + pg_cron    │  │                      │  │
│  │          │  │  + timescaledb│  │                      │  │
│  │          │  │  + age (graph)│  │                      │  │
│  └──────────┘  └───────────────┘  └──────────────────────┘  │
│                                                              │
│  ┌──────────────┐  ┌───────────────┐                        │
│  │ Meilisearch  │  │    NATS       │                        │
│  │ search:7700  │  │    msg:4222   │                        │
│  │ (Rust, 57k★) │  │               │                        │
│  └──────────────┘  └───────────────┘                        │
└─────────────────────────────────────────────────────────────┘
          │                              │
          ▼                              ▼
┌─────────────────────┐    ┌──────────────────────────────────┐
│ CLOUDFLARE FREE TIER│    │ DIRECT PROVIDER APIs             │
│                     │    │                                  │
│ cloud-agent-next    │    │ Anthropic, OpenAI, OpenRouter    │
│ code-review-infra   │    │ (via Kilo Gateway self-hosted    │
│ gastown             │    │  or direct BYOK)                 │
│ session-ingest      │    │                                  │
│ git-token-service   │    │                                  │
│ auto-triage         │    │                                  │
│ auto-fix            │    │                                  │
│ app-builder         │    │                                  │
│ webhook-ingest      │    │                                  │
│ o11y (observability)│    │                                  │
│ deploy-infra        │    │                                  │
│ (100K req/day free) │    │                                  │
└─────────────────────┘    └──────────────────────────────────┘
```

### Quickstart

```bash
# 1. Clone
gh repo fork Kilo-Org/cloud --clone && cd cloud

# 2. Start OrbStack services
cat > dev/docker-compose.yml << 'EOF'
services:
  postgres:
    image: pgvector/pgvector:pg16
    ports: ["5432:5432"]
    environment: { POSTGRES_PASSWORD: postgres }
    volumes: ["pgdata:/var/lib/postgresql/data"]

  dragonfly:
    image: dragonflydb/dragonfly:v1.37.2
    ports: ["6379:6379"]
    command: dragonfly --requirepass=kilocache --maxmemory=2gb --cache_mode=true

  meilisearch:
    image: getmeili/meilisearch:v1.13
    ports: ["7700:7700"]

  nats:
    image: nats:2-alpine
    ports: ["4222:4222"]

volumes:
  pgdata:
EOF

cd dev && docker compose up -d

# 3. Setup app
nvm use 22 && pnpm install
cp .env .env.local  # then edit with local URLs
pnpm drizzle migrate

# 4. Deploy CF workers (free)
npx wrangler login
for d in cloud-agent-next cloudflare-code-review-infra cloudflare-gastown \
         cloudflare-session-ingest cloudflare-git-token-service; do
  cd $d && npx wrangler deploy && cd ..
done

# 5. Run web app
pnpm dev

# 6. Login
open http://localhost:3000/users/sign_in?fakeUser=you@admin.example.com
```

### Alternative: Full SurrealDB Stack (Minimal)

If you want to go radical and replace PG + Redis + Vector + Graph with one engine:

```yaml
services:
  surrealdb:
    image: surrealdb/surrealdb:v2
    command: start --user root --pass root rocksdb:/data/db
    ports: ["8000:8000"]
```

SurrealDB gives you: SQL, graph queries, vector search, document store, KV — all in one Rust binary. Trade-off: less mature ecosystem than PG extensions.
