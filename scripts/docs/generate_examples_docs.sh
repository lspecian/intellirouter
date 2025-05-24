#!/bin/bash
# Examples and tutorials documentation generator for IntelliRouter
# This script generates documentation for examples and tutorials

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
EXAMPLES_DOCS_DIR="$DOCS_DIR/examples"
EXAMPLES_DIR="$PROJECT_ROOT/examples"
TUTORIALS_DIR="$DOCS_DIR/tutorials"

# Create directories if they don't exist
mkdir -p "$EXAMPLES_DOCS_DIR" "$TUTORIALS_DIR"

# Check if pandoc is installed
if ! command -v pandoc &> /dev/null; then
    echo -e "${RED}Error: pandoc is not installed. Please install it to generate examples documentation.${NC}"
    exit 1
fi

# Function to count examples documentation coverage
count_examples_coverage() {
    local total_examples=0
    local documented_examples=0
    
    # Count total examples
    total_examples=$(find "$EXAMPLES_DIR" -type f -name "*.rs" -o -name "*.py" -o -name "*.ts" | wc -l)
    
    # Count documented examples
    documented_examples=$(find "$EXAMPLES_DOCS_DIR" -type f -name "*.html" | wc -l)
    
    # Calculate coverage percentage
    if [ "$total_examples" -gt 0 ]; then
        coverage=$(echo "scale=2; ($documented_examples / $total_examples) * 100" | bc)
    else
        coverage="0.0"
    fi
    
    echo "Total examples: $total_examples"
    echo "Documented examples: $documented_examples"
    echo "Coverage: $coverage%"
    
    # Return the coverage percentage
    echo "$coverage"
}

# Function to extract documentation from Rust examples
extract_rust_doc() {
    local file="$1"
    local output_file="$2"
    
    # Extract documentation comments
    awk '
    BEGIN { in_comment = 0; print "# " FILENAME; print ""; }
    /^\/\/\/ / {
        # Remove the comment marker and print the line
        gsub(/^\/\/\/ /, "");
        print;
        in_comment = 1;
    }
    /^\/\/\/$/ {
        # Empty comment line
        print "";
        in_comment = 1;
    }
    /^[^\/]/ && in_comment {
        # End of comment block
        in_comment = 0;
        print "";
        print "```rust";
    }
    /^[^\/]/ {
        # Code line
        print $0;
    }
    END {
        if (!in_comment) {
            print "```";
        }
    }
    ' "$file" > "$output_file"
}

# Function to extract documentation from Python examples
extract_python_doc() {
    local file="$1"
    local output_file="$2"
    
    # Extract documentation comments
    awk '
    BEGIN { in_comment = 0; print "# " FILENAME; print ""; }
    /^# / {
        # Remove the comment marker and print the line
        gsub(/^# /, "");
        print;
        in_comment = 1;
    }
    /^#$/ {
        # Empty comment line
        print "";
        in_comment = 1;
    }
    /^[^#]/ && in_comment {
        # End of comment block
        in_comment = 0;
        print "";
        print "```python";
    }
    /^[^#]/ {
        # Code line
        print $0;
    }
    END {
        if (!in_comment) {
            print "```";
        }
    }
    ' "$file" > "$output_file"
}

# Function to extract documentation from TypeScript examples
extract_typescript_doc() {
    local file="$1"
    local output_file="$2"
    
    # Extract documentation comments
    awk '
    BEGIN { in_comment = 0; print "# " FILENAME; print ""; }
    /^\/\/ / {
        # Remove the comment marker and print the line
        gsub(/^\/\/ /, "");
        print;
        in_comment = 1;
    }
    /^\/\/$/ {
        # Empty comment line
        print "";
        in_comment = 1;
    }
    /^[^\/]/ && in_comment {
        # End of comment block
        in_comment = 0;
        print "";
        print "```typescript";
    }
    /^[^\/]/ {
        # Code line
        print $0;
    }
    END {
        if (!in_comment) {
            print "```";
        }
    }
    ' "$file" > "$output_file"
}

# Generate examples documentation
echo -e "${YELLOW}Generating examples documentation...${NC}"

# Process Rust examples
find "$EXAMPLES_DIR" -name "*.rs" | while read -r file; do
    # Get relative path
    rel_path="${file#$EXAMPLES_DIR/}"
    # Create output directory
    output_dir="$EXAMPLES_DOCS_DIR/$(dirname "$rel_path")"
    mkdir -p "$output_dir"
    # Extract documentation
    temp_file=$(mktemp)
    extract_rust_doc "$file" "$temp_file"
    # Convert Markdown to HTML
    output_file="$EXAMPLES_DOCS_DIR/${rel_path%.rs}.html"
    pandoc "$temp_file" -f markdown -t html -s -o "$output_file" --metadata title="$(basename "${file%.rs}")"
    rm "$temp_file"
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ Generated $output_file${NC}"
    else
        echo -e "${RED}✗ Failed to generate $output_file${NC}"
        exit 1
    fi
done

# Process Python examples
find "$EXAMPLES_DIR" -name "*.py" | while read -r file; do
    # Get relative path
    rel_path="${file#$EXAMPLES_DIR/}"
    # Create output directory
    output_dir="$EXAMPLES_DOCS_DIR/$(dirname "$rel_path")"
    mkdir -p "$output_dir"
    # Extract documentation
    temp_file=$(mktemp)
    extract_python_doc "$file" "$temp_file"
    # Convert Markdown to HTML
    output_file="$EXAMPLES_DOCS_DIR/${rel_path%.py}.html"
    pandoc "$temp_file" -f markdown -t html -s -o "$output_file" --metadata title="$(basename "${file%.py}")"
    rm "$temp_file"
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ Generated $output_file${NC}"
    else
        echo -e "${RED}✗ Failed to generate $output_file${NC}"
        exit 1
    fi
