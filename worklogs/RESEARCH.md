# RESEARCH Worklogs

Technology research, starred repository analysis, and innovation tracking.

---

## 2026-03-29 - Starred Repos Analysis Summary

**Project:** [cross-repo]
**Category:** research
**Status:** completed
**Priority:** P1

### Summary

Comprehensive analysis of 30 starred GitHub repositories for potential integration.

### Top Repos by Category

| Category | Repos | Key Insights |
|----------|-------|--------------|
| Agent Skills | harbor-framework/skills | Standardized skill framework |
| Code Gen | antinomyhq/forgecode | Claude Code tooling |
| Knowledge | khoj-ai/khoj | Local RAG pipeline |
| Data | pathwaycom/pathway | Real-time ML streaming |
| Runtime | nitrojs/nitro | Edge/serverless framework |
| BI | lightdash/lightdash | YAML-first analytics |
| Learning | codecrafters-io | Build-your-own-X education |
| Data Quality | great-expectations | Expectation suites |

### Priority Integrations

1. **harbor-framework/skills** - Agent skills standard
2. **khoj-ai/khoj** - Local knowledge base
3. **pathwaycom/pathway** - Real-time ML
4. **great-expectations** - Agent validation

---

## 2026-03-29 - harbor-framework/skills Deep Dive

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P1

### Overview

Agent skills framework for standardized tool/skill definitions.

### Key Features

| Feature | Description | Phenotype Fit |
|---------|-------------|---------------|
| Skill Registry | Centralized skill definitions | High |
| Tool Definitions | Standardized schemas | High |
| Skill Versioning | Semantic versioning | Medium |
| Runtime Adapters | Pluggable runtime | Medium |

### Architecture

```
Skill Definition (SKILL.md)
    |
    v
Skill Registry (registry.json)
    |
    v
Runtime Adapter (Rust/Python/JS)
    |
    v
Agent Execution
```

### Phenotype Integration

**Fork Option:** Create `platforms/harbor-skills`

```yaml
# Phenotype skill example
name: spec-generator
version: 1.0.0
description: Generate specifications from requirements
tools:
  - name: parse_requirements
    input: Requirements
    output: StructuredRequirements
  - name: generate_spec
    input: StructuredRequirements
    output: Spec
runtime: phenotype
```

### Next Steps
- [ ] Fork harbor-framework/skills
- [ ] Add Phenotype-specific skills
- [ ] Implement runtime adapter

---

## 2026-03-29 - antinomyhq/forgecode Deep Dive

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P1

### Overview

Claude Code tooling for code generation and agent workflows.

### Key Features

| Feature | Description | Phenotype Fit |
|---------|-------------|---------------|
| Code Generation | Template-based code gen | High |
| Agent Workflows | Structured agent loops | High |
| PR Integration | Automated PR workflows | High |
| CLI Tooling | Developer experience | Medium |

### Architecture

```python
# Forge code workflow
class ForgeWorkflow:
    def __init__(self, config):
        self.agent = ClaudeAgent(config)
        self.generator = CodeGenerator(config)
    
    async def run(self, spec: Specification):
        # 1. Parse specification
        ast = self.parser.parse(spec)
        
        # 2. Generate code
        code = self.generator.generate(ast)
        
        # 3. Validate output
        self.validator.validate(code)
        
        # 4. Create PR
        return self.pr.create(code)
```

### Phenotype Integration

**Learn from:**
- Structured agent loops
- Code generation patterns
- CLI tooling design

### Next Steps
- [ ] Study forgecode patterns
- [ ] Apply to spec-driven development
- [ ] Improve CLI tooling

---

## 2026-03-29 - khoj-ai/khoj Deep Dive

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P1

### Overview

Local AI knowledge base with RAG pipeline.

### Key Features

| Feature | Description | Phenotype Fit |
|---------|-------------|---------------|
| Local Embeddings | No API needed | High |
| Multiple Backends | PostgreSQL, SQLite | High |
| RAG Pipeline | Retrieval augmented | High |
| API-first | REST + GraphQL | Medium |
| Chat Interface | Conversational | Medium |

### Architecture

```
Documents
    |
    v
Document Processing (chunk, parse)
    |
    v
Embedding Generation (local model)
    |
    v
Vector Store (pgvector/sqlite)
    |
    v
Retrieval (semantic search)
    |
    v
LLM Context (augmented generation)
```

### Phenotype Integration

**Fork Option:** Create `platforms/knowledge-base`

```python
# Phenotype knowledge integration
from khoj import Khoj

khoj = Khoj(
    embedder="ollama",
    model="nomic-embed-text"
)

# Index Phenotype docs
khoj.index(
    documents=phenotype_docs,
    filters={"project": "agileplus"}
)

# Query with context
results = khoj.search(
    query="spec-driven development",
    filters={"type": "spec"}
)
```

### Next Steps
- [ ] Evaluate khoj for knowledge base
- [ ] Set up local embedding model
- [ ] Integrate with agent dispatch

---

## 2026-03-29 - pathwaycom/pathway Deep Dive

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P1

### Overview

Real-time ML data processing with streaming connectors.

### Key Features

| Feature | Description | Phenotype Fit |
|---------|-------------|---------------|
| Stream Processing | Real-time data | High |
| 40+ Connectors | Databases, APIs, files | High |
| LLM xpack | RAG pipelines | High |
| MCP Server | Model Context Protocol | High |

### Architecture

```
Data Sources
    |
    v
Pathway Pipeline
    |--- Transformer
    |--- Join
    |--- Aggregate
    |
    v
LLM Integration (xpack)
    |
    v
Output (Sinks)
```

### Pathway MCP Pattern

```python
from pathway.xpacks.llm import MCP_server
from pathway.xpacks.llm.vector_store import VectorStoreIndex

# Create RAG pipeline
index = VectorStoreIndex.from_documents(
    documents,
    embedder=embedder
)

# Serve as MCP server
server = MCP_server(
    name="phenotype-knowledge",
    index=index,
    metadata={"source": "agileplus"}
)

server.run()
```

### Phenotype Integration

**Fork Option:** Create `platforms/pathway-xpack`

### Next Steps
- [ ] Evaluate Pathway for data pipelines
- [ ] Create custom connectors
- [ ] Build MCP server

---

## 2026-03-29 - nitrojs/nitro Deep Dive

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P2

### Overview

Edge/serverless framework with hybrid rendering.

### Key Features

| Feature | Description | Phenotype Fit |
|---------|-------------|---------------|
| Edge Deployment | Cloudflare, Vercel | Medium |
| Hybrid Rendering | SSR + API | Low |
| Smart Defaults | Zero-config | Medium |
| AI Routes | Built-in LLM support | Medium |

### Nitro AI Routes

```typescript
// Built-in AI support
export default defineEventHandler(async (event) => {
  const body = await readBody(event);
  
  return ai.complete('gpt-4', {
    messages: body.messages,
    temperature: 0.7,
  });
});

// Streaming support
export default defineEventHandler(async (event) => {
  const stream = await ai.stream('claude-3-sonnet', {
    messages: body.messages
  });
  
  return sendStream(event, stream);
});
```

### Phenotype Integration

