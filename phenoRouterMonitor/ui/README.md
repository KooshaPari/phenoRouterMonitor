# Helios Router UI

Interactive Pareto analysis dashboard for LLM provider selection and ledger management.

## Features

- **Pareto Analysis** - Compute optimal provider/model combinations based on cost, latency, and quality
- **Provider Ledger** - Track and visualize token usage across multiple providers
- **Streamlit UI** - Interactive dashboard with data editors, weight sliders, and metrics
- **NATS Integration** - Real-time event streaming and KV caching
- **Export** - Export to Excel/spreadsheet formats

## Quick Start

```bash
# Install dependencies
task install

# Start Streamlit dev server
task dev

# Or use make
make dev
```

## Installation

### Requirements

- Python 3.12+
- [uv](https://github.com/astral-sh/uv) package manager
- NATS server (optional, for event streaming)

### Install NATS

```bash
# macOS
brew install nats-server

# Linux
go install github.com/nats-io/nats-server/v2@latest
```

## Usage

### Development

```bash
# Start Streamlit with hot reload
task dev

# Run with custom port
STREAMIT_PORT=8502 task dev
```

### Database

```bash
# Initialize database
task db-migrate

# Load sample data
task sample-data

# Reset database
task db-reset
```

### Testing

```bash
# Run tests
task test

# With coverage
task test-cov
```

### Code Quality

```bash
# Lint
task lint

# Format
task format

# Type check
task typecheck

# All checks
task check
```

## Service Management

### Makefile

```bash
make dev          # Start all services
make dev-tui      # Start with log tailing
make status       # Check status
make stop         # Stop all
make restart      # Restart
make health       # Health check
```

### Shell Script

```bash
source scripts/services.sh
start          # Start all
stop           # Stop all  
status         # Status
logs           # Tail logs
```

### Process Compose

```bash
# If you have process-compose installed
process-compose up -f process-compose.yaml
```

## Architecture

```
helios_router_data/
├── app.py                    # Main Streamlit application
├── sample_data.py            # Sample data loader
├── helios.db                # SQLite database
├── src/helios_router_ui/
│   ├── __init__.py
│   ├── db/
│   │   └── schema.py        # Database layer
│   ├── pareto/
│   │   └── engine.py       # Pareto computation engine
│   ├── ui/
│   │   └── components.py    # Streamlit UI components
│   └── nats_client.py       # NATS JetStream client
└── migrations/             # Database migrations
```

## Services

| Service | Port | Description |
|---------|------|-------------|
| NATS | 4222 | Event bus + KV cache |
| Streamlit | 8501 | Dashboard UI |
| NATS Monitor | 8222 | Optional monitoring |

## NATS KV Stores

- `provider_cache` - Provider status (5min TTL)
- `model_cache` - Model data (1hr TTL)
- `pareto_cache` - Computed frontiers (1min TTL)
- `price_cache` - Real-time pricing (1min TTL)

## Configuration

Copy `.env.example` to `.env` and configure:

```bash
cp .env.example .env
```

## Dependencies

### Core

- streamlit >= 1.40
- pandas >= 2.0
- numpy >= 1.24
- openpyxl >= 3.1

### Optional

- nats >= 0.24 (for NATS integration)
- asyncio-mqtt >= 0.16 (for MQTT bridge)

### Dev

- ruff >= 0.3
- mypy >= 1.19
- pytest >= 8.0
- pytest-cov >= 6.0

## Available Tasks

Run `task --list` to see all available tasks:

```
task install          Install dependencies with uv
task dev             Start Streamlit dev server
task lint             Run ruff linter
task format           Format code with ruff
task typecheck        Run mypy type checker
task test             Run pytest
task check            Run all checks (lint, typecheck, test)
task clean            Clean up cache files
```

## License

MIT
