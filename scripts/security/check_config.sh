#!/bin/bash
# IntelliRouter Configuration Security Check
# This script checks for security issues in configuration files

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
    echo "IntelliRouter Configuration Security Check"
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
REPORT_FILENAME="config_security_audit_${TIMESTAMP}.${FORMAT}"
REPORT_PATH="${OUTPUT_DIR}/${REPORT_FILENAME}"

# Print configuration if verbose
if [[ "$VERBOSE" == true ]]; then
    echo "IntelliRouter Configuration Security Check"
    echo "=========================================="
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

# Initialize issue counters
CRITICAL_COUNT=0
HIGH_COUNT=0
MEDIUM_COUNT=0
LOW_COUNT=0
TOTAL_COUNT=0

# Initialize issues array
ISSUES=()

# Function to add an issue
add_issue() {
    local severity="$1"
    local file="$2"
    local issue="$3"
    local description="$4"
    local recommendation="$5"
    
    # Increment the appropriate counter
    case "$severity" in
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
    
    # Add to total count
    TOTAL_COUNT=$((TOTAL_COUNT + 1))
    
    # Create issue JSON
    local issue_json=$(jq -n \
        --arg severity "$severity" \
        --arg file "$file" \
        --arg issue "$issue" \
        --arg description "$description" \
        --arg recommendation "$recommendation" \
        '{
            "severity": $severity,
            "file": $file,
            "issue": $issue,
            "description": $description,
            "recommendation": $recommendation
        }')
    
    # Add to issues array
    ISSUES+=("$issue_json")
}

echo "Checking configuration files for security issues..."

# Check for hardcoded secrets in configuration files
echo "Checking for hardcoded secrets..."

# Define patterns to search for
SECRET_PATTERNS=(
    "password"
    "secret"
    "token"
    "key"
    "credential"
    "api_key"
    "apikey"
    "auth"
)

# Check TOML configuration files
for config_file in $(find config -name "*.toml"); do
    if [[ "$VERBOSE" == true ]]; then
        echo "Checking $config_file..."
    fi
    
    # Check for hardcoded secrets
    for pattern in "${SECRET_PATTERNS[@]}"; do
        # Look for pattern followed by = and a string literal
        matches=$(grep -i -E "${pattern}.*=.*\"[^\"]+\"" "$config_file" || true)
        if [[ -n "$matches" ]]; then
            # Check if it's an environment variable reference
            if ! echo "$matches" | grep -q "\${.*}"; then
                add_issue "high" "$config_file" "Hardcoded Secret" \
                    "Found potential hardcoded secret: $pattern" \
                    "Use environment variables or a secure secret management solution instead of hardcoding secrets."
                
                # Fix the issue if requested
                if [[ "$FIX_ISSUES" == true ]]; then
                    # Replace with environment variable reference
                    sed -i -E "s/(${pattern}.*=.*\"[^\"]+\")/\1 # TODO: Replace with environment variable/" "$config_file"
                fi
            fi
        fi
    done
    
    # Check for insecure defaults
    if grep -q -i "ssl_verify.*=.*false" "$config_file"; then
        add_issue "high" "$config_file" "SSL Verification Disabled" \
            "SSL verification is disabled, which makes connections vulnerable to MITM attacks." \
            "Enable SSL verification in production environments."
    fi
    
    if grep -q -i "debug.*=.*true" "$config_file"; then
        add_issue "medium" "$config_file" "Debug Mode Enabled" \
            "Debug mode is enabled, which may expose sensitive information." \
            "Disable debug mode in production environments."
    fi
    
    # Check for missing required security settings
    if ! grep -q -i "timeout" "$config_file"; then
        add_issue "medium" "$config_file" "Missing Timeout Configuration" \
            "No timeout configuration found, which may lead to resource exhaustion." \
            "Add appropriate timeout settings for all connections and operations."
    fi
    
    if ! grep -q -i "rate_limit" "$config_file" && ! grep -q -i "throttle" "$config_file"; then
        add_issue "medium" "$config_file" "Missing Rate Limiting" \
            "No rate limiting configuration found, which may make the system vulnerable to DoS attacks." \
            "Add rate limiting for all public-facing endpoints."
    fi
done

# Check for .env files with sensitive information
if [[ -f ".env" ]]; then
    add_issue "medium" ".env" "Environment File in Repository" \
        "Found .env file which may contain sensitive information." \
        "Add .env to .gitignore and use .env.example with placeholder values instead."
fi

