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

# Parse command line arguments
OUTPUT_FORMAT="markdown"
OUTPUT_FILE="metrics/code_quality_report.md"
COMPARE_WITH_PREVIOUS=false
STORE_METRICS=true

for arg in "$@"; do
    case $arg in
        --format=*)
            OUTPUT_FORMAT="${arg#*=}"
            shift
            ;;
        --output=*)
            OUTPUT_FILE="${arg#*=}"
            shift
            ;;
        --compare)
            COMPARE_WITH_PREVIOUS=true
            shift
            ;;
        --no-store)
            STORE_METRICS=false
            shift
            ;;
    esac
done

echo -e "${BOLD}${BLUE}IntelliRouter Code Quality Report Generator${RESET}"
echo -e "${CYAN}============================================${RESET}\n"

# Create metrics directory if it doesn't exist
METRICS_DIR="metrics"
if [ ! -d "$METRICS_DIR" ]; then
    echo -e "${YELLOW}Creating metrics directory...${RESET}"
    mkdir -p "$METRICS_DIR"
fi

# Get current date and time for the report
CURRENT_DATE=$(date +"%Y-%m-%d")
CURRENT_TIME=$(date +"%H:%M:%S")
TIMESTAMP=$(date +"%Y%m%d%H%M%S")
METRICS_FILE="$METRICS_DIR/metrics_$TIMESTAMP.json"

# Create a temporary file for the warning analysis
TEMP_WARNING_FILE=$(mktemp)

# Run warning analyzer and capture output
echo -e "${YELLOW}Running warning analyzer...${RESET}"
./scripts/analyze_warnings.sh --allow-errors > "$TEMP_WARNING_FILE" 2>&1 || {
    echo -e "${RED}Warning analyzer failed. Continuing with other metrics...${RESET}"
}

# Extract warning counts from the warning report
WARNING_REPORT="warning_report.md"
if [ -f "$WARNING_REPORT" ]; then
    TOTAL_WARNINGS=$(grep -m 1 "Total Warnings:" "$WARNING_REPORT" | sed 's/[^0-9]*//g')
    UNUSED_VARIABLE_WARNINGS=$(grep -A 1 "Unused Variables" "$WARNING_REPORT" | grep -o "([0-9]* warnings)" | sed 's/[^0-9]*//g')
    UNUSED_IMPORT_WARNINGS=$(grep -A 1 "Unused Imports" "$WARNING_REPORT" | grep -o "([0-9]* warnings)" | sed 's/[^0-9]*//g')
    DEAD_CODE_WARNINGS=$(grep -A 1 "Dead Code" "$WARNING_REPORT" | grep -o "([0-9]* warnings)" | sed 's/[^0-9]*//g')
    NAMING_CONVENTION_WARNINGS=$(grep -A 1 "Naming Convention" "$WARNING_REPORT" | grep -o "([0-9]* warnings)" | sed 's/[^0-9]*//g')
else
    TOTAL_WARNINGS="N/A"
    UNUSED_VARIABLE_WARNINGS="N/A"
    UNUSED_IMPORT_WARNINGS="N/A"
    DEAD_CODE_WARNINGS="N/A"
    NAMING_CONVENTION_WARNINGS="N/A"
fi

# Run clippy and count warnings
echo -e "${YELLOW}Running clippy to collect lints...${RESET}"
CLIPPY_TEMP=$(mktemp)
cargo clippy --message-format=json > "$CLIPPY_TEMP" 2>&1 || {
    echo -e "${YELLOW}Clippy reported issues. Continuing with analysis...${RESET}"
}
CLIPPY_WARNINGS=$(grep -c "\"level\":\"warning\"" "$CLIPPY_TEMP" || echo "0")

# Count lines of code
echo -e "${YELLOW}Counting lines of code...${RESET}"
RUST_LOC=$(find src -name "*.rs" -exec cat {} \; | wc -l)
PROTO_LOC=$(find proto -name "*.proto" -exec cat {} \; | wc -l)
PYTHON_LOC=$(find sdk/python -name "*.py" -exec cat {} \; | wc -l)
TS_LOC=$(find sdk/typescript/src -name "*.ts" -exec cat {} \; | wc -l)
TOTAL_LOC=$((RUST_LOC + PROTO_LOC + PYTHON_LOC + TS_LOC))

# Calculate warning density (warnings per 1000 lines of code)
if [ "$TOTAL_WARNINGS" != "N/A" ]; then
    WARNING_DENSITY=$(echo "scale=2; $TOTAL_WARNINGS * 1000 / $TOTAL_LOC" | bc)
else
    WARNING_DENSITY="N/A"
fi

