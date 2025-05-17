#!/bin/bash
# IntelliRouter Audit System Runner
# This script provides a simple interface for running the audit system in different environments

set -e

# Default values
DEPLOYMENT="local-dev"
DASHBOARD=false
DASHBOARD_HOST="127.0.0.1"
DASHBOARD_PORT="8090"
OUTPUT_DIR="audit_reports"
FORMAT="json"
STORE_RESULTS=false
COMPARE=false
VERBOSE=false
CI_MODE=false
SPECIFIC_TESTS=""
CONFIG_PATH="config/audit.toml"

# Help message
show_help() {
    echo "IntelliRouter Audit System Runner"
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -h, --help                 Show this help message"
    echo "  -d, --deployment SCENARIO  Deployment scenario (single-node, distributed, cloud, local-dev)"
    echo "  --dashboard                Start the dashboard server"
    echo "  --dashboard-host HOST      Dashboard host (default: 127.0.0.1)"
    echo "  --dashboard-port PORT      Dashboard port (default: 8090)"
    echo "  -o, --output DIR           Directory to save reports (default: audit_reports)"
    echo "  -f, --format FORMAT        Report format (json, markdown, html)"
    echo "  -s, --store                Store test results for historical comparison"
    echo "  -c, --compare              Compare with previous test results"
    echo "  -v, --verbose              Enable verbose output"
    echo "  --ci                       Run in CI mode (non-interactive, exit code reflects test status)"
    echo "  -t, --tests TESTS          Specific tests to run (comma-separated)"
    echo "  --config PATH              Path to the audit configuration file"
    echo ""
    echo "Examples:"
    echo "  $0 --deployment cloud --dashboard"
    echo "  $0 --tests basic,rag,e2e --store --verbose"
    echo "  $0 --ci --deployment distributed"
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
        -d|--deployment)
            DEPLOYMENT="$2"
            shift
            shift
            ;;
        --dashboard)
            DASHBOARD=true
            shift
            ;;
        --dashboard-host)
            DASHBOARD_HOST="$2"
            shift
            shift
            ;;
        --dashboard-port)
            DASHBOARD_PORT="$2"
            shift
            shift
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
        -s|--store)
            STORE_RESULTS=true
            shift
            ;;
        -c|--compare)
            COMPARE=true
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
        -t|--tests)
            SPECIFIC_TESTS="$2"
            shift
            shift
            ;;
        --config)
            CONFIG_PATH="$2"
            shift
            shift
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            ;;
    esac
done

# Validate deployment scenario
if [[ "$DEPLOYMENT" != "single-node" && "$DEPLOYMENT" != "distributed" && "$DEPLOYMENT" != "cloud" && "$DEPLOYMENT" != "local-dev" ]]; then
    echo "Error: Invalid deployment scenario: $DEPLOYMENT"
    echo "Valid scenarios: single-node, distributed, cloud, local-dev"
    exit 1
fi

# Validate format
if [[ "$FORMAT" != "json" && "$FORMAT" != "markdown" && "$FORMAT" != "html" ]]; then
    echo "Error: Invalid format: $FORMAT"
    echo "Valid formats: json, markdown, html"
    exit 1
fi

# Create output directory if it doesn't exist
mkdir -p "$OUTPUT_DIR"

# Generate timestamp for report filename
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
REPORT_FILENAME="audit_report_${DEPLOYMENT}_${TIMESTAMP}.${FORMAT}"
REPORT_PATH="${OUTPUT_DIR}/${REPORT_FILENAME}"

# Build command
CMD="cargo run -- audit run --deployment $DEPLOYMENT"

# Add options
if [[ "$DASHBOARD" == true ]]; then
    CMD="$CMD --dashboard --dashboard-host $DASHBOARD_HOST --dashboard-port $DASHBOARD_PORT"
fi

CMD="$CMD --output $REPORT_PATH --format $FORMAT --config $CONFIG_PATH"

if [[ "$STORE_RESULTS" == true ]]; then
    CMD="$CMD --store-results"
fi

if [[ "$COMPARE" == true ]]; then
    CMD="$CMD --compare"
fi

if [[ "$VERBOSE" == true ]]; then
    CMD="$CMD --verbose"
fi

if [[ "$CI_MODE" == true ]]; then
    CMD="$CMD --ci"
fi

if [[ -n "$SPECIFIC_TESTS" ]]; then
    CMD="$CMD --tests $SPECIFIC_TESTS"
fi

# Print configuration
echo "IntelliRouter Audit System Runner"
echo "================================="
echo "Deployment scenario: $DEPLOYMENT"
echo "Dashboard: $DASHBOARD"
if [[ "$DASHBOARD" == true ]]; then
    echo "Dashboard URL: http://$DASHBOARD_HOST:$DASHBOARD_PORT"
fi
echo "Report path: $REPORT_PATH"
echo "Report format: $FORMAT"
echo "Store results: $STORE_RESULTS"
echo "Compare with previous: $COMPARE"
echo "Verbose output: $VERBOSE"
echo "CI mode: $CI_MODE"
if [[ -n "$SPECIFIC_TESTS" ]]; then
    echo "Specific tests: $SPECIFIC_TESTS"
fi
echo "Configuration file: $CONFIG_PATH"
echo ""
echo "Command: $CMD"
echo ""

# Confirm execution
if [[ "$CI_MODE" != true ]]; then
    read -p "Do you want to proceed? (y/n) " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted."
        exit 1
    fi
fi

# Run the command
echo "Running audit system..."
echo ""
eval "$CMD"

# Check exit code
EXIT_CODE=$?
if [[ $EXIT_CODE -eq 0 ]]; then
    echo ""
    echo "Audit completed successfully."
    echo "Report saved to: $REPORT_PATH"
    
    # Open report if dashboard is not enabled
    if [[ "$DASHBOARD" != true && "$CI_MODE" != true ]]; then
        if [[ "$FORMAT" == "html" ]]; then
            if command -v xdg-open &> /dev/null; then
                xdg-open "$REPORT_PATH"
            elif command -v open &> /dev/null; then
                open "$REPORT_PATH"
            else
                echo "Report is available at: $REPORT_PATH"
            fi
        else
            echo "Report is available at: $REPORT_PATH"
        fi
    fi
else
    echo ""
    echo "Audit failed with exit code: $EXIT_CODE"
    echo "Check the logs for details."
fi

exit $EXIT_CODE