# Check Docker configuration for security issues
if [[ -f "Dockerfile" ]]; then
    # Check if running as root
    if ! grep -q "USER" "Dockerfile" || grep -q "USER root" "Dockerfile"; then
        add_issue "medium" "Dockerfile" "Running as Root" \
            "Container is running as root, which is a security risk." \
            "Add a non-root user and use the USER directive to run as that user."
    fi
    
    # Check for latest tag
    if grep -q "FROM.*:latest" "Dockerfile"; then
        add_issue "low" "Dockerfile" "Using Latest Tag" \
            "Using the 'latest' tag for base images, which may lead to unexpected changes." \
            "Use specific version tags for base images to ensure reproducibility and security."
    fi
    
    # Check for sensitive information
    for pattern in "${SECRET_PATTERNS[@]}"; do
        if grep -i -q "$pattern" "Dockerfile"; then
            add_issue "high" "Dockerfile" "Sensitive Information in Dockerfile" \
                "Found potential sensitive information in Dockerfile: $pattern" \
                "Use build arguments or environment variables instead of hardcoding sensitive information."
        fi
    done
fi

# Check docker-compose files
for compose_file in $(find . -name "docker-compose*.yml"); do
    # Check for exposed ports
    if grep -q "ports:" "$compose_file"; then
        add_issue "low" "$compose_file" "Exposed Ports" \
            "Services have exposed ports, which may increase attack surface." \
            "Limit exposed ports to only those necessary and consider using internal networks."
    fi
    
    # Check for sensitive information
    for pattern in "${SECRET_PATTERNS[@]}"; do
        if grep -i -q "$pattern" "$compose_file"; then
            add_issue "high" "$compose_file" "Sensitive Information in Docker Compose" \
                "Found potential sensitive information in Docker Compose file: $pattern" \
                "Use environment variables or secrets management instead of hardcoding sensitive information."
        fi
    done
    
    # Check for privileged mode
    if grep -q "privileged: true" "$compose_file"; then
        add_issue "high" "$compose_file" "Privileged Mode Enabled" \
            "Container is running in privileged mode, which gives it full access to the host." \
            "Avoid using privileged mode and use specific capabilities instead if needed."
    fi
end

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
            --argjson issues "$(printf '%s\n' "${ISSUES[@]}" | jq -s '.')" \
            '{
                "timestamp": $timestamp,
                "check_type": "config",
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
            echo "# IntelliRouter Configuration Security Audit"
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
            echo "## Issues"
            echo ""
            
            # Process each issue
            for issue_json in "${ISSUES[@]}"; do
                SEVERITY=$(echo "$issue_json" | jq -r '.severity')
                FILE=$(echo "$issue_json" | jq -r '.file')
                ISSUE=$(echo "$issue_json" | jq -r '.issue')
                DESCRIPTION=$(echo "$issue_json" | jq -r '.description')
                RECOMMENDATION=$(echo "$issue_json" | jq -r '.recommendation')
                
                echo "### $ISSUE ($SEVERITY)"
                echo ""
                echo "**File:** $FILE"
                echo ""
                echo "**Description:** $DESCRIPTION"
                echo ""
                echo "**Recommendation:** $RECOMMENDATION"
                echo ""
            done
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
            echo "  <title>IntelliRouter Configuration Security Audit</title>"
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
            echo "  <h1>IntelliRouter Configuration Security Audit</h1>"
            echo "  <p><strong>Date:</strong> $(date)</p>"
            echo "  <h2>Summary</h2>"
            echo "  <div class=\"summary\">"
            echo "    <div class=\"summary-item\"><strong>Total issues:</strong> $TOTAL_COUNT</div>"
            echo "    <div class=\"summary-item\"><strong>Critical issues:</strong> <span class=\"critical\">$CRITICAL_COUNT</span></div>"
            echo "    <div class=\"summary-item\"><strong>High issues:</strong> <span class=\"high\">$HIGH_COUNT</span></div>"
            echo "    <div class=\"summary-item\"><strong>Medium issues:</strong> <span class=\"medium\">$MEDIUM_COUNT</span></div>"
            echo "    <div class=\"summary-item\"><strong>Low issues:</strong> <span class=\"low\">$LOW_COUNT</span></div>"
            echo "  </div>"
            echo "  <h2>Issues</h2>"
            
            # Process each issue
            for issue_json in "${ISSUES[@]}"; do
                SEVERITY=$(echo "$issue_json" | jq -r '.severity')
                FILE=$(echo "$issue_json" | jq -r '.file')
                ISSUE=$(echo "$issue_json" | jq -r '.issue')
                DESCRIPTION=$(echo "$issue_json" | jq -r '.description')
                RECOMMENDATION=$(echo "$issue_json" | jq -r '.recommendation')
                
                echo "  <div class=\"issue issue-$SEVERITY\">"
                echo "    <h3>$ISSUE <span class=\"$SEVERITY\">($SEVERITY)</span></h3>"
                echo "    <p><strong>File:</strong> $FILE</p>"
                echo "    <p><strong>Description:</strong> $DESCRIPTION</p>"
                echo "    <p><strong>Recommendation:</strong> $RECOMMENDATION</p>"
                echo "  </div>"
            done
            
            echo "</body>"
            echo "</html>"
        } > "$REPORT_PATH"
        ;;
esac

# Print summary
echo "Configuration Security Check Summary"
echo "===================================="
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