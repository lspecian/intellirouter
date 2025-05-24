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

# Parse command line arguments
OUTPUT_DIR="metrics/charts"
METRICS_DIR="metrics"
CHART_FORMAT="html"

for arg in "$@"; do
    case $arg in
        --output=*)
            OUTPUT_DIR="${arg#*=}"
            shift
            ;;
        --metrics-dir=*)
            METRICS_DIR="${arg#*=}"
            shift
            ;;
        --format=*)
            CHART_FORMAT="${arg#*=}"
            shift
            ;;
    esac
done

echo -e "${BOLD}${BLUE}IntelliRouter Code Quality Metrics Chart Generator${RESET}"
echo -e "${CYAN}==================================================${RESET}\n"

# Check if metrics directory exists
if [ ! -d "$METRICS_DIR" ]; then
    echo -e "${RED}Error: Metrics directory '$METRICS_DIR' does not exist.${RESET}"
    echo -e "${YELLOW}Run generate_code_quality_report.sh first to generate metrics.${RESET}"
    exit 1
fi

# Create output directory if it doesn't exist
if [ ! -d "$OUTPUT_DIR" ]; then
    echo -e "${YELLOW}Creating output directory '$OUTPUT_DIR'...${RESET}"
    mkdir -p "$OUTPUT_DIR"
fi

# Find all metrics files and sort them by timestamp
METRICS_FILES=$(find "$METRICS_DIR" -name "metrics_*.json" | sort)
FILE_COUNT=$(echo "$METRICS_FILES" | wc -l)

if [ "$FILE_COUNT" -lt 1 ]; then
    echo -e "${RED}Error: No metrics files found in '$METRICS_DIR'.${RESET}"
    echo -e "${YELLOW}Run generate_code_quality_report.sh first to generate metrics.${RESET}"
    exit 1
fi

echo -e "${GREEN}Found $FILE_COUNT metrics files.${RESET}"

# Extract data from metrics files
echo -e "${YELLOW}Extracting data from metrics files...${RESET}"

# Arrays to store data
TIMESTAMPS=()
TOTAL_WARNINGS=()
WARNING_DENSITY=()
TEST_COVERAGE=()
DOC_COVERAGE=()
TOTAL_LOC=()

