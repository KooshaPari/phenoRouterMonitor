#!/usr/bin/env bash
# =============================================================================
# lib/mesh/inject.sh — Reliable tmux Command Injection
# =============================================================================

MESH_INJECT_DELAY="${MESH_INJECT_DELAY:-1.5}"

mesh_inject::command() {
    local target="$1" # session:window.pane or session
    local command="$2"
    local delay="${3:-$MESH_INJECT_DELAY}"

    # Send keys literally (-l) to avoid tmux interpretation of special chars
    tmux send-keys -t "$target" -l "$command"

    # Critical delay - prevents command loss in rapid fire or slow agents
    sleep "$delay"

    # Send Enter to execute
    tmux send-keys -t "$target" Enter
}

mesh_inject::wait_for_ready() {
    local target="$1"
    local pattern="$2"
    local timeout="${3:-120}"
    local interval="${4:-1}"
    local elapsed=0

    while [ "$elapsed" -lt "$timeout" ]; do
        local output
        output=$(tmux capture-pane -p -t "$target" -S -10 2>/dev/null)
        if echo "$output" | grep -qE "$pattern"; then
            return 0
        fi
        sleep "$interval"
        elapsed=$((elapsed + interval))
    done
    return 1 # Timeout
}

# Common agent readiness patterns
declare -gA MESH_READY_PATTERNS=(
    ["aider"]="^aider>|^───"
    ["claude-code"]="^>|claude>"
    ["shell"]="\\$|%|#|➜|❯|>"
)

mesh_inject::get_ready_pattern() {
    local agent_type="$1"
    echo "${MESH_READY_PATTERNS[$agent_type]:-\$|%|#|➜|❯|>}"
}

mesh_inject::wait_for_agent() {
    local target="$1"
    local agent_type="$2"
    local timeout="${3:-120}"

    local pattern
    pattern=$(mesh_inject::get_ready_pattern "$agent_type")

    mesh_inject::wait_for_ready "$target" "$pattern" "$timeout"
}
