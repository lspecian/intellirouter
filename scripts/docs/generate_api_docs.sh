#!/bin/bash
# API documentation generator for IntelliRouter
# This script generates API documentation using rustdoc for Rust code and sphinx for Python SDK

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
API_DOCS_DIR="$DOCS_DIR/api"
RUST_DOCS_DIR="$API_DOCS_DIR/rust"
PYTHON_DOCS_DIR="$API_DOCS_DIR/python"
TS_DOCS_DIR="$API_DOCS_DIR/typescript"

# Create directories if they don't exist
mkdir -p "$RUST_DOCS_DIR" "$PYTHON_DOCS_DIR" "$TS_DOCS_DIR"

# Function to count documented items
count_documented_items() {
    local total=0
    local documented=0
    
    # Count total public items and documented public items
    while IFS= read -r line; do
        if [[ "$line" =~ ^pub\ (struct|enum|trait|fn|type|mod|const|static|use) ]]; then
            ((total++))
            
            # Check if the previous line contains documentation
            if [[ "${prev_line}" =~ ^///.*$ ]]; then
                ((documented++))
            fi
        fi
        prev_line="$line"
    done < <(find "$PROJECT_ROOT/src" -name "*.rs" -type f -exec cat {} \;)
    
    # Calculate coverage percentage
    if [ "$total" -gt 0 ]; then
        coverage=$(echo "scale=2; ($documented / $total) * 100" | bc)
    else
        coverage="0.0"
    fi
    
    echo "Total public items: $total"
    echo "Documented public items: $documented"
    echo "Coverage: $coverage%"
    
    # Return the coverage percentage
    echo "$coverage"
}

# Generate Rust API documentation
echo -e "${YELLOW}Generating Rust API documentation...${NC}"
cargo doc --no-deps --document-private-items --target-dir "$RUST_DOCS_DIR"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Rust API documentation generated successfully${NC}"
else
    echo -e "${RED}✗ Failed to generate Rust API documentation${NC}"
    exit 1
fi

# Generate Python SDK documentation
echo -e "${YELLOW}Generating Python SDK documentation...${NC}"
cd "$DOCS_DIR"
make html BUILDDIR="$PYTHON_DOCS_DIR"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Python SDK documentation generated successfully${NC}"
else
    echo -e "${RED}✗ Failed to generate Python SDK documentation${NC}"
    exit 1
fi

# Generate TypeScript SDK documentation
echo -e "${YELLOW}Generating TypeScript SDK documentation...${NC}"
if [ -f "$PROJECT_ROOT/sdk/typescript/package.json" ]; then
    cd "$PROJECT_ROOT/sdk/typescript"
    if command -v typedoc &> /dev/null; then
        npx typedoc --out "$TS_DOCS_DIR" src/
        
        if [ $? -eq 0 ]; then
            echo -e "${GREEN}✓ TypeScript SDK documentation generated successfully${NC}"
        else
            echo -e "${RED}✗ Failed to generate TypeScript SDK documentation${NC}"
            exit 1
        fi
    else
        echo -e "${RED}✗ typedoc not found. Please install it with 'npm install -g typedoc'${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}⚠ TypeScript SDK not found. Skipping TypeScript documentation generation.${NC}"
fi

# Calculate documentation coverage
echo -e "${YELLOW}Calculating API documentation coverage...${NC}"
coverage=$(count_documented_items)

# Create index.html file
echo -e "${YELLOW}Creating API documentation index...${NC}"
cat > "$API_DOCS_DIR/index.html" << EOF
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>IntelliRouter API Documentation</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            line-height: 1.6;
            margin: 0;
            padding: 20px;
            color: #333;
        }
        h1, h2 {
            color: #0066cc;
        }
        .container {
            max-width: 1200px;
            margin: 0 auto;
        }
        .card {
            border: 1px solid #ddd;
            border-radius: 4px;
            padding: 20px;
            margin-bottom: 20px;
            background-color: #f9f9f9;
        }
        .coverage {
            display: inline-block;
            padding: 5px 10px;
            border-radius: 4px;
            font-weight: bold;
            color: white;
            background-color: ${coverage >= 80 ? '#4CAF50' : coverage >= 60 ? '#FFC107' : '#F44336'};
        }
        a {
            color: #0066cc;
            text-decoration: none;
        }
        a:hover {
            text-decoration: underline;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>IntelliRouter API Documentation</h1>
        <p>Generated on $(date)</p>
        
        <div class="card">
            <h2>Documentation Coverage</h2>
            <p>Current API documentation coverage: <span class="coverage">${coverage}%</span></p>
        </div>
        
        <div class="card">
            <h2>Available Documentation</h2>
            <ul>
                <li><a href="./rust/doc/intellirouter/index.html">Rust API Documentation</a></li>
                <li><a href="./python/html/index.html">Python SDK Documentation</a></li>
                <li><a href="./typescript/index.html">TypeScript SDK Documentation</a></li>
            </ul>
        </div>
    </div>
</body>
</html>
EOF

echo "Coverage: $coverage%"