# Run tests and calculate test coverage
echo -e "${YELLOW}Running tests and calculating coverage...${RESET}"
TARPAULIN_TEMP=$(mktemp)
cargo tarpaulin --out json --output-dir . > "$TARPAULIN_TEMP" 2>&1 || {
    echo -e "${YELLOW}Tarpaulin reported issues. Continuing with analysis...${RESET}"
}

if [ -f "tarpaulin-report.json" ]; then
    TEST_COVERAGE=$(grep -o '"line_coverage":[0-9.]*' tarpaulin-report.json | cut -d':' -f2)
    TEST_COVERAGE_FORMATTED=$(printf "%.2f" "$TEST_COVERAGE")
else
    TEST_COVERAGE_FORMATTED="N/A"
fi

# Count number of tests
TEST_COUNT=$(grep -r --include="*.rs" "#\[test\]" src tests | wc -l)

# Count documentation coverage (percentage of public items with documentation)
echo -e "${YELLOW}Analyzing documentation coverage...${RESET}"
DOC_TEMP=$(mktemp)
cargo doc --no-deps > "$DOC_TEMP" 2>&1
DOC_WARNINGS=$(grep -c "warning: missing documentation" "$DOC_TEMP" || echo "0")
PUBLIC_ITEMS=$(grep -r --include="*.rs" "pub " src | grep -v "pub(crate)" | wc -l)

if [ "$PUBLIC_ITEMS" -gt 0 ]; then
    DOC_COVERAGE=$(echo "scale=2; 100 - ($DOC_WARNINGS * 100 / $PUBLIC_ITEMS)" | bc)
else
    DOC_COVERAGE="N/A"
fi

# Create JSON metrics file
echo -e "${YELLOW}Creating metrics file...${RESET}"
cat > "$METRICS_FILE" << EOF
{
  "timestamp": "$CURRENT_DATE $CURRENT_TIME",
  "metrics": {
    "total_warnings": $TOTAL_WARNINGS,
    "unused_variable_warnings": $UNUSED_VARIABLE_WARNINGS,
    "unused_import_warnings": $UNUSED_IMPORT_WARNINGS,
    "dead_code_warnings": $DEAD_CODE_WARNINGS,
    "naming_convention_warnings": $NAMING_CONVENTION_WARNINGS,
    "clippy_warnings": $CLIPPY_WARNINGS,
    "lines_of_code": {
      "rust": $RUST_LOC,
      "proto": $PROTO_LOC,
      "python": $PYTHON_LOC,
      "typescript": $TS_LOC,
      "total": $TOTAL_LOC
    },
    "warning_density": $WARNING_DENSITY,
    "test_coverage": $TEST_COVERAGE_FORMATTED,
    "test_count": $TEST_COUNT,
    "documentation_coverage": $DOC_COVERAGE
  }
}
EOF

# Compare with previous metrics if requested
COMPARISON_TEXT=""
if [ "$COMPARE_WITH_PREVIOUS" = true ]; then
    echo -e "${YELLOW}Comparing with previous metrics...${RESET}"
    PREVIOUS_METRICS=$(find "$METRICS_DIR" -name "metrics_*.json" -not -name "metrics_$TIMESTAMP.json" | sort -r | head -n 1)
    
    if [ -n "$PREVIOUS_METRICS" ]; then
        PREV_TOTAL_WARNINGS=$(grep -o '"total_warnings":[0-9]*' "$PREVIOUS_METRICS" | cut -d':' -f2)
        PREV_WARNING_DENSITY=$(grep -o '"warning_density":[0-9.]*' "$PREVIOUS_METRICS" | cut -d':' -f2)
        PREV_TEST_COVERAGE=$(grep -o '"test_coverage":[0-9.]*' "$PREVIOUS_METRICS" | cut -d':' -f2)
        PREV_DOC_COVERAGE=$(grep -o '"documentation_coverage":[0-9.]*' "$PREVIOUS_METRICS" | cut -d':' -f2)
        
        # Calculate changes
        if [ "$TOTAL_WARNINGS" != "N/A" ] && [ "$PREV_TOTAL_WARNINGS" != "N/A" ]; then
            WARNING_CHANGE=$((TOTAL_WARNINGS - PREV_TOTAL_WARNINGS))
            if [ $WARNING_CHANGE -lt 0 ]; then
                WARNING_CHANGE_TEXT="${GREEN}$WARNING_CHANGE${RESET}"
            elif [ $WARNING_CHANGE -gt 0 ]; then
                WARNING_CHANGE_TEXT="${RED}+$WARNING_CHANGE${RESET}"
            else
                WARNING_CHANGE_TEXT="No change"
            fi
        else
            WARNING_CHANGE_TEXT="N/A"
        fi
        
        if [ "$TEST_COVERAGE_FORMATTED" != "N/A" ] && [ "$PREV_TEST_COVERAGE" != "N/A" ]; then
            TEST_COVERAGE_CHANGE=$(echo "scale=2; $TEST_COVERAGE_FORMATTED - $PREV_TEST_COVERAGE" | bc)
            if (( $(echo "$TEST_COVERAGE_CHANGE > 0" | bc -l) )); then
                TEST_COVERAGE_CHANGE_TEXT="${GREEN}+$TEST_COVERAGE_CHANGE%${RESET}"
            elif (( $(echo "$TEST_COVERAGE_CHANGE < 0" | bc -l) )); then
                TEST_COVERAGE_CHANGE_TEXT="${RED}$TEST_COVERAGE_CHANGE%${RESET}"
            else
                TEST_COVERAGE_CHANGE_TEXT="No change"
            fi
        else
            TEST_COVERAGE_CHANGE_TEXT="N/A"
        fi
        
        if [ "$DOC_COVERAGE" != "N/A" ] && [ "$PREV_DOC_COVERAGE" != "N/A" ]; then
            DOC_COVERAGE_CHANGE=$(echo "scale=2; $DOC_COVERAGE - $PREV_DOC_COVERAGE" | bc)
            if (( $(echo "$DOC_COVERAGE_CHANGE > 0" | bc -l) )); then
                DOC_COVERAGE_CHANGE_TEXT="${GREEN}+$DOC_COVERAGE_CHANGE%${RESET}"
            elif (( $(echo "$DOC_COVERAGE_CHANGE < 0" | bc -l) )); then
                DOC_COVERAGE_CHANGE_TEXT="${RED}$DOC_COVERAGE_CHANGE%${RESET}"
            else
                DOC_COVERAGE_CHANGE_TEXT="No change"
            fi
        else
            DOC_COVERAGE_CHANGE_TEXT="N/A"
        fi
        
        # Create comparison text
        COMPARISON_TEXT="
