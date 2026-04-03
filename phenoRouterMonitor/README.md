# phenoRouterMonitor

LLM router with monitoring backend (Rust) and Pareto analysis dashboard (Streamlit).

## Components

### Backend (Rust)
- **Location**: `src/`
- **Purpose**: Core routing logic, metrics collection, monitoring

### UI (Python/Streamlit)
- **Location**: `ui/`
- **Purpose**: Interactive Pareto dashboard for LLM provider selection and ledger management
- **Features**:
  - **Pareto Analysis** - Compute optimal provider/model combinations based on cost, latency, quality
  - **Provider Ledger** - Track and visualize token usage across multiple providers
  - **Streamlit UI** - Interactive dashboard

## Usage

### Start Backend
```bash
cargo run
```

### Start UI
```bash
cd ui && streamlit run app.py
```

## Merged Repositories

- `helios-router` (Streamlit dashboard) → `ui/`
- `phenoRouterMonitor` (Rust backend) → `src/`

Unified into single LLM router platform.
