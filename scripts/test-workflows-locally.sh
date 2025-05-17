#!/bin/bash
# Script to test GitHub Actions workflows locally using act
# https://github.com/nektos/act

# Check if act is installed
if ! command -v act &> /dev/null; then
    echo "Error: 'act' is not installed. Please install it first:"
    echo "  macOS: brew install act"
    echo "  Linux: curl -s https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash"
    exit 1
fi

# Create .actrc file if it doesn't exist
if [ ! -f .actrc ]; then
    echo "Creating .actrc file with recommended settings..."
    cat > .actrc << EOF
-P ubuntu-latest=nektos/act-environments-ubuntu:18.04
--env FONTCONFIG_NO_PKG_CONFIG=1
EOF
    echo ".actrc file created."
fi

# Function to run a specific workflow
run_workflow() {
    local workflow=$1
    echo "Running workflow: $workflow"
    act -W .github/workflows/$workflow.yml push
}

# Function to run a specific job from a workflow
run_job() {
    local workflow=$1
    local job=$2
    echo "Running job '$job' from workflow: $workflow"
    act -W .github/workflows/$workflow.yml -j $job
}

# Display menu
echo "IntelliRouter Workflow Tester"
echo "============================"
echo "1. Run all workflows"
echo "2. Run test workflow"
echo "3. Run e2e-tests workflow"
echo "4. Run codeql-analysis workflow"
echo "5. Run specific job"
echo "6. Exit"
echo

read -p "Enter your choice (1-6): " choice

case $choice in
    1)
        echo "Running all workflows..."
        act push
        ;;
    2)
        run_workflow "test"
        ;;
    3)
        run_workflow "e2e-tests"
        ;;
    4)
        run_workflow "codeql-analysis"
        ;;
    5)
        echo "Available workflows:"
        ls -1 .github/workflows/*.yml | sed 's/\.github\/workflows\///' | sed 's/\.yml//'
        echo
        read -p "Enter workflow name: " workflow
        
        echo "Available jobs in $workflow:"
        grep -A 1 "jobs:" .github/workflows/$workflow.yml | grep -v "jobs:" | grep -v "--" | sed 's/://' | sed 's/^[ \t]*//'
        echo
        read -p "Enter job name: " job
        
        run_job $workflow $job
        ;;
    6)
        echo "Exiting..."
        exit 0
        ;;
    *)
        echo "Invalid choice. Exiting..."
        exit 1
        ;;
esac