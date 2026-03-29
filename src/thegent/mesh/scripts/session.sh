#!/usr/bin/env bash
# =============================================================================
# lib/mesh/session.sh — tmux Session Management for Agents
# =============================================================================

MESH_SESSION_PREFIX="${MESH_SESSION_PREFIX:-mesh-}"

mesh_session::create() {
    local agent_id="$1"
    local session_name="${MESH_SESSION_PREFIX}${agent_id}"

    if tmux has-session -t "$session_name" 2>/dev/null; then
        # Session already exists
        return 0
    fi

    tmux new-session -d -s "$session_name"
    return $?
}

mesh_session::exists() {
    local agent_id="$1"
    local session_name="${MESH_SESSION_PREFIX}${agent_id}"
    tmux has-session -t "$session_name" 2>/dev/null
}

mesh_session::kill() {
    local agent_id="$1"
    local session_name="${MESH_SESSION_PREFIX}${agent_id}"
    tmux kill-session -t "$session_name" 2>/dev/null
}

mesh_session::list() {
    tmux list-sessions -F '#{session_name}' 2>/dev/null | grep "^${MESH_SESSION_PREFIX}" || true
}

mesh_session::get_name() {
    local agent_id="$1"
    echo "${MESH_SESSION_PREFIX}${agent_id}"
}
