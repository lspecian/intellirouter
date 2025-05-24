#!/bin/bash
# IntelliRouter Code Security Check
# This script performs static code analysis for security issues

set -e

# Default values
OUTPUT_DIR="metrics/security"
FORMAT="json"
VERBOSE=false
CI_MODE=false
SEVERITY_THRESHOLD="medium"
FIX_ISSUES=false

# Help message
show_help() {
    echo "IntelliRouter Code Security Check"
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -h, --help                   Show this help message"
    echo "  -o, --output DIR             Directory to save reports (default: metrics/security)"
    echo "  -f, --format FORMAT          Report format (json, markdown, html)"
    echo "  -v, --verbose                Enable verbose output"
    echo "  --ci                         Run in CI mode (non-interactive, exit code reflects security status)"
    echo "  -s, --severity LEVEL         Minimum severity level to report (low, medium, high, critical)"
    echo "  --fix                        Attempt to automatically fix issues where possible"
    echo ""
    exit 0
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    key="$1"
    case $key in
        -h|--help)
            show_help
            ;;
        -o|--output)
            OUTPUT_DIR="$2"
            shift
            shift
            ;;
        -f|--format)
            FORMAT="$2"
            shift
            shift
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        --ci)
            CI_MODE=true
            shift
            ;;
        -s|--severity)
            SEVERITY_THRESHOLD="$2"
            shift
            shift
            ;;
        --fix)
            FIX_ISSUES=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            ;;
    esac
done

# Create output directory if it doesn't exist
mkdir -p "$OUTPUT_DIR"

# Generate timestamp for report filename
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
REPORT_FILENAME="code_security_audit_${TIMESTAMP}.${FORMAT}"
REPORT_PATH="${OUTPUT_DIR}/${REPORT_FILENAME}"

# Check if cargo-audit is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo is not installed."
    exit 1
fi

# Print configuration if verbose
if [[ "$VERBOSE" == true ]]; then
    echo "IntelliRouter Code Security Check"
    echo "================================"
    echo "Output directory: $OUTPUT_DIR"
    echo "Report path: $REPORT_PATH"
    echo "Report format: $FORMAT"
    echo "Severity threshold: $SEVERITY_THRESHOLD"
    echo "Fix issues: $FIX_ISSUES"
    echo "CI mode: $CI_MODE"
    echo ""
fi

# Create a temporary directory for results
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

# Define security lints to check
SECURITY_LINTS=(
    "clippy::unwrap_used"
    "clippy::expect_used"
    "clippy::panic"
    "clippy::unreachable"
    "clippy::todo"
    "clippy::unimplemented"
    "clippy::indexing_slicing"
    "clippy::unchecked_duration_subtraction"
    "clippy::float_arithmetic"
    "clippy::integer_arithmetic"
    "clippy::option_unwrap_used"
    "clippy::result_unwrap_used"
    "clippy::missing_safety_doc"
    "clippy::mem_forget"
    "clippy::missing_panics_doc"
    "clippy::exit"
    "clippy::dbg_macro"
    "clippy::print_stdout"
    "clippy::print_stderr"
    "clippy::wildcard_imports"
    "clippy::empty_loop"
    "clippy::cast_possible_truncation"
    "clippy::cast_possible_wrap"
    "clippy::cast_sign_loss"
    "clippy::cast_precision_loss"
    "clippy::mutex_atomic"
    "clippy::mutex_integer"
    "clippy::let_underscore_drop"
)

# Define severity levels for each lint
declare -A LINT_SEVERITY
LINT_SEVERITY["clippy::unwrap_used"]="medium"
LINT_SEVERITY["clippy::expect_used"]="medium"
LINT_SEVERITY["clippy::panic"]="high"
LINT_SEVERITY["clippy::unreachable"]="low"
LINT_SEVERITY["clippy::todo"]="low"
LINT_SEVERITY["clippy::unimplemented"]="medium"
LINT_SEVERITY["clippy::indexing_slicing"]="medium"
LINT_SEVERITY["clippy::unchecked_duration_subtraction"]="medium"
LINT_SEVERITY["clippy::float_arithmetic"]="low"
LINT_SEVERITY["clippy::integer_arithmetic"]="low"
LINT_SEVERITY["clippy::option_unwrap_used"]="medium"
LINT_SEVERITY["clippy::result_unwrap_used"]="medium"
LINT_SEVERITY["clippy::missing_safety_doc"]="medium"
LINT_SEVERITY["clippy::mem_forget"]="high"
LINT_SEVERITY["clippy::missing_panics_doc"]="low"
LINT_SEVERITY["clippy::exit"]="medium"
LINT_SEVERITY["clippy::dbg_macro"]="low"
LINT_SEVERITY["clippy::print_stdout"]="low"
LINT_SEVERITY["clippy::print_stderr"]="low"
LINT_SEVERITY["clippy::wildcard_imports"]="low"
LINT_SEVERITY["clippy::empty_loop"]="low"
LINT_SEVERITY["clippy::cast_possible_truncation"]="medium"
LINT_SEVERITY["clippy::cast_possible_wrap"]="medium"
LINT_SEVERITY["clippy::cast_sign_loss"]="medium"
LINT_SEVERITY["clippy::cast_precision_loss"]="medium"
LINT_SEVERITY["clippy::mutex_atomic"]="medium"
LINT_SEVERITY["clippy::mutex_integer"]="medium"
LINT_SEVERITY["clippy::let_underscore_drop"]="medium"

