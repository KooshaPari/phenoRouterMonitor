# Small Repo Census

Date: 2026-04-02

## Scope

- GitHub owner checked: `KooshaPari`
- Owned GitHub repos discovered via authenticated `gh api`: `178`
- Local git repos measured in this shelf: `60`
- Local shelf repos under `10,000` code LOC: `43`

## Method

- LOC was measured with `scc`.
- This pass counts code-like languages and excludes prose/data-heavy languages such as Markdown, JSON, YAML, TOML, plain text, CSV, XML, HTML, CSS, and lock/build/vendor directories.
- “Published on owned GitHub” means the local repo name appeared in the authenticated owned-repo export from GitHub.
- “No-or-unverified” means the repo exists locally but was not present in the owned GitHub export at measurement time.

## Verified Small Repos In Shelf

| Repo | Code LOC | Published on owned GitHub | Top languages |
|---|---:|---|---|
| Hexacore | 0 | yes | none |
| Httpora | 0 | yes | none |
| phenotype-xdd | 0 | yes | none |
| thegent-sharecli | 0 | yes | none |
| Zerokit | 0 | yes | none |
| Quillr | 1 | yes | TypeScript 1 |
| phenotype-cipher | 10 | yes | Rust 10 |
| phenotype-forge | 26 | yes | Rust 26 |
| helMo | 71 | yes | Shell 41, JavaScript 20, TypeScript 8 |
| helix-logging | 75 | yes | Rust 75 |
| forgecode | 90 | yes | JavaScript 42, Shell 26, TypeScript 22 |
| Apisync | 111 | yes | Rust 111 |
| template-lang-typescript | 116 | yes | Shell 49, TypeScript 36, JavaScript 29 |
| phenotypeActions | 229 | yes | Shell 200, JavaScript 22, TypeScript 5 |
| tracely | 248 | yes | Rust 248 |
| heliosApp | 284 | yes | TypeScript 250, Shell 34 |
| phenotype-design | 320 | yes | TypeScript 235, JavaScript 42, Shell 41 |
| agileplus-plugin-core | 351 | yes | Rust 351 |
| phenotype-nexus | 363 | yes | Rust 193, JavaScript 85, TypeScript 85 |
| devenv-abstraction | 374 | no-or-unverified | Go 374 |
| agileplus-plugin-git | 414 | yes | Rust 414 |
| agent-wave | 426 | yes | Shell 323, TypeScript 83, JavaScript 20 |
| agileplus-plugin-sqlite | 430 | yes | Rust 430 |
| thegent-plugin-host | 590 | yes | Rust 590 |
| worktree-manager | 614 | yes | Rust 614 |
| Agentora | 782 | yes | Rust 782 |
| HexaType | 944 | yes | TypeScript 919, JavaScript 25 |
| thegent-shm | 952 | yes | Rust 952 |
| thegent-metrics | 1029 | yes | Rust 1029 |
| thegent-subprocess | 1031 | yes | Rust 1031 |
| phenotype-xdd-lib | 1112 | yes | Rust 978, JavaScript 92, TypeScript 42 |
| HexaPy | 1115 | yes | Python 1115 |
| phenotype-gauge | 1138 | yes | Rust 994, TypeScript 100, JavaScript 44 |
| thegent-cache | 1157 | yes | Rust 1157 |
| Authvault | 1197 | yes | Rust 1197 |
| HexaGo | 1468 | yes | Go 1404, TypeScript 36, JavaScript 28 |
| Cmdra | 1546 | yes | Rust 1546 |
| thegent-mesh | 2752 | yes | Python 2366, Shell 386 |
| Profila | 3394 | yes | Python 2736, Shell 637, TypeScript 21 |
| agent-devops-setups | 5966 | yes | Patch 4668, Shell 651, Python 547 |
| template-commons | 6331 | no-or-unverified | Go 1700, Python 1559, Shell 1304 |
| phenotype-shared | 7348 | yes | Rust 6373, TypeScript 940, Shell 23 |
| AgentMCP | 8139 | yes | Python 6895, Shell 504, Go 401 |

## Unpublished Local Repos Under 10k

- `claude-api` | `0` | `none` | `/Users/kooshapari/CodeProjects/archive/ai-agents/claude-experiments/claude-api`
- `ch2` | `10` | `Rust 10` | `/Users/kooshapari/CodeProjects/archive/Rust/webApp/test-rust/ch2`
- `hw4` | `3162` | `Java 3017, FXML 145` | `/Users/kooshapari/CodeProjects/learning/courses/prior/360/hw4`

## Likely Foundational Small Repos

These are the small repos most likely to sit underneath the larger projects by name and role:

- `phenotype-shared` | `7348`
- `AgentMCP` | `8139`
- `template-commons` | `6331`
- `agileplus-plugin-core` | `351`
- `agileplus-plugin-git` | `414`
- `agileplus-plugin-sqlite` | `430`
- `worktree-manager` | `614`
- `thegent-cache` | `1157`
- `thegent-plugin-host` | `590`
- `thegent-shm` | `952`
- `thegent-subprocess` | `1031`
- `thegent-metrics` | `1029`
- `phenotype-xdd-lib` | `1112`
- `devenv-abstraction` | `374`
- `phenotype-gauge` | `1138`
- `phenotypeActions` | `229`
- `helix-logging` | `75`

## Not Yet Measured

- Owned GitHub repos not cloned in this shelf: `118`
- Those are not LOC-verified in this pass.
- If needed, next pass should clone or shallow-fetch the smallest owned-but-not-local repos first and add them to this census.
