#!/usr/bin/env bash
# =============================================================================
# lib/mesh/scanner.sh — Process Scanner for Agent Discovery
# =============================================================================

MESH_HOME="${MESH_HOME:-/tmp/agent-mesh}"
MESH_SCANNER_INTERVAL="${MESH_SCANNER_INTERVAL:-5}"
MESH_HEARTBEAT_THRESHOLD="${MESH_HEARTBEAT_THRESHOLD:-15}"

# Agent patterns (substring matches in cmdline)
declare -gA MESH_AGENT_PATTERNS=(
    ["claude-code"]="claude"
    ["aider"]="aider"
    ["cursor"]="cursor"
    ["cline"]="cline"
    ["devin"]="devin"
    ["continue"]="continue"
)

# =============================================================================
# Process Detection
# =============================================================================

mesh_scanner::detect_agent_type() {
    local cmdline="$1"
    for type in "${!MESH_AGENT_PATTERNS[@]}"; do
        local pattern="${MESH_AGENT_PATTERNS[$type]}"
        if [[ "$cmdline" == *"$pattern"* ]]; then
            echo "$type"
            return 0
        fi
    done

    # Generic check for Python agents
    if [[ "$cmdline" == *"python"* ]] && [[ "$cmdline" == *"agent"* ]]; then
        echo "python-agent"
        return 0
    fi
    return 1
}

mesh_scanner::get_raw_processes() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS: ps -Ao pid,args (skip header)
        ps -Ao pid,args | sed 1d
    else
        # Linux: loop over /proc
        for proc_dir in /proc/[0-9]*; do
            [[ -d "$proc_dir" ]] || continue
            local pid=$(basename "$proc_dir")
            local cmdline=$(tr '\0' ' ' < "$proc_dir/cmdline" 2>/dev/null)
            # Skip kernel threads and self
            [[ -z "$cmdline" || "$cmdline" == *"["* ]] && continue
            echo "$pid $cmdline"
        done
    fi
}

mesh_scanner::scan_all() {
    while read -r pid args; do
        [[ -z "$pid" || -z "$args" ]] && continue
        local agent_type
        agent_type=$(mesh_scanner::detect_agent_type "$args")
        if [[ -n "$agent_type" ]]; then
            echo "$pid:$agent_type"
        fi
    done < <(mesh_scanner::get_raw_processes)
}