# Run clippy with security lints
echo "Running Clippy with security lints..."

# Build the clippy command with all security lints
CLIPPY_CMD="cargo clippy --message-format=json -- "
for lint in "${SECURITY_LINTS[@]}"; do
    CLIPPY_CMD+="--warn $lint "
done

# Run clippy and save output
if [[ "$VERBOSE" == true ]]; then
    echo "Command: $CLIPPY_CMD"
fi

eval "$CLIPPY_CMD" > "$TEMP_DIR/clippy_output.json" || true

# Parse the clippy output to count issues by severity
CRITICAL_COUNT=0
HIGH_COUNT=0
MEDIUM_COUNT=0
LOW_COUNT=0

# Process each line of the JSON output
cat "$TEMP_DIR/clippy_output.json" | while read -r line; do
    # Check if the line contains a warning or error message
    if echo "$line" | jq -e '.reason == "compiler-message" and .message.level == "warning"' > /dev/null; then
        # Extract the lint name
        LINT_NAME=$(echo "$line" | jq -r '.message.code.code // ""')
        
        # Determine severity based on lint name
        SEVERITY="low" # Default severity
        for lint in "${!LINT_SEVERITY[@]}"; do
            if [[ "$LINT_NAME" == "$lint" ]]; then
                SEVERITY="${LINT_SEVERITY[$lint]}"
                break
            fi
        done
        
        # Increment the appropriate counter
        case "$SEVERITY" in
            "critical")
                CRITICAL_COUNT=$((CRITICAL_COUNT + 1))
                ;;
            "high")
                HIGH_COUNT=$((HIGH_COUNT + 1))
                ;;
            "medium")
                MEDIUM_COUNT=$((MEDIUM_COUNT + 1))
                ;;
            "low")
                LOW_COUNT=$((LOW_COUNT + 1))
                ;;
        esac
        
        # Save the issue details
        echo "$line" | jq --arg severity "$SEVERITY" '. + {severity: $severity}' >> "$TEMP_DIR/issues.json"
    fi
done

# Calculate total
TOTAL_COUNT=$((CRITICAL_COUNT + HIGH_COUNT + MEDIUM_COUNT + LOW_COUNT))

