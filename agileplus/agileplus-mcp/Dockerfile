FROM python:3.12-slim

# Install uv
COPY --from=ghcr.io/astral-sh/uv:latest /uv /uvx /usr/local/bin/

WORKDIR /app

# Copy dependency files first for layer caching
COPY pyproject.toml .python-version ./

# Install dependencies (no dev extras in production image)
RUN uv sync --no-dev

# Copy source code
COPY src/ ./src/

# Run the MCP server
CMD ["uv", "run", "python", "-m", "agileplus_mcp"]
