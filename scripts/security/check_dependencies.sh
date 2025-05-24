#!/bin/bash
# IntelliRouter Dependency Security Check
# This script checks for vulnerable dependencies using cargo-audit

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
    echo "IntelliRouter Dependency Security Check"
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
REPORT_FILENAME="dependency_audit_${TIMESTAMP}.${FORMAT}"
REPORT_PATH="${OUTPUT_DIR}/${REPORT_FILENAME}"

# Check if cargo-audit is installed
if ! command -v cargo-audit &> /dev/null; then
    echo "Error: cargo-audit is not installed."
    echo "Please install it with: cargo install cargo-audit"
    exit 1
fi

# Print configuration if verbose
if [[ "$VERBOSE" == true ]]; then
    echo "IntelliRouter Dependency Security Check"
    echo "======================================"
    echo "Output directory: $OUTPUT_DIR"
    echo "Report path: $REPORT_PATH"
    echo "Report format: $FORMAT"
    echo "Severity threshold: $SEVERITY_THRESHOLD"
    echo "Fix issues: $FIX_ISSUES"
    echo "CI mode: $CI_MODE"
    echo ""
fi

# Run cargo audit
echo "Running cargo audit..."

# Build command
AUDIT_CMD="cargo audit"

# Add options based on severity threshold
case "$SEVERITY_THRESHOLD" in
    "low")
        AUDIT_CMD="$AUDIT_CMD --severity low"
        ;;
    "medium")
        AUDIT_CMD="$AUDIT_CMD --severity medium"
        ;;
    "high")
        AUDIT_CMD="$AUDIT_CMD --severity high"
        ;;
    "critical")
        AUDIT_CMD="$AUDIT_CMD --severity critical"
        ;;
esac

# Add fix option if requested
if [[ "$FIX_ISSUES" == true ]]; then
    AUDIT_CMD="$AUDIT_CMD --fix"
fi

# Add JSON output for parsing
AUDIT_CMD="$AUDIT_CMD --json"

# Run the command and capture output
if [[ "$VERBOSE" == true ]]; then
    echo "Command: $AUDIT_CMD"
fi

AUDIT_OUTPUT=$(eval "$AUDIT_CMD" 2>&1 || true)
AUDIT_EXIT_CODE=$?

# Parse the JSON output to count vulnerabilities by severity
CRITICAL_COUNT=$(echo "$AUDIT_OUTPUT" | jq -r '.vulnerabilities.list | map(select(.severity == "critical")) | length')
HIGH_COUNT=$(echo "$AUDIT_OUTPUT" | jq -r '.vulnerabilities.list | map(select(.severity == "high")) | length')
MEDIUM_COUNT=$(echo "$AUDIT_OUTPUT" | jq -r '.vulnerabilities.list | map(select(.severity == "medium")) | length')
LOW_COUNT=$(echo "$AUDIT_OUTPUT" | jq -r '.vulnerabilities.list | map(select(.severity == "low")) | length')

# Default to 0 if jq returns null
CRITICAL_COUNT=${CRITICAL_COUNT:-0}
HIGH_COUNT=${HIGH_COUNT:-0}
MEDIUM_COUNT=${MEDIUM_COUNT:-0}
LOW_COUNT=${LOW_COUNT:-0}

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
            --argjson raw_output "$(echo "$AUDIT_OUTPUT" | jq '.')" \
            '{
                "timestamp": $timestamp,
                "check_type": "dependencies",
                "summary": {
                    "total": $total | tonumber,
                    "critical": $critical | tonumber,
                    "high": $high | tonumber,
                    "medium": $medium | tonumber,
                    "low": $low | tonumber
                },
                "raw_output": $raw_output
            }' > "$REPORT_PATH"
        ;;
    "markdown")
        # Create Markdown report
        {
            echo "# IntelliRouter Dependency Security Audit"
            echo ""
            echo "**Date:** $(date)"
            echo ""
            echo "## Summary"
            echo ""
            echo "- **Total vulnerabilities:** $TOTAL_COUNT"
            echo "- **Critical vulnerabilities:** $CRITICAL_COUNT"
            echo "- **High vulnerabilities:** $HIGH_COUNT"
            echo "- **Medium vulnerabilities:** $MEDIUM_COUNT"
            echo "- **Low vulnerabilities:** $LOW_COUNT"
            echo ""
            echo "## Details"
            echo ""
            echo '```'
            echo "$AUDIT_OUTPUT" | jq -r '.vulnerabilities.list[] | "### " + .advisory.id + "\n\n**Package:** " + .package.name + "\n**Version:** " + .package.version + "\n**Severity:** " + .severity + "\n\n" + .advisory.description + "\n\n**Solution:** " + (.advisory.solution // "No solution provided") + "\n"'
            echo '```'
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
            echo "  <title>IntelliRouter Dependency Security Audit</title>"
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
            echo "    .vulnerability { border: 1px solid #ddd; padding: 15px; margin-bottom: 15px; border-radius: 4px; }"
            echo "  </style>"
            echo "</head>"
            echo "<body>"
            echo "  <h1>IntelliRouter Dependency Security Audit</h1>"
            echo "  <p><strong>Date:</strong> $(date)</p>"
            echo "  <h2>Summary</h2>"
            echo "  <div class=\"summary\">"
            echo "    <div class=\"summary-item\"><strong>Total vulnerabilities:</strong> $TOTAL_COUNT</div>"
            echo "    <div class=\"summary-item\"><strong>Critical vulnerabilities:</strong> <span class=\"critical\">$CRITICAL_COUNT</span></div>"
            echo "    <div class=\"summary-item\"><strong>High vulnerabilities:</strong> <span class=\"high\">$HIGH_COUNT</span></div>"
            echo "    <div class=\"summary-item\"><strong>Medium vulnerabilities:</strong> <span class=\"medium\">$MEDIUM_COUNT</span></div>"
            echo "    <div class=\"summary-item\"><strong>Low vulnerabilities:</strong> <span class=\"low\">$LOW_COUNT</span></div>"
            echo "  </div>"
            echo "  <h2>Details</h2>"
            echo "  <div class=\"vulnerabilities\">"
            echo "$AUDIT_OUTPUT" | jq -r '.vulnerabilities.list[] | "<div class=\"vulnerability " + .severity + "\">\n  <h3>" + .advisory.id + "</h3>\n  <p><strong>Package:</strong> " + .package.name + "</p>\n  <p><strong>Version:</strong> " + .package.version + "</p>\n  <p><strong>Severity:</strong> <span class=\"" + .severity + "\">" + .severity + "</span></p>\n  <p>" + .advisory.description + "</p>\n  <p><strong>Solution:</strong> " + (.advisory.solution // "No solution provided") + "</p>\n</div>"'
            echo "  </div>"
            echo "</body>"
            echo "</html>"
        } > "$REPORT_PATH"
        ;;
esac

# Print summary
echo "Dependency Security Check Summary"
echo "================================="
echo "Total vulnerabilities: $TOTAL_COUNT"
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