**Use Case:** Agent runtime at edge

### Next Steps
- [ ] Evaluate for edge deployment
- [ ] Consider for lightweight agent runtime

---

## 2026-03-29 - lightdash/lightdash Deep Dive

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P2

### Overview

BI tool with YAML-first approach and semantic layer.

### Key Features

| Feature | Description | Phenotype Fit |
|---------|-------------|---------------|
| dbt Integration | SQL-first BI | Medium |
| YAML Config | Declarative | High |
| Semantic Layer | Metrics as code | Medium |
| MCP Server | AI integration | High |

### YAML-First Approach

```yaml
# lightdash YAML pattern
version: 2
models:
  - name: users
    meta:
      label: Users
    dimensions:
      - name: created_at
        sql: created_at
        type: timestamp
        meta:
          format: 'YYYY-MM-DD'
    metrics:
      - name: total_users
        sql: user_id
        type: count_distinct
        meta:
          format: ',~r'
```

### Phenotype Integration

**Learn from:**
- YAML configuration patterns
- Semantic layer design
- MCP integration for AI

### Next Steps
- [ ] Study YAML patterns for config
- [ ] Evaluate semantic layer concepts

---

## 2026-03-29 - codecrafters-io/build-your-own-x Deep Dive

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P2

### Overview

Educational platform with build-your-own-X tutorials.

### Key Features

| Feature | Description | Phenotype Fit |
|---------|-------------|---------------|
| Hands-on Learning | Build from scratch | High |
| Multiple Techs | 60+ technologies | Medium |
| Progressive Difficulty | Step-by-step | Medium |
| Test-driven | Verify implementation | High |

### Topics Covered

| Category | Examples |
|----------|----------|
| Languages | Python, Go, Rust, JavaScript |
| Databases | Redis, PostgreSQL, Git |
| Networking | Docker, WebSocket, DNS |
| Web | Web framework, ORM, template engine |
| AI | GitHub Copilot, ChatGPT bots |

### Phenotype Integration

**Use Case:** Agent training materials

```rust
// Example: Build your own agent
#[derive(Debug)]
pub struct AgentSpec {
    name: String,
    capabilities: Vec<Capability>,
    tools: Vec<ToolDefinition>,
}

impl AgentSpec {
    pub fn from_tutorial() -> Self {
        // Step 1: Basic agent loop
        // Step 2: Add tool calling
        // Step 3: Implement memory
        // Step 4: Add planning
    }
}
```

### Next Steps
- [ ] Create agent development tutorial
- [ ] Build agent playground

---

## 2026-03-29 - great-expectations Deep Dive

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P2

### Overview

Data validation library with expectation suites.

### Key Features

| Feature | Description | Phenotype Fit |
|---------|-------------|---------------|
| Expectation Suites | Declarative validation | High |
| Checkpoints | Automated validation | High |
| Data Connectors | Multiple sources | Medium |
| Profiling | Auto-generate expectations | Medium |

### Expectation Suite Pattern

```python
from great_expectations import ExpectationSuite

suite = ExpectationSuite("user_validation")
suite.add_expectation(
    ExpectColumnValuesToBeUnique("user_id")
)
suite.add_expectation(
    ExpectColumnValuesToNotBeNull("email")
)
suite.add_expectation(
    ExpectColumnValueLengthsToBeBetween(
        "name", min_value=1, max_value=100
    )
)

# Run validation
checkpoint = Checkpoint("user_checkpoint", suites=[suite])
result = checkpoint.run(data_context=context)
```

### Phenotype Integration

**Fork Option:** Create `platforms/llm-eval`

```python
# Agent output validation
from great_expectations import ExpectationSuite

agent_suite = ExpectationSuite("agent_output_validation")

# For code generation
agent_suite.add_expectation(
    ExpectValidPython("generated_code")
)
agent_suite.add_expectation(
    ExpectNoSyntaxErrors("generated_code")
)

# For spec generation
agent_suite.add_expectation(
    ExpectValidSpecMarkdown("generated_spec")
)
agent_suite.add_expectation(
    ExpectCompleteRequirements("generated_spec")
)
```

### Next Steps
- [ ] Evaluate for agent output validation
- [ ] Create expectation library for agent outputs

---

## 2026-03-29 - rustdesk/rustdesk Deep Dive

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P3

### Overview

Open source remote desktop with Rust/H264.

### Key Features

| Feature | Description | Phenotype Fit |
|---------|-------------|---------------|
| Low Latency | Custom protocol | Low |
| Rust-based | Performance | Medium |
| P2P Option | No server needed | Low |
| Cross-platform | All major OS | Medium |

### Phenotype Relevance

**Use Case:** Remote agent control/debugging

### Next Steps
- [ ] Consider for remote debugging
- [ ] Low priority

---

## 2026-03-29 - hoppscotch/hoppscotch Deep Dive

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P2

### Overview

API testing tool with modern UX.

### Key Features

| Feature | Description | Phenotype Fit |
|---------|-------------|---------------|
| REST/GraphQL | Multiple protocols | High |
| WebSocket | Real-time testing | High |
| Environment Vars | Dev/Staging/Prod | High |
| Collections | Test organization | Medium |

### Phenotype Integration

**Learn from:**
- API testing UX
- Environment management
- Collection patterns

### Next Steps
- [ ] Improve API testing tooling
- [ ] Consider for MCP testing

---

## 2026-03-29 - donnemartin/system-design-primer Deep Dive

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P2

### Overview

Educational resource for system design.

### Key Topics

| Topic | Description |
|-------|-------------|
| Scalability | Load balancing, caching |
| Storage | Databases, CDNs |
| Messaging | Queues, pub/sub |
| Distributed | Consensus, replication |

### Phenotype Integration

**Use Case:** Agent system design capabilities

### Next Steps
- [ ] Use for agent training
- [ ] Build system design agent

---

## 2026-03-29 - f/prompts.chat Deep Dive

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P3

### Overview

Curated prompt collection.

### Categories

| Category | Examples |
|----------|----------|
| Coding | Code review, refactoring |
| Writing | Documentation, emails |
| Analysis | Data analysis, debugging |

### Phenotype Integration

**Use Case:** Prompt engineering

### Next Steps
- [ ] Collect useful prompts
- [ ] Build prompt library

---

## 2026-03-29 - Widelands Deep Dive

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P3

### Overview

Open source RTS game in C++.

### Relevance

Low - gaming domain

---

## 2026-03-29 - Directus Deep Dive

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P3

### Overview

Headless CMS with SQL database.

### Key Features

| Feature | Description |
|---------|-------------|
| REST/GraphQL API | Auto-generated |
| Role-based Access | Fine-grained |
| File Assets | Media management |

### Phenotype Relevance

Potential for content management use cases.

---

## 2026-03-29 - Neon Database Deep Dive

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P2

### Overview

Serverless Postgres with branching.

### Key Features

| Feature | Description | Phenotype Fit |
|---------|-------------|---------------|
| Serverless | Auto-scale | High |
| Branching | Like Git | High |
| Point-in-time | Time travel | High |

### Architecture

```
Main Branch (production)
    |
    +-- feature-branch-1 (dev)
    +-- feature-branch-2 (testing)
    +-- preview-branch (staging)
```

