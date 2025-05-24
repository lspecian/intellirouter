#!/bin/bash
# IntelliRouter CI Security Audit
# This script runs security audits in CI environments

set -e

# Default values
OUTPUT_DIR="metrics/security"
FORMAT="json"
SEVERITY_THRESHOLD="medium"
CHECKS="dependencies,code,config"
FAIL_ON_ISSUES=true
STORE_ARTIFACTS=true
COMPARE_WITH_PREVIOUS=true
VERBOSE=false

# Help message
show_help() {
    echo "IntelliRouter CI Security Audit"
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -h, --help                   Show this help message"
    echo "  -o, --output DIR             Directory to save reports (default: metrics/security)"
    echo "  -f, --format FORMAT          Report format (json, markdown, html)"
    echo "  -s, --severity LEVEL         Minimum severity level to report (low, medium, high, critical)"
    echo "  -c, --checks CHECKS          Specific security checks to run (comma-separated)"
    echo "  --no-fail                    Don't fail the build if security issues are found"
    echo "  --no-store                   Don't store artifacts"
    echo "  --no-compare                 Don't compare with previous results"
    echo "  -v, --verbose                Enable verbose output"
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
        -s|--severity)
            SEVERITY_THRESHOLD="$2"
            shift
            shift
            ;;
        -c|--checks)
            CHECKS="$2"
            shift
            shift
            ;;
        --no-fail)
            FAIL_ON_ISSUES=false
            shift
            ;;
        --no-store)
            STORE_ARTIFACTS=false
            shift
            ;;
        --no-compare)
            COMPARE_WITH_PREVIOUS=false
            shift
            ;;
        -v|--verbose)
            VERBOSE=true
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
REPORT_FILENAME="ci_security_audit_${TIMESTAMP}.${FORMAT}"
REPORT_PATH="${OUTPUT_DIR}/${REPORT_FILENAME}"

# Print configuration
echo "IntelliRouter CI Security Audit"
echo "=============================="
echo "Output directory: $OUTPUT_DIR"
echo "Report path: $REPORT_PATH"
echo "Report format: $FORMAT"
echo "Severity threshold: $SEVERITY_THRESHOLD"
echo "Checks to run: $CHECKS"
echo "Fail on issues: $FAIL_ON_ISSUES"
echo "Store artifacts: $STORE_ARTIFACTS"
echo "Compare with previous: $COMPARE_WITH_PREVIOUS"
echo "Verbose: $VERBOSE"
echo ""

# Build the command
CMD="scripts/security/run_security_audit.sh --ci --output $OUTPUT_DIR --format $FORMAT --severity $SEVERITY_THRESHOLD --checks $CHECKS"

if [[ "$VERBOSE" == true ]]; then
    CMD="$CMD --verbose"
fi

# Run the security audit
echo "Running security audit..."
echo "Command: $CMD"
echo ""

eval "$CMD"
AUDIT_EXIT_CODE=$?

# Determine if we should fail the build
if [[ "$FAIL_ON_ISSUES" == true && "$AUDIT_EXIT_CODE" -ne 0 ]]; then
    echo "Security audit failed with exit code: $AUDIT_EXIT_CODE"
    echo "See the report for details: $REPORT_PATH"
    
    # Extract issue counts from the report
    if [[ "$FORMAT" == "json" ]]; then
        CRITICAL=$(jq -r '.summary.critical // 0' "$REPORT_PATH")
        HIGH=$(jq -r '.summary.high // 0' "$REPORT_PATH")
        MEDIUM=$(jq -r '.summary.medium // 0' "$REPORT_PATH")
        LOW=$(jq -r '.summary.low // 0' "$REPORT_PATH")
        
        echo "Critical issues: $CRITICAL"
        echo "High issues: $HIGH"
        echo "Medium issues: $MEDIUM"
        echo "Low issues: $LOW"
    fi
    
    # Fail the build
    echo "Failing the build due to security issues."
    exit 1
else
    echo "Security audit completed."
    echo "See the report for details: $REPORT_PATH"
fi

# Store artifacts if enabled
if [[ "$STORE_ARTIFACTS" == true ]]; then
    echo "Storing security audit artifacts..."
    
    # Create a directory for artifacts
    ARTIFACTS_DIR="${OUTPUT_DIR}/artifacts"
    mkdir -p "$ARTIFACTS_DIR"
    
    # Copy the report
    cp "$REPORT_PATH" "$ARTIFACTS_DIR/"
    
    # Copy any charts
    if [[ -f "${OUTPUT_DIR}/security_trends.png" ]]; then
        cp "${OUTPUT_DIR}/security_trends.png" "$ARTIFACTS_DIR/"
    fi
    
    if [[ -f "${OUTPUT_DIR}/security_trends_dashboard.html" ]]; then
        cp "${OUTPUT_DIR}/security_trends_dashboard.html" "$ARTIFACTS_DIR/"
    fi
    
    echo "Artifacts stored in: $ARTIFACTS_DIR"
