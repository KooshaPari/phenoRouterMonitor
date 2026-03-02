.PHONY: help install install-dev sync test lint typecheck docs clean

PYTHON := uv run python
UV := uv

help:
	@echo "AgilePlus MCP Service - Available targets:"
	@echo "  install      Install production dependencies"
	@echo "  install-dev  Install all dependencies including dev extras"
	@echo "  sync         Sync dependencies from lockfile"
	@echo "  test         Run pytest test suite"
	@echo "  lint         Run ruff linter"
	@echo "  typecheck    Run mypy type checker"
	@echo "  docs         Build Sphinx HTML API reference"
	@echo "  clean        Remove build artifacts"

install:
	$(UV) sync --no-dev

install-dev:
	$(UV) sync --all-extras

sync:
	$(UV) sync

test:
	$(UV) run pytest tests/ -v --cov=src/agileplus_mcp --cov-report=term-missing

lint:
	$(UV) run ruff check src/ tests/

typecheck:
	$(UV) run mypy src/

docs:
	$(UV) run sphinx-build -b html docs/ docs/_build/html
	@echo "Docs built at docs/_build/html/index.html"

clean:
	rm -rf docs/_build
	rm -rf .pytest_cache
	rm -rf htmlcov
	rm -rf .coverage
	find . -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null || true
	find . -type f -name "*.pyc" -delete 2>/dev/null || true