### Phenotype Integration

**Use Case:** Development workflow

```sql
-- Create feature branch
CREATE BRANCH feature-auth FROM main;

-- Use in tests
SET DATABASE = 'feature-auth';

-- Merge back
MERGE BRANCH feature-auth INTO main;
```

### Next Steps
- [ ] Evaluate Neon for development
- [ ] Consider for preview environments

---

## 2026-03-29 - Nitrojs/Edge Functions Deep Dive

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P2

### Overview

Edge deployment patterns with Nitro.

### Deployment Targets

| Target | Provider | Use Case |
|--------|----------|----------|
| cloudflare-workers | Cloudflare | Global edge |
| vercel-edge | Vercel | JAMstack |
| deno-deploy | Deno | Deno runtime |
| aws-lambda | AWS | Serverless |

### Phenotype Integration

**Use Case:** Edge agent runtime

### Next Steps
- [ ] Evaluate for agent deployment
- [ ] Consider lightweight agent at edge

---

## 2026-03-29 - Mautic Deep Dive

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P3

### Overview

Marketing automation platform.

### Key Features

| Feature | Description |
|---------|-------------|
| Email Marketing | Campaigns, automation |
| Lead Scoring | Behavior tracking |
| Analytics | Attribution |

### Phenotype Relevance

Low - marketing domain

---

## 2026-03-29 - Syft Deep Dive

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P2

### Overview

SBOM generation for container images.

### Key Features

| Feature | Description |
|---------|-------------|
| SBOM Generation | Software bill of materials |
| Vulnerability Scanning | CVE detection |
| Multiple Formats | SPDX, CycloneDX |

### Phenotype Integration

**Use Case:** Security compliance

```bash
# Generate SBOM
syft packages dir:. -o spdx-json > sbom.json

# Scan for vulnerabilities
syft packages dir:. -o json | grype
```

### Next Steps
- [ ] Add to CI/CD pipeline
- [ ] Track dependencies

---

## 2026-03-29 - Buf CLI Deep Dive

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P2

### Overview

Modern Protocol Buffers tooling.

### Key Features

| Feature | Description |
|---------|-------------|
| Linting | Proto lint rules |
| Breaking Change | API compatibility |
| Dependency Management | buf.build |
| Schema Registry | Cloud registry |

### Phenotype Integration

**Use Case:** Proto management

```yaml
# buf.yaml
version: v2
lint:
  use:
    - DEFAULT
  except:
    - PACKAGE_VERSION_SUFFIX
breaking:
  use:
    - FILE
```

### Next Steps
- [ ] Set up buf workspace
- [ ] Add to CI

---

## 2026-03-29 - SSVC Deep Dive

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P3

### Overview

Stakeholder-specific vulnerability categorization.

### Key Concepts

| Concept | Description |
|---------|-------------|
| Decision Tree | Structured triage |
| Exploitation | Current status |
| Automatable | Manual vs auto |

### Phenotype Relevance

Low - security domain

---

## 2026-03-29 - Agent Skills Framework Research

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P1

### Summary

Research into standardized agent skills frameworks.

### Framework Comparison

| Framework | Language | Tool Support | Registry |
|-----------|----------|--------------|----------|
| harbor-skills | Multiple | Standardized | Yes |
| MCP | JSON-RPC | Native | Yes |
| LangChain Tools | Python | Dynamic | No |
| OpenAI Plugins | JSON | OpenAPI | Limited |

### Skill Definition Schema

```yaml
# Standard skill definition
apiVersion: skills.io/v1
kind: Skill
metadata:
  name: spec-generator
  version: 1.0.0
spec:
  description: Generate specifications
  inputs:
    - name: requirements
      type: text
      required: true
  outputs:
    - name: spec
      type: markdown
  tools:
    - name: parser
      type: function
    - name: generator
      type: function
  runtime:
    type: phenotype
```

### Recommendation

Fork harbor-framework/skills for Phenotype-specific skills.

---

## 2026-03-29 - MCP (Model Context Protocol) Research

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P1

### Summary

Research into Model Context Protocol for agent tool integration.

### Protocol Overview

```
Client (Agent)
    |
    v (JSON-RPC)
MCP Server (Tools/Resources)
    |
    v
Tool Execution
    |
    v
Response (JSON-RPC)
```

### Capabilities

| Capability | Description |
|------------|-------------|
| Tools | Function calling |
| Resources | Data access |
| Prompts | Templated prompts |

### Phenotype Integration

```python
# MCP server for Phenotype
from mcp.server import MCPServer
from mcp.types import Tool

server = MCPServer("phenotype")

@server.tool("generate_spec")
def generate_spec(requirements: str) -> str:
    """Generate specification from requirements."""
    # ...
```

### Next Steps
- [ ] Implement MCP server for tools
- [ ] Add MCP client to agent

---

## 2026-03-29 - RAG Pipeline Research

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P1

### Summary

Research into Retrieval Augmented Generation pipelines.

### Pipeline Stages

```
Query → Embed → Retrieve → Rerank → Generate
```

### Key Components

| Component | Options | Phenotype Fit |
|-----------|---------|---------------|
| Embedder | OpenAI, local | High |
| Vector Store | Pinecone, pgvector | High |
| Retriever | BM25, semantic | High |
| Reranker | Cohere, local | Medium |

### Phenotype Implementation

```python
from pathway.xpacks.llm import VectorStoreIndex
from pathway import StreamInput

# Create index
index = VectorStoreIndex.from_documents(
    documents=specs_and_docs,
    embedder=local_embedder
)

# Query with context
context = index.query(
    query="spec-driven development patterns",
    metadata_filter={"type": "spec"}
)
```

### Next Steps
- [ ] Set up vector store
- [ ] Create document loader
- [ ] Implement reranking

---

## 2026-03-29 - Semantic Search Research

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P1

### Summary

Research into semantic search implementations.

### Approaches

| Approach | Use Case | Implementation |
|----------|----------|----------------|
| Dense | General semantic | Embeddings |
| Sparse | Keyword matching | BM25 |
| Hybrid | Best of both | Ensemble |

### Hybrid Search

```python
# Combine dense and sparse
def hybrid_search(query, documents):
    # Dense: Semantic similarity
    dense_scores = embedder.similarity(query, documents)
    
    # Sparse: Keyword matching
    sparse_scores = bm25.score(query, documents)
    
    # Combine with RRF
    combined = reciprocal_rank_fusion(
        [dense_scores, sparse_scores],
        k=60
    )
    
    return sorted(documents, key=combined, reverse=True)
```

### Phenotype Use Cases

1. Spec search
2. Code search
3. Documentation search
4. Issue search

---

## 2026-03-29 - Agent Memory Architecture Research

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P1

### Summary

Research into agent memory systems.

### Memory Types

| Type | Duration | Capacity | Use Case |
|------|----------|----------|----------|
| Short-term | Current session | Limited | Context |
| Working | Current task | Medium | Subgoals |
| Long-term | Persistent | Unlimited | Learned |

### Implementation Patterns

