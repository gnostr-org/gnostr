#!/bin/bash
# scripts/splittest.sh

TARGET_DIR="$(pwd)/target"
BINARY="$TARGET_DIR/debug/gnostr-bitcoin"
SESSION_NAME="gnostr-cluster"
TOP_CMD="cargo build --target-dir $TARGET_DIR --bin gnostr-bitcoin"

# Help Bar Content
HELP_CMD="while true; do echo ' [o] Switch Pane | [z] Zoom | [d] Detach | [Ctrl+b + %] Split | tmux kill-session -t $SESSION_NAME'; sleep 60; done"

# 1. Cleanup & Build
pkill -f gnostr-bitcoin 2>/dev/null
tmux kill-session -t "$SESSION_NAME" 2>/dev/null

# 2. Setup Data Dirs
for i in {1..4}; do
    mkdir -p "test_data_$i"
done

# 3. Define Node Commands
CMD1="$BINARY --datadir ./test_data_1 --listen"
CMD2="sleep 2; $BINARY --datadir ./test_data_2 --target-peer-addr 127.0.0.1:8333"
CMD3="sleep 4; $BINARY --datadir ./test_data_3 --target-peer-addr 127.0.0.1:8333"
CMD4="sleep 6; $BINARY --datadir ./test_data_4 --target-peer-addr 127.0.0.1:8333"

# 4. Launch Layout
echo "[+] Launching 2x2 Grid + Slim Header/Footer..."

# Start Session with the Top Monitor (Pane 0)
tmux new-session -d -s "$SESSION_NAME" -n "Cluster" "$TOP_CMD"

# Create the main work area (Pane 1)
tmux split-window -v -t "$SESSION_NAME" -p 90 "$CMD1"

# Create the slim Help Bar at the very bottom (Split from the NEW Pane 1)
# tmux split-window -v -t "$SESSION_NAME:0.1" -l 2 "$HELP_CMD"

# --- GRID LOGIC ---

# 1. Split Pane 1 vertically to create Node 3 (Bottom-Left)
# Pane 1 = Node 1 (Top-Left), Pane 2 = Node 3 (Bottom-Left)
tmux split-window -v -t "$SESSION_NAME:0.1" "$CMD3"

# 2. Split Node 1 (Top-Left) horizontally to create Node 2 (Top-Right)
# Pane 1 = Node 1, Pane 3 = Node 2
tmux split-window -h -t "$SESSION_NAME:0.1" "$CMD2"

# 3. Split Node 3 (Bottom-Left) horizontally to create Node 4 (Bottom-Right)
# Pane 2 = Node 3, Pane 4 = Node 4
tmux split-window -h -t "$SESSION_NAME:0.2" "$CMD4"

# 5. Finalize Focus & Styling
# Set help bar colors
tmux select-pane -t 5 -P 'bg=black,fg=cyan'

# TARGET THE BOTTOM MOST NODE PANEL (Pane 4)
# This ensures keyboard input goes to Node 4 immediately
tmux select-pane -t 4

# 6. Attach
tmux attach-session -t "$SESSION_NAME"
