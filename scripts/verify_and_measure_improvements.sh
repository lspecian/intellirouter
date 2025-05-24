#!/bin/bash
# Script to verify that all tests still pass with the new structure and measure improvements
# in binary size and compilation time.

set -e  # Exit on error

# Create directories for results
RESULTS_DIR="./metrics/test_restructuring"
mkdir -p "$RESULTS_DIR"
REPORT_FILE="$RESULTS_DIR/improvements_report_$(date +"%Y-%m-%d_%H-%M-%S").md"

echo "===== Verifying Tests and Measuring Improvements ====="
echo "Results will be saved to: $REPORT_FILE"
echo

# Initialize report file
cat > "$REPORT_FILE" << EOF
# Test Restructuring Improvements Report

Generated: $(date)

## Overview

This report documents the verification of tests and measurements of improvements
after the test code restructuring project.

EOF

# Function to measure compilation time
measure_compilation_time() {
  echo "Measuring compilation time..."
  
  # Clean first to ensure a full rebuild
  cargo clean
  
  # Measure time for debug build
  echo "Building debug version..."
  DEBUG_START=$(date +%s.%N)
  cargo build
  DEBUG_END=$(date +%s.%N)
  DEBUG_TIME=$(echo "$DEBUG_END - $DEBUG_START" | bc)
  
  # Measure time for release build
  echo "Building release version..."
  RELEASE_START=$(date +%s.%N)
  cargo build --release
  RELEASE_END=$(date +%s.%N)
  RELEASE_TIME=$(echo "$RELEASE_END - $RELEASE_START" | bc)
  
  echo "Debug build time: $DEBUG_TIME seconds"
  echo "Release build time: $RELEASE_TIME seconds"
  
  # Add to report
  cat >> "$REPORT_FILE" << EOF
## Compilation Time

| Build Type | Time (seconds) |
|------------|---------------|
| Debug      | $DEBUG_TIME   |
| Release    | $RELEASE_TIME |

EOF
}

# Function to measure binary size
measure_binary_size() {
  echo "Measuring binary size..."
  
  # Ensure release binary is built
  if [ ! -f "target/release/intellirouter" ]; then
    cargo build --release
  fi
  
  # Get binary size
  BINARY_SIZE=$(ls -lh target/release/intellirouter | awk '{print $5}')
  BINARY_SIZE_BYTES=$(ls -l target/release/intellirouter | awk '{print $5}')
  
  echo "Binary size: $BINARY_SIZE ($BINARY_SIZE_BYTES bytes)"
  
  # Add to report
  cat >> "$REPORT_FILE" << EOF
## Binary Size

| Metric      | Value         |
|-------------|---------------|
| Size        | $BINARY_SIZE  |
| Size (bytes)| $BINARY_SIZE_BYTES |

EOF
}