```python
class AgentMemory:
    def __init__(self):
        self.short_term = VectorStore()  # Recent context
        self.working = Scratchpad()      # Current task
        self.long_term = GraphStore()     # Persistent
    
    def remember(self, experience):
        # Store in appropriate memory
        self.short_term.add(experience)
        
        if experience.important:
            self.long_term.add(experience)
    
    def recall(self, query):
        # Retrieve from all memories
        return self.short_term.search(query) + \
               self.long_term.search(query)
```

### Phenotype Integration

- [ ] Implement episodic memory for agents
- [ ] Add semantic memory for learned patterns
- [ ] Build working memory for task state

---

## 2026-03-29 - Tool Calling Research

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P1

### Summary

Research into tool calling patterns for agents.

### Approaches

| Approach | Pros | Cons |
|----------|------|------|
| Function calling | Structured | Schema required |
| Code generation | Flexible | Error-prone |
| API mapping | Simple | Limited |

### Function Calling Pattern

```python
# Define tool schema
tools = [
    {
        "name": "generate_spec",
        "description": "Generate specification from requirements",
        "parameters": {
            "type": "object",
            "properties": {
                "requirements": {
                    "type": "string",
                    "description": "User requirements"
                }
            },
            "required": ["requirements"]
        }
    }
]

# Agent uses tools
response = agent.chat(
    messages=[{"role": "user", "content": prompt}],
    tools=tools
)

if response.tool_calls:
    for call in response.tool_calls:
        result = execute_tool(call.function.name, call.function.arguments)
```

### Phenotype Tools

1. Spec generator
2. Code reviewer
3. Test generator
4. Documentation builder

---

## 2026-03-29 - Prompt Engineering Research

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P2

### Summary

Research into prompt engineering techniques.

### Techniques

| Technique | Description | Effectiveness |
|-----------|-------------|----------------|
| Chain-of-thought | Step-by-step reasoning | High |
| Few-shot | Examples in prompt | High |
| System prompts | Role definition | High |
| Chain-of-density | Progressive summarization | Medium |

### Prompt Templates

```python
# System prompt
SYSTEM_PROMPT = """You are {role}, an expert in {domain}.

Guidelines:
{guidelines}

Format your responses as:
1. {format_requirement}
2. {format_requirement}
"""

# Few-shot examples
FEW_SHOT = """
Example 1:
Input: {example_input_1}
Output: {example_output_1}

Example 2:
Input: {example_input_2}
Output: {example_output_2}
"""
```

### Phenotype Prompts

1. **Spec Generation:** Structured requirements to spec
2. **Code Review:** PR description + code to review
3. **Test Generation:** Function signature + code to tests

---

## 2026-03-29 - Vector Database Research

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P1

### Summary

Research into vector database options.

### Options

| Database | Pros | Cons | Deployment |
|----------|------|------|-------------|
| pgvector | Postgres native | Limited scale | Self-hosted |
| Pinecone | Managed | Vendor lock-in | Cloud |
| Weaviate | Rich features | Complex | Both |
| Qdrant | High performance | Newer | Both |
| Chroma | Simple | Limited | Local |

### pgvector Setup

```sql
-- Enable extension
CREATE EXTENSION IF NOT EXISTS vector;

-- Create table
CREATE TABLE documents (
    id SERIAL PRIMARY KEY,
    content TEXT,
    embedding vector(1536)
);

-- Create index
CREATE INDEX ON documents USING ivfflat (embedding vector_cosine_ops);

-- Query
SELECT * FROM documents
ORDER BY embedding <=> '[0.1, 0.2, ...]'
LIMIT 5;
```

### Recommendation

Start with pgvector (existing Postgres), evaluate Qdrant for scale.

---

## 2026-03-29 - Embedding Model Research

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P1

### Summary

Research into embedding model options.

### Models

| Model | Dimensions | Use Case | Deployment |
|-------|------------|----------|-------------|
| text-embedding-3-large | 3072 | Best quality | API |
| text-embedding-3-small | 1536/512 | Balance | API |
| nomic-embed-text | 768 | Local | Self-hosted |
| all-MiniLM-L6-v2 | 384 | Fast | Self-hosted |

### Local Embedding

```python
# Using Ollama for local embeddings
from ollama import Client

client = Client(host='http://localhost:11434')

response = client.embeddings(
    model='nomic-embed-text',
    prompt='spec-driven development'
)

embedding = response['embedding']
```

### Recommendation

1. Start with API-based (OpenAI/Cohere)
2. Add local option for privacy/cost
3. Use quantization for local models

---

## 2026-03-29 - Agent Orchestration Research

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P1

### Summary

Research into agent orchestration patterns.

### Patterns

| Pattern | Description | Use Case |
|---------|-------------|----------|
| Single agent | One agent handles all | Simple tasks |
| Multi-agent | Multiple specialized agents | Complex workflows |
| Hierarchical | Agent manages sub-agents | Delegation |
| Debate | Agents argue and vote | Consensus |

### Multi-Agent Pattern

```python
class AgentOrchestrator:
    def __init__(self):
        self.agents = {
            "planner": PlannerAgent(),
            "coder": CoderAgent(),
            "reviewer": ReviewerAgent(),
        }
    
    async def execute(self, task: Task) -> Result:
        # Planner decomposes task
        plan = await self.agents["planner"].plan(task)
        
        # Execute subtasks
        results = []
        for subtask in plan.subtasks:
            agent = self.agents[subtask.type]
            result = await agent.execute(subtask)
            results.append(result)
        
        # Reviewer validates
        return await self.agents["reviewer"].review(results)
```

### Phenotype Use Case

1. Spec → Code → Tests pipeline
2. Code review workflow
3. Documentation generation

---

## 2026-03-29 - Streaming LLM Research

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P1

### Summary

Research into streaming LLM responses.

### Implementation

```python
async def stream_response(prompt: str):
    async for chunk in llm.stream(prompt):
        yield chunk.content

# SSE endpoint
@app.post("/chat/stream")
async def chat_stream(request: ChatRequest):
    async def event_generator():
        async for chunk in stream_response(request.prompt):
            yield f"data: {chunk}\n\n"
    
    return StreamingResponse(
        event_generator(),
        media_type="text/event-stream"
    )
```

### Client Handling

```javascript
// Frontend streaming
const response = await fetch('/chat/stream', {
    method: 'POST',
    body: JSON.stringify({ prompt }),
});

const reader = response.body.getReader();
const decoder = new TextDecoder();

while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    console.log(decoder.decode(value));
}
```

### Phenotype Integration

- [ ] Add streaming to agent endpoints
- [ ] Implement SSE for real-time updates

---

## 2026-03-29 - LLM Evaluation Research

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P2

### Summary

Research into LLM/agent evaluation methods.

### Evaluation Methods

| Method | Description | Use Case |
|--------|-------------|----------|
| Unit tests | Assert on output | Code generation |
| LLM-as-judge | Another LLM evaluates | Open-ended |
| Human eval | Expert review | Quality critical |
| Benchmarks | Standard datasets | Comparison |

### LLM-as-Judge

