#!/bin/bash
# Architecture documentation generator for IntelliRouter
# This script generates architecture documentation using diagrams and Markdown

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
ARCH_DOCS_DIR="$DOCS_DIR/architecture"
ARCH_DOCS_SRC_DIR="$DOCS_DIR/architecture_src"
DIAGRAMS_DIR="$ARCH_DOCS_DIR/diagrams"

# Create directories if they don't exist
mkdir -p "$ARCH_DOCS_DIR" "$ARCH_DOCS_SRC_DIR" "$DIAGRAMS_DIR"

# Check if pandoc is installed
if ! command -v pandoc &> /dev/null; then
    echo -e "${RED}Error: pandoc is not installed. Please install it to generate architecture documentation.${NC}"
    exit 1
fi

# Check if plantuml is installed
if ! command -v plantuml &> /dev/null; then
    echo -e "${YELLOW}Warning: plantuml is not installed. Diagrams will not be generated.${NC}"
    PLANTUML_INSTALLED=false
else
    PLANTUML_INSTALLED=true
fi

# Function to count architecture documentation coverage
count_architecture_coverage() {
    local total_components=0
    local documented_components=0
    
    # Define expected architecture documents
    expected_docs=(
        "overview"
        "system_architecture"
        "component_architecture"
        "data_flow"
        "deployment_architecture"
        "security_architecture"
        "scalability"
        "modules/llm_proxy"
        "modules/model_registry"
        "modules/router_core"
        "modules/persona_layer"
        "modules/chain_engine"
        "modules/memory"
        "modules/rag_manager"
        "modules/authentication"
        "modules/telemetry"
        "modules/plugin_sdk"
    )
    
    # Count total expected documents
    total_components=${#expected_docs[@]}
    
    # Count documented components
    for doc in "${expected_docs[@]}"; do
        if [ -f "$ARCH_DOCS_SRC_DIR/$doc.md" ]; then
            ((documented_components++))
        fi
    done
    
    # Calculate coverage percentage
    if [ "$total_components" -gt 0 ]; then
        coverage=$(echo "scale=2; ($documented_components / $total_components) * 100" | bc)
    else
        coverage="0.0"
    fi
    
    echo "Total expected architecture documents: $total_components"
    echo "Documented architecture components: $documented_components"
    echo "Coverage: $coverage%"
    
    # Return the coverage percentage
    echo "$coverage"
}

# Create architecture documentation source directory structure if it doesn't exist
if [ ! -d "$ARCH_DOCS_SRC_DIR/modules" ]; then
    echo -e "${YELLOW}Creating architecture documentation source directory structure...${NC}"
    mkdir -p "$ARCH_DOCS_SRC_DIR/modules"
    
    # Create template files if they don't exist
    if [ ! -f "$ARCH_DOCS_SRC_DIR/overview.md" ]; then
        cat > "$ARCH_DOCS_SRC_DIR/overview.md" << EOF
# IntelliRouter Architecture Overview

IntelliRouter is a programmable LLM gateway that provides an OpenAI-compatible API endpoint for chat completions. This document provides an overview of the IntelliRouter architecture.

## System Context

IntelliRouter sits between client applications and LLM backends, providing a unified interface for routing requests to appropriate models based on customizable strategies.

\`\`\`plantuml
@startuml
!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Context.puml

Person(user, "User", "A user of the system")
System_Boundary(c1, "Client Applications") {
    System(clientApp, "Client Application", "Uses IntelliRouter to access LLM capabilities")
}

System_Boundary(c2, "IntelliRouter") {
    System(intelliRouter, "IntelliRouter", "Programmable LLM gateway")
}

System_Boundary(c3, "LLM Backends") {
    System(openai, "OpenAI API", "GPT models")
    System(anthropic, "Anthropic API", "Claude models")
    System(local, "Local Models", "Self-hosted models")
}

Rel(user, clientApp, "Uses")
Rel(clientApp, intelliRouter, "Sends requests to", "HTTP/WebSocket")
Rel(intelliRouter, openai, "Routes requests to", "HTTP")
Rel(intelliRouter, anthropic, "Routes requests to", "HTTP")
Rel(intelliRouter, local, "Routes requests to", "HTTP/gRPC")

@enduml
\`\`\`

## Key Components

IntelliRouter consists of several key components:

- **LLM Proxy**: OpenAI-compatible API endpoint
- **Model Registry**: Tracks available LLM backends
- **Router Core**: Routes requests to appropriate model backends
- **Persona Layer**: Injects system prompts and guardrails
- **Chain Engine**: Orchestrates multi-step inference flows
- **Memory**: Provides short-term and long-term memory capabilities
- **RAG Manager**: Manages Retrieval Augmented Generation
- **Authentication**: Handles API key validation and RBAC
- **Telemetry**: Collects logs, costs, and usage metrics
- **Plugin SDK**: Provides extensibility for custom components

## Architecture Principles

1. **Modularity**: Components are designed to be modular and replaceable
2. **Extensibility**: Plugin system allows for custom extensions
3. **Scalability**: Components can be scaled independently
4. **Resilience**: Fault tolerance and graceful degradation
5. **Security**: Secure by design with proper authentication and authorization
EOF
    fi
    
    if [ ! -f "$ARCH_DOCS_SRC_DIR/system_architecture.md" ]; then
        cat > "$ARCH_DOCS_SRC_DIR/system_architecture.md" << EOF
# IntelliRouter System Architecture

This document describes the system architecture of IntelliRouter.

## Component Diagram

\`\`\`plantuml
@startuml
!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Component.puml

Container_Boundary(intelliRouter, "IntelliRouter") {
    Component(llmProxy, "LLM Proxy", "Axum", "OpenAI-compatible API endpoint")
    Component(modelRegistry, "Model Registry", "Rust", "Tracks available LLM backends")
    Component(routerCore, "Router Core", "Rust", "Routes requests to appropriate model backends")
    Component(personaLayer, "Persona Layer", "Rust", "Injects system prompts and guardrails")
    Component(chainEngine, "Chain Engine", "Rust", "Orchestrates multi-step inference flows")
    Component(memory, "Memory", "Rust", "Provides short-term and long-term memory capabilities")
    Component(ragManager, "RAG Manager", "Rust", "Manages Retrieval Augmented Generation")
    Component(authentication, "Authentication", "Rust", "Handles API key validation and RBAC")
    Component(telemetry, "Telemetry", "Rust", "Collects logs, costs, and usage metrics")
    Component(pluginSdk, "Plugin SDK", "Rust", "Provides extensibility for custom components")
}

System_Ext(clientApp, "Client Application", "Uses IntelliRouter to access LLM capabilities")
System_Ext(openai, "OpenAI API", "GPT models")
System_Ext(anthropic, "Anthropic API", "Claude models")
System_Ext(local, "Local Models", "Self-hosted models")
System_Ext(redis, "Redis", "Cache and message broker")
System_Ext(chromadb, "ChromaDB", "Vector database for RAG")

Rel(clientApp, llmProxy, "Sends requests to", "HTTP/WebSocket")
Rel(llmProxy, routerCore, "Forwards requests to")
Rel(routerCore, modelRegistry, "Queries")
Rel(routerCore, personaLayer, "Applies personas")
Rel(personaLayer, chainEngine, "Executes chains")
Rel(chainEngine, memory, "Stores/retrieves context")
Rel(chainEngine, ragManager, "Retrieves relevant information")
Rel(llmProxy, authentication, "Validates requests")
Rel(llmProxy, telemetry, "Logs events")
Rel(routerCore, openai, "Routes requests to", "HTTP")
Rel(routerCore, anthropic, "Routes requests to", "HTTP")
Rel(routerCore, local, "Routes requests to", "HTTP/gRPC")
Rel(memory, redis, "Uses")
Rel(ragManager, chromadb, "Uses")
Rel(pluginSdk, routerCore, "Extends")
Rel(pluginSdk, personaLayer, "Extends")
Rel(pluginSdk, chainEngine, "Extends")
Rel(pluginSdk, ragManager, "Extends")

@enduml
\`\`\`

## Deployment Options

IntelliRouter supports various deployment options:

1. **Monolithic Deployment**: All components run in a single process
2. **Multi-Role Deployment**: Components run as separate services with secure IPC
3. **Edge Deployment**: Optimized for edge computing environments
4. **Cloud Deployment**: Optimized for cloud environments (Kubernetes)
EOF
    fi
    
    # Create template module architecture docs
    modules=("llm_proxy" "model_registry" "router_core" "persona_layer" "chain_engine" "memory" "rag_manager" "authentication" "telemetry" "plugin_sdk")
    for module in "${modules[@]}"; do
        if [ ! -f "$ARCH_DOCS_SRC_DIR/modules/$module.md" ]; then
            cat > "$ARCH_DOCS_SRC_DIR/modules/$module.md" << EOF
# $module Architecture

## Overview

Description of the $module component.

## Responsibilities

- Responsibility 1
- Responsibility 2
- Responsibility 3

## Internal Structure

\`\`\`plantuml
@startuml
package "$module" {
    [Component 1]
    [Component 2]
    [Component 3]
    
    [Component 1] --> [Component 2]
    [Component 2] --> [Component 3]
}
@enduml
\`\`\`

## Interfaces

### Input Interfaces

- Interface 1
- Interface 2

### Output Interfaces

- Interface 1
- Interface 2

## Dependencies

- Dependency 1
- Dependency 2

## Configuration

How to configure the $module component.
EOF
        fi
    done
fi

# Generate PlantUML diagrams if installed
if [ "$PLANTUML_INSTALLED" = true ]; then
    echo -e "${YELLOW}Generating PlantUML diagrams...${NC}"
    
    # Find all PlantUML diagrams in Markdown files
    find "$ARCH_DOCS_SRC_DIR" -name "*.md" | while read -r file; do
        # Extract PlantUML diagrams
        grep -n "^\`\`\`plantuml" -A 1000 "$file" | while read -r line; do
            if [[ "$line" =~ ^([0-9]+):\`\`\`plantuml$ ]]; then
                start_line="${BASH_REMATCH[1]}"
                end_line=$(tail -n +$start_line "$file" | grep -n "^\`\`\`$" | head -1 | cut -d: -f1)
                end_line=$((start_line + end_line - 1))
                
                # Extract diagram content
                diagram_content=$(sed -n "$((start_line + 1)),$((end_line - 1))p" "$file")
                
                # Generate diagram filename
                diagram_name=$(basename "$file" .md)_$(echo "$diagram_content" | md5sum | cut -d' ' -f1)
                diagram_path="$DIAGRAMS_DIR/$diagram_name.png"
                
                # Generate diagram
                echo "$diagram_content" | plantuml -pipe > "$diagram_path"
                
                if [ $? -eq 0 ]; then
                    echo -e "${GREEN}✓ Generated diagram: $diagram_path${NC}"
                else
                    echo -e "${RED}✗ Failed to generate diagram from $file${NC}"
                fi
            fi
        done
    done
else
    echo -e "${YELLOW}Skipping PlantUML diagram generation...${NC}"
fi

# Generate HTML from Markdown
echo -e "${YELLOW}Generating HTML from Markdown...${NC}"
find "$ARCH_DOCS_SRC_DIR" -name "*.md" | while read -r file; do
    # Get relative path
    rel_path="${file#$ARCH_DOCS_SRC_DIR/}"
    # Create output directory
    output_dir="$ARCH_DOCS_DIR/$(dirname "$rel_path")"
    mkdir -p "$output_dir"
    # Convert Markdown to HTML
    output_file="$ARCH_DOCS_DIR/${rel_path%.md}.html"
    
    # Replace PlantUML code blocks with image tags if diagrams were generated
    if [ "$PLANTUML_INSTALLED" = true ]; then
        # Create a temporary file
        temp_file=$(mktemp)
        
        # Copy the original file
        cp "$file" "$temp_file"
        
        # Replace PlantUML code blocks with image tags
        awk -v diagrams_dir="$DIAGRAMS_DIR" -v file_basename="$(basename "$file" .md)" '
        BEGIN { in_plantuml = 0; diagram_content = ""; }
        /^```plantuml$/ {
            in_plantuml = 1;
            diagram_content = "";
            next;
        }
        /^```$/ && in_plantuml {
            in_plantuml = 0;
            diagram_name = file_basename "_" md5(diagram_content);
            diagram_path = diagrams_dir "/" diagram_name ".png";
            print "![" file_basename " diagram](/architecture/diagrams/" diagram_name ".png)";
            next;
        }
        in_plantuml {
            diagram_content = diagram_content $0 "\n";
            next;
        }
        { print; }
        function md5(str) {
            cmd = "echo \"" str "\" | md5sum | cut -d\" \" -f1";
            cmd | getline hash;
            close(cmd);
            return hash;
        }
        ' "$temp_file" > "$temp_file.new"
        
        # Replace the original file with the modified one
        mv "$temp_file.new" "$temp_file"
        
        # Use the modified file for pandoc
        pandoc "$temp_file" -f markdown -t html -s -o "$output_file" --metadata title="$(basename "${file%.md}")"
        
        # Remove the temporary file
        rm "$temp_file"
    else
        # Use the original file for pandoc
        pandoc "$file" -f markdown -t html -s -o "$output_file" --metadata title="$(basename "${file%.md}")"
    fi
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ Generated $output_file${NC}"
    else
        echo -e "${RED}✗ Failed to generate $output_file${NC}"
        exit 1
    fi
done

# Create CSS file
cat > "$ARCH_DOCS_DIR/style.css" << EOF
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

img {
    max-width: 100%;
    height: auto;
    display: block;
    margin: 20px auto;
    border: 1px solid #ddd;
    border-radius: 4px;
    padding: 5px;
}
EOF

# Add CSS to all HTML files
find "$ARCH_DOCS_DIR" -name "*.html" | while read -r file; do
    sed -i 's/<\/head>/<link rel="stylesheet" href="\/style.css"><\/head>/' "$file"
done

# Create index.html file
cat > "$ARCH_DOCS_DIR/index.html" << EOF
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>IntelliRouter Architecture Documentation</title>
    <link rel="stylesheet" href="/style.css">
</head>
<body>
    <h1>IntelliRouter Architecture Documentation</h1>
    
    <p>This documentation provides an overview of the IntelliRouter architecture.</p>
    
    <h2>System Architecture</h2>
    <ul>
        <li><a href="overview.html">Architecture Overview</a></li>
        <li><a href="system_architecture.html">System Architecture</a></li>
        <li><a href="component_architecture.html">Component Architecture</a></li>
        <li><a href="data_flow.html">Data Flow</a></li>
        <li><a href="deployment_architecture.html">Deployment Architecture</a></li>
        <li><a href="security_architecture.html">Security Architecture</a></li>
        <li><a href="scalability.html">Scalability</a></li>
    </ul>
    
    <h2>Module Architecture</h2>
    <ul>
        <li><a href="modules/llm_proxy.html">LLM Proxy</a></li>
        <li><a href="modules/model_registry.html">Model Registry</a></li>
        <li><a href="modules/router_core.html">Router Core</a></li>
        <li><a href="modules/persona_layer.html">Persona Layer</a></li>
        <li><a href="modules/chain_engine.html">Chain Engine</a></li>
        <li><a href="modules/memory.html">Memory</a></li>
        <li><a href="modules/rag_manager.html">RAG Manager</a></li>
        <li><a href="modules/authentication.html">Authentication</a></li>
        <li><a href="modules/telemetry.html">Telemetry</a></li>
        <li><a href="modules/plugin_sdk.html">Plugin SDK</a></li>
    </ul>
</body>
</html>
EOF

# Calculate architecture documentation coverage
echo -e "${YELLOW}Calculating architecture documentation coverage...${NC}"
coverage=$(count_architecture_coverage)

echo "Coverage: $coverage%"