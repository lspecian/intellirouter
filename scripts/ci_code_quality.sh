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

echo -e "${BOLD}${BLUE}IntelliRouter CI Code Quality Check${RESET}"
echo -e "${CYAN}===================================${RESET}\n"

# Create metrics directory if it doesn't exist
METRICS_DIR="metrics"
if [ ! -d "$METRICS_DIR" ]; then
    echo -e "${YELLOW}Creating metrics directory...${RESET}"
    mkdir -p "$METRICS_DIR"
fi

# Generate code quality report
echo -e "${YELLOW}Generating code quality report...${RESET}"
./scripts/generate_code_quality_report.sh --format=json --output="$METRICS_DIR/ci_report.json"

# Generate metrics charts
echo -e "${YELLOW}Generating metrics charts...${RESET}"
./scripts/generate_metrics_charts.sh --format=html

# Check if we're running in CI
if [ -n "$CI" ]; then
    echo -e "${YELLOW}Running in CI environment. Checking against quality goals...${RESET}"
    
    # Load the current metrics
    CURRENT_REPORT="$METRICS_DIR/ci_report.json"
    
    if [ ! -f "$CURRENT_REPORT" ]; then
        echo -e "${RED}Error: Report file not found at $CURRENT_REPORT${RESET}"
        exit 1
    fi
    
    # Extract metrics
    TOTAL_WARNINGS=$(grep -o '"total_warnings":[0-9]*' "$CURRENT_REPORT" | cut -d':' -f2)
    TEST_COVERAGE=$(grep -o '"test_coverage":[0-9.]*' "$CURRENT_REPORT" | cut -d':' -f2)
    DOC_COVERAGE=$(grep -o '"documentation_coverage":[0-9.]*' "$CURRENT_REPORT" | cut -d':' -f2)
    WARNING_DENSITY=$(grep -o '"warning_density":[0-9.]*' "$CURRENT_REPORT" | cut -d':' -f2)
    
    # Load goals from code_quality_goals.md
    GOALS_FILE="docs/code_quality_goals.md"
    
    if [ -f "$GOALS_FILE" ]; then
        echo -e "${YELLOW}Loading quality goals from $GOALS_FILE...${RESET}"
        
        # Extract goals
        WARNING_GOAL=$(grep -A 10 "## Short-term Goals" "$GOALS_FILE" | grep "Total Warnings:" | grep -o "[0-9]*")
        COVERAGE_GOAL=$(grep -A 10 "## Short-term Goals" "$GOALS_FILE" | grep "Test Coverage:" | grep -o "[0-9]*")
        DOC_COVERAGE_GOAL=$(grep -A 10 "## Short-term Goals" "$GOALS_FILE" | grep "Documentation Coverage:" | grep -o "[0-9]*")
        DENSITY_GOAL=$(grep -A 10 "## Short-term Goals" "$GOALS_FILE" | grep "Warning Density:" | grep -o "[0-9.]*")
        
        # Check if goals are met
        FAILURES=0
        
        echo -e "\n${BOLD}Checking metrics against goals:${RESET}"
        
        if [ -n "$WARNING_GOAL" ] && [ "$TOTAL_WARNINGS" -gt "$WARNING_GOAL" ]; then
            echo -e "${RED}✗ Total Warnings: $TOTAL_WARNINGS (Goal: $WARNING_GOAL)${RESET}"
            FAILURES=$((FAILURES + 1))
        else
            echo -e "${GREEN}✓ Total Warnings: $TOTAL_WARNINGS (Goal: $WARNING_GOAL)${RESET}"
        fi
        
        if [ -n "$COVERAGE_GOAL" ] && (( $(echo "$TEST_COVERAGE < $COVERAGE_GOAL" | bc -l) )); then
            echo -e "${RED}✗ Test Coverage: $TEST_COVERAGE% (Goal: $COVERAGE_GOAL%)${RESET}"
            FAILURES=$((FAILURES + 1))
        else
            echo -e "${GREEN}✓ Test Coverage: $TEST_COVERAGE% (Goal: $COVERAGE_GOAL%)${RESET}"
        fi
        
        if [ -n "$DOC_COVERAGE_GOAL" ] && (( $(echo "$DOC_COVERAGE < $DOC_COVERAGE_GOAL" | bc -l) )); then
            echo -e "${RED}✗ Documentation Coverage: $DOC_COVERAGE% (Goal: $DOC_COVERAGE_GOAL%)${RESET}"
            FAILURES=$((FAILURES + 1))
        else
            echo -e "${GREEN}✓ Documentation Coverage: $DOC_COVERAGE% (Goal: $DOC_COVERAGE_GOAL%)${RESET}"
        fi
        
        if [ -n "$DENSITY_GOAL" ] && (( $(echo "$WARNING_DENSITY > $DENSITY_GOAL" | bc -l) )); then
            echo -e "${RED}✗ Warning Density: $WARNING_DENSITY (Goal: $DENSITY_GOAL)${RESET}"
            FAILURES=$((FAILURES + 1))
        else
            echo -e "${GREEN}✓ Warning Density: $WARNING_DENSITY (Goal: $DENSITY_GOAL)${RESET}"
        fi
        
        # Determine if we should fail the build
        if [ "$FAILURES" -gt 0 ]; then
            echo -e "\n${RED}${BOLD}Code quality check failed with $FAILURES metric(s) not meeting goals.${RESET}"
            
            # Check if we should enforce goals
            ENFORCE_GOALS=$(grep -A 10 "## Configuration" "$GOALS_FILE" | grep "Enforce Goals:" | grep -o "true\|false")
            
            if [ "$ENFORCE_GOALS" = "true" ]; then
                echo -e "${RED}Goals enforcement is enabled. CI build will fail.${RESET}"
                echo -e "${YELLOW}See $GOALS_FILE for more information on quality goals.${RESET}"
                exit 1
            else
                echo -e "${YELLOW}Goals enforcement is disabled. CI build will continue despite quality issues.${RESET}"
                echo -e "${YELLOW}See $GOALS_FILE for more information on quality goals.${RESET}"
            fi
        else
            echo -e "\n${GREEN}${BOLD}All code quality metrics meet or exceed goals!${RESET}"
        fi
    else
        echo -e "${YELLOW}Warning: Goals file not found at $GOALS_FILE. Skipping goals check.${RESET}"
    fi
    
    # Archive the metrics for CI artifacts
    if [ -d "$METRICS_DIR/charts" ]; then
        echo -e "${YELLOW}Preparing metrics artifacts for CI...${RESET}"
        
        # Create a directory for artifacts if it doesn't exist
        ARTIFACTS_DIR="artifacts"
        if [ ! -d "$ARTIFACTS_DIR" ]; then
            mkdir -p "$ARTIFACTS_DIR"
        fi
        
        # Create a zip file with all metrics and charts
        ZIP_FILE="$ARTIFACTS_DIR/code_quality_metrics.zip"
        zip -r "$ZIP_FILE" "$METRICS_DIR" > /dev/null
        
        echo -e "${GREEN}Metrics artifacts prepared at $ZIP_FILE${RESET}"
    fi
else
    echo -e "${YELLOW}Not running in CI environment. Skipping goals check.${RESET}"
fi

echo -e "\n${GREEN}${BOLD}Code quality check completed!${RESET}"

# Make the script executable
chmod +x "$0"