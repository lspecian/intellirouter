#!/bin/bash
# Make security audit scripts executable

set -e

# Get the directory of this script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Make all scripts in the security directory executable
echo "Making security audit scripts executable..."
chmod +x "$SCRIPT_DIR"/*.sh

echo "Done!"