mesh_scanner::scan() {
    echo "Scanning for agents..."
    local agents=()
    mapfile -t agents < <(mesh_scanner::scan_all)

    if [[ ${#agents[@]} -eq 0 || -z "${agents[0]}" ]]; then
        echo "  No agents found"
        return 1
    fi

    echo "  Found ${#agents[@]} agent(s):"
    for agent in "${agents[@]}"; do
        local pid="${agent%%:*}"
        local type="${agent#*:}"
        echo "    - PID $pid: $type"
    done
}

# =============================================================================
# Agent Details
# =============================================================================

mesh_scanner::get_workdir() {
    local pid="$1"
    if [[ "$OSTYPE" == "darwin"* ]]; then
        lsof -p "$pid" -a -d cwd -Fn 2>/dev/null | sed -n 's/^n//p' || echo "unknown"
    else
        readlink "/proc/$pid/cwd" 2>/dev/null || echo "unknown"
    fi
}

mesh_scanner::get_env() {
    local pid="$1"
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # On macOS, getting env of another process is restricted without special privileges
        echo "OS=darwin"
    else
        if [[ -f "/proc/$pid/environ" ]]; then
            # Extract relevant environment variables
            tr '\0' '\n' < "/proc/$pid/environ" 2>/dev/null | grep -E "^(HOME|PATH|AGENT_|MESH_|CLAUDE_|AIDER_)" || true
        fi
    fi
}

mesh_scanner::generate_manifest() {
    local pid="$1"
    local agent_type="$2"
    local workdir=$(mesh_scanner::get_workdir "$pid")

    local manifest_file="$MESH_HOME/agents/agent-${pid}.yaml"
    mkdir -p "$MESH_HOME/agents"

    {
        echo "id: agent-${pid}"
        echo "type: ${agent_type}"
        echo "pid: ${pid}"
        echo "started_at: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
        echo "working_directory: ${workdir}"
        echo "status: running"
    } > "$manifest_file"

    echo "$manifest_file"
}

# =============================================================================
# Heartbeat Monitoring
# =============================================================================

mesh_scanner::touch_heartbeat() {
    local pid="$1"
    local heartbeat_file="$MESH_HOME/heartbeats/agent-${pid}"
    mkdir -p "$MESH_HOME/heartbeats"
    touch "$heartbeat_file"
}

mesh_scanner::check_heartbeats() {
    local now=$(date +%s)
    local stale=()

    for hb in "$MESH_HOME"/heartbeats/agent-*; do
        [[ -f "$hb" ]] || continue

        local mtime
        if [[ "$OSTYPE" == "darwin"* ]]; then
            mtime=$(stat -f %m "$hb" 2>/dev/null || echo 0)
        else
            mtime=$(stat -c %Y "$hb" 2>/dev/null || echo 0)
        fi

        local age=$((now - mtime))
        if [[ "$age" -gt "$MESH_HEARTBEAT_THRESHOLD" ]]; then
            local pid=$(basename "$hb" | sed 's/agent-//')
            stale+=("$pid")
        fi
    done

    if [[ ${#stale[@]} -gt 0 ]]; then
        echo "Stale agents: ${stale[*]}"
        return 1
    fi

    echo "All heartbeats fresh"
    return 0
}

mesh_scanner::cleanup_stale() {
    local stale_threshold=$((MESH_HEARTBEAT_THRESHOLD * 2))
    local now=$(date +%s)
    local cleaned=0

    for hb in "$MESH_HOME"/heartbeats/agent-*; do
        [[ -f "$hb" ]] || continue

        local mtime
        if [[ "$OSTYPE" == "darwin"* ]]; then
            mtime=$(stat -f %m "$hb" 2>/dev/null || echo 0)
        else
            mtime=$(stat -c %Y "$hb" 2>/dev/null || echo 0)
        fi

        local age=$((now - mtime))
        if [[ "$age" -gt "$stale_threshold" ]]; then
            local pid=$(basename "$hb" | sed 's/agent-//')
            rm -f "$hb"
            rm -f "$MESH_HOME/agents/agent-${pid}.yaml"
            cleaned=$((cleaned + 1))
        fi
    done

    echo "Cleaned $stale_threshold+ second stale agents: $cleaned"
}

# =============================================================================
# Daemon Mode
# =============================================================================

mesh_scanner::daemon() {
    echo "Starting scanner daemon (interval: ${MESH_SCANNER_INTERVAL}s)..."

    while true; do
        while read -r agent; do
            [[ -z "$agent" ]] && continue
            local pid="${agent%%:*}"
            local type="${agent#*:}"

            # Check if already registered
            if [[ ! -f "$MESH_HOME/agents/agent-${pid}.yaml" ]]; then
                echo "[$(date '+%H:%M:%S')] Detected new agent: $type (PID $pid)"
                mesh_scanner::generate_manifest "$pid" "$type"
            fi

            # Touch heartbeat
            mesh_scanner::touch_heartbeat "$pid"
        done < <(mesh_scanner::scan_all)

        # Check for stale agents
        mesh_scanner::check_heartbeats >/dev/null 2>&1 || true

        sleep "$MESH_SCANNER_INTERVAL"
    done
}

# =============================================================================
# CLI Helpers
# =============================================================================

mesh_scanner::list() {
    echo "Registered agents:"

    for manifest in "$MESH_HOME"/agents/agent-*.yaml; do
        [[ -f "$manifest" ]] || continue

        local pid=$(basename "$manifest" .yaml | sed 's/agent-//')
        local type=$(grep '^type: ' "$manifest" | cut -d: -f2 | tr -d ' ')
        local status=$(grep '^status: ' "$manifest" | cut -d: -f2 | tr -d ' ')

        echo "  - PID $pid: $type ($status)"
    done

    if [[ ! -d "$MESH_HOME/agents" ]] || [[ -z "$(ls -A "$MESH_HOME/agents" 2>/dev/null)" ]]; then
        echo "  (no agents registered)"
    fi
}

mesh_scanner::status() {
    echo "Scanner Status:"
    echo "  Home: $MESH_HOME"
    echo "  Scan interval: ${MESH_SCANNER_INTERVAL}s"
    echo "  Heartbeat threshold: ${MESH_HEARTBEAT_THRESHOLD}s"
    echo ""

    mesh_scanner::list
    echo ""

    echo "Heartbeat Status:"
    local total fresh
    total=$(find "$MESH_HOME/heartbeats" -name "agent-*" -type f 2>/dev/null | wc -l)
    fresh=$(mesh_scanner::check_heartbeats 2>&1 | grep -c "fresh" || echo 0)

    echo "  Total: $total"
    echo "  Fresh: $fresh"
}

mesh_scanner::register_all() {
    echo "Registering all detected agents..."
    while read -r agent; do
        [[ -z "$agent" ]] && continue
        local pid="${agent%%:*}"
        local type="${agent#*:}"
        echo "  - Registering $type (PID $pid)..."
        mesh_scanner::generate_manifest "$pid" "$type"
        mesh_scanner::touch_heartbeat "$pid"
    done < <(mesh_scanner::scan_all)
}

# =============================================================================
# Main Entry Point
# =============================================================================

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    case "${1:-scan}" in
        scan)
            mesh_scanner::scan
            ;;
        register)
            mesh_scanner::register_all
            ;;
        list)
            mesh_scanner::list
            ;;
        status)
            mesh_scanner::status
            ;;
        daemon)
            mesh_scanner::daemon
            ;;
        check-heartbeats)
            mesh_scanner::check_heartbeats
            ;;
        cleanup)
            mesh_scanner::cleanup_stale
            ;;
        generate_manifest)
            mesh_scanner::generate_manifest "$2" "$3"
            ;;
        *)
            echo "Usage: $0 [scan|list|status|daemon|check-heartbeats|cleanup|generate_manifest]"
            exit 1
            ;;
    esac
fi