# Function to run all tests
run_all_tests() {
  echo "Running all tests..."
  
  # Create a section in the report
  cat >> "$REPORT_FILE" << EOF
## Test Results

EOF
  
  # Run unit tests
  echo "Running unit tests..."
  if cargo test --lib; then
    echo "✅ Unit tests passed"
    echo "- ✅ Unit tests: PASSED" >> "$REPORT_FILE"
  else
    echo "❌ Unit tests failed"
    echo "- ❌ Unit tests: FAILED" >> "$REPORT_FILE"
    TEST_FAILURES=true
  fi
  
  # Run integration tests
  echo "Running integration tests..."
  if cargo test --test integration_tests; then
    echo "✅ Integration tests passed"
    echo "- ✅ Integration tests: PASSED" >> "$REPORT_FILE"
  else
    echo "❌ Integration tests failed"
    echo "- ❌ Integration tests: FAILED" >> "$REPORT_FILE"
    TEST_FAILURES=true
  fi
  
  # Run e2e tests
  echo "Running e2e tests..."
  if cargo test --test e2e_tests; then
    echo "✅ E2E tests passed"
    echo "- ✅ E2E tests: PASSED" >> "$REPORT_FILE"
  else
    echo "❌ E2E tests failed"
    echo "- ❌ E2E tests: FAILED" >> "$REPORT_FILE"
    TEST_FAILURES=true
  fi
  
  # Run property tests
  echo "Running property tests..."
  if cargo test --test property_tests; then
    echo "✅ Property tests passed"
    echo "- ✅ Property tests: PASSED" >> "$REPORT_FILE"
  else
    echo "❌ Property tests failed"
    echo "- ❌ Property tests: FAILED" >> "$REPORT_FILE"
    TEST_FAILURES=true
  fi
  
  # Run custom test runner
  echo "Running custom test runner..."
  if cargo run --bin run_tests --features test-utils -- test all; then
    echo "✅ Custom test runner passed"
    echo "- ✅ Custom test runner: PASSED" >> "$REPORT_FILE"
  else
    echo "❌ Custom test runner failed"
    echo "- ❌ Custom test runner: FAILED" >> "$REPORT_FILE"
    TEST_FAILURES=true
  fi
  
  # Check for test failures
  if [ "$TEST_FAILURES" = true ]; then
    echo "❌ Some tests failed. See report for details."
    cat >> "$REPORT_FILE" << EOF

⚠️ **Warning**: Some tests failed. The test restructuring may have introduced issues that need to be fixed.

EOF
  else
    echo "✅ All tests passed!"
    cat >> "$REPORT_FILE" << EOF

✅ **All tests passed successfully with the new structure!**

EOF
  fi
}

# Function to check for test code in production binary
check_test_code_in_binary() {
  echo "Checking for test code in production binary..."
  
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
  TEST_CODE_FOUND=()
  
  for str in "${TEST_STRINGS[@]}"; do
    if strings target/release/intellirouter | grep -q "$str"; then
      echo "❌ Found test code in production binary: $str"
      FOUND_TEST_CODE=true
      TEST_CODE_FOUND+=("$str")
    fi
  done
  
  # Check if the binary depends on the test-utils crate
  if ldd target/release/intellirouter 2>/dev/null | grep -q "intellirouter-test-utils"; then
    echo "❌ Production binary depends on test-utils crate"
    FOUND_TEST_CODE=true
    TEST_CODE_FOUND+=("intellirouter-test-utils dependency")
  fi
  
  # Add to report
  cat >> "$REPORT_FILE" << EOF
## Test Code in Production Binary

EOF
  
  if [ "$FOUND_TEST_CODE" = true ]; then
    echo "❌ Test code found in production binary"
    cat >> "$REPORT_FILE" << EOF
❌ **Test code found in production binary**

The following test-related strings were found in the production binary:

EOF
    for found in "${TEST_CODE_FOUND[@]}"; do
      echo "- $found" >> "$REPORT_FILE"
    done
    
    cat >> "$REPORT_FILE" << EOF

This suggests that some test code is still being included in the production build.
Further investigation is needed to identify and remove this test code.

EOF
  else
    echo "✅ No test code found in production binary"
    cat >> "$REPORT_FILE" << EOF
✅ **No test code found in production binary**

The production binary does not contain any test-related strings or dependencies.
This confirms that the test restructuring has successfully separated test code from production code.

EOF
  fi
}