fi

# Compare with previous results if enabled
if [[ "$COMPARE_WITH_PREVIOUS" == true ]]; then
    echo "Comparing with previous security audit results..."
    
    # Find the previous report
    PREVIOUS_REPORT=$(find "$OUTPUT_DIR" -name "ci_security_audit_*.${FORMAT}" -not -name "$(basename "$REPORT_PATH")" | sort -r | head -n 1)
    
    if [[ -n "$PREVIOUS_REPORT" ]]; then
        echo "Previous report found: $PREVIOUS_REPORT"
        
        # Compare the reports
        if [[ "$FORMAT" == "json" ]]; then
            CURRENT_TOTAL=$(jq -r '.summary.total // 0' "$REPORT_PATH")
            PREVIOUS_TOTAL=$(jq -r '.summary.total // 0' "$PREVIOUS_REPORT")
            
            CURRENT_CRITICAL=$(jq -r '.summary.critical // 0' "$REPORT_PATH")
            PREVIOUS_CRITICAL=$(jq -r '.summary.critical // 0' "$PREVIOUS_REPORT")
            
            CURRENT_HIGH=$(jq -r '.summary.high // 0' "$REPORT_PATH")
            PREVIOUS_HIGH=$(jq -r '.summary.high // 0' "$PREVIOUS_REPORT")
            
            CURRENT_MEDIUM=$(jq -r '.summary.medium // 0' "$REPORT_PATH")
            PREVIOUS_MEDIUM=$(jq -r '.summary.medium // 0' "$PREVIOUS_REPORT")
            
            CURRENT_LOW=$(jq -r '.summary.low // 0' "$REPORT_PATH")
            PREVIOUS_LOW=$(jq -r '.summary.low // 0' "$PREVIOUS_REPORT")
            
            # Calculate differences
            TOTAL_DIFF=$((CURRENT_TOTAL - PREVIOUS_TOTAL))
            CRITICAL_DIFF=$((CURRENT_CRITICAL - PREVIOUS_CRITICAL))
            HIGH_DIFF=$((CURRENT_HIGH - PREVIOUS_HIGH))
            MEDIUM_DIFF=$((CURRENT_MEDIUM - PREVIOUS_MEDIUM))
            LOW_DIFF=$((CURRENT_LOW - PREVIOUS_LOW))
            
            # Print comparison
            echo "Comparison with previous results:"
            echo "--------------------------------"
            echo "Total issues: $CURRENT_TOTAL ($(format_diff $TOTAL_DIFF))"
            echo "Critical issues: $CURRENT_CRITICAL ($(format_diff $CRITICAL_DIFF))"
            echo "High issues: $CURRENT_HIGH ($(format_diff $HIGH_DIFF))"
            echo "Medium issues: $CURRENT_MEDIUM ($(format_diff $MEDIUM_DIFF))"
            echo "Low issues: $CURRENT_LOW ($(format_diff $LOW_DIFF))"
            echo ""
            
            # Generate comparison report
            COMPARISON_REPORT="${OUTPUT_DIR}/security_comparison_${TIMESTAMP}.json"
            jq -n \
                --arg timestamp "$(date -u +"%Y-%m-%dT%H:%M:%SZ")" \
                --arg current_report "$(basename "$REPORT_PATH")" \
                --arg previous_report "$(basename "$PREVIOUS_REPORT")" \
                --arg total_diff "$TOTAL_DIFF" \
                --arg critical_diff "$CRITICAL_DIFF" \
                --arg high_diff "$HIGH_DIFF" \
                --arg medium_diff "$MEDIUM_DIFF" \
                --arg low_diff "$LOW_DIFF" \
                '{
                    "timestamp": $timestamp,
                    "current_report": $current_report,
                    "previous_report": $previous_report,
                    "differences": {
                        "total": $total_diff | tonumber,
                        "critical": $critical_diff | tonumber,
                        "high": $high_diff | tonumber,
                        "medium": $medium_diff | tonumber,
                        "low": $low_diff | tonumber
                    }
                }' > "$COMPARISON_REPORT"
            
            echo "Comparison report saved to: $COMPARISON_REPORT"
        else
            echo "Comparison is only supported for JSON reports."
        fi
    else
        echo "No previous report found for comparison."
    fi
fi

# Helper function to format difference values
format_diff() {
    local diff=$1
    if [[ "$diff" -gt 0 ]]; then
        echo "+$diff"
    elif [[ "$diff" -lt 0 ]]; then
        echo "$diff"
    else
        echo "no change"
    fi
}

exit 0