```python
async def evaluate_with_judge(
    response: str,
    criteria: list[str]
) -> EvaluationResult:
    judge_prompt = f"""Evaluate the following response:
    
    Response: {response}
    
    Criteria:
    {chr(10).join(f'- {c}' for c in criteria)}
    
    Rate each criterion 1-10 and provide feedback."""
    
    result = await llm.complete(judge_prompt)
    return parse_evaluation(result)
```

### Phenotype Evaluation

1. Spec completeness
2. Code correctness
3. Test coverage
4. Documentation quality

---

## 2026-03-29 - Agentic SWE Bench Research

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P2

### Summary

Research into software engineering agent benchmarks.

### Benchmarks

| Benchmark | Tasks | Success Rate | Description |
|-----------|-------|--------------|-------------|
| SWE-bench | 2290 | 25% | GitHub issues |
| HumanEval | 164 | 90%+ | Python coding |
| MBPP | 974 | 80%+ | Python problems |
| BigCodeBench | 114 | 70% | Code generation |

### SWE-bench Pattern

```python
async def solve_swe_bench_task(instance):
    # 1. Read issue description
    issue = read_issue(instance)
    
    # 2. Explore codebase
    repo = checkout_repo(instance)
    
    # 3. Understand problem
    context = await understand(issue, repo)
    
    # 4. Generate fix
    fix = await agent.generate_fix(context)
    
    # 5. Apply and test
    repo.apply(fix)
    result = repo.run_tests()
    
    return result.success
```

### Phenotype Integration

- [ ] Build internal benchmark
- [ ] Track agent performance over time

---

## 2026-03-29 - Claude Code Patterns Research

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P1

### Summary

Research into Claude Code workflows and patterns.

### Key Patterns

| Pattern | Description | Phenotype Fit |
|---------|-------------|---------------|
| Ghostty integration | Terminal-native | Medium |
| Worktree per task | Isolated branches | High |
| PR automation | Automated reviews | High |
| Session continuity | Context preservation | High |

### Worktree Pattern

```bash
# Claude Code workflow
claude --worktree feature/spec-gen

# Inside Claude Code session
/spec Generate specification generator
git commit -m "feat: spec generator"
git push origin feature/spec-gen
claude /pr Create PR with generated code
```

### Phenotype Integration

1. Use worktrees for agent sessions
2. Automate PR creation
3. Track agent contributions

---

## 2026-03-29 - Hexagonal Architecture Research

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P1

### Summary

Research into hexagonal/ports-and-adapters architecture.

### Pattern

```
       External World
             |
    +--------v--------+
    |    Adapters     | (Primary: REST, CLI)
    +--------+--------+
             |
    +--------v--------+
    |     Ports       | (Inbound: UseCases)
    |                 |
    |   Application   |
    |     Core        |
    |                 |
    +--------+--------+
             |
    +--------v--------+
    |     Ports       | (Outbound: Repositories)
    +--------+--------+
             |
    +--------v--------+
    |   Adapters     | (Secondary: DB, External)
    +-----------------+
```

### Phenotype Application

```rust
// Port (inbound)
pub trait SpecGeneratorPort {
    async fn generate(&self, req: GenerateSpecRequest) -> Result<Spec>;
}

// Use case
pub struct GenerateSpecUseCase<R: Repository> {
    repo: R,
}

impl<R: Repository> SpecGeneratorPort for GenerateSpecUseCase<R> {
    async fn generate(&self, req: GenerateSpecRequest) -> Result<Spec> {
        // Business logic
    }
}

// Adapter (primary)
pub struct RestSpecGeneratorAdapter<U: SpecGeneratorPort> {
    usecase: U,
}

impl<U: SpecGeneratorPort> SpecGeneratorPort for RestSpecGeneratorAdapter<U> {
    // Adapts REST to use case
}
```

### Next Steps
- [ ] Apply to spec generation service
- [ ] Define ports for agent operations

---

## 2026-03-29 - Database Branching Research

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P2

### Summary

Research into database branching patterns (like Git).

### Solutions

| Solution | Approach | Phenotype Fit |
|----------|----------|---------------|
| Neon | Logical branching | High |
| PlanetScale | Vitess | Medium |
| GitHub Actions | Point-in-time | Medium |
| Manual | Snapshot/restore | Low |

### Neon Branching

```bash
# Create branch
psql "postgresql://user:pass@db.neon.tech/main"
> CREATE BRANCH feature-auth FROM main;

# Connect to branch
psql "postgresql://user:pass@ep-xxx.feature-auth/auth"

# Use in tests
export DATABASE_URL="postgresql://.../feature-auth"

# Merge back
psql "postgresql://user:pass@db.neon.tech/main"
> MERGE BRANCH feature-auth INTO main;
```

### Phenotype Use Cases

1. Preview environments per PR
2. Feature development isolation
3. A/B testing data

---

## 2026-03-29 - Event Sourcing Research

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P1

### Summary

Research into event sourcing patterns.

### Pattern

```
Command → Aggregate → Event → Event Store
                               |
                               +-- Projection → Read Model
                               |
                               +-- Projection → Projection
```

### Implementation

```rust
#[derive(Event)]
enum DomainEvent {
    SpecCreated { id: Uuid, name: String },
    SpecUpdated { id: Uuid, changes: Changes },
    SpecApproved { id: Uuid, approver: UserId },
}

pub struct SpecAggregate {
    id: Uuid,
    version: i64,
    events: Vec<DomainEvent>,
}

impl Aggregate for SpecAggregate {
    type Event = DomainEvent;
    
    fn apply(&mut self, event: DomainEvent) {
        match event {
            DomainEvent::SpecCreated { id, name } => {
                self.id = id;
                self.name = name;
            }
            // ...
        }
    }
}
```

### Phenotype Application

- [ ] Event-driven spec lifecycle
- [ ] Audit trail for all changes
- [ ] Temporal queries

---

## 2026-03-29 - CQRS Research

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P1

### Summary

Research into Command Query Responsibility Segregation.

### Pattern

```
Commands (Write)              Queries (Read)
    |                              |
    v                              |
+-----------------+                |
| Command Handler |                |
+-----------------+                |
        |                          |
        v                          |
+-----------------+                |
|    Aggregate    |                |
+-----------------+                |
        |                          |
        v                          |
+-----------------+                |
|  Event Store    |-------------->+
+-----------------+
                       |
                       v
              +-----------------+
              |   Projections   |
              +-----------------+
                       |
                       v
              +-----------------+
              |   Read Models   |
              +-----------------+
```

### Phenotype Application

```rust
// Commands
pub async fn create_spec(cmd: CreateSpecCommand) -> Result<SpecId> {
    let spec = SpecAggregate::new(cmd.name)?;
    event_store.save(spec.drain_events()).await?;
    Ok(spec.id())
}

// Queries
pub async fn get_spec_view(id: SpecId) -> Result<SpecView> {
    read_model.get(id).await
}
```

---

## 2026-03-29 - Zero Trust Security Research

**Project:** [cross-repo]
**Category:** research
**Status:** pending
**Priority:** P1

### Summary

Research into zero trust security patterns.

### Principles

| Principle | Implementation |
|-----------|----------------|
| Verify explicitly | AuthN/AuthZ every request |
| Least privilege | Role-based access |
| Assume breach | Encrypt at rest/transit |
| Continuous verification | Session validation |