## Comparison with Previous Report

| Metric | Previous | Current | Change |
|--------|----------|---------|--------|
| Total Warnings | $PREV_TOTAL_WARNINGS | $TOTAL_WARNINGS | $WARNING_CHANGE_TEXT |
| Warning Density | $PREV_WARNING_DENSITY | $WARNING_DENSITY | - |
| Test Coverage | $PREV_TEST_COVERAGE% | $TEST_COVERAGE_FORMATTED% | $TEST_COVERAGE_CHANGE_TEXT |
| Documentation Coverage | $PREV_DOC_COVERAGE% | $DOC_COVERAGE% | $DOC_COVERAGE_CHANGE_TEXT |
"
    else
        COMPARISON_TEXT="
## Comparison with Previous Report

No previous metrics found for comparison.
"
    fi
fi

# Generate the report based on the requested format
echo -e "${YELLOW}Generating $OUTPUT_FORMAT report...${RESET}"

if [ "$OUTPUT_FORMAT" = "markdown" ]; then
    cat > "$OUTPUT_FILE" << EOF
# IntelliRouter Code Quality Report

**Generated on:** $CURRENT_DATE at $CURRENT_TIME

## Summary

| Metric | Value |
|--------|-------|
| Total Warnings | $TOTAL_WARNINGS |
| Clippy Warnings | $CLIPPY_WARNINGS |
| Warning Density | $WARNING_DENSITY warnings per 1000 lines |
| Test Coverage | $TEST_COVERAGE_FORMATTED% |
| Test Count | $TEST_COUNT tests |
| Documentation Coverage | $DOC_COVERAGE% |
| Total Lines of Code | $TOTAL_LOC |

## Warning Breakdown

| Warning Type | Count |
|--------------|-------|
| Unused Variables | $UNUSED_VARIABLE_WARNINGS |
| Unused Imports | $UNUSED_IMPORT_WARNINGS |
| Dead Code | $DEAD_CODE_WARNINGS |
| Naming Convention Issues | $NAMING_CONVENTION_WARNINGS |

## Lines of Code by Language

| Language | Lines of Code |
|----------|---------------|
| Rust | $RUST_LOC |
| Protocol Buffers | $PROTO_LOC |
| Python | $PYTHON_LOC |
| TypeScript | $TS_LOC |
| **Total** | **$TOTAL_LOC** |

$COMPARISON_TEXT

## Code Quality Goals

See [Code Quality Goals](../docs/code_quality_goals.md) for our targets and progress.

## How to Contribute to Code Quality

