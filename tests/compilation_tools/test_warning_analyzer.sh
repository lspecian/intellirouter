#!/bin/bash
set -e

# ANSI color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

# Temporary directory for test files
TEMP_DIR="target/test_warnings"
TEMP_REPORT="$TEMP_DIR/warning_report.md"
ANALYZE_WARNINGS_SCRIPT="scripts/analyze_warnings.sh"

# Check if analyze_warnings.sh exists
if [ ! -f "$ANALYZE_WARNINGS_SCRIPT" ]; then
    echo -e "${RED}Error: $ANALYZE_WARNINGS_SCRIPT not found.${RESET}"
    exit 1
fi

# Setup function to create a temporary directory for test files
setup() {
    mkdir -p "$TEMP_DIR/src"
    
    # Create a simple Cargo.toml
    cat > "$TEMP_DIR/Cargo.toml" << EOF
[package]
name = "test_warnings"
version = "0.1.0"
edition = "2021"

[dependencies]
EOF
}

# Cleanup function
cleanup() {
    if [ -d "$TEMP_DIR" ]; then
        rm -rf "$TEMP_DIR"
    fi
}

# Run a test and check if the warning analyzer correctly identifies the warning
run_test() {
    local test_name="$1"
    local rust_file="$2"
    local warning_type="$3"
    local expected_count="$4"
    
    echo -e "${BLUE}Running test: $test_name${RESET}"
    
    # Create the Rust file
    echo "$rust_file" > "$TEMP_DIR/src/lib.rs"
    
    # Run the warning analyzer script on the test directory
    (cd "$TEMP_DIR" && bash "../../$ANALYZE_WARNINGS_SCRIPT" --allow-errors > /dev/null)
    
    # Check if the warning report was generated
    if [ ! -f "$TEMP_REPORT" ]; then
        echo -e "${RED}Error: Warning report not generated.${RESET}"
        return 1
    fi
    
    # Check if the warning type is in the report
    if ! grep -q "$warning_type" "$TEMP_REPORT"; then
        echo -e "${RED}Error: Warning type '$warning_type' not found in the report.${RESET}"
        return 1
    fi
    
    # Check if the warning count matches the expected count
    local actual_count=$(grep -A 1 "$warning_type" "$TEMP_REPORT" | grep -o "[0-9]\+" | head -1)
    if [ "$actual_count" -ne "$expected_count" ]; then
        echo -e "${RED}Error: Expected $expected_count warnings of type '$warning_type', but found $actual_count.${RESET}"
        return 1
    fi
    
    echo -e "${GREEN}Test passed: $test_name${RESET}"
    return 0
}

# Main test function
run_tests() {
    local failures=0
    
    # Test 1: Unused variable warning
    run_test "Unused Variable Warning" '
fn main() {
    let unused = 42;
    println!("Hello, world!");
}
' "unused_variable" 1 || ((failures++))
    
    # Test 2: Unused import warning
    run_test "Unused Import Warning" '
use std::collections::HashMap;
use std::time::Duration;

fn main() {
    let _map = HashMap::new();
    // Duration is unused
}
' "unused_import" 1 || ((failures++))
    
    # Test 3: Dead code warning
    run_test "Dead Code Warning" '
fn main() {
    println!("Hello, world!");
}

fn unused_function() {
    println!("This function is never called");
}
' "dead_code" 1 || ((failures++))
    
    # Test 4: Multiple warnings of different types
    run_test "Multiple Warning Types" '
use std::collections::HashMap;
use std::time::Duration;
use std::io::Read;

fn main() {
    let unused1 = 42;
    let unused2 = "hello";
    let _map = HashMap::new();
    // Duration and Read are unused
}

fn unused_function() {
    println!("This function is never called");
}

struct Config {
    unused_field: String,
}
' "unused_variable" 2 || ((failures++))
    
    # Test 5: No warnings
    run_test "No Warnings" '
fn main() {
    let used_var = 42;
    println!("The value is: {}", used_var);
}
' "unused_variable" 0 || ((failures++))
    
    # Test 6: Naming convention warning
    run_test "Naming Convention Warning" '
fn badName() {
    println!("This function name does not follow Rust naming conventions");
}

fn main() {
    badName();
}
' "naming_convention" 1 || ((failures++))
    
    # Test 7: Unused field warning
    run_test "Unused Field Warning" '
struct Config {
    name: String,
    unused_field: u32,
}

fn main() {
    let config = Config {
        name: "test".to_string(),
        unused_field: 42,
    };
    
    println!("Name: {}", config.name);
    // unused_field is never read
}
' "unused_field" 1 || ((failures++))
    
    # Report test results
    if [ $failures -eq 0 ]; then
        echo -e "${GREEN}${BOLD}All tests passed!${RESET}"
        return 0
    else
        echo -e "${RED}${BOLD}$failures test(s) failed.${RESET}"
        return 1
    fi
}

# Run the tests
echo -e "${BOLD}${BLUE}Running Warning Analyzer Tests${RESET}"
echo -e "${CYAN}=============================${RESET}\n"

# Setup, run tests, and cleanup
setup
run_tests
result=$?
cleanup

exit $result