### Implementation

```rust
// Middleware
async fn verify_auth(
    request: Request,
    next: Next,
) -> Result<Response, AuthError> {
    // Extract token
    let token = request.headers()
        .get("Authorization")?
        .ok_or(AuthError::NoToken)?;
    
    // Verify token
    let claims = verify_jwt(token)?;
    
    // Check permissions
    let user = User::from_claims(&claims)?;
    if !user.can_access(&request.resource()) {
        return Err(AuthError::Forbidden);
    }
    
    // Continuous validation
    if user.session_expired() {
        return Err(AuthError::SessionExpired);
    }
    
    next.run(request.with_user(user)).await
}
```

### Phenotype Requirements

1. All agent actions require authentication
2. Scoped tokens for tool access
3. Audit logging for all operations

---

## 2026-03-29 - Ecosystem Gap Research: Go, Rust, AI Infra, TypeScript, Cross-cutting (2025-2026)

**Project:** [cross-repo]
**Category:** research
**Status:** completed
**Priority:** P1

### Summary

Targeted research into packages and protocols released or significantly updated in 2025-2026 that fill gaps in the Phenotype ecosystem not covered by the existing worklog entries (ra2a, mentisdb, forza-core, eventually, casbin, figment, koanf, chi, go-retryablehttp, gobreaker, mockery v3, orval, openapi-fetch, react-error-boundary, xstate, temporal-sdk, tauri, pathway, khoj, gix/gitoxide, command-group, indicatif, mcp-sdk, anthropic crate, llm-chain, tiktoken-rs, pydantic, miette, zod).

---

### Go: HTTP Frameworks and Middleware

#### `go.opentelemetry.io/contrib/instrumentation/github.com/go-chi/chi/v5/otelchi`

- **GitHub:** https://github.com/riandyrn/otelchi (community fork; official contrib: opentelemetry-go-contrib)
- **Stars:** ~320 (riandyrn fork); official contrib lives in opentelemetry-go-contrib
- **Last update:** Active through Q1 2026
- **What it replaces/improves:** Fills the OTel gap for chi-based services in AgilePlus and byteport. The official `otelhttp` wrapper requires wrapping each handler individually; otelchi attaches as a single router-level middleware that auto-names spans from chi route patterns.
- **Recommendation:** ADOPT
- **LOC savings:** ~40-80 lines of manual span boilerplate per service

#### Go 1.22+ `net/http` ServeMux with pattern routing

- **GitHub:** https://github.com/golang/go (stdlib)
- **Stars:** N/A (stdlib)
- **Last update:** Go 1.22 (Feb 2024), Go 1.23 (Aug 2024), Go 1.24 (Feb 2025)
- **What it replaces/improves:** For simple internal APIs (health, metrics endpoints) within Phenotype services, the Go 1.22+ stdlib `http.ServeMux` now supports method-prefixed and wildcard patterns (`GET /items/{id}`). Eliminates the chi dependency for services with fewer than 10 routes.
- **Recommendation:** EVALUATE — use stdlib for leaf microservices; retain chi for services with middleware stacks.
- **LOC savings:** Removes one dependency for simple HTTP handlers.

#### `connectrpc.com/connect` (connect-go) v1.x

- **GitHub:** https://github.com/connectrpc/connect-go
- **Stars:** ~5.7k
- **Last update:** Active, v1.18+ in 2025
- **What it replaces/improves:** Replaces raw gRPC + grpc-gateway setups. Connect-go generates handlers that work over gRPC, gRPC-Web, and a plain JSON-over-HTTP protocol from the same `.proto` file. Pairs with `buf` for schema management. Directly relevant to any future byteport or AgilePlus inter-service communication.
- **Recommendation:** ADOPT for new service boundaries. Use alongside buf.
- **LOC savings:** Eliminates grpc-gateway + HTTP gateway glue (~200-500 lines per service boundary).

#### `github.com/bufbuild/buf` (buf CLI + BSR)

- **GitHub:** https://github.com/bufbuild/buf
- **Stars:** ~10k
- **Last update:** Active through 2026
- **What it replaces/improves:** Replaces raw `protoc` invocations and hand-managed plugin chains. Provides schema linting, breaking-change detection, and a schema registry (BSR). Required companion to connect-go for any proto-based service. Relevant if byteport or AgilePlus adopt proto contracts.
- **Recommendation:** ADOPT alongside connect-go.
- **LOC savings:** Replaces multi-step protoc Makefiles with a single `buf generate`.

---

### Go: Auth Libraries

#### `github.com/aidantwoods/go-paseto`

- **GitHub:** https://github.com/aidantwoods/go-paseto
- **Stars:** ~370
- **Last update:** Active 2024-2025; supports PASETO v3 and v4
- **What it replaces/improves:** Replaces `golang-jwt/jwt` (already referenced via `jwtauth` in the existing worklog). PASETO removes algorithm negotiation vulnerabilities by binding version to primitive (v4 = Ed25519 + XChaCha20-Poly1305). For AgilePlus agent tokens and heliosCLI session tokens this is a strict improvement.
- **Recommendation:** ADOPT for new token issuance. Migrate existing JWT issuance in a phased rollout.
- **LOC savings:** Net neutral on LOC; gain: eliminates `alg:none` attack surface.

---

### Go: CLI Frameworks

#### `github.com/charmbracelet/bubbletea` (Bubble Tea)

- **GitHub:** https://github.com/charmbracelet/bubbletea
- **Stars:** ~30k
- **Last update:** Active through Q1 2026, v1.x stable
- **What it replaces/improves:** Complements cobra (not replaces). thegent CLI uses cobra for command dispatch; Bubble Tea provides the interactive TUI layer for progress dashboards, agent activity monitors, and interactive pickers. Pattern: cobra routes commands, Bubble Tea renders interactive sub-flows.
- **Recommendation:** ADOPT for thegent interactive commands (install wizard, status dashboard, config editor).
- **LOC savings:** Replaces ~300-600 lines of hand-rolled ANSI output and readline loops.

---

### Go: Observability

The Go OTel SDK (`go.opentelemetry.io/otel`) reached stable logs API in 2025. Key packages for Phenotype:

| Package | Purpose | Status |
|---------|---------|--------|
| `go.opentelemetry.io/otel/trace` | Distributed tracing | Stable |
| `go.opentelemetry.io/otel/metric` | Metrics | Stable |
| `go.opentelemetry.io/otel/log` | Structured logs | Stable (2025) |
| `go.opentelemetry.io/contrib/.../otelchi` | Chi middleware | ADOPT |
| `go.opentelemetry.io/contrib/.../otelhttp` | net/http middleware | ADOPT |

- **Recommendation:** ADOPT full OTel stack (traces + metrics + logs) for AgilePlus and byteport services. The logs API stabilization in 2025 means the three-signal stack is now consistent.

---

### Rust: TUI

#### `ratatui` v0.30+