# Function to compare with previous measurements
compare_with_previous() {
  echo "Comparing with previous measurements..."
  
  # Check if previous measurements exist
  PREV_MEASUREMENTS="$RESULTS_DIR/previous_measurements.txt"
  
  if [ -f "$PREV_MEASUREMENTS" ]; then
    # Load previous measurements
    source "$PREV_MEASUREMENTS"
    
    # Calculate improvements
    BINARY_SIZE_DIFF=$((PREV_BINARY_SIZE_BYTES - BINARY_SIZE_BYTES))
    BINARY_SIZE_DIFF_PERCENT=$(echo "scale=2; $BINARY_SIZE_DIFF / $PREV_BINARY_SIZE_BYTES * 100" | bc)
    
    DEBUG_TIME_DIFF=$(echo "$PREV_DEBUG_TIME - $DEBUG_TIME" | bc)
    DEBUG_TIME_DIFF_PERCENT=$(echo "scale=2; $DEBUG_TIME_DIFF / $PREV_DEBUG_TIME * 100" | bc)
    
    RELEASE_TIME_DIFF=$(echo "$PREV_RELEASE_TIME - $RELEASE_TIME" | bc)
    RELEASE_TIME_DIFF_PERCENT=$(echo "scale=2; $RELEASE_TIME_DIFF / $PREV_RELEASE_TIME * 100" | bc)
    
    # Add to report
    cat >> "$REPORT_FILE" << EOF
## Improvements

### Binary Size Improvement

| Metric | Before | After | Difference | Improvement |
|--------|--------|-------|------------|-------------|
| Size (bytes) | $PREV_BINARY_SIZE_BYTES | $BINARY_SIZE_BYTES | $BINARY_SIZE_DIFF | $BINARY_SIZE_DIFF_PERCENT% |

### Compilation Time Improvement

| Build Type | Before (seconds) | After (seconds) | Difference | Improvement |
|------------|------------------|----------------|------------|-------------|
| Debug | $PREV_DEBUG_TIME | $DEBUG_TIME | $DEBUG_TIME_DIFF | $DEBUG_TIME_DIFF_PERCENT% |
| Release | $PREV_RELEASE_TIME | $RELEASE_TIME | $RELEASE_TIME_DIFF | $RELEASE_TIME_DIFF_PERCENT% |

EOF
  else
    echo "No previous measurements found. This will be the baseline for future comparisons."
    cat >> "$REPORT_FILE" << EOF
## Baseline Measurements

No previous measurements were found. These measurements will serve as the baseline for future comparisons.

EOF
  fi
  
  # Save current measurements for future comparisons
  cat > "$PREV_MEASUREMENTS" << EOF
PREV_BINARY_SIZE_BYTES=$BINARY_SIZE_BYTES
PREV_DEBUG_TIME=$DEBUG_TIME
PREV_RELEASE_TIME=$RELEASE_TIME
EOF
}

# Main execution
echo "Step 1: Running all tests"
run_all_tests

echo "Step 2: Measuring binary size"
measure_binary_size

echo "Step 3: Measuring compilation time"
measure_compilation_time

echo "Step 4: Checking for test code in production binary"
check_test_code_in_binary

echo "Step 5: Comparing with previous measurements"
compare_with_previous

echo "Step 6: Generating summary"
cat >> "$REPORT_FILE" << EOF
## Summary

The test restructuring project aimed to separate test code from production code to reduce binary size and improve compilation time. This report documents the verification of tests and measurements of improvements after the restructuring.

Key findings:
- Test Status: All tests $([ "$TEST_FAILURES" = true ] && echo "did not pass" || echo "passed successfully") with the new structure.
- Test Code in Production: $([ "$FOUND_TEST_CODE" = true ] && echo "Still found test code in the production binary" || echo "Successfully removed all test code from the production binary").
EOF

if [ -f "$PREV_MEASUREMENTS" ]; then
  cat >> "$REPORT_FILE" << EOF
- Binary Size: Reduced by $BINARY_SIZE_DIFF bytes ($BINARY_SIZE_DIFF_PERCENT%).
- Debug Compilation Time: Improved by $DEBUG_TIME_DIFF seconds ($DEBUG_TIME_DIFF_PERCENT%).
- Release Compilation Time: Improved by $RELEASE_TIME_DIFF seconds ($RELEASE_TIME_DIFF_PERCENT%).
EOF
else
  cat >> "$REPORT_FILE" << EOF
- Baseline measurements established for future comparisons.
EOF
fi

echo "Done! Report generated at: $REPORT_FILE"