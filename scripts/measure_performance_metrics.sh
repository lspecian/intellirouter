#!/bin/bash
# Script to measure binary size and compilation time for IntelliRouter
# This script can be run regularly to track changes in these metrics over time

set -e  # Exit on error

# Create directories for results
METRICS_DIR="./metrics/performance_metrics"
mkdir -p "$METRICS_DIR"
DATE=$(date +"%Y-%m-%d_%H-%M-%S")
REPORT_FILE="$METRICS_DIR/metrics_report_$DATE.md"
CSV_FILE="$METRICS_DIR/metrics_history.csv"

echo "===== Measuring Performance Metrics ====="
echo "Results will be saved to: $REPORT_FILE"
echo "History will be updated in: $CSV_FILE"
echo

# Initialize report file
cat > "$REPORT_FILE" << EOF
# IntelliRouter Performance Metrics Report

Generated: $(date)

## Overview

This report documents the current binary size and compilation time metrics for IntelliRouter.

EOF

# Create CSV file header if it doesn't exist
if [ ! -f "$CSV_FILE" ]; then
  echo "Date,Binary Size (bytes),Debug Build Time (s),Release Build Time (s)" > "$CSV_FILE"
fi

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
  else
    echo "✅ No test code found in production binary"
    cat >> "$REPORT_FILE" << EOF
✅ **No test code found in production binary**

EOF
  fi
}

# Function to compare with previous measurements
compare_with_previous() {
  echo "Comparing with previous measurements..."
  
  # Get the most recent metrics from the CSV file (excluding the header)
  PREV_METRICS=$(tail -n 1 "$CSV_FILE" 2>/dev/null || echo "")
  
  if [ -n "$PREV_METRICS" ]; then
    # Parse previous metrics
    PREV_DATE=$(echo "$PREV_METRICS" | cut -d, -f1)
    PREV_BINARY_SIZE=$(echo "$PREV_METRICS" | cut -d, -f2)
    PREV_DEBUG_TIME=$(echo "$PREV_METRICS" | cut -d, -f3)
    PREV_RELEASE_TIME=$(echo "$PREV_METRICS" | cut -d, -f4)
    
    # Calculate changes
    BINARY_SIZE_DIFF=$((PREV_BINARY_SIZE - BINARY_SIZE_BYTES))
    BINARY_SIZE_DIFF_PERCENT=$(echo "scale=2; $BINARY_SIZE_DIFF / $PREV_BINARY_SIZE * 100" | bc 2>/dev/null || echo "N/A")
    
    DEBUG_TIME_DIFF=$(echo "$PREV_DEBUG_TIME - $DEBUG_TIME" | bc 2>/dev/null || echo "N/A")
    DEBUG_TIME_DIFF_PERCENT=$(echo "scale=2; $DEBUG_TIME_DIFF / $PREV_DEBUG_TIME * 100" | bc 2>/dev/null || echo "N/A")
    
    RELEASE_TIME_DIFF=$(echo "$PREV_RELEASE_TIME - $RELEASE_TIME" | bc 2>/dev/null || echo "N/A")
    RELEASE_TIME_DIFF_PERCENT=$(echo "scale=2; $RELEASE_TIME_DIFF / $PREV_RELEASE_TIME * 100" | bc 2>/dev/null || echo "N/A")
    
    # Add to report
    cat >> "$REPORT_FILE" << EOF
## Comparison with Previous Measurement ($PREV_DATE)

### Binary Size Change

| Metric | Before | After | Difference | Change |
|--------|--------|-------|------------|--------|
| Size (bytes) | $PREV_BINARY_SIZE | $BINARY_SIZE_BYTES | $BINARY_SIZE_DIFF | $BINARY_SIZE_DIFF_PERCENT% |

### Compilation Time Change

| Build Type | Before (seconds) | After (seconds) | Difference | Change |
|------------|------------------|----------------|------------|--------|
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
  
  # Add current metrics to the CSV file
  echo "$DATE,$BINARY_SIZE_BYTES,$DEBUG_TIME,$RELEASE_TIME" >> "$CSV_FILE"
}

# Function to generate charts (placeholder)
generate_charts() {
  echo "Generating charts..."
  
  # This is a placeholder for chart generation
  # In a real implementation, you would use a charting library or tool
  
  cat >> "$REPORT_FILE" << EOF
## Historical Trends

Charts showing the historical trends of binary size and compilation time would be generated here.
To implement this, you could use a tool like gnuplot, matplotlib, or a web-based charting library.

EOF
}

# Main execution
echo "Step 1: Measuring binary size"
measure_binary_size

echo "Step 2: Measuring compilation time"
measure_compilation_time

echo "Step 3: Checking for test code in production binary"
check_test_code_in_binary

echo "Step 4: Comparing with previous measurements"
compare_with_previous

echo "Step 5: Generating charts (placeholder)"
generate_charts

echo "Step 6: Generating summary"
cat >> "$REPORT_FILE" << EOF
## Summary

This report documents the current binary size and compilation time metrics for IntelliRouter.

Key metrics:
- Binary Size: $BINARY_SIZE ($BINARY_SIZE_BYTES bytes)
- Debug Compilation Time: $DEBUG_TIME seconds
- Release Compilation Time: $RELEASE_TIME seconds
- Test Code in Production: $([ "$FOUND_TEST_CODE" = true ] && echo "Found test code in the production binary" || echo "No test code found in the production binary")

EOF

echo "Done! Report generated at: $REPORT_FILE"
echo "Metrics history updated in: $CSV_FILE"