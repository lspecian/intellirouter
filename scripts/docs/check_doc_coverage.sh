#!/bin/bash
# Documentation coverage check script for IntelliRouter
# This script analyzes the codebase and checks for documentation coverage

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
METRICS_DIR="$PROJECT_ROOT/metrics/docs"
TIMESTAMP=$(date +"%Y-%m-%d_%H-%M-%S")
COVERAGE_FILE="$METRICS_DIR/doc_coverage_$TIMESTAMP.json"

# Create metrics directory if it doesn't exist
mkdir -p "$METRICS_DIR"

# Print header
echo -e "${BLUE}=========================================${NC}"
echo -e "${BLUE}IntelliRouter Documentation Coverage Check${NC}"
echo -e "${BLUE}=========================================${NC}"
echo

# Initialize coverage file
echo "{" > "$COVERAGE_FILE"
echo "  \"timestamp\": \"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\"," >> "$COVERAGE_FILE"
echo "  \"coverage\": {" >> "$COVERAGE_FILE"

# Function to check Rust documentation coverage
check_rust_doc_coverage() {
    echo -e "${YELLOW}Checking Rust documentation coverage...${NC}"
    
    local total_items=0
    local documented_items=0
    local undocumented_items=()
    
    # Find all Rust files
    find "$PROJECT_ROOT/src" -name "*.rs" | while read -r file; do
        # Get relative path
        rel_path="${file#$PROJECT_ROOT/}"
        
        # Count public items
        while IFS= read -r line; do
            if [[ "$line" =~ ^pub\ (struct|enum|trait|fn|type|mod|const|static|use) ]]; then
                ((total_items++))
                
                # Extract item name
                if [[ "$line" =~ ^pub\ (struct|enum|trait|fn|type|mod|const|static|use)\ ([a-zA-Z0-9_]+) ]]; then
                    item_type="${BASH_REMATCH[1]}"
                    item_name="${BASH_REMATCH[2]}"
                    
                    # Check if the previous line contains documentation
                    if [[ "${prev_line}" =~ ^///.*$ ]]; then
                        ((documented_items++))
                    else
                        undocumented_items+=("$rel_path: $item_type $item_name")
                    fi
                fi
            fi
            prev_line="$line"
        done < "$file"
    done
    
    # Calculate coverage percentage
    if [ "$total_items" -gt 0 ]; then
        coverage=$(echo "scale=2; ($documented_items / $total_items) * 100" | bc)
    else
        coverage="0.0"
    fi
    
    # Print results
    echo -e "${GREEN}Total public items: $total_items${NC}"
    echo -e "${GREEN}Documented public items: $documented_items${NC}"
    echo -e "${GREEN}Documentation coverage: $coverage%${NC}"
    
    # Print undocumented items
    if [ ${#undocumented_items[@]} -gt 0 ]; then
        echo -e "${YELLOW}Undocumented items:${NC}"
        for item in "${undocumented_items[@]}"; do
            echo -e "${YELLOW}  - $item${NC}"
        done
    fi
    
    # Add to coverage file
    echo "    \"rust\": {" >> "$COVERAGE_FILE"
    echo "      \"total_items\": $total_items," >> "$COVERAGE_FILE"
    echo "      \"documented_items\": $documented_items," >> "$COVERAGE_FILE"
    echo "      \"coverage_percentage\": $coverage," >> "$COVERAGE_FILE"
    echo "      \"undocumented_items\": [" >> "$COVERAGE_FILE"
    
    # Add undocumented items to coverage file
    for ((i=0; i<${#undocumented_items[@]}; i++)); do
        echo "        \"${undocumented_items[$i]}\"$([ $i -lt $((${#undocumented_items[@]}-1)) ] && echo ",")" >> "$COVERAGE_FILE"
    done
    
    echo "      ]" >> "$COVERAGE_FILE"
    echo "    }," >> "$COVERAGE_FILE"
    
    echo
}

# Function to check Python documentation coverage
check_python_doc_coverage() {
    echo -e "${YELLOW}Checking Python documentation coverage...${NC}"
    
    local total_items=0
    local documented_items=0
    local undocumented_items=()
    
    # Find all Python files
    find "$PROJECT_ROOT/sdk/python" -name "*.py" 2>/dev/null | while read -r file; do
        # Get relative path
        rel_path="${file#$PROJECT_ROOT/}"
        
        # Count classes and functions
        while IFS= read -r line; do
            if [[ "$line" =~ ^(class|def)\ ([a-zA-Z0-9_]+) ]]; then
                ((total_items++))
                
                # Extract item name
                item_type="${BASH_REMATCH[1]}"
                item_name="${BASH_REMATCH[2]}"
                
                # Check if the previous lines contain documentation
                if [[ "${prev_lines}" =~ \"\"\".*\"\"\" ]]; then
                    ((documented_items++))
                else
                    undocumented_items+=("$rel_path: $item_type $item_name")
                fi
            fi
            
            # Keep track of previous lines for docstring detection
            prev_lines="$prev_lines
$line"
            # Keep only the last 10 lines
            prev_lines=$(echo "$prev_lines" | tail -n 10)
        done < "$file"
    done
    
    # Calculate coverage percentage
    if [ "$total_items" -gt 0 ]; then
        coverage=$(echo "scale=2; ($documented_items / $total_items) * 100" | bc)
    else
        coverage="0.0"
    fi
    
    # Print results
    echo -e "${GREEN}Total classes and functions: $total_items${NC}"
    echo -e "${GREEN}Documented classes and functions: $documented_items${NC}"
    echo -e "${GREEN}Documentation coverage: $coverage%${NC}"
    
    # Print undocumented items
    if [ ${#undocumented_items[@]} -gt 0 ]; then
        echo -e "${YELLOW}Undocumented items:${NC}"
        for item in "${undocumented_items[@]}"; do
            echo -e "${YELLOW}  - $item${NC}"
        done
    fi
    
    # Add to coverage file
    echo "    \"python\": {" >> "$COVERAGE_FILE"
    echo "      \"total_items\": $total_items," >> "$COVERAGE_FILE"
    echo "      \"documented_items\": $documented_items," >> "$COVERAGE_FILE"
    echo "      \"coverage_percentage\": $coverage," >> "$COVERAGE_FILE"
    echo "      \"undocumented_items\": [" >> "$COVERAGE_FILE"
    
    # Add undocumented items to coverage file
    for ((i=0; i<${#undocumented_items[@]}; i++)); do
        echo "        \"${undocumented_items[$i]}\"$([ $i -lt $((${#undocumented_items[@]}-1)) ] && echo ",")" >> "$COVERAGE_FILE"
    done
    
    echo "      ]" >> "$COVERAGE_FILE"
    echo "    }," >> "$COVERAGE_FILE"
    
    echo
}

# Function to check TypeScript documentation coverage
check_typescript_doc_coverage() {
    echo -e "${YELLOW}Checking TypeScript documentation coverage...${NC}"
    
    local total_items=0
    local documented_items=0
    local undocumented_items=()
    
    # Find all TypeScript files
    find "$PROJECT_ROOT/sdk/typescript" -name "*.ts" 2>/dev/null | while read -r file; do
        # Get relative path
        rel_path="${file#$PROJECT_ROOT/}"
        
        # Count classes, interfaces, and functions
        while IFS= read -r line; do
            if [[ "$line" =~ ^(export\ )?(class|interface|function|const)\ ([a-zA-Z0-9_]+) ]]; then
                ((total_items++))
                
                # Extract item name
                item_type="${BASH_REMATCH[2]}"
                item_name="${BASH_REMATCH[3]}"
                
                # Check if the previous lines contain documentation
                if [[ "${prev_lines}" =~ /\*\*.*\*/ ]]; then
                    ((documented_items++))
                else
                    undocumented_items+=("$rel_path: $item_type $item_name")
                fi
            fi
            
            # Keep track of previous lines for JSDoc detection
            prev_lines="$prev_lines
$line"
            # Keep only the last 10 lines
            prev_lines=$(echo "$prev_lines" | tail -n 10)
        done < "$file"
    done
    
    # Calculate coverage percentage
    if [ "$total_items" -gt 0 ]; then
        coverage=$(echo "scale=2; ($documented_items / $total_items) * 100" | bc)
    else
        coverage="0.0"
    fi
    
    # Print results
    echo -e "${GREEN}Total classes, interfaces, and functions: $total_items${NC}"
    echo -e "${GREEN}Documented classes, interfaces, and functions: $documented_items${NC}"
    echo -e "${GREEN}Documentation coverage: $coverage%${NC}"
    
    # Print undocumented items
    if [ ${#undocumented_items[@]} -gt 0 ]; then
        echo -e "${YELLOW}Undocumented items:${NC}"
        for item in "${undocumented_items[@]}"; do
            echo -e "${YELLOW}  - $item${NC}"
        done
    fi
    
    # Add to coverage file
    echo "    \"typescript\": {" >> "$COVERAGE_FILE"
    echo "      \"total_items\": $total_items," >> "$COVERAGE_FILE"
    echo "      \"documented_items\": $documented_items," >> "$COVERAGE_FILE"
    echo "      \"coverage_percentage\": $coverage," >> "$COVERAGE_FILE"
    echo "      \"undocumented_items\": [" >> "$COVERAGE_FILE"
    
    # Add undocumented items to coverage file
    for ((i=0; i<${#undocumented_items[@]}; i++)); do
        echo "        \"${undocumented_items[$i]}\"$([ $i -lt $((${#undocumented_items[@]}-1)) ] && echo ",")" >> "$COVERAGE_FILE"
    done
    
    echo "      ]" >> "$COVERAGE_FILE"
    echo "    }," >> "$COVERAGE_FILE"
    
    echo
}

# Function to check Markdown documentation coverage
check_markdown_doc_coverage() {
    echo -e "${YELLOW}Checking Markdown documentation coverage...${NC}"
    
    local total_modules=0
    local documented_modules=0
    local undocumented_modules=()
    
    # Get list of modules from src/modules
    modules=$(find "$PROJECT_ROOT/src/modules" -maxdepth 1 -type d | grep -v "src/modules$" | xargs -n1 basename)
    
    # Count total modules
    total_modules=$(echo "$modules" | wc -l)
    
    # Check if each module has a corresponding Markdown file
    for module in $modules; do
        if [ -f "$PROJECT_ROOT/docs/$module.md" ]; then
            ((documented_modules++))
        else
            undocumented_modules+=("$module")
        fi
    done
    
    # Calculate coverage percentage
    if [ "$total_modules" -gt 0 ]; then
        coverage=$(echo "scale=2; ($documented_modules / $total_modules) * 100" | bc)
    else
        coverage="0.0"
    fi
    
    # Print results
    echo -e "${GREEN}Total modules: $total_modules${NC}"
    echo -e "${GREEN}Documented modules: $documented_modules${NC}"
    echo -e "${GREEN}Documentation coverage: $coverage%${NC}"
    
    # Print undocumented modules
    if [ ${#undocumented_modules[@]} -gt 0 ]; then
        echo -e "${YELLOW}Undocumented modules:${NC}"
        for module in "${undocumented_modules[@]}"; do
            echo -e "${YELLOW}  - $module${NC}"
        done
    fi
    
    # Add to coverage file
    echo "    \"markdown\": {" >> "$COVERAGE_FILE"
    echo "      \"total_modules\": $total_modules," >> "$COVERAGE_FILE"
    echo "      \"documented_modules\": $documented_modules," >> "$COVERAGE_FILE"
    echo "      \"coverage_percentage\": $coverage," >> "$COVERAGE_FILE"
    echo "      \"undocumented_modules\": [" >> "$COVERAGE_FILE"
    
    # Add undocumented modules to coverage file
    for ((i=0; i<${#undocumented_modules[@]}; i++)); do
        echo "        \"${undocumented_modules[$i]}\"$([ $i -lt $((${#undocumented_modules[@]}-1)) ] && echo ",")" >> "$COVERAGE_FILE"
    done
    
    echo "      ]" >> "$COVERAGE_FILE"
    echo "    }" >> "$COVERAGE_FILE"
    
    echo
}

# Run coverage checks
check_rust_doc_coverage
check_python_doc_coverage
check_typescript_doc_coverage
check_markdown_doc_coverage

# Close the coverage file
echo "  }" >> "$COVERAGE_FILE"
echo "}" >> "$COVERAGE_FILE"

# Print summary
echo -e "${BLUE}=========================================${NC}"
echo -e "${GREEN}Documentation coverage check complete${NC}"
echo -e "${BLUE}Coverage data saved to:${NC} $COVERAGE_FILE"
echo -e "${BLUE}=========================================${NC}"