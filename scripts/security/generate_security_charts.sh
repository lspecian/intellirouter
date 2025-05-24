#!/bin/bash
# IntelliRouter Security Metrics Chart Generator
# This script generates charts from security metrics data

set -e

# Default values
INPUT_FILE="metrics/security/security_metrics.csv"
OUTPUT_FILE="metrics/security/security_trends.png"
CHART_TYPE="line"
TITLE="IntelliRouter Security Trends"
WIDTH=1200
HEIGHT=800
VERBOSE=false

# Help message
show_help() {
    echo "IntelliRouter Security Metrics Chart Generator"
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  -h, --help                   Show this help message"
    echo "  -i, --input FILE             Input CSV file (default: metrics/security/security_metrics.csv)"
    echo "  -o, --output FILE            Output image file (default: metrics/security/security_trends.png)"
    echo "  -t, --type TYPE              Chart type (line, bar) (default: line)"
    echo "  --title TITLE                Chart title (default: IntelliRouter Security Trends)"
    echo "  --width WIDTH                Chart width in pixels (default: 1200)"
    echo "  --height HEIGHT              Chart height in pixels (default: 800)"
    echo "  -v, --verbose                Enable verbose output"
    echo ""
    exit 0
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    key="$1"
    case $key in
        -h|--help)
            show_help
            ;;
        -i|--input)
            INPUT_FILE="$2"
            shift
            shift
            ;;
        -o|--output)
            OUTPUT_FILE="$2"
            shift
            shift
            ;;
        -t|--type)
            CHART_TYPE="$2"
            shift
            shift
            ;;
        --title)
            TITLE="$2"
            shift
            shift
            ;;
        --width)
            WIDTH="$2"
            shift
            shift
            ;;
        --height)
            HEIGHT="$2"
            shift
            shift
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            ;;
    esac
done

# Check if input file exists
if [[ ! -f "$INPUT_FILE" ]]; then
    echo "Error: Input file not found: $INPUT_FILE"
    exit 1
fi

# Create output directory if it doesn't exist
OUTPUT_DIR=$(dirname "$OUTPUT_FILE")
mkdir -p "$OUTPUT_DIR"

# Check if gnuplot is installed
if ! command -v gnuplot &> /dev/null; then
    echo "Error: gnuplot is not installed."
    echo "Please install it with your package manager (e.g., apt-get install gnuplot)"
    exit 1
fi

# Print configuration if verbose
if [[ "$VERBOSE" == true ]]; then
    echo "IntelliRouter Security Metrics Chart Generator"
    echo "=============================================="
    echo "Input file: $INPUT_FILE"
    echo "Output file: $OUTPUT_FILE"
    echo "Chart type: $CHART_TYPE"
    echo "Chart title: $TITLE"
    echo "Chart dimensions: ${WIDTH}x${HEIGHT}"
    echo ""
fi

# Create a temporary gnuplot script
TEMP_SCRIPT=$(mktemp)
trap 'rm -f "$TEMP_SCRIPT"' EXIT

# Count the number of data points
DATA_POINTS=$(wc -l < "$INPUT_FILE")
DATA_POINTS=$((DATA_POINTS - 1)) # Subtract header row

if [[ "$DATA_POINTS" -lt 2 ]]; then
    echo "Warning: Not enough data points for a meaningful chart (found $DATA_POINTS, need at least 2)."
    echo "Will generate a chart anyway, but it may not be very informative."
fi

# Create the gnuplot script
cat > "$TEMP_SCRIPT" << EOF
set terminal pngcairo enhanced font "Arial,12" size $WIDTH,$HEIGHT
set output "$OUTPUT_FILE"
set title "$TITLE" font "Arial,16"
set datafile separator ","
set key outside right top
set grid
set xdata time
set timefmt "%Y-%m-%d"
set format x "%Y-%m-%d"
set xlabel "Date"
set ylabel "Number of Issues"
set style data $CHART_TYPE
set style fill transparent solid 0.5
set xtics rotate by 45 right
EOF

# Add different plot commands based on chart type
if [[ "$CHART_TYPE" == "line" ]]; then
    cat >> "$TEMP_SCRIPT" << EOF
plot "$INPUT_FILE" using 1:3 title "Critical" with linespoints lw 2 lc rgb "#d9534f", \
     "$INPUT_FILE" using 1:4 title "High" with linespoints lw 2 lc rgb "#f0ad4e", \
     "$INPUT_FILE" using 1:5 title "Medium" with linespoints lw 2 lc rgb "#5bc0de", \
     "$INPUT_FILE" using 1:6 title "Low" with linespoints lw 2 lc rgb "#5cb85c", \
     "$INPUT_FILE" using 1:2 title "Total" with linespoints lw 3 lc rgb "#333333"
EOF
elif [[ "$CHART_TYPE" == "bar" ]]; then
    cat >> "$TEMP_SCRIPT" << EOF
