#!/bin/bash
# Enhanced Testing Script for IntelliRouter
# This script runs the enhanced tests for error conditions, recovery scenarios,
# load testing, and integration tests between components.

set -e

echo "Running Enhanced Tests for IntelliRouter"
echo "----------------------------------------"

# Set environment variables for testing
export RUST_BACKTRACE=1
export RUST_LOG=info

# Create a directory for test results
RESULTS_DIR="./test_results"
mkdir -p $RESULTS_DIR

echo "Running Error Recovery Tests..."
cargo test --package intellirouter --lib -- modules::test_harness::error_recovery_tests --nocapture | tee $RESULTS_DIR/error_recovery_tests.log

echo "Running Load Tests..."
cargo test --package intellirouter --lib -- modules::test_harness::load_tests --nocapture | tee $RESULTS_DIR/load_tests.log

echo "Running Integration Tests..."
cargo test --package intellirouter --lib -- modules::test_harness::integration_tests --nocapture | tee $RESULTS_DIR/integration_tests.log

echo "Running Error Recovery Integration Tests..."
cargo test --package intellirouter --lib -- modules::test_harness::integration_tests::error_recovery_integration_tests --nocapture | tee $RESULTS_DIR/error_recovery_integration_tests.log

# Run the test harness directly for more comprehensive testing
echo "Running Test Harness with All Test Suites..."
cargo run --bin run_tests -- --category all --output $RESULTS_DIR/all_tests.json

echo "----------------------------------------"
echo "Enhanced Tests Completed"
echo "Results saved to $RESULTS_DIR"

# Check for failures
if grep -q "FAILED" $RESULTS_DIR/*.log; then
  echo "Some tests failed. Please check the logs for details."
  exit 1
else
  echo "All tests passed!"
  exit 0
fi