See [Contributing to Code Quality](../CONTRIBUTING.md#code-quality-contributions) for information on how to help improve code quality.
EOF

elif [ "$OUTPUT_FORMAT" = "html" ]; then
    cat > "$OUTPUT_FILE" << EOF
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>IntelliRouter Code Quality Report</title>
    <style>
        body { font-family: Arial, sans-serif; line-height: 1.6; margin: 0; padding: 20px; color: #333; }
        h1 { color: #2c3e50; }
        h2 { color: #3498db; margin-top: 30px; }
        table { border-collapse: collapse; width: 100%; margin-bottom: 20px; }
        th, td { text-align: left; padding: 12px; border-bottom: 1px solid #ddd; }
        th { background-color: #f2f2f2; }
        tr:hover { background-color: #f5f5f5; }
        .good { color: green; }
        .bad { color: red; }
        .warning { color: orange; }
    </style>
</head>
<body>
    <h1>IntelliRouter Code Quality Report</h1>
    <p><strong>Generated on:</strong> $CURRENT_DATE at $CURRENT_TIME</p>
    
    <h2>Summary</h2>
    <table>
        <tr><th>Metric</th><th>Value</th></tr>
        <tr><td>Total Warnings</td><td>$TOTAL_WARNINGS</td></tr>
        <tr><td>Clippy Warnings</td><td>$CLIPPY_WARNINGS</td></tr>
        <tr><td>Warning Density</td><td>$WARNING_DENSITY warnings per 1000 lines</td></tr>
        <tr><td>Test Coverage</td><td>$TEST_COVERAGE_FORMATTED%</td></tr>
        <tr><td>Test Count</td><td>$TEST_COUNT tests</td></tr>
        <tr><td>Documentation Coverage</td><td>$DOC_COVERAGE%</td></tr>
        <tr><td>Total Lines of Code</td><td>$TOTAL_LOC</td></tr>
    </table>
    
    <h2>Warning Breakdown</h2>
    <table>
        <tr><th>Warning Type</th><th>Count</th></tr>
        <tr><td>Unused Variables</td><td>$UNUSED_VARIABLE_WARNINGS</td></tr>
        <tr><td>Unused Imports</td><td>$UNUSED_IMPORT_WARNINGS</td></tr>
        <tr><td>Dead Code</td><td>$DEAD_CODE_WARNINGS</td></tr>
        <tr><td>Naming Convention Issues</td><td>$NAMING_CONVENTION_WARNINGS</td></tr>
    </table>
    
    <h2>Lines of Code by Language</h2>
    <table>
        <tr><th>Language</th><th>Lines of Code</th></tr>
        <tr><td>Rust</td><td>$RUST_LOC</td></tr>
        <tr><td>Protocol Buffers</td><td>$PROTO_LOC</td></tr>
        <tr><td>Python</td><td>$PYTHON_LOC</td></tr>
        <tr><td>TypeScript</td><td>$TS_LOC</td></tr>
        <tr><th>Total</th><th>$TOTAL_LOC</th></tr>
    </table>
    
    <h2>Code Quality Goals</h2>
    <p>See <a href="../docs/code_quality_goals.md">Code Quality Goals</a> for our targets and progress.</p>
    
    <h2>How to Contribute to Code Quality</h2>
    <p>See <a href="../CONTRIBUTING.md#code-quality-contributions">Contributing to Code Quality</a> for information on how to help improve code quality.</p>
</body>
</html>
EOF

elif [ "$OUTPUT_FORMAT" = "json" ]; then
    cat > "$OUTPUT_FILE" << EOF
{
  "report": {
    "generated_on": "$CURRENT_DATE $CURRENT_TIME",
    "summary": {
      "total_warnings": $TOTAL_WARNINGS,
      "clippy_warnings": $CLIPPY_WARNINGS,
      "warning_density": $WARNING_DENSITY,
      "test_coverage": $TEST_COVERAGE_FORMATTED,
      "test_count": $TEST_COUNT,
      "documentation_coverage": $DOC_COVERAGE,
      "total_lines_of_code": $TOTAL_LOC
    },
    "warning_breakdown": {
      "unused_variables": $UNUSED_VARIABLE_WARNINGS,
      "unused_imports": $UNUSED_IMPORT_WARNINGS,
      "dead_code": $DEAD_CODE_WARNINGS,
      "naming_convention_issues": $NAMING_CONVENTION_WARNINGS
    },
    "lines_of_code_by_language": {
      "rust": $RUST_LOC,
      "proto": $PROTO_LOC,
      "python": $PYTHON_LOC,
      "typescript": $TS_LOC,
      "total": $TOTAL_LOC
    }
  }
}
EOF
fi

echo -e "${GREEN}Report generated successfully!${RESET}"
echo -e "Report saved to: ${BOLD}$OUTPUT_FILE${RESET}"

# Clean up temporary files
rm -f "$TEMP_WARNING_FILE" "$CLIPPY_TEMP" "$TARPAULIN_TEMP" "$DOC_TEMP"

# Make the script executable
chmod +x "$0"