set style data histograms
set style histogram rowstacked
set boxwidth 0.8 relative
set style fill solid 1.0 border -1
plot "$INPUT_FILE" using 3:xtic(1) title "Critical" lc rgb "#d9534f", \
     "" using 4 title "High" lc rgb "#f0ad4e", \
     "" using 5 title "Medium" lc rgb "#5bc0de", \
     "" using 6 title "Low" lc rgb "#5cb85c"
EOF
fi

# Run gnuplot
if [[ "$VERBOSE" == true ]]; then
    echo "Running gnuplot..."
    cat "$TEMP_SCRIPT"
fi

gnuplot "$TEMP_SCRIPT"

echo "Chart generated: $OUTPUT_FILE"

# Generate additional charts for each severity level
if [[ "$VERBOSE" == true ]]; then
    echo "Generating individual severity charts..."
fi

# Critical issues trend
CRITICAL_OUTPUT="${OUTPUT_FILE%.*}_critical.png"
cat > "$TEMP_SCRIPT" << EOF
set terminal pngcairo enhanced font "Arial,12" size $WIDTH,$HEIGHT
set output "$CRITICAL_OUTPUT"
set title "Critical Security Issues Trend" font "Arial,16"
set datafile separator ","
set key outside right top
set grid
set xdata time
set timefmt "%Y-%m-%d"
set format x "%Y-%m-%d"
set xlabel "Date"
set ylabel "Number of Issues"
set style data $CHART_TYPE
set style fill transparent solid 0.5
set xtics rotate by 45 right
plot "$INPUT_FILE" using 1:3 title "Critical" with linespoints lw 2 lc rgb "#d9534f"
EOF

gnuplot "$TEMP_SCRIPT"
echo "Critical issues chart generated: $CRITICAL_OUTPUT"

# High issues trend
HIGH_OUTPUT="${OUTPUT_FILE%.*}_high.png"
cat > "$TEMP_SCRIPT" << EOF
set terminal pngcairo enhanced font "Arial,12" size $WIDTH,$HEIGHT
set output "$HIGH_OUTPUT"
set title "High Security Issues Trend" font "Arial,16"
set datafile separator ","
set key outside right top
set grid
set xdata time
set timefmt "%Y-%m-%d"
set format x "%Y-%m-%d"
set xlabel "Date"
set ylabel "Number of Issues"
set style data $CHART_TYPE
set style fill transparent solid 0.5
set xtics rotate by 45 right
plot "$INPUT_FILE" using 1:4 title "High" with linespoints lw 2 lc rgb "#f0ad4e"
EOF

gnuplot "$TEMP_SCRIPT"
echo "High issues chart generated: $HIGH_OUTPUT"

# Generate an HTML dashboard with all charts
DASHBOARD_OUTPUT="${OUTPUT_FILE%.*}_dashboard.html"
cat > "$DASHBOARD_OUTPUT" << EOF
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>IntelliRouter Security Metrics Dashboard</title>
  <style>
    body { font-family: Arial, sans-serif; margin: 0; padding: 20px; }
    h1 { color: #333; }
    h2 { color: #555; margin-top: 30px; }
    .chart-container { margin-bottom: 40px; }
    .chart { max-width: 100%; height: auto; border: 1px solid #ddd; }
    table { border-collapse: collapse; width: 100%; margin-bottom: 20px; }
    th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
    th { background-color: #f2f2f2; }
    tr:nth-child(even) { background-color: #f9f9f9; }
    .critical { color: #d9534f; }
    .high { color: #f0ad4e; }
    .medium { color: #5bc0de; }
    .low { color: #5cb85c; }
  </style>
</head>
<body>
  <h1>IntelliRouter Security Metrics Dashboard</h1>
  <p><strong>Generated:</strong> $(date)</p>
  
  <h2>Security Issues Trend</h2>
  <div class="chart-container">
    <img src="$(basename "$OUTPUT_FILE")" alt="Security Issues Trend" class="chart">
  </div>
  
  <h2>Critical Issues Trend</h2>
  <div class="chart-container">
    <img src="$(basename "$CRITICAL_OUTPUT")" alt="Critical Issues Trend" class="chart">
  </div>
  
  <h2>High Issues Trend</h2>
  <div class="chart-container">
    <img src="$(basename "$HIGH_OUTPUT")" alt="High Issues Trend" class="chart">
  </div>
  
  <h2>Raw Data</h2>
  <table>
    <tr>
      <th>Date</th>
      <th>Total</th>
      <th>Critical</th>
      <th>High</th>
      <th>Medium</th>
      <th>Low</th>
    </tr>
EOF

# Add data rows
tail -n +2 "$INPUT_FILE" | while IFS=, read -r date total critical high medium low; do
    cat >> "$DASHBOARD_OUTPUT" << EOF
    <tr>
      <td>$date</td>
      <td>$total</td>
      <td class="critical">$critical</td>
      <td class="high">$high</td>
      <td class="medium">$medium</td>
      <td class="low">$low</td>
    </tr>
EOF
done

# Close the HTML file
cat >> "$DASHBOARD_OUTPUT" << EOF
  </table>
</body>
</html>
EOF

echo "Dashboard generated: $DASHBOARD_OUTPUT"

exit 0