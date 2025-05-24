#!/bin/bash

# Pre-commit hook to enforce test-first development
# This script checks if tests exist for the Rust files being committed

set -e

echo "Running test-first check..."

# Get list of Rust files being committed
RUST_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep '\.rs$' || true)

if [ -z "$RUST_FILES" ]; then
    echo "No Rust files being committed, skipping test-first check."
    exit 0
fi

FAIL=0

for FILE in $RUST_FILES; do
    # Skip test files themselves
    if [[ "$FILE" == *"tests.rs" ]] || [[ "$FILE" == *"_tests.rs" ]] || [[ "$FILE" == *"/tests/"* ]]; then
        continue
    fi

    # Extract module name and path
    MODULE_PATH=$(dirname "$FILE")
    MODULE_NAME=$(basename "$FILE" .rs)
    
    # Check for test module in the same file
    if grep -q "#\[cfg(test)\]" "$FILE"; then
        echo "✅ $FILE contains tests"
        continue
    fi

    # Check for separate test file
    TEST_FILE="$MODULE_PATH/tests.rs"
    if [ -f "$TEST_FILE" ] && grep -q "$MODULE_NAME" "$TEST_FILE"; then
        echo "✅ $FILE has tests in $TEST_FILE"
        continue
    fi

    # Check for dedicated test file
    TEST_FILE="$MODULE_PATH/${MODULE_NAME}_tests.rs"
    if [ -f "$TEST_FILE" ]; then
        echo "✅ $FILE has tests in $TEST_FILE"
        continue
    fi

    # Check for tests directory
    TEST_FILE="$MODULE_PATH/tests/${MODULE_NAME}_tests.rs"
    if [ -f "$TEST_FILE" ]; then
        echo "✅ $FILE has tests in $TEST_FILE"
        continue
    fi

    # If we get here, no tests were found
    echo "❌ No tests found for $FILE"
    echo "   Please write tests before implementing functionality."
    FAIL=1
done

if [ $FAIL -eq 1 ]; then
    echo "Test-first check failed. Please write tests before implementing functionality."
    echo "To bypass this check (not recommended), use git commit with --no-verify"
    exit 1
fi

echo "Test-first check passed!"
exit 0