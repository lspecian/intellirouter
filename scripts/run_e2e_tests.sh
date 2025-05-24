#!/bin/bash
# Script for running end-to-end tests in CI environments

set -e

# Set environment variables for testing
export RUST_LOG=intellirouter=debug,test=debug
export RUST_BACKTRACE=1

echo "Running model routing tests..."
cargo test --test e2e_tests -- test_model_routing --nocapture

echo "Running multi-step chain tests (non-ignored)..."
cargo test --test integration_tests -- test_chat_completions_endpoint --nocapture

echo "Running RAG injection tests (non-ignored)..."
cargo test --test integration_tests -- test_rag_injection --nocapture

echo "Running all non-ignored tests in the test directory..."
cargo test --test integration_tests --test e2e_tests --test property_tests -- --nocapture --skip-ignored

echo "Running custom test runner..."
cargo run --bin run_tests --features test-utils -- test integration

echo "Generating test report..."
# Create a simple test report
mkdir -p logs
echo "# E2E Test Results" > logs/test_report.md
echo "## Test Run: $(date)" >> logs/test_report.md
echo "## Test Summary" >> logs/test_report.md

# Count test results
TOTAL=$(cargo test 2>&1 | grep -o 'test result: ok. [0-9]\+ passed' | awk '{print $4}')
IGNORED=$(cargo test 2>&1 | grep -o '[0-9]\+ ignored' | awk '{print $1}')
FAILED=$(cargo test 2>&1 | grep -o '[0-9]\+ failed' | awk '{print $1}' || echo "0")

echo "- Total Tests: $TOTAL" >> logs/test_report.md
echo "- Passed: $((TOTAL - FAILED))" >> logs/test_report.md
echo "- Failed: ${FAILED:-0}" >> logs/test_report.md
echo "- Ignored: ${IGNORED:-0}" >> logs/test_report.md

echo "E2E tests completed!"