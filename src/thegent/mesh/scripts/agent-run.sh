#!/usr/bin/env bash
# =============================================================================
# lib/mesh/agent-run.sh — Start Agent in Managed Mesh Session
# =============================================================================

# Source library dependencies
LIB_DIR=$(dirname "${BASH_SOURCE[0]}")
source "$LIB_DIR/session.sh"
source "$LIB_DIR/inject.sh"
source "$LIB_DIR/scanner.sh"

MESH_AGENT_TYPE="${1:-claude-code}"
MESH_AGENT_PROMPT="${2:-}"
MESH_AGENT_WORKDIR="${3:-$(pwd)}"

# 1. Create session
# Use a short UUID for session identification
SESSION_ID=$(uuidgen | tr '[:upper:]' '[:lower:]' | cut -d- -f1)
SESSION_NAME=$(mesh_session::get_name "$SESSION_ID")

echo "Starting agent $MESH_AGENT_TYPE in session $SESSION_NAME..."

# Create new tmux session in specified working directory
tmux new-session -d -s "$SESSION_NAME" -c "$MESH_AGENT_WORKDIR"

# 2. Wait for shell ready
if ! mesh_inject::wait_for_agent "$SESSION_NAME" "shell" 30; then
    echo "Timeout waiting for shell in $SESSION_NAME"
    exit 1
fi

# 3. Start agent
# Command mapping for common agents
declare -gA MESH_AGENT_COMMANDS=(
    ["claude-code"]="claude"
    ["aider"]="aider"
    ["cursor"]="cursor"
)
CMD="${MESH_AGENT_COMMANDS[$MESH_AGENT_TYPE]:-$MESH_AGENT_TYPE}"

# Inject start command
mesh_inject::command "$SESSION_NAME" "$CMD"

# 4. Wait for agent ready
if ! mesh_inject::wait_for_agent "$SESSION_NAME" "$MESH_AGENT_TYPE" 60; then
    echo "Timeout waiting for agent $MESH_AGENT_TYPE in $SESSION_NAME"
    # Don't exit, maybe it started but prompt didn't match
fi

# 5. Send initial prompt
if [[ -n "$MESH_AGENT_PROMPT" ]]; then
    mesh_inject::command "$SESSION_NAME" "$MESH_AGENT_PROMPT"
fi

echo "Agent $MESH_AGENT_TYPE started successfully in session $SESSION_NAME"
echo "To attach: tmux attach -t $SESSION_NAME"
echo "$SESSION_NAME"