# Process each metrics file
for file in $METRICS_FILES; do
    # Extract timestamp from filename
    filename=$(basename "$file")
    ts=${filename#metrics_}
    ts=${ts%.json}
    
    # Format timestamp for display
    formatted_ts=$(date -d "@$(echo "$ts" | sed 's/^\([0-9]\{8\}\)\([0-9]\{6\}\)$/\1 \2/' | sed 's/\([0-9]\{4\}\)\([0-9]\{2\}\)\([0-9]\{2\}\) \([0-9]\{2\}\)\([0-9]\{2\}\)\([0-9]\{2\}\)/\1-\2-\3 \4:\5:\6/')" +"%Y-%m-%d" 2>/dev/null || echo "$ts")
    
    TIMESTAMPS+=("$formatted_ts")
    
    # Extract metrics
    if [ -f "$file" ]; then
        total_warnings=$(grep -o '"total_warnings":[0-9]*' "$file" | cut -d':' -f2)
        TOTAL_WARNINGS+=("$total_warnings")
        
        warning_density=$(grep -o '"warning_density":[0-9.]*' "$file" | cut -d':' -f2)
        WARNING_DENSITY+=("$warning_density")
        
        test_coverage=$(grep -o '"test_coverage":[0-9.]*' "$file" | cut -d':' -f2)
        TEST_COVERAGE+=("$test_coverage")
        
        doc_coverage=$(grep -o '"documentation_coverage":[0-9.]*' "$file" | cut -d':' -f2)
        DOC_COVERAGE+=("$doc_coverage")
        
        total_loc=$(grep -o '"total": [0-9]*' "$file" | head -1 | cut -d' ' -f2)
        TOTAL_LOC+=("$total_loc")
    else
        TOTAL_WARNINGS+=("null")
        WARNING_DENSITY+=("null")
        TEST_COVERAGE+=("null")
        DOC_COVERAGE+=("null")
        TOTAL_LOC+=("null")
    fi
done

# Generate HTML chart
if [ "$CHART_FORMAT" = "html" ]; then
    echo -e "${YELLOW}Generating HTML charts...${RESET}"
    
    # Create HTML file with Chart.js
    HTML_FILE="$OUTPUT_DIR/metrics_charts.html"
    
    cat > "$HTML_FILE" << EOF
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>IntelliRouter Code Quality Metrics</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        body {
            font-family: Arial, sans-serif;
            margin: 0;
            padding: 20px;
            background-color: #f5f5f5;
        }
        .container {
            max-width: 1200px;
            margin: 0 auto;
            background-color: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 0 10px rgba(0,0,0,0.1);
        }
        h1 {
            color: #2c3e50;
            text-align: center;
        }
        .chart-container {
            position: relative;
            height: 400px;
            margin-bottom: 40px;
        }
        .row {
            display: flex;
            flex-wrap: wrap;
            margin: 0 -15px;
        }
        .col {
            flex: 1;
            padding: 0 15px;
            min-width: 300px;
        }
        .card {
            background-color: white;
            border-radius: 8px;
            box-shadow: 0 0 5px rgba(0,0,0,0.1);
            padding: 15px;
            margin-bottom: 20px;
        }
        .card h2 {
            margin-top: 0;
            color: #3498db;
            font-size: 1.2rem;
        }
        .summary {
            display: flex;
            justify-content: space-between;
            margin-bottom: 30px;
        }
        .summary-item {
            text-align: center;
            padding: 15px;
            background-color: #f8f9fa;
            border-radius: 8px;
            flex: 1;
            margin: 0 10px;
        }
        .summary-item h3 {
            margin: 0;
            font-size: 1rem;
            color: #555;
        }
        .summary-item p {
            margin: 10px 0 0;
            font-size: 1.8rem;
            font-weight: bold;
            color: #2c3e50;
        }
        .summary-item .change {
            font-size: 0.9rem;
            margin-top: 5px;
        }
        .positive {
            color: #27ae60;
        }
        .negative {
            color: #e74c3c;
        }
        .neutral {
            color: #7f8c8d;
        }
        @media (max-width: 768px) {
            .col {
                flex: 100%;
            }
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>IntelliRouter Code Quality Metrics</h1>
        <p style="text-align: center;">Generated on $(date '+%Y-%m-%d at %H:%M:%S')</p>
        
        <div class="summary">
            <div class="summary-item">
                <h3>Total Warnings</h3>
                <p>${TOTAL_WARNINGS[-1]}</p>
                <div class="change">
EOF

# Calculate change for total warnings
if [ ${#TOTAL_WARNINGS[@]} -gt 1 ]; then
    prev_warnings=${TOTAL_WARNINGS[-2]}
    curr_warnings=${TOTAL_WARNINGS[-1]}
    if [ "$prev_warnings" != "null" ] && [ "$curr_warnings" != "null" ]; then
        warnings_change=$((curr_warnings - prev_warnings))
        if [ $warnings_change -lt 0 ]; then
            echo "                    <span class=\"positive\">${warnings_change} ($(echo "scale=1; $warnings_change * 100 / $prev_warnings" | bc)%)</span>" >> "$HTML_FILE"
        elif [ $warnings_change -gt 0 ]; then
            echo "                    <span class=\"negative\">+${warnings_change} ($(echo "scale=1; $warnings_change * 100 / $prev_warnings" | bc)%)</span>" >> "$HTML_FILE"
        else
            echo "                    <span class=\"neutral\">No change</span>" >> "$HTML_FILE"
        fi
    else
        echo "                    <span class=\"neutral\">N/A</span>" >> "$HTML_FILE"
    fi
else
    echo "                    <span class=\"neutral\">N/A</span>" >> "$HTML_FILE"
fi

cat >> "$HTML_FILE" << EOF
                </div>
            </div>
            <div class="summary-item">
                <h3>Test Coverage</h3>
                <p>${TEST_COVERAGE[-1]}%</p>
                <div class="change">
EOF

# Calculate change for test coverage
if [ ${#TEST_COVERAGE[@]} -gt 1 ]; then
    prev_coverage=${TEST_COVERAGE[-2]}
    curr_coverage=${TEST_COVERAGE[-1]}
    if [ "$prev_coverage" != "null" ] && [ "$curr_coverage" != "null" ]; then
        coverage_change=$(echo "scale=1; $curr_coverage - $prev_coverage" | bc)
        if (( $(echo "$coverage_change > 0" | bc -l) )); then
            echo "                    <span class=\"positive\">+${coverage_change}%</span>" >> "$HTML_FILE"
        elif (( $(echo "$coverage_change < 0" | bc -l) )); then
            echo "                    <span class=\"negative\">${coverage_change}%</span>" >> "$HTML_FILE"
        else
            echo "                    <span class=\"neutral\">No change</span>" >> "$HTML_FILE"
        fi
    else
        echo "                    <span class=\"neutral\">N/A</span>" >> "$HTML_FILE"
    fi
else
    echo "                    <span class=\"neutral\">N/A</span>" >> "$HTML_FILE"
fi

cat >> "$HTML_FILE" << EOF
                </div>
            </div>
            <div class="summary-item">
                <h3>Doc Coverage</h3>
                <p>${DOC_COVERAGE[-1]}%</p>
                <div class="change">
EOF

# Calculate change for doc coverage
if [ ${#DOC_COVERAGE[@]} -gt 1 ]; then
    prev_doc_coverage=${DOC_COVERAGE[-2]}
    curr_doc_coverage=${DOC_COVERAGE[-1]}
    if [ "$prev_doc_coverage" != "null" ] && [ "$curr_doc_coverage" != "null" ]; then
        doc_coverage_change=$(echo "scale=1; $curr_doc_coverage - $prev_doc_coverage" | bc)
        if (( $(echo "$doc_coverage_change > 0" | bc -l) )); then
            echo "                    <span class=\"positive\">+${doc_coverage_change}%</span>" >> "$HTML_FILE"
        elif (( $(echo "$doc_coverage_change < 0" | bc -l) )); then
            echo "                    <span class=\"negative\">${doc_coverage_change}%</span>" >> "$HTML_FILE"
        else
            echo "                    <span class=\"neutral\">No change</span>" >> "$HTML_FILE"
        fi
    else
        echo "                    <span class=\"neutral\">N/A</span>" >> "$HTML_FILE"
    fi
else
    echo "                    <span class=\"neutral\">N/A</span>" >> "$HTML_FILE"
fi

cat >> "$HTML_FILE" << EOF
                </div>
            </div>
            <div class="summary-item">
                <h3>Warning Density</h3>
                <p>${WARNING_DENSITY[-1]}</p>
                <div class="change">
EOF

# Calculate change for warning density
if [ ${#WARNING_DENSITY[@]} -gt 1 ]; then
    prev_density=${WARNING_DENSITY[-2]}
    curr_density=${WARNING_DENSITY[-1]}
    if [ "$prev_density" != "null" ] && [ "$curr_density" != "null" ]; then
        density_change=$(echo "scale=2; $curr_density - $prev_density" | bc)
        if (( $(echo "$density_change < 0" | bc -l) )); then
            echo "                    <span class=\"positive\">${density_change}</span>" >> "$HTML_FILE"
        elif (( $(echo "$density_change > 0" | bc -l) )); then
            echo "                    <span class=\"negative\">+${density_change}</span>" >> "$HTML_FILE"
        else
            echo "                    <span class=\"neutral\">No change</span>" >> "$HTML_FILE"
        fi
    else
        echo "                    <span class=\"neutral\">N/A</span>" >> "$HTML_FILE"
    fi
else
    echo "                    <span class=\"neutral\">N/A</span>" >> "$HTML_FILE"
fi

cat >> "$HTML_FILE" << EOF
                </div>
            </div>
        </div>
        
        <div class="row">
            <div class="col">
                <div class="card">
                    <h2>Total Warnings</h2>
                    <div class="chart-container">
                        <canvas id="warningsChart"></canvas>
                    </div>
                </div>
            </div>
            <div class="col">
                <div class="card">
                    <h2>Warning Density (per 1000 LOC)</h2>
                    <div class="chart-container">
                        <canvas id="densityChart"></canvas>
                    </div>
                </div>
            </div>
        </div>
        
        <div class="row">
            <div class="col">
                <div class="card">
                    <h2>Test Coverage (%)</h2>
                    <div class="chart-container">
                        <canvas id="coverageChart"></canvas>
                    </div>
                </div>
            </div>
            <div class="col">
                <div class="card">
                    <h2>Documentation Coverage (%)</h2>
                    <div class="chart-container">
                        <canvas id="docCoverageChart"></canvas>
                    </div>
                </div>
            </div>
        </div>
        
        <div class="row">
            <div class="col">
                <div class="card">
                    <h2>Lines of Code</h2>
                    <div class="chart-container">
                        <canvas id="locChart"></canvas>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <script>
        // Common chart options
        const commonOptions = {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                legend: {
                    position: 'top',
                },
            },
            scales: {
                x: {
                    grid: {
                        display: false
                    }
                },
                y: {
                    beginAtZero: false
                }
            },
            elements: {
                line: {
                    tension: 0.3
                },
                point: {
                    radius: 4,
                    hoverRadius: 6
                }
            }
        };
        
        // Data
        const labels = [$(printf "'%s', " "${TIMESTAMPS[@]}")];
        
        // Total Warnings Chart
        const warningsData = {
            labels: labels,
            datasets: [{
                label: 'Total Warnings',
                data: [$(printf "%s, " "${TOTAL_WARNINGS[@]}")],
                borderColor: '#e74c3c',
                backgroundColor: 'rgba(231, 76, 60, 0.2)',
                fill: true
            }]
        };
        
        new Chart(
            document.getElementById('warningsChart'),
            {
                type: 'line',
                data: warningsData,
                options: {
                    ...commonOptions,
                    scales: {
                        ...commonOptions.scales,
                        y: {
                            beginAtZero: true,
                            title: {
                                display: true,
                                text: 'Number of Warnings'
                            }
                        }
                    }
                }
            }
        );
        
        // Warning Density Chart
        const densityData = {
            labels: labels,
            datasets: [{
                label: 'Warning Density',
                data: [$(printf "%s, " "${WARNING_DENSITY[@]}")],
                borderColor: '#f39c12',
                backgroundColor: 'rgba(243, 156, 18, 0.2)',
                fill: true
            }]
        };
        
        new Chart(
            document.getElementById('densityChart'),
            {
                type: 'line',
                data: densityData,
                options: {
                    ...commonOptions,
                    scales: {
                        ...commonOptions.scales,
                        y: {
                            beginAtZero: true,
                            title: {
                                display: true,
                                text: 'Warnings per 1000 LOC'
                            }
                        }
                    }
                }
            }
        );
        
        // Test Coverage Chart
        const coverageData = {
            labels: labels,
            datasets: [{
                label: 'Test Coverage',
                data: [$(printf "%s, " "${TEST_COVERAGE[@]}")],
                borderColor: '#2ecc71',
                backgroundColor: 'rgba(46, 204, 113, 0.2)',
                fill: true
            }]
        };
        
        new Chart(
            document.getElementById('coverageChart'),
            {
                type: 'line',
                data: coverageData,
                options: {
                    ...commonOptions,
                    scales: {
                        ...commonOptions.scales,
                        y: {
                            min: 0,
                            max: 100,
                            title: {
                                display: true,
                                text: 'Coverage (%)'
                            }
                        }
                    }
                }
            }
        );
        
        // Documentation Coverage Chart
        const docCoverageData = {
            labels: labels,
            datasets: [{
                label: 'Documentation Coverage',
                data: [$(printf "%s, " "${DOC_COVERAGE[@]}")],
                borderColor: '#3498db',
                backgroundColor: 'rgba(52, 152, 219, 0.2)',
                fill: true
            }]
        };
        
        new Chart(
            document.getElementById('docCoverageChart'),
            {
                type: 'line',
                data: docCoverageData,
                options: {
                    ...commonOptions,
                    scales: {
                        ...commonOptions.scales,
                        y: {
                            min: 0,
                            max: 100,
                            title: {
                                display: true,
                                text: 'Coverage (%)'
                            }
                        }
                    }
                }
            }
        );
        
        // Lines of Code Chart
        const locData = {
            labels: labels,
            datasets: [{
                label: 'Lines of Code',
                data: [$(printf "%s, " "${TOTAL_LOC[@]}")],
                borderColor: '#9b59b6',
                backgroundColor: 'rgba(155, 89, 182, 0.2)',
                fill: true
            }]
        };
        
        new Chart(
            document.getElementById('locChart'),
            {
                type: 'line',
                data: locData,
                options: {
                    ...commonOptions,
                    scales: {
                        ...commonOptions.scales,
                        y: {
                            beginAtZero: false,
                            title: {
                                display: true,
                                text: 'Lines of Code'
                            }
                        }
                    }
                }
            }
        );
    </script>
</body>
</html>
EOF

    echo -e "${GREEN}HTML charts generated successfully!${RESET}"
    echo -e "Charts saved to: ${BOLD}$HTML_FILE${RESET}"

elif [ "$CHART_FORMAT" = "markdown" ]; then
    echo -e "${YELLOW}Generating Markdown charts...${RESET}"
    
    # Create Markdown file with ASCII charts
    MD_FILE="$OUTPUT_DIR/metrics_charts.md"
    
    cat > "$MD_FILE" << EOF
# IntelliRouter Code Quality Metrics Charts

Generated on $(date '+%Y-%m-%d at %H:%M:%S')

## Summary of Latest Metrics

| Metric | Value |
|--------|-------|
| Total Warnings | ${TOTAL_WARNINGS[-1]} |
| Warning Density | ${WARNING_DENSITY[-1]} warnings per 1000 lines |
| Test Coverage | ${TEST_COVERAGE[-1]}% |
| Documentation Coverage | ${DOC_COVERAGE[-1]}% |
| Total Lines of Code | ${TOTAL_LOC[-1]} |

## Metrics Over Time

### Total Warnings

Date | Warnings
-----|--------
EOF

    # Add data rows for total warnings
    for i in $(seq 0 $((${#TIMESTAMPS[@]} - 1))); do
        echo "${TIMESTAMPS[$i]} | ${TOTAL_WARNINGS[$i]}" >> "$MD_FILE"
    done

    cat >> "$MD_FILE" << EOF

### Warning Density

Date | Density
-----|--------
EOF

    # Add data rows for warning density
    for i in $(seq 0 $((${#TIMESTAMPS[@]} - 1))); do
        echo "${TIMESTAMPS[$i]} | ${WARNING_DENSITY[$i]}" >> "$MD_FILE"
    done

    cat >> "$MD_FILE" << EOF

### Test Coverage

Date | Coverage (%)
-----|------------
EOF

    # Add data rows for test coverage
    for i in $(seq 0 $((${#TIMESTAMPS[@]} - 1))); do
        echo "${TIMESTAMPS[$i]} | ${TEST_COVERAGE[$i]}" >> "$MD_FILE"
    done

    cat >> "$MD_FILE" << EOF

### Documentation Coverage

Date | Coverage (%)
-----|------------
EOF

    # Add data rows for documentation coverage
    for i in $(seq 0 $((${#TIMESTAMPS[@]} - 1))); do
        echo "${TIMESTAMPS[$i]} | ${DOC_COVERAGE[$i]}" >> "$MD_FILE"
    done

    cat >> "$MD_FILE" << EOF

### Lines of Code

Date | LOC
-----|----
EOF

    # Add data rows for lines of code
    for i in $(seq 0 $((${#TIMESTAMPS[@]} - 1))); do
        echo "${TIMESTAMPS[$i]} | ${TOTAL_LOC[$i]}" >> "$MD_FILE"
    done

    echo -e "${GREEN}Markdown charts generated successfully!${RESET}"
    echo -e "Charts saved to: ${BOLD}$MD_FILE${RESET}"
fi

# Make the script executable
chmod +x "$0"