# Helios Router - Service Management
# Uses process-compose for native service orchestration

.PHONY: dev dev-tui status restart clean nats-status nats-logs

# Services
NATS=nats-server
STREAMLIT=streamlit
PYTHON=python3

# Paths
APP_DIR:=$(shell pwd)
LOGS_DIR:=$(APP_DIR)/.process-compose/logs
LOCKS_DIR:=$(APP_DIR)/.process-compose/locks

# Check if running on macOS
IS_DARWIN:=$(shell uname -s == Darwin)

# Preflight checks
preflight:
	@echo "[PREFLIGHT] Checking dependencies..."
	@which $(NATS) >/dev/null 2>&1 || { echo "[PREFLIGHT] ERROR: nats-server not found. Install with: brew install nats-server"; exit 1; }
	@which $(PYTHON) >/dev/null 2>&1 || { echo "[PREFLIGHT] ERROR: python3 not found"; exit 1; }
	@echo "[PREFLIGHT] OK"

# Start NATS (homebrew service)
start-nats:
	@echo "[LIFECYCLE] START nats-server $$(date)"
	@if pgrep -f "nats-server" > /dev/null; then \
		echo "[INFO] nats-server already running"; \
	else \
		nats-server -c /dev/null & \
		echo "$$!" > $(LOCKS_DIR)/nats.pid; \
	fi
	@sleep 1
	@nats-server -v || echo "[INFO] NATS started"

# Stop NATS
stop-nats:
	@if [ -f $(LOCKS_DIR)/nats.pid ]; then \
		pid=$$(cat $(LOCKS_DIR)/nats.pid); \
		kill $$pid 2>/dev/null || true; \
		rm -f $(LOCKS_DIR)/nats.pid; \
	fi
	@pgrep -f "nats-server" | xargs kill 2>/dev/null || true
	@echo "[LIFECYCLE] STOP nats-server"

# NATS status
nats-status:
	@nats-server -v 2>/dev/null && echo "NATS: installed" || echo "NATS: not installed"
	@pgrep -f "nats-server" > /dev/null && echo "NATS: running" || echo "NATS: not running"
	@nats context list 2>/dev/null || true

# NATS logs
nats-logs:
	@if pgrep -f "nats-server" > /dev/null; then \
		nats server report jetstream; \
	else \
		echo "NATS not running"; \
	fi

# Start Streamlit
start-streamlit:
	@echo "[LIFECYCLE] START streamlit $$(date)"
	@cd $(APP_DIR) && \
		PYTHONPATH=$(APP_DIR)/src:$$PYTHONPATH \
		$(PYTHON) -m streamlit run app.py --server.port 8501 & \
		echo "$$!" > $(LOCKS_DIR)/streamlit.pid
	@echo "Streamlit started on http://localhost:8501"

# Stop Streamlit
stop-streamlit:
	@if [ -f $(LOCKS_DIR)/streamlit.pid ]; then \
		pid=$$(cat $(LOCKS_DIR)/streamlit.pid); \
		kill $$pid 2>/dev/null || true; \
		rm -f $(LOCKS_DIR)/streamlit.pid; \
	fi
	@pgrep -f "streamlit run" | xargs kill 2>/dev/null || true
	@echo "[LIFECYCLE] STOP streamlit"

# Start all services
dev: preflight start-nats
	@echo "[INFO] Waiting for NATS..."
	@sleep 2
	@$(MAKE) start-streamlit
	@echo ""
	@echo "[DEV] Services started:"
	@echo "  - NATS: nats://localhost:4222"
	@echo "  - Streamlit: http://localhost:8501"
	@echo "  - Logs: $(LOGS_DIR)/"

# Dev with TUI (tail -f logs)
dev-tui: dev
	@echo "[INFO] Tailing logs... (Ctrl+C to exit)"
	@tail -f $(LOGS_DIR)/*.log

# Status
status:
	@echo "=== Helios Router Status ==="
	@$(MAKE) nats-status
	@echo ""
	@echo "Streamlit:"
	@pgrep -f "streamlit" > /dev/null && echo "  Running" || echo "  Not running"

# Stop all
stop: stop-streamlit stop-nats
	@echo "[LIFECYCLE] All services stopped"

# Restart service
restart:
	@$(MAKE) stop
	@$(MAKE) dev

# Clean logs
clean:
	@rm -f $(LOGS_DIR)/*.log
	@echo "[CLEAN] Logs cleared"

# Health check
health:
	@echo "=== Health Check ==="
	@nc -z localhost 4222 && echo "NATS: healthy" || echo "NATS: unhealthy"
	@curl -s http://localhost:8501/_stcore/health 2>/dev/null | grep -q "ok" && echo "Streamlit: healthy" || echo "Streamlit: not responding"

# Install dependencies
install:
	@echo "[INSTALL] Installing dependencies..."
	@which brew >/dev/null 2>&1 && brew install nats-server 2>/dev/null || echo "[WARN] Homebrew not found"
	@cd $(APP_DIR) && uv sync 2>/dev/null || pip install -r requirements.txt 2>/dev/null || true

help:
	@echo "Helios Router Makefile"
	@echo ""
	@echo "  make dev          - Start all services"
	@echo "  make dev-tui      - Start with log tailing"
	@echo "  make status       - Check service status"
	@echo "  make restart      - Restart all services"
	@echo "  make stop         - Stop all services"
	@echo "  make health       - Health check"
	@echo "  make install      - Install dependencies"