- **GitHub:** https://github.com/ratatui/ratatui
- **Stars:** ~19.4k
- **crates.io downloads:** 21.7M
- **Last update:** v0.30 shipped 2025 with modular workspace split; active Q1 2026
- **What it replaces/improves:** The canonical Rust TUI library. For thegent's Rust CLI components (if pheno-shell or heliosCLI gain interactive modes), ratatui provides immediate-mode rendering: charts, tables, progress bars, scrollable lists. Used by 2,100+ crates and trusted by Netflix, OpenAI, AWS, Vercel.
- **Recommendation:** ADOPT for any Rust CLI components in thegent or heliosCLI that need interactive dashboards.
- **LOC savings:** ~500-1000 lines vs hand-rolled ANSI rendering.

#### `tachyonfx` (ratatui effects/animations)

- **GitHub:** https://github.com/junkdog/tachyonfx
- **Stars:** ~700+
- **Last update:** Active 2025-2026, integrates with ratatui 0.30+
- **What it replaces/improves:** Adds smooth transitions, 50+ visual effects to ratatui apps. Optional polish layer for thegent CLI dashboards.
- **Recommendation:** EVALUATE — add after ratatui baseline is stable.

---

### Rust: Builder Patterns

#### `bon` crate

- **GitHub:** https://github.com/elastio/bon
- **Stars:** ~2k+
- **Last update:** Active 2025
- **What it replaces/improves:** Alternative to `typed-builder`. `bon` uses a step-by-step builder pattern with compile-time enforcement of required fields. Unlike `typed-builder` which encodes state in generics, `bon` generates a named-step API (`builder.field_name(value)`). Ergonomically preferred for large config structs in Phenotype CLI and SDK code.
- **Recommendation:** EVALUATE vs `typed-builder`. Use `bon` for new structs with >5 required fields; retain `typed-builder` for existing code.
- **LOC savings:** ~20-40 lines per large config struct vs hand-rolled builders.

---

### Rust: Validation

#### `garde` v0.22+

- **GitHub:** https://github.com/jprochazk/garde
- **Stars:** ~800
- **crates.io downloads:** 1.1M total, 327k recent
- **Last update:** v0.22.0 active 2025
- **What it replaces/improves:** Full rewrite of the `validator` crate. Derive-macro-based struct validation with rules like `#[garde(length(min=1), email)]`. More ergonomic than `validator` with better error messages. Directly applicable to AgilePlus spec/config validation and byteport payload validation.
- **Recommendation:** ADOPT for new Rust services. Migrate validator usages incrementally.
- **LOC savings:** ~30-60 lines of manual validation per struct replaced by derive macros.

---

### Agent/AI Infra: Official MCP SDKs

#### `modelcontextprotocol/go-sdk` (official)

- **GitHub:** https://github.com/modelcontextprotocol/go-sdk
- **Stars:** ~3k
- **Last update:** Active Q1 2026; maintained in collaboration with Google
- **What it replaces/improves:** Replaces `mark3labs/mcp-go` (community). The official Go MCP SDK provides server and client APIs, stdio and HTTP transports, OAuth primitives, and the full JSON-RPC layer. AgilePlus and byteport should migrate from community MCP wrappers to this once it reaches v1.0.
- **Recommendation:** EVALUATE now, ADOPT when v1.0 lands. Track milestone at github.com/modelcontextprotocol/go-sdk/releases.

#### `modelcontextprotocol/rust-sdk` (official, `rmcp` crate)

- **GitHub:** https://github.com/modelcontextprotocol/rust-sdk
- **Stars:** ~2.7k
- **Last update:** Active Q1 2026
- **What it replaces/improves:** Official Rust MCP SDK with tokio async runtime, proc-macros for server implementation, stdio transport. Replaces the community `mcp-sdk` crate already in the worklog. The official SDK is the forward path.
- **Recommendation:** ADOPT — migrate from community mcp-sdk to official rmcp crate.
- **LOC savings:** Net neutral on LOC; gain: reduced maintenance burden, official protocol compliance.

---

### Agent/AI Infra: Anthropic SDKs

#### `anthropic-sdk-rust` (tmikus, community)

- **GitHub:** https://github.com/tmikus/anthropic-sdk-rust
- **crates.io:** `anthropic-sdk-rust`
- **Last update:** Active 2025-2026
- **What it replaces/improves:** The most feature-complete community Rust SDK for the Anthropic API. Provides type-safe, async-first access (tokio), streaming responses, tool-use support. Supplements the `anthropic` crate already in the worklog.
- **Recommendation:** EVALUATE alongside existing `anthropic` crate. Pick one and standardize. `tmikus` SDK tracks the official API more closely.

#### `anthropic-agent-sdk` (Rust)

- **crates.io:** `anthropic-agent-sdk`
- **GitHub:** (community, based on anthropic-sdk-rust)
- **Last update:** 2025
- **What it replaces/improves:** Adds agent loop abstractions (tool dispatch, conversation state) on top of the base SDK. Relevant for any Rust agent runtime in the Phenotype SDK.
- **Recommendation:** EVALUATE for pheno-sdk Rust agent runtime.

---

### Agent/AI Infra: Agent Communication Protocols

#### AAIF / A2A v2 (Linux Foundation)

- **GitHub:** https://github.com/google-a2a/A2A (now Linux Foundation)
- **Stars:** ~7k+ (original Google repo)
- **Last update:** A2A moved to Linux Foundation AAIF in Dec 2025; IBM ACP merged into A2A Aug 2025
- **What it replaces/improves:** Extends the MCP+A2A landscape. Both MCP (Anthropic) and A2A (Google, now LF AAIF) are now co-stewarded under the Linux Foundation's Agentic AI Foundation alongside OpenAI, Microsoft, AWS, Block. MCP = tool/resource access; A2A = agent-to-agent task delegation. Together they cover the full agentic communication stack.
- **Recommendation:** EVALUATE A2A for multi-agent coordination in AgilePlus (e.g., planner-agent -> impl-agent handoff). MCP already covers tool access.
- **LOC savings:** N/A (protocol adoption, not library substitution).

#### Agent Commerce Protocol (ACP) - now merged into A2A

- Now part of A2A via the Linux Foundation merger (Aug 2025). No separate adoption needed.

---

### TypeScript: Routing and Full-Stack

#### `TanStack Start` + `TanStack Router` v1/v2

- **GitHub:** https://github.com/TanStack/router
- **Stars:** ~9k
- **Last update:** Active Q1 2026; TanStack Start in beta
- **What it replaces/improves:** End-to-end type-safe routing for AgilePlus dashboard and TraceRTM UI. Unlike Next.js App Router, TanStack Router generates a fully-typed route tree at build time — every route param, search param, and loader return type is known to TypeScript without manual annotation. Vite-based (faster DX than webpack Next.js). Full-stack via TanStack Start (Vinxi + Nitro).
- **Recommendation:** EVALUATE for greenfield UI work (new AgilePlus dashboard pages, TraceRTM). Existing Next.js pages: no forced migration.
- **LOC savings:** ~50-100 lines of manual type assertions per route eliminated.

#### `Hono` v4+

