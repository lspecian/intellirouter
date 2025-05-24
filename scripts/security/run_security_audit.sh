#!/bin/bash
# IntelliRouter Security Audit System
# This script provides a comprehensive security audit framework for IntelliRouter

set -e

# Default values
OUTPUT_DIR="metrics/security"
FORMAT="json"
VERBOSE=false
CI_MODE=false
SPECIFIC_CHECKS=""
CONFIG_PATH="config/security_audit.toml"
SEVERITY_THRESHOLD="medium"
FIX_ISSUES=false
REPORT_ONLY=false
TRACK_METRICS=true

# Help message
show_help() {
    echo "IntelliRouter Security Audit System"
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -h, --help                   Show this help message"
    echo "  -o, --output DIR             Directory to save reports (default: metrics/security)"
    echo "  -f, --format FORMAT          Report format (json, markdown, html)"
    echo "  -v, --verbose                Enable verbose output"
    echo "  --ci                         Run in CI mode (non-interactive, exit code reflects security status)"
    echo "  -c, --checks CHECKS          Specific security checks to run (comma-separated)"
    echo "  --config PATH                Path to the security audit configuration file"
    echo "  -s, --severity LEVEL         Minimum severity level to report (low, medium, high, critical)"
    echo "  --fix                        Attempt to automatically fix issues where possible"
    echo "  --report-only                Only generate report, don't track metrics"
    echo "  --no-metrics                 Don't track metrics"
    echo ""
    echo "Available security checks:"
    echo "  dependencies                 Check for vulnerable dependencies"
    echo "  code                         Static code analysis for security issues"
    echo "  config                       Configuration security checks"
    echo "  auth                         Authentication and authorization checks"
    echo "  data                         Data validation and sanitization checks"
    echo "  network                      Network security checks"
    echo "  all                          Run all security checks"
    echo ""
    echo "Examples:"
    echo "  $0 --checks dependencies,code --severity high"
    echo "  $0 --ci --checks all"
    echo "  $0 --fix --checks dependencies"
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
        -c|--checks)
            SPECIFIC_CHECKS="$2"
            shift
            shift
            ;;
        --config)
            CONFIG_PATH="$2"
            shift
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
        --report-only)
            REPORT_ONLY=true
            shift
            ;;
        --no-metrics)
            TRACK_METRICS=false
            shift
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            ;;
    esac
done

# Validate format
if [[ "$FORMAT" != "json" && "$FORMAT" != "markdown" && "$FORMAT" != "html" ]]; then
    echo "Error: Invalid format: $FORMAT"
    echo "Valid formats: json, markdown, html"
    exit 1
fi

# Validate severity threshold
if [[ "$SEVERITY_THRESHOLD" != "low" && "$SEVERITY_THRESHOLD" != "medium" && "$SEVERITY_THRESHOLD" != "high" && "$SEVERITY_THRESHOLD" != "critical" ]]; then
    echo "Error: Invalid severity threshold: $SEVERITY_THRESHOLD"
    echo "Valid thresholds: low, medium, high, critical"
    exit 1
fi

# Create output directory if it doesn't exist
mkdir -p "$OUTPUT_DIR"

# Generate timestamp for report filename
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
REPORT_FILENAME="security_audit_${TIMESTAMP}.${FORMAT}"
REPORT_PATH="${OUTPUT_DIR}/${REPORT_FILENAME}"

# Determine which checks to run
if [[ "$SPECIFIC_CHECKS" == "all" || -z "$SPECIFIC_CHECKS" ]]; then
    CHECKS=("dependencies" "code" "config" "auth" "data" "network")
else
    IFS=',' read -ra CHECKS <<< "$SPECIFIC_CHECKS"
fi

# Print configuration
echo "IntelliRouter Security Audit System"
echo "=================================="
echo "Output directory: $OUTPUT_DIR"
echo "Report path: $REPORT_PATH"
echo "Report format: $FORMAT"
echo "Severity threshold: $SEVERITY_THRESHOLD"
echo "Fix issues: $FIX_ISSUES"
echo "Track metrics: $TRACK_METRICS"
echo "CI mode: $CI_MODE"
echo "Checks to run: ${CHECKS[*]}"
echo "Configuration file: $CONFIG_PATH"
echo ""

# Confirm execution if not in CI mode
if [[ "$CI_MODE" != true ]]; then
    read -p "Do you want to proceed? (y/n) " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted."
        exit 1
    fi
fi

# Initialize results array
declare -A RESULTS
TOTAL_ISSUES=0
CRITICAL_ISSUES=0
HIGH_ISSUES=0
MEDIUM_ISSUES=0
LOW_ISSUES=0

# Run security checks
echo "Running security checks..."
echo ""