done

# Process TypeScript examples
find "$EXAMPLES_DIR" -name "*.ts" | while read -r file; do
    # Get relative path
    rel_path="${file#$EXAMPLES_DIR/}"
    # Create output directory
    output_dir="$EXAMPLES_DOCS_DIR/$(dirname "$rel_path")"
    mkdir -p "$output_dir"
    # Extract documentation
    temp_file=$(mktemp)
    extract_typescript_doc "$file" "$temp_file"
    # Convert Markdown to HTML
    output_file="$EXAMPLES_DOCS_DIR/${rel_path%.ts}.html"
    pandoc "$temp_file" -f markdown -t html -s -o "$output_file" --metadata title="$(basename "${file%.ts}")"
    rm "$temp_file"
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ Generated $output_file${NC}"
    else
        echo -e "${RED}✗ Failed to generate $output_file${NC}"
        exit 1
    fi
done

# Create tutorials directory structure if it doesn't exist
if [ ! -d "$TUTORIALS_DIR/getting_started" ]; then
    echo -e "${YELLOW}Creating tutorials directory structure...${NC}"
    mkdir -p "$TUTORIALS_DIR/getting_started"
    mkdir -p "$TUTORIALS_DIR/advanced"
    
    # Create template tutorial files
    cat > "$TUTORIALS_DIR/index.md" << EOF
# IntelliRouter Tutorials

Welcome to the IntelliRouter tutorials. These tutorials provide step-by-step guides for common tasks.

## Getting Started

- [Setting Up IntelliRouter](getting_started/setup.md)
- [Creating Your First Router](getting_started/first_router.md)
- [Configuring Model Routing](getting_started/model_routing.md)

## Advanced Tutorials

- [Implementing Custom Routing Strategies](advanced/custom_routing.md)
- [Building Personas](advanced/personas.md)
- [Creating Chain Templates](advanced/chain_templates.md)
- [Integrating RAG](advanced/rag_integration.md)
EOF
    
    # Convert index.md to HTML
    pandoc "$TUTORIALS_DIR/index.md" -f markdown -t html -s -o "$TUTORIALS_DIR/index.html" --metadata title="IntelliRouter Tutorials"
    
    # Create template tutorial files
    cat > "$TUTORIALS_DIR/getting_started/setup.md" << EOF
# Setting Up IntelliRouter

This tutorial guides you through the process of setting up IntelliRouter.

## Prerequisites

- Rust 1.70 or later
- Docker (for containerized deployment)
- Kubernetes (for production deployment)

## Installation Steps

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

## Configuration

Create a \`config.toml\` file in the \`config\` directory:

\`\`\`toml
[server]
host = "0.0.0.0"
port = 8000

[logging]
level = "info"

[redis]
host = "localhost"
port = 6379

[chromadb]
host = "localhost"
port = 8001
\`\`\`

## Verification

To verify that IntelliRouter is running correctly, send a test request:

\`\`\`bash
curl -X POST http://localhost:8000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-3.5-turbo",
    "messages": [
      {"role": "system", "content": "You are a helpful assistant."},
      {"role": "user", "content": "Hello!"}
    ]
  }'
\`\`\`

You should receive a response from the router.
EOF
    
    # Convert setup.md to HTML
    pandoc "$TUTORIALS_DIR/getting_started/setup.md" -f markdown -t html -s -o "$TUTORIALS_DIR/getting_started/setup.html" --metadata title="Setting Up IntelliRouter"
fi

# Create CSS file
cat > "$EXAMPLES_DOCS_DIR/style.css" << EOF
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

# Copy CSS to tutorials directory
cp "$EXAMPLES_DOCS_DIR/style.css" "$TUTORIALS_DIR/"

# Add CSS to all HTML files
find "$EXAMPLES_DOCS_DIR" -name "*.html" | while read -r file; do
    sed -i 's/<\/head>/<link rel="stylesheet" href="\/style.css"><\/head>/' "$file"
done

find "$TUTORIALS_DIR" -name "*.html" | while read -r file; do
    sed -i 's/<\/head>/<link rel="stylesheet" href="\/style.css"><\/head>/' "$file"
done

# Create examples index.html file
cat > "$EXAMPLES_DOCS_DIR/index.html" << EOF
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>IntelliRouter Examples</title>
    <link rel="stylesheet" href="/style.css">
</head>
<body>
    <h1>IntelliRouter Examples</h1>
    
    <p>This documentation provides examples of using IntelliRouter.</p>
    
    <h2>Rust Examples</h2>
    <ul>
EOF

# Add links to Rust examples
find "$EXAMPLES_DOCS_DIR" -name "*.html" -not -path "*/\.*" -not -name "index.html" | sort | while read -r file; do
    rel_path="${file#$EXAMPLES_DOCS_DIR/}"
    example_name=$(basename "${file%.html}")
    echo "        <li><a href=\"$rel_path\">$example_name</a></li>" >> "$EXAMPLES_DOCS_DIR/index.html"
done

# Close the HTML file
cat >> "$EXAMPLES_DOCS_DIR/index.html" << EOF
    </ul>
</body>
</html>
EOF

# Calculate examples documentation coverage
echo -e "${YELLOW}Calculating examples documentation coverage...${NC}"
coverage=$(count_examples_coverage)

echo "Coverage: $coverage%"