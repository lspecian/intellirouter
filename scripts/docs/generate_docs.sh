#!/bin/bash
# Main documentation generation script for IntelliRouter
# This script coordinates the generation of different types of documentation and collects metrics

set -e

# Define colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Define paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
DOCS_DIR="$PROJECT_ROOT/docs"
METRICS_DIR="$PROJECT_ROOT/metrics/docs"
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")
METRICS_FILE="$METRICS_DIR/doc_metrics_$TIMESTAMP.json"

# Create metrics directory if it doesn't exist
mkdir -p "$METRICS_DIR"

# Print header
echo -e "${BLUE}=========================================${NC}"
echo -e "${BLUE}IntelliRouter Documentation Generator${NC}"
echo -e "${BLUE}=========================================${NC}"
echo

# Initialize metrics
echo "{" > "$METRICS_FILE"
echo "  \"timestamp\": \"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\"," >> "$METRICS_FILE"
echo "  \"metrics\": {" >> "$METRICS_FILE"

# Function to run a documentation generator and collect metrics
run_generator() {
    local generator_name="$1"
    local generator_script="$2"
    local metrics_key="$3"
    
    echo -e "${YELLOW}Generating $generator_name documentation...${NC}"
    
    # Run the generator script and capture its output
    if output=$("$SCRIPT_DIR/$generator_script" 2>&1); then
        echo -e "${GREEN}✓ $generator_name documentation generated successfully${NC}"
        success=true
    else
        echo -e "${RED}✗ $generator_name documentation generation failed${NC}"
        echo -e "${RED}Error: $output${NC}"
        success=false
    fi
    
    # Extract metrics from the output
    if [[ "$output" =~ Coverage:\ ([0-9]+(\.[0-9]+)?)% ]]; then
        coverage="${BASH_REMATCH[1]}"
    else
        coverage="0.0"
    fi
    
    # Add metrics to the metrics file
    echo "    \"$metrics_key\": {" >> "$METRICS_FILE"
    echo "      \"success\": $success," >> "$METRICS_FILE"
    echo "      \"coverage\": $coverage," >> "$METRICS_FILE"
    echo "      \"timestamp\": \"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\"" >> "$METRICS_FILE"
    echo "    }," >> "$METRICS_FILE"
    
    echo
}

# Run documentation generators
run_generator "API" "generate_api_docs.sh" "api_docs"
run_generator "User Guides" "generate_user_guides.sh" "user_guides"
run_generator "Architecture" "generate_architecture_docs.sh" "architecture_docs"
run_generator "Examples" "generate_examples_docs.sh" "examples_docs"

# Fix the trailing comma in the metrics file
sed -i '$ s/,$//' "$METRICS_FILE"

# Close the metrics file
echo "  }" >> "$METRICS_FILE"
echo "}" >> "$METRICS_FILE"

# Run documentation coverage check
echo -e "${YELLOW}Checking documentation coverage...${NC}"
"$SCRIPT_DIR/check_doc_coverage.sh"

# Generate documentation report
echo -e "${YELLOW}Generating documentation report...${NC}"
"$SCRIPT_DIR/generate_doc_report.sh" "$METRICS_FILE"

echo -e "${BLUE}=========================================${NC}"
echo -e "${GREEN}Documentation generation complete${NC}"
echo -e "${BLUE}Documentation metrics saved to:${NC} $METRICS_FILE"
echo -e "${BLUE}Documentation report available at:${NC} $METRICS_DIR/doc_report_$TIMESTAMP.html"
echo -e "${BLUE}=========================================${NC}"