#!/bin/bash
# User guides documentation generator for IntelliRouter
# This script generates user guides documentation from Markdown files

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
USER_GUIDES_DIR="$DOCS_DIR/user_guides"
USER_GUIDES_SRC_DIR="$DOCS_DIR/user_guides_src"

# Create directories if they don't exist
mkdir -p "$USER_GUIDES_DIR" "$USER_GUIDES_SRC_DIR"

# Check if pandoc is installed
if ! command -v pandoc &> /dev/null; then
    echo -e "${RED}Error: pandoc is not installed. Please install it to generate user guides.${NC}"
    exit 1
fi

# Function to count user guide coverage
count_user_guide_coverage() {
    local total_modules=0
    local documented_modules=0
    
    # Get list of modules from src/modules
    modules=$(find "$PROJECT_ROOT/src/modules" -maxdepth 1 -type d | grep -v "src/modules$" | xargs -n1 basename)
    
    # Count total modules
    total_modules=$(echo "$modules" | wc -l)
    
    # Count documented modules
    for module in $modules; do
        if [ -f "$USER_GUIDES_SRC_DIR/$module.md" ]; then
            ((documented_modules++))
        fi
    done
    
    # Calculate coverage percentage
    if [ "$total_modules" -gt 0 ]; then
        coverage=$(echo "scale=2; ($documented_modules / $total_modules) * 100" | bc)
    else
        coverage="0.0"
    fi
    
    echo "Total modules: $total_modules"
    echo "Documented modules: $documented_modules"
    echo "Coverage: $coverage%"
    
    # Return the coverage percentage
    echo "$coverage"
}

# Create user guides source directory structure if it doesn't exist
if [ ! -d "$USER_GUIDES_SRC_DIR/getting_started" ]; then
    echo -e "${YELLOW}Creating user guides source directory structure...${NC}"
    mkdir -p "$USER_GUIDES_SRC_DIR/getting_started"
    mkdir -p "$USER_GUIDES_SRC_DIR/configuration"
    mkdir -p "$USER_GUIDES_SRC_DIR/deployment"
    mkdir -p "$USER_GUIDES_SRC_DIR/modules"
    
    # Create template files if they don't exist
    if [ ! -f "$USER_GUIDES_SRC_DIR/index.md" ]; then
        cat > "$USER_GUIDES_SRC_DIR/index.md" << EOF
# IntelliRouter User Guides

Welcome to the IntelliRouter User Guides. These guides provide detailed information on how to use and configure IntelliRouter.

## Getting Started

- [Installation](getting_started/installation.md)
- [Quick Start](getting_started/quick_start.md)
- [Basic Concepts](getting_started/basic_concepts.md)

## Configuration

- [Configuration Overview](configuration/overview.md)
- [Environment Variables](configuration/environment_variables.md)
- [TOML Configuration](configuration/toml_configuration.md)

## Deployment

- [Local Development](deployment/local_development.md)
- [Docker Deployment](deployment/docker_deployment.md)
- [Kubernetes Deployment](deployment/kubernetes_deployment.md)
- [Edge Deployment](deployment/edge_deployment.md)

## Modules

EOF
        
        # Add module links to index.md
        for module in $modules; do
            echo "- [$module](modules/$module.md)" >> "$USER_GUIDES_SRC_DIR/index.md"
            
            # Create template module guide
            cat > "$USER_GUIDES_SRC_DIR/modules/$module.md" << EOF
# $module Module

## Overview

Description of the $module module.

## Configuration

How to configure the $module module.

## Usage

How to use the $module module.

## Examples

Examples of using the $module module.
EOF
        done
    fi
    
    # Create template getting started guides
    if [ ! -f "$USER_GUIDES_SRC_DIR/getting_started/installation.md" ]; then
        cat > "$USER_GUIDES_SRC_DIR/getting_started/installation.md" << EOF
# Installation

This guide provides instructions for installing IntelliRouter.

## Prerequisites

- Rust 1.70 or later
- Docker (for containerized deployment)
- Kubernetes (for production deployment)

## From Source

1. Clone the repository:
   \`\`\`bash
   git clone https://github.com/yourusername/intellirouter.git
   cd intellirouter
   \`\`\`

2. Build the project:
   \`\`\`bash
   cargo build --release
   \`\`\`

3. Run the router:
   \`\`\`bash
   ./target/release/intellirouter --role router
   \`\`\`

## Using Docker

1. Build the Docker image:
   \`\`\`bash
   docker build -t intellirouter .
   \`\`\`

2. Run the container:
   \`\`\`bash
   docker run -p 8000:8000 -e INTELLIROUTER_ROLE=router intellirouter
   \`\`\`

## Using Docker Compose

1. Start all services:
   \`\`\`bash
   docker-compose up -d
   \`\`\`
EOF
    fi
fi

# Generate HTML from Markdown
echo -e "${YELLOW}Generating HTML from Markdown...${NC}"
find "$USER_GUIDES_SRC_DIR" -name "*.md" | while read -r file; do
    # Get relative path
    rel_path="${file#$USER_GUIDES_SRC_DIR/}"
    # Create output directory
    output_dir="$USER_GUIDES_DIR/$(dirname "$rel_path")"
    mkdir -p "$output_dir"
    # Convert Markdown to HTML
    output_file="$USER_GUIDES_DIR/${rel_path%.md}.html"
    pandoc "$file" -f markdown -t html -s -o "$output_file" --metadata title="$(basename "${file%.md}")"
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ Generated $output_file${NC}"
    else
        echo -e "${RED}✗ Failed to generate $output_file${NC}"
        exit 1
    fi
done

# Create CSS file
cat > "$USER_GUIDES_DIR/style.css" << EOF
body {
    font-family: Arial, sans-serif;
    line-height: 1.6;
    margin: 0;
    padding: 20px;
    color: #333;
    max-width: 1200px;
    margin: 0 auto;
}

h1, h2, h3, h4, h5, h6 {
    color: #0066cc;
}

a {
    color: #0066cc;
    text-decoration: none;
}

a:hover {
    text-decoration: underline;
}

pre {
    background-color: #f5f5f5;
    padding: 10px;
    border-radius: 4px;
    overflow-x: auto;
}

code {
    font-family: 'Courier New', Courier, monospace;
    background-color: #f5f5f5;
    padding: 2px 4px;
    border-radius: 4px;
}

table {
    border-collapse: collapse;
    width: 100%;
    margin-bottom: 20px;
}

th, td {
    border: 1px solid #ddd;
    padding: 8px;
    text-align: left;
}

th {
    background-color: #f2f2f2;
}

tr:nth-child(even) {
    background-color: #f9f9f9;
}
EOF

# Add CSS to all HTML files
find "$USER_GUIDES_DIR" -name "*.html" | while read -r file; do
    sed -i 's/<\/head>/<link rel="stylesheet" href="\/style.css"><\/head>/' "$file"
done

# Calculate user guide coverage
echo -e "${YELLOW}Calculating user guide coverage...${NC}"
coverage=$(count_user_guide_coverage)

echo "Coverage: $coverage%"