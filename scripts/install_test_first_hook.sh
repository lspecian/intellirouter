#!/bin/bash

# Script to install the test-first pre-commit hook

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HOOK_SOURCE="$SCRIPT_DIR/test_first_check.sh"
HOOK_TARGET=".git/hooks/pre-commit"

# Make the test-first check script executable
chmod +x "$HOOK_SOURCE"

# Check if the hook already exists
if [ -f "$HOOK_TARGET" ]; then
    echo "Pre-commit hook already exists at $HOOK_TARGET"
    echo "Checking if it already includes the test-first check..."
    
    if grep -q "test_first_check.sh" "$HOOK_TARGET"; then
        echo "Test-first check is already included in the pre-commit hook."
        exit 0
    else
        echo "Existing pre-commit hook found, but it doesn't include the test-first check."
        echo "Backing up existing hook to ${HOOK_TARGET}.bak"
        cp "$HOOK_TARGET" "${HOOK_TARGET}.bak"
        
        # Append to existing hook
        echo "" >> "$HOOK_TARGET"
        echo "# Added by install_test_first_hook.sh" >> "$HOOK_TARGET"
        echo "\"$HOOK_SOURCE\" || exit 1" >> "$HOOK_TARGET"
        echo "Test-first check added to existing pre-commit hook."
    fi
else
    # Create a new hook
    echo "#!/bin/bash" > "$HOOK_TARGET"
    echo "" >> "$HOOK_TARGET"
    echo "# Test-first pre-commit hook" >> "$HOOK_TARGET"
    echo "\"$HOOK_SOURCE\" || exit 1" >> "$HOOK_TARGET"
    
    # Make the hook executable
    chmod +x "$HOOK_TARGET"
    echo "Test-first pre-commit hook installed at $HOOK_TARGET"
fi

echo ""
echo "The test-first pre-commit hook will now check that tests exist for all Rust files being committed."
echo "This helps enforce the test-first development approach."
echo "To bypass this check (not recommended), use git commit with --no-verify"