#!/bin/bash

# CI script to enforce the test-first approach
# This script checks if tests exist for all Rust files and if test coverage meets the minimum threshold

set -e

echo "Running test-first CI check..."

# Check if tests exist for all Rust files
echo "Checking if tests exist for all Rust files..."

# Get list of all Rust files (excluding test files and generated code)
RUST_FILES=$(find src -name "*.rs" | grep -v "tests.rs" | grep -v "_tests.rs" | grep -v "/tests/" | grep -v "/generated/" || true)

if [ -z "$RUST_FILES" ]; then
    echo "No Rust files found, skipping test-first check."
    exit 0
fi

FAIL=0

for FILE in $RUST_FILES; do
    # Skip files in the test_templates directory
    if [[ "$FILE" == *"test_templates"* ]]; then
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
    FAIL=1
done

# Run test coverage check
echo "Running test coverage check..."

# Install tarpaulin if not already installed
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "Installing cargo-tarpaulin..."
    cargo install cargo-tarpaulin
fi

# Run tarpaulin to generate coverage report
cargo tarpaulin --verbose --workspace --out Xml --output-dir coverage

# Check if coverage meets the minimum threshold
COVERAGE=$(grep -oP 'line-rate="\K[0-9.]+' coverage/cobertura.xml | head -1)
COVERAGE_PERCENT=$(echo "$COVERAGE * 100" | bc)
MINIMUM_COVERAGE=80

echo "Current test coverage: ${COVERAGE_PERCENT}%"
echo "Minimum required coverage: ${MINIMUM_COVERAGE}%"

if (( $(echo "$COVERAGE_PERCENT < $MINIMUM_COVERAGE" | bc -l) )); then
    echo "❌ Test coverage is below the minimum threshold of ${MINIMUM_COVERAGE}%"
    FAIL=1
else
    echo "✅ Test coverage meets the minimum threshold of ${MINIMUM_COVERAGE}%"
fi

# Generate HTML report for easier viewing
cargo tarpaulin --verbose --workspace --out Html --output-dir coverage

if [ $FAIL -eq 1 ]; then
    echo "Test-first check failed. Please ensure all code has tests and coverage meets the minimum threshold."
    exit 1
fi

echo "Test-first check passed!"
exit 0