- **GitHub:** https://github.com/honojs/hono
- **Stars:** ~22k
- **Last update:** Active Q1 2026, v4.x
- **What it replaces/improves:** Ultra-lightweight TypeScript-first HTTP framework targeting edge runtimes (Cloudflare Workers, Bun, Deno, Node). For AgilePlus API routes or byteport edge functions, Hono provides first-class TypeScript inference for request/response types, built-in validation (zod integration), and zero-config multi-runtime deployment. Directly replaces Express in new TS services; ~2x faster than Fastify on Bun benchmarks.
- **Recommendation:** ADOPT for new TypeScript API services and edge workers.
- **LOC savings:** ~100-200 lines vs Express boilerplate per service; eliminates type assertion layers.

---

### TypeScript: Error Handling and Typed Effects

#### `Effect` (effect-ts) v3+

- **GitHub:** https://github.com/Effect-TS/effect
- **Stars:** ~9k
- **Last update:** Active Q1 2026, v3.x production-ready
- **What it replaces/improves:** Typed error channels for TypeScript. `Effect<A, E, R>` encodes success type, error type, and requirements in the signature — no more `try/catch` with `unknown`. Used in production by Vercel, Prisma, and fintech companies. For AgilePlus TypeScript backend, Effect eliminates the `any`-typed error problem and provides structured concurrency, dependency injection, and retry logic built-in.
- **Recommendation:** EVALUATE for AgilePlus TypeScript API layer. Steep learning curve; adopt in phases starting with error modeling.
- **LOC savings:** ~30-50% reduction in error handling boilerplate for complex async flows.

---

### TypeScript: OpenAPI Codegen

#### `@hey-api/openapi-ts`

- **GitHub:** https://github.com/hey-api/openapi-ts
- **Stars:** ~5k+
- **Last update:** Active Q1 2026
- **What it replaces/improves:** Spiritual successor to `openapi-typescript-codegen`, with a plugin-based architecture. Used by Vercel, OpenCode, and PayPal. Unlike orval (which generates hooks by default), hey-api uses an options-based pattern compatible with `queryClient.prefetchQuery`, `getQueryData`, and `useQueries` for SSR and cache reads — fully typed. Orval's differentiator remains built-in mock generation.
- **Recommendation:** ADOPT for new OpenAPI client generation. Retain orval where mock generation is needed. Migrate existing orval clients to hey-api incrementally.
- **LOC savings:** ~20-30% fewer generated files vs orval for non-mock workflows.

---

### TypeScript: MCP Server Frameworks

#### `mcp-framework` (QuantGeekDev)

- **GitHub:** https://github.com/QuantGeekDev/mcp-framework
- **Stars:** ~857 (growing rapidly)
- **Last update:** Active Q1 2026, v0.2.11+
- **What it replaces/improves:** Opinionated TypeScript framework for building MCP servers with automatic directory-based discovery of tools, resources, and prompts. Reduces boilerplate vs the bare `@modelcontextprotocol/sdk`. For thegent agent skills or AgilePlus MCP server, this provides the scaffolding layer.
- **Recommendation:** EVALUATE for any new TypeScript MCP server. Official SDK remains the foundation; mcp-framework adds DX conventions on top.

#### `@modelcontextprotocol/sdk` (official TypeScript)

- **GitHub:** https://github.com/modelcontextprotocol/typescript-sdk
- **npm:** `@modelcontextprotocol/sdk`
- **Stars:** ~7k+
- **Last update:** Active Q1 2026; v2 release anticipated Q1 2026
- **What it replaces/improves:** Already in the worklog as `mcp-sdk`. Noting that v2 introduces stable Streamable HTTP transport, auth helpers, and improved client APIs. Upgrade path from v1 is non-breaking for server implementations.
- **Recommendation:** ADOPT v2 when released. Watch releases at github.com/modelcontextprotocol/typescript-sdk/releases.

---

### Cross-cutting: Proto/RPC Schema Management

#### `buf` CLI + Buf Schema Registry (BSR)

- **GitHub:** https://github.com/bufbuild/buf
- **Stars:** ~10k
- **Last update:** Active Q1 2026
- **What it replaces/improves:** Replaces raw `protoc` + hand-written Makefiles for proto compilation. Provides: schema linting (`buf lint`), breaking change detection (`buf breaking`), code generation via `buf generate` (with connect-go and connect-es plugins), and a centralized schema registry (BSR). If byteport or AgilePlus adopt gRPC/connect service contracts, buf is the required toolchain.
- **Recommendation:** ADOPT alongside connect-go and connect-es.

#### `@connectrpc/connect-es` v2

- **GitHub:** https://github.com/connectrpc/connect-es (via bufbuild)
- **Stars:** ~1.5k
- **npm:** `@connectrpc/connect-es`
- **Last update:** v2.0 GA in 2025; uses Protobuf-ES 2.0 reflection APIs
- **What it replaces/improves:** Replaces grpc-web for browser-to-backend RPC in TypeScript. Connect-ES v2 works with fetch, supports streaming, and integrates with TanStack Query. Not backward compatible with v1. For AgilePlus dashboard communicating with Go services via proto contracts.
- **Recommendation:** ADOPT for new TypeScript clients against connect-go services.

---

### Summary Table

| Package | Lang | Stars | Recommendation | Primary Phenotype Target |
|---------|------|-------|----------------|--------------------------|
| otelchi | Go | ~320 | ADOPT | AgilePlus, byteport |
| Go stdlib ServeMux (1.22+) | Go | N/A | EVALUATE | Leaf services |
| connect-go v1.x | Go | ~5.7k | ADOPT | byteport, AgilePlus |
| buf CLI | Go/multi | ~10k | ADOPT | byteport proto contracts |
| go-paseto (aidantwoods) | Go | ~370 | ADOPT | AgilePlus agent tokens |
| bubbletea | Go | ~30k | ADOPT | thegent CLI |
| ratatui v0.30+ | Rust | ~19.4k | ADOPT | thegent, heliosCLI |
| tachyonfx | Rust | ~700 | EVALUATE | thegent (polish) |
| bon | Rust | ~2k | EVALUATE | pheno-sdk config structs |
| garde v0.22+ | Rust | ~800 | ADOPT | byteport, AgilePlus Rust |
| mcp go-sdk (official) | Go | ~3k | EVALUATE->ADOPT | AgilePlus MCP server |
| mcp rust-sdk/rmcp (official) | Rust | ~2.7k | ADOPT | pheno-sdk |
| anthropic-sdk-rust (tmikus) | Rust | ~200 | EVALUATE | pheno-sdk agent runtime |
| AAIF / A2A v2 (LF) | Protocol | ~7k | EVALUATE | AgilePlus multi-agent |
| TanStack Start/Router | TypeScript | ~9k | EVALUATE | AgilePlus dashboard |
| Hono v4+ | TypeScript | ~22k | ADOPT | AgilePlus API, byteport edge |
| Effect v3+ | TypeScript | ~9k | EVALUATE | AgilePlus TS backend |
| hey-api/openapi-ts | TypeScript | ~5k | ADOPT | all TS services |
| mcp-framework (QuantGeekDev) | TypeScript | ~857 | EVALUATE | thegent skill server |
| connect-es v2 | TypeScript | ~1.5k | ADOPT | AgilePlus dashboard RPC |
| OTel Go logs API (stable 2025) | Go | N/A | ADOPT | all Go services |
