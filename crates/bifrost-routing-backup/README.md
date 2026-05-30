# bifrost-routing

> **DEPRECATED / SUPERSEDED**
>
> This directory contains an early prototype of the Rust routing abstraction layer.
> It was never added to the workspace (no `Cargo.toml`).
>
> **Canonical routing now lives in two places:**
> - **LLM proxy / runtime routing**: [OmniRoute](https://github.com/KooshaPari/OmniRoute) — TypeScript, 14 strategies, 160+ providers.
> - **Rust routing adapter + Pareto scoring**: [`tokenledger::routing`](https://github.com/KooshaPari/Tokn) (`crates/tokenledger/src/routing/`).
>
> **Pareto dashboard**: [helios-router](https://github.com/KooshaPari/helios-router) — Streamlit app for provider/model selection analysis.
>
> Do not add new routing logic here. Contribute to OmniRoute (JS) or Tokn (Rust) instead.
