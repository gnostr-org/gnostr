#!/bin/bash
# scripts/test.sh

# Target directory for builds
TARGET_DIR="$(pwd)/target"

# Kill any existing gnostr-bitcoin processes
echo "Cleaning up existing gnostr-bitcoin processes..."
pkill -f gnostr-bitcoin 2>/dev/null
sleep 1

# Build once to avoid race conditions in concurrent cargo run
echo "Building gnostr-bitcoin..."
cargo build --target-dir "$TARGET_DIR"

if [ $? -ne 0 ]; then
    echo "Build failed."
    exit 1
fi

echo "Build successful."
echo "Configuring multiple instances with separate data directories..."

# Create absolute paths for data directories
DATADIR1="$(pwd)/test_data_1"
DATADIR2="$(pwd)/test_data_2"

# Clean up previous run data
rm -rf "$DATADIR1" "$DATADIR2"
mkdir -p "$DATADIR1"
mkdir -p "$DATADIR2"

echo "Instance 1 will use: $DATADIR1"
echo "Instance 2 will use: $DATADIR2"

# Construct commands using cargo run
CMD1="cargo run --target-dir $TARGET_DIR --bin gnostr-bitcoin -- --datadir $DATADIR1 --listen"
CMD2="cargo run --target-dir $TARGET_DIR --bin gnostr-bitcoin -- --datadir $DATADIR2 --target-peer-addr 127.0.0.1:8333"

if [[ "$(uname)" == "Darwin" ]]; then
    echo ""
    echo "Detected macOS. Launching separate Terminal windows with cargo run..."
    
    # Create temporary executable scripts for the terminals to run
    SCRIPT1="$DATADIR1/run_node1.command"
    echo "#!/bin/bash" > "$SCRIPT1"
    echo "cd '$(pwd)'" >> "$SCRIPT1"
    echo "echo 'Starting Node 1 (Listener)...'" >> "$SCRIPT1"
    echo "$CMD1" >> "$SCRIPT1"
    chmod +x "$SCRIPT1"
    
    SCRIPT2="$DATADIR2/run_node2.command"
    echo "#!/bin/bash" > "$SCRIPT2"
    echo "cd '$(pwd)'" >> "$SCRIPT2"
    echo "echo 'Starting Node 2 (Connector)...'" >> "$SCRIPT2"
    echo "echo 'Waiting 3 seconds for Node 1...'" >> "$SCRIPT2"
    echo "sleep 3" >> "$SCRIPT2"
    echo "$CMD2" >> "$SCRIPT2"
    chmod +x "$SCRIPT2"
    
    # Open the scripts in Terminal.app
    open "$SCRIPT1"
    open "$SCRIPT2"
    
    echo "Terminals launched."
else
    echo ""
    echo "Not on macOS. Please run these commands in separate terminals:"
    echo ""
    echo "Terminal 1:"
    echo "  $CMD1"
    echo ""
    echo "Terminal 2:"
    echo "  $CMD2"
fi