# Generate report based on format
case "$FORMAT" in
    "json")
        # Create JSON report
        jq -n \
            --arg timestamp "$(date -u +"%Y-%m-%dT%H:%M:%SZ")" \
            --arg total "$TOTAL_COUNT" \
            --arg critical "$CRITICAL_COUNT" \
            --arg high "$HIGH_COUNT" \
            --arg medium "$MEDIUM_COUNT" \
            --arg low "$LOW_COUNT" \
            --slurpfile issues "$TEMP_DIR/issues.json" \
            '{
                "timestamp": $timestamp,
                "check_type": "code",
                "summary": {
                    "total": $total | tonumber,
                    "critical": $critical | tonumber,
                    "high": $high | tonumber,
                    "medium": $medium | tonumber,
                    "low": $low | tonumber
                },
                "issues": $issues
            }' > "$REPORT_PATH"
        ;;
    "markdown")
        # Create Markdown report
        {
            echo "# IntelliRouter Code Security Audit"
            echo ""
            echo "**Date:** $(date)"
            echo ""
            echo "## Summary"
            echo ""
            echo "- **Total issues:** $TOTAL_COUNT"
            echo "- **Critical issues:** $CRITICAL_COUNT"
            echo "- **High issues:** $HIGH_COUNT"
            echo "- **Medium issues:** $MEDIUM_COUNT"
            echo "- **Low issues:** $LOW_COUNT"
            echo ""
            echo "## Details"
            echo ""
            
            # Process each issue
            if [[ -f "$TEMP_DIR/issues.json" ]]; then
                cat "$TEMP_DIR/issues.json" | while read -r issue; do
                    SEVERITY=$(echo "$issue" | jq -r '.severity')
                    MESSAGE=$(echo "$issue" | jq -r '.message.message')
                    CODE=$(echo "$issue" | jq -r '.message.code.code // "unknown"')
                    FILE=$(echo "$issue" | jq -r '.message.spans[0].file_name // "unknown"')
                    LINE=$(echo "$issue" | jq -r '.message.spans[0].line_start // "unknown"')
                    COLUMN=$(echo "$issue" | jq -r '.message.spans[0].column_start // "unknown"')
                    
                    echo "### $CODE ($SEVERITY)"
                    echo ""
                    echo "**Location:** $FILE:$LINE:$COLUMN"
                    echo ""
                    echo "**Message:** $MESSAGE"
                    echo ""
                done
            fi
        } > "$REPORT_PATH"
        ;;
    "html")
        # Create HTML report
        {
            echo "<!DOCTYPE html>"
            echo "<html lang=\"en\">"
            echo "<head>"
            echo "  <meta charset=\"UTF-8\">"
            echo "  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">"
            echo "  <title>IntelliRouter Code Security Audit</title>"
            echo "  <style>"
            echo "    body { font-family: Arial, sans-serif; margin: 0; padding: 20px; }"
            echo "    h1 { color: #333; }"
            echo "    h2 { color: #555; margin-top: 30px; }"
            echo "    h3 { color: #666; }"
            echo "    .summary { display: flex; flex-wrap: wrap; margin-bottom: 20px; }"
            echo "    .summary-item { margin-right: 20px; margin-bottom: 10px; }"
            echo "    .critical { color: #d9534f; }"
            echo "    .high { color: #f0ad4e; }"
            echo "    .medium { color: #5bc0de; }"
            echo "    .low { color: #5cb85c; }"
            echo "    .issue { border: 1px solid #ddd; padding: 15px; margin-bottom: 15px; border-radius: 4px; }"
            echo "    .issue-critical { border-left: 5px solid #d9534f; }"
            echo "    .issue-high { border-left: 5px solid #f0ad4e; }"
            echo "    .issue-medium { border-left: 5px solid #5bc0de; }"
            echo "    .issue-low { border-left: 5px solid #5cb85c; }"
            echo "  </style>"
            echo "</head>"
            echo "<body>"
            echo "  <h1>IntelliRouter Code Security Audit</h1>"
            echo "  <p><strong>Date:</strong> $(date)</p>"
            echo "  <h2>Summary</h2>"
            echo "  <div class=\"summary\">"
            echo "    <div class=\"summary-item\"><strong>Total issues:</strong> $TOTAL_COUNT</div>"
            echo "    <div class=\"summary-item\"><strong>Critical issues:</strong> <span class=\"critical\">$CRITICAL_COUNT</span></div>"
            echo "    <div class=\"summary-item\"><strong>High issues:</strong> <span class=\"high\">$HIGH_COUNT</span></div>"
            echo "    <div class=\"summary-item\"><strong>Medium issues:</strong> <span class=\"medium\">$MEDIUM_COUNT</span></div>"
            echo "    <div class=\"summary-item\"><strong>Low issues:</strong> <span class=\"low\">$LOW_COUNT</span></div>"
            echo "  </div>"
            echo "  <h2>Details</h2>"
            echo "  <div class=\"issues\">"
            
            # Process each issue
            if [[ -f "$TEMP_DIR/issues.json" ]]; then
                cat "$TEMP_DIR/issues.json" | while read -r issue; do
                    SEVERITY=$(echo "$issue" | jq -r '.severity')
                    MESSAGE=$(echo "$issue" | jq -r '.message.message')
                    CODE=$(echo "$issue" | jq -r '.message.code.code // "unknown"')
                    FILE=$(echo "$issue" | jq -r '.message.spans[0].file_name // "unknown"')
                    LINE=$(echo "$issue" | jq -r '.message.spans[0].line_start // "unknown"')
                    COLUMN=$(echo "$issue" | jq -r '.message.spans[0].column_start // "unknown"')
                    
                    echo "    <div class=\"issue issue-$SEVERITY\">"
                    echo "      <h3>$CODE <span class=\"$SEVERITY\">($SEVERITY)</span></h3>"
                    echo "      <p><strong>Location:</strong> $FILE:$LINE:$COLUMN</p>"
                    echo "      <p><strong>Message:</strong> $MESSAGE</p>"
                    echo "    </div>"
                done
            fi
            
            echo "  </div>"
            echo "</body>"
            echo "</html>"
        } > "$REPORT_PATH"
        ;;
esac

# Print summary
echo "Code Security Check Summary"
echo "==========================="
echo "Total issues: $TOTAL_COUNT"
echo "Critical issues: $CRITICAL_COUNT"
echo "High issues: $HIGH_COUNT"
echo "Medium issues: $MEDIUM_COUNT"
echo "Low issues: $LOW_COUNT"
echo ""
echo "Report saved to: $REPORT_PATH"

# Determine exit code based on severity threshold
EXIT_CODE=0
if [[ "$CRITICAL_COUNT" -gt 0 ]]; then
    EXIT_CODE=4
elif [[ "$HIGH_COUNT" -gt 0 && "$SEVERITY_THRESHOLD" != "critical" ]]; then
    EXIT_CODE=3
elif [[ "$MEDIUM_COUNT" -gt 0 && "$SEVERITY_THRESHOLD" != "critical" && "$SEVERITY_THRESHOLD" != "high" ]]; then
    EXIT_CODE=2
elif [[ "$LOW_COUNT" -gt 0 && "$SEVERITY_THRESHOLD" == "low" ]]; then
    EXIT_CODE=1
fi

exit $EXIT_CODE