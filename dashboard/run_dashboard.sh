#!/bin/bash
# Script to run the IntelliRouter dashboard server

set -e

# Define paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Print header
echo "IntelliRouter Dashboard Server"
echo "============================="
echo "Date: $(date)"
echo "Project root: $PROJECT_ROOT"
echo ""

# Check if the dashboard is built
if [ ! -f "$SCRIPT_DIR/target/release/intellirouter-dashboard" ]; then
    echo "Building dashboard..."
    cd "$SCRIPT_DIR"
    cargo build --release
    echo "Dashboard built."
fi

# Collect metrics before starting the server
echo "Collecting metrics..."
"$SCRIPT_DIR/collect_metrics.sh"
echo "Metrics collected."

# Start the dashboard server
echo "Starting dashboard server..."
cd "$SCRIPT_DIR"
"$SCRIPT_DIR/target/release/intellirouter-dashboard"