# Function to run a security check
run_check() {
    local check=$1
    local script="scripts/security/check_${check}.sh"
    
    if [[ ! -f "$script" ]]; then
        echo "Error: Security check script not found: $script"
        return 1
    fi
    
    echo "Running $check security check..."
    
    # Build command
    local cmd="$script --output $OUTPUT_DIR --format $FORMAT --severity $SEVERITY_THRESHOLD"
    
    if [[ "$VERBOSE" == true ]]; then
        cmd="$cmd --verbose"
    fi
    
    if [[ "$FIX_ISSUES" == true ]]; then
        cmd="$cmd --fix"
    fi
    
    if [[ "$CI_MODE" == true ]]; then
        cmd="$cmd --ci"
    fi
    
    # Run the command
    if [[ "$VERBOSE" == true ]]; then
        echo "Command: $cmd"
    fi
    
    # Execute the check and capture output and exit code
    local output
    output=$(bash "$cmd" 2>&1)
    local exit_code=$?
    
    # Parse results
    local critical=$(echo "$output" | grep -oP "Critical issues: \K\d+")
    local high=$(echo "$output" | grep -oP "High issues: \K\d+")
    local medium=$(echo "$output" | grep -oP "Medium issues: \K\d+")
    local low=$(echo "$output" | grep -oP "Low issues: \K\d+")
    
    # Store results
    RESULTS["$check"]="$exit_code"
    CRITICAL_ISSUES=$((CRITICAL_ISSUES + (critical || 0)))
    HIGH_ISSUES=$((HIGH_ISSUES + (high || 0)))
    MEDIUM_ISSUES=$((MEDIUM_ISSUES + (medium || 0)))
    LOW_ISSUES=$((LOW_ISSUES + (low || 0)))
    
    # Print output
    echo "$output"
    echo ""
    
    return $exit_code
}

# Run each check
FAILED_CHECKS=0
for check in "${CHECKS[@]}"; do
    if ! run_check "$check"; then
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
    fi
done

# Calculate total issues
TOTAL_ISSUES=$((CRITICAL_ISSUES + HIGH_ISSUES + MEDIUM_ISSUES + LOW_ISSUES))

# Generate summary report
echo "Security Audit Summary"
echo "======================"
echo "Total issues: $TOTAL_ISSUES"
echo "Critical issues: $CRITICAL_ISSUES"
echo "High issues: $HIGH_ISSUES"
echo "Medium issues: $MEDIUM_ISSUES"
echo "Low issues: $LOW_ISSUES"
echo ""
echo "Failed checks: $FAILED_CHECKS"
echo ""

# Track metrics if enabled
if [[ "$TRACK_METRICS" == true && "$REPORT_ONLY" != true ]]; then
    echo "Tracking security metrics..."
    
    # Create metrics file if it doesn't exist
    METRICS_FILE="${OUTPUT_DIR}/security_metrics.csv"
    if [[ ! -f "$METRICS_FILE" ]]; then
        echo "Date,Total,Critical,High,Medium,Low" > "$METRICS_FILE"
    fi
    
    # Append metrics
    DATE=$(date +"%Y-%m-%d")
    echo "$DATE,$TOTAL_ISSUES,$CRITICAL_ISSUES,$HIGH_ISSUES,$MEDIUM_ISSUES,$LOW_ISSUES" >> "$METRICS_FILE"
    
    echo "Metrics saved to: $METRICS_FILE"
    echo ""
    
    # Generate metrics chart
    if command -v gnuplot &> /dev/null; then
        echo "Generating metrics chart..."
        scripts/security/generate_security_charts.sh --input "$METRICS_FILE" --output "${OUTPUT_DIR}/security_trends.png"
        echo "Chart saved to: ${OUTPUT_DIR}/security_trends.png"
        echo ""
    fi
fi

# Determine exit code based on severity threshold
EXIT_CODE=0
if [[ "$CRITICAL_ISSUES" -gt 0 ]]; then
    EXIT_CODE=4
elif [[ "$HIGH_ISSUES" -gt 0 && "$SEVERITY_THRESHOLD" != "critical" ]]; then
    EXIT_CODE=3
elif [[ "$MEDIUM_ISSUES" -gt 0 && "$SEVERITY_THRESHOLD" != "critical" && "$SEVERITY_THRESHOLD" != "high" ]]; then
    EXIT_CODE=2
elif [[ "$LOW_ISSUES" -gt 0 && "$SEVERITY_THRESHOLD" == "low" ]]; then
    EXIT_CODE=1
fi

# Print final status
if [[ $EXIT_CODE -eq 0 ]]; then
    echo "Security audit completed successfully with no issues above the $SEVERITY_THRESHOLD severity threshold."
else
    echo "Security audit completed with issues above the $SEVERITY_THRESHOLD severity threshold."
fi

echo "Report saved to: $REPORT_PATH"

# Open report if not in CI mode
if [[ "$CI_MODE" != true && "$FORMAT" == "html" ]]; then
    if command -v xdg-open &> /dev/null; then
        xdg-open "$REPORT_PATH"
    elif command -v open &> /dev/null; then
        open "$REPORT_PATH"
    fi
fi

exit $EXIT_CODE