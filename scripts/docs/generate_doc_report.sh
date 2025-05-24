#!/bin/bash
# Documentation report generator for IntelliRouter
# This script generates a report based on the documentation metrics

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
REPORT_FILE="$METRICS_DIR/doc_report_$TIMESTAMP.html"

# Check if a metrics file was provided
if [ $# -eq 0 ]; then
    echo -e "${RED}Error: No metrics file provided${NC}"
    echo -e "${YELLOW}Usage: $0 <metrics_file>${NC}"
    exit 1
fi

METRICS_FILE="$1"

# Check if the metrics file exists
if [ ! -f "$METRICS_FILE" ]; then
    echo -e "${RED}Error: Metrics file not found: $METRICS_FILE${NC}"
    exit 1
fi

# Print header
echo -e "${BLUE}=========================================${NC}"
echo -e "${BLUE}IntelliRouter Documentation Report Generator${NC}"
echo -e "${BLUE}=========================================${NC}"
echo

# Function to get coverage data from metrics file
get_coverage() {
    local key="$1"
    local metrics_file="$2"
    
    # Extract coverage percentage
    coverage=$(jq -r ".metrics.$key.coverage" "$metrics_file" 2>/dev/null)
    
    # If the key doesn't exist, try the coverage file
    if [ "$coverage" == "null" ]; then
        coverage_file=$(find "$METRICS_DIR" -name "doc_coverage_*.json" -type f -exec ls -t {} \; | head -1)
        if [ -n "$coverage_file" ]; then
            coverage=$(jq -r ".coverage.$key.coverage_percentage" "$coverage_file" 2>/dev/null)
        fi
    fi
    
    # If still null, return 0
    if [ "$coverage" == "null" ]; then
        coverage="0.0"
    fi
    
    echo "$coverage"
}

# Function to get historical coverage data
get_historical_coverage() {
    local key="$1"
    local days="$2"
    
    # Find metrics files from the last N days
    files=$(find "$METRICS_DIR" -name "doc_metrics_*.json" -type f -mtime -$days -exec ls -t {} \;)
    
    # If no metrics files found, try coverage files
    if [ -z "$files" ]; then
        files=$(find "$METRICS_DIR" -name "doc_coverage_*.json" -type f -mtime -$days -exec ls -t {} \;)
    fi
    
    # Extract coverage percentages
    coverage_data=""
    for file in $files; do
        # Get timestamp from filename
        timestamp=$(basename "$file" | sed -E 's/doc_(metrics|coverage)_([0-9-_]+)\..*/\2/')
        timestamp=$(echo "$timestamp" | tr '_' ' ')
        
        # Get coverage
        coverage=$(get_coverage "$key" "$file")
        
        # Add to coverage data
        coverage_data="$coverage_data
$timestamp,$coverage"
    done
    
    # Return coverage data
    echo "$coverage_data"
}

# Generate report
echo -e "${YELLOW}Generating documentation report...${NC}"

# Get coverage data
api_coverage=$(get_coverage "api_docs" "$METRICS_FILE")
user_guides_coverage=$(get_coverage "user_guides" "$METRICS_FILE")
architecture_coverage=$(get_coverage "architecture_docs" "$METRICS_FILE")
examples_coverage=$(get_coverage "examples_docs" "$METRICS_FILE")
rust_coverage=$(get_coverage "rust" "$METRICS_FILE")
python_coverage=$(get_coverage "python" "$METRICS_FILE")
typescript_coverage=$(get_coverage "typescript" "$METRICS_FILE")
markdown_coverage=$(get_coverage "markdown" "$METRICS_FILE")

# Calculate overall coverage
overall_coverage=$(echo "scale=2; ($api_coverage + $user_guides_coverage + $architecture_coverage + $examples_coverage) / 4" | bc)

# Get historical coverage data
api_history=$(get_historical_coverage "api_docs" 30)
user_guides_history=$(get_historical_coverage "user_guides" 30)
architecture_history=$(get_historical_coverage "architecture_docs" 30)
examples_history=$(get_historical_coverage "examples_docs" 30)
overall_history=""

# Calculate historical overall coverage
while IFS=',' read -r timestamp api_cov; do
    if [ -n "$timestamp" ] && [ -n "$api_cov" ]; then
        user_cov=$(echo "$user_guides_history" | grep "$timestamp" | cut -d',' -f2)
        arch_cov=$(echo "$architecture_history" | grep "$timestamp" | cut -d',' -f2)
        examples_cov=$(echo "$examples_history" | grep "$timestamp" | cut -d',' -f2)
        
        # If any coverage is missing, use the current value
        [ -z "$user_cov" ] && user_cov=$user_guides_coverage
        [ -z "$arch_cov" ] && arch_cov=$architecture_coverage
        [ -z "$examples_cov" ] && examples_cov=$examples_coverage
        
        # Calculate overall coverage
        overall_cov=$(echo "scale=2; ($api_cov + $user_cov + $arch_cov + $examples_cov) / 4" | bc)
        
        # Add to overall history
        overall_history="$overall_history
$timestamp,$overall_cov"
    fi
done <<< "$api_history"

# Create report file
cat > "$REPORT_FILE" << EOF
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>IntelliRouter Documentation Report</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        body {
            font-family: Arial, sans-serif;
            line-height: 1.6;
            margin: 0;
            padding: 20px;
            color: #333;
        }
        
        .container {
            max-width: 1200px;
            margin: 0 auto;
        }
        
        h1, h2, h3 {
            color: #0066cc;
        }
        
        .card {
            border: 1px solid #ddd;
            border-radius: 4px;
            padding: 20px;
            margin-bottom: 20px;
            background-color: #f9f9f9;
        }
        
        .coverage-bar {
            height: 20px;
            background-color: #f2f2f2;
            border-radius: 4px;
            margin-bottom: 10px;
            overflow: hidden;
        }
        
        .coverage-progress {
            height: 100%;
            background-color: #4CAF50;
            text-align: center;
            line-height: 20px;
            color: white;
            font-weight: bold;
        }
        
        .low {
            background-color: #F44336;
        }
        
        .medium {
            background-color: #FFC107;
        }
        
        .high {
            background-color: #4CAF50;
        }
        
        .chart-container {
            position: relative;
            height: 300px;
            margin-bottom: 20px;
        }
        
        table {
            width: 100%;
            border-collapse: collapse;
            margin-bottom: 20px;
        }
        
        th, td {
            padding: 8px;
            text-align: left;
            border-bottom: 1px solid #ddd;
        }
        
        th {
            background-color: #f2f2f2;
        }
        
        tr:hover {
            background-color: #f5f5f5;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>IntelliRouter Documentation Report</h1>
        <p>Generated on $(date)</p>
        
        <div class="card">
            <h2>Overall Documentation Coverage</h2>
            <div class="coverage-bar">
                <div class="coverage-progress ${overall_coverage >= 80 ? 'high' : overall_coverage >= 60 ? 'medium' : 'low'}" style="width: ${overall_coverage}%">
                    ${overall_coverage}%
                </div>
            </div>
            <p>Overall documentation coverage: <strong>${overall_coverage}%</strong></p>
        </div>
        
        <div class="card">
            <h2>Documentation Coverage by Type</h2>
            <table>
                <tr>
                    <th>Documentation Type</th>
                    <th>Coverage</th>
                    <th>Status</th>
                </tr>
                <tr>
                    <td>API Documentation</td>
                    <td>${api_coverage}%</td>
                    <td>
                        <div class="coverage-bar">
                            <div class="coverage-progress ${api_coverage >= 80 ? 'high' : api_coverage >= 60 ? 'medium' : 'low'}" style="width: ${api_coverage}%"></div>
                        </div>
                    </td>
                </tr>
                <tr>
                    <td>User Guides</td>
                    <td>${user_guides_coverage}%</td>
                    <td>
                        <div class="coverage-bar">
                            <div class="coverage-progress ${user_guides_coverage >= 80 ? 'high' : user_guides_coverage >= 60 ? 'medium' : 'low'}" style="width: ${user_guides_coverage}%"></div>
                        </div>
                    </td>
                </tr>
                <tr>
                    <td>Architecture Documentation</td>
                    <td>${architecture_coverage}%</td>
                    <td>
                        <div class="coverage-bar">
                            <div class="coverage-progress ${architecture_coverage >= 80 ? 'high' : architecture_coverage >= 60 ? 'medium' : 'low'}" style="width: ${architecture_coverage}%"></div>
                        </div>
                    </td>
                </tr>
                <tr>
                    <td>Examples & Tutorials</td>
                    <td>${examples_coverage}%</td>
                    <td>
                        <div class="coverage-bar">
                            <div class="coverage-progress ${examples_coverage >= 80 ? 'high' : examples_coverage >= 60 ? 'medium' : 'low'}" style="width: ${examples_coverage}%"></div>
                        </div>
                    </td>
                </tr>
            </table>
        </div>
        
        <div class="card">
            <h2>Code Documentation Coverage</h2>
            <table>
                <tr>
                    <th>Language</th>
                    <th>Coverage</th>
                    <th>Status</th>
                </tr>
                <tr>
                    <td>Rust</td>
                    <td>${rust_coverage}%</td>
                    <td>
                        <div class="coverage-bar">
                            <div class="coverage-progress ${rust_coverage >= 80 ? 'high' : rust_coverage >= 60 ? 'medium' : 'low'}" style="width: ${rust_coverage}%"></div>
                        </div>
                    </td>
                </tr>
                <tr>
                    <td>Python</td>
                    <td>${python_coverage}%</td>
                    <td>
                        <div class="coverage-bar">
                            <div class="coverage-progress ${python_coverage >= 80 ? 'high' : python_coverage >= 60 ? 'medium' : 'low'}" style="width: ${python_coverage}%"></div>
                        </div>
                    </td>
                </tr>
                <tr>
                    <td>TypeScript</td>
                    <td>${typescript_coverage}%</td>
                    <td>
                        <div class="coverage-bar">
                            <div class="coverage-progress ${typescript_coverage >= 80 ? 'high' : typescript_coverage >= 60 ? 'medium' : 'low'}" style="width: ${typescript_coverage}%"></div>
                        </div>
                    </td>
                </tr>
                <tr>
                    <td>Markdown</td>
                    <td>${markdown_coverage}%</td>
                    <td>
                        <div class="coverage-bar">
                            <div class="coverage-progress ${markdown_coverage >= 80 ? 'high' : markdown_coverage >= 60 ? 'medium' : 'low'}" style="width: ${markdown_coverage}%"></div>
                        </div>
                    </td>
                </tr>
            </table>
        </div>
        
        <div class="card">
            <h2>Documentation Coverage Trends</h2>
            <div class="chart-container">
                <canvas id="coverageTrends"></canvas>
            </div>
        </div>
        
        <div class="card">
            <h2>Recommendations</h2>
            <ul>
                ${api_coverage < 80 ? '<li>Improve API documentation coverage by documenting more public items.</li>' : ''}
                ${user_guides_coverage < 80 ? '<li>Create more user guides to cover all modules and features.</li>' : ''}
                ${architecture_coverage < 80 ? '<li>Enhance architecture documentation with more diagrams and detailed explanations.</li>' : ''}
                ${examples_coverage < 80 ? '<li>Add more examples and tutorials to demonstrate usage patterns.</li>' : ''}
                ${rust_coverage < 80 ? '<li>Add documentation comments to undocumented Rust items.</li>' : ''}
                ${python_coverage < 80 ? '<li>Add docstrings to undocumented Python classes and functions.</li>' : ''}
                ${typescript_coverage < 80 ? '<li>Add JSDoc comments to undocumented TypeScript items.</li>' : ''}
                ${markdown_coverage < 80 ? '<li>Create Markdown documentation for undocumented modules.</li>' : ''}
                ${overall_coverage >= 80 ? '<li>Maintain the current high level of documentation coverage.</li>' : ''}
            </ul>
        </div>
    </div>
    
    <script>
        // Parse historical data
        const apiHistory = \`${api_history}\`.trim().split('\\n').map(line => {
            const [date, coverage] = line.split(',');
            return { date, coverage: parseFloat(coverage) };
        });
        
        const userGuidesHistory = \`${user_guides_history}\`.trim().split('\\n').map(line => {
            const [date, coverage] = line.split(',');
            return { date, coverage: parseFloat(coverage) };
        });
        
        const architectureHistory = \`${architecture_history}\`.trim().split('\\n').map(line => {
            const [date, coverage] = line.split(',');
            return { date, coverage: parseFloat(coverage) };
        });
        
        const examplesHistory = \`${examples_history}\`.trim().split('\\n').map(line => {
            const [date, coverage] = line.split(',');
            return { date, coverage: parseFloat(coverage) };
        });
        
        const overallHistory = \`${overall_history}\`.trim().split('\\n').map(line => {
            const [date, coverage] = line.split(',');
            return { date, coverage: parseFloat(coverage) };
        });
        
        // Create chart
        const ctx = document.getElementById('coverageTrends').getContext('2d');
        const chart = new Chart(ctx, {
            type: 'line',
            data: {
                labels: overallHistory.map(item => item.date),
                datasets: [
                    {
                        label: 'Overall',
                        data: overallHistory.map(item => item.coverage),
                        borderColor: '#0066cc',
                        backgroundColor: 'rgba(0, 102, 204, 0.1)',
                        tension: 0.4,
                        fill: true
                    },
                    {
                        label: 'API',
                        data: apiHistory.map(item => item.coverage),
                        borderColor: '#4CAF50',
                        backgroundColor: 'transparent',
                        tension: 0.4
                    },
                    {
                        label: 'User Guides',
                        data: userGuidesHistory.map(item => item.coverage),
                        borderColor: '#FFC107',
                        backgroundColor: 'transparent',
                        tension: 0.4
                    },
                    {
                        label: 'Architecture',
                        data: architectureHistory.map(item => item.coverage),
                        borderColor: '#F44336',
                        backgroundColor: 'transparent',
                        tension: 0.4
                    },
                    {
                        label: 'Examples',
                        data: examplesHistory.map(item => item.coverage),
                        borderColor: '#9C27B0',
                        backgroundColor: 'transparent',
                        tension: 0.4
                    }
                ]
            },
            options: {
                responsive: true,
                maintainAspectRatio: false,
                scales: {
                    y: {
                        beginAtZero: true,
                        max: 100,
                        title: {
                            display: true,
                            text: 'Coverage (%)'
                        }
                    },
                    x: {
                        title: {
                            display: true,
                            text: 'Date'
                        }
                    }
                },
                plugins: {
                    title: {
                        display: true,
                        text: 'Documentation Coverage Trends'
                    },
                    tooltip: {
                        mode: 'index',
                        intersect: false
                    }
                }
            }
        });
    </script>
</body>
</html>
EOF

echo -e "${GREEN}Documentation report generated: $REPORT_FILE${NC}"