#!/bin/bash
# Script to verify that test code is not included in the production build

set -e  # Exit on error

echo "===== Verifying Production Build ====="
echo

# Build the production binary
echo "Building production binary..."
cargo build --release --features production --no-default-features

# Get the binary size
BINARY_SIZE=$(ls -lh target/release/intellirouter | awk '{print $5}')
echo "Production binary size: $BINARY_SIZE"

# Check for test code in the binary
echo "Checking for test code in the binary..."

# List of test-related strings to check for
TEST_STRINGS=(
  "test_utils"
  "#[test]"
  "test_harness"
  "mock_"
  "fixture_"
)

# Check each string
FOUND_TEST_CODE=false
for str in "${TEST_STRINGS[@]}"; do
  if strings target/release/intellirouter | grep -q "$str"; then
    echo "❌ Found test code in production binary: $str"
    FOUND_TEST_CODE=true
  fi
done

# Check if the binary depends on the test-utils crate
if ldd target/release/intellirouter 2>/dev/null | grep -q "intellirouter-test-utils"; then
  echo "❌ Production binary depends on test-utils crate"
  FOUND_TEST_CODE=true
fi

# Final result
if [ "$FOUND_TEST_CODE" = true ]; then
  echo
  echo "❌ Verification FAILED: Test code found in production build"
  exit 1
else
  echo
  echo "✅ Verification PASSED: No test code found in production build"
  exit 0
fi