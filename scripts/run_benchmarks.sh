#!/bin/bash
# Script to run benchmarks and store results

# Set variables
METRICS_DIR="metrics/performance"
REPORTS_DIR="$METRICS_DIR/reports"
CHARTS_DIR="$METRICS_DIR/charts"
DATE=$(date +"%Y-%m-%d_%H-%M-%S")
BENCHMARK_RESULT_FILE="$METRICS_DIR/benchmark_results_$DATE.csv"
PERFORMANCE_REPORT="$REPORTS_DIR/performance_report_$DATE.md"
REGRESSION_REPORT="$REPORTS_DIR/regression_report_$DATE.md"

# Create directories if they don't exist
mkdir -p "$METRICS_DIR"
mkdir -p "$REPORTS_DIR"
mkdir -p "$CHARTS_DIR"

# Print header
echo "Running IntelliRouter Performance Benchmarks"
echo "============================================"
echo "Date: $DATE"
echo "Results will be stored in: $BENCHMARK_RESULT_FILE"
echo "Performance report will be generated at: $PERFORMANCE_REPORT"
echo "Regression report will be generated at: $REGRESSION_REPORT"
echo

# Run all benchmarks
echo "Running router benchmarks..."
cargo bench --bench router_benchmarks -- --verbose

echo "Running model registry benchmarks..."
cargo bench --bench model_registry_benchmarks -- --verbose

echo "Running chain engine benchmarks..."
cargo bench --bench chain_engine_benchmarks -- --verbose

echo "Running memory benchmarks..."
cargo bench --bench memory_benchmarks -- --verbose

echo "Running RAG manager benchmarks..."
cargo bench --bench rag_manager_benchmarks -- --verbose

# Copy benchmark results to metrics directory
echo "Copying benchmark results to metrics directory..."
cp target/criterion/*/new/estimates.json "$METRICS_DIR/$(date +"%Y-%m-%d_%H-%M-%S")_estimates.json"

# Generate performance report
echo "Generating performance report..."
cat > "$PERFORMANCE_REPORT" << EOF
# IntelliRouter Performance Report

Generated: $(date)

## Summary

This report contains the results of performance benchmarks for the IntelliRouter system.

## Benchmark Results

| Component | Benchmark | Median | Mean | Std Dev | Min | Max |
|-----------|-----------|--------|------|---------|-----|-----|
EOF

# Parse benchmark results and add to report
for COMPONENT in "router" "model_registry" "chain_engine" "memory" "rag_manager"; do
    for RESULT_FILE in $(find target/criterion/${COMPONENT}_* -name estimates.json); do
        BENCHMARK=$(echo "$RESULT_FILE" | sed -E 's/.*\/([^\/]+)\/estimates.json/\1/')
        
        # Extract metrics from JSON file
        MEDIAN=$(jq '.median.point_estimate' "$RESULT_FILE")
        MEAN=$(jq '.mean.point_estimate' "$RESULT_FILE")
        STD_DEV=$(jq '.std_dev.point_estimate' "$RESULT_FILE")
        MIN=$(jq '.slope.point_estimate' "$RESULT_FILE")
        MAX=$(jq '.mean.point_estimate + .std_dev.point_estimate' "$RESULT_FILE")
        
        # Convert from nanoseconds to milliseconds for readability
        MEDIAN_MS=$(echo "scale=3; $MEDIAN / 1000000" | bc)
        MEAN_MS=$(echo "scale=3; $MEAN / 1000000" | bc)
        STD_DEV_MS=$(echo "scale=3; $STD_DEV / 1000000" | bc)
        MIN_MS=$(echo "scale=3; $MIN / 1000000" | bc)
        MAX_MS=$(echo "scale=3; $MAX / 1000000" | bc)
        
        # Add to report
        echo "| $COMPONENT | $BENCHMARK | $MEDIAN_MS ms | $MEAN_MS ms | $STD_DEV_MS ms | $MIN_MS ms | $MAX_MS ms |" >> "$PERFORMANCE_REPORT"
        
        # Add to CSV file
        echo "$COMPONENT,$BENCHMARK,$MEDIAN_MS,$MEAN_MS,$STD_DEV_MS,$MIN_MS,$MAX_MS,$(date +"%Y-%m-%d %H:%M:%S")" >> "$BENCHMARK_RESULT_FILE"
    done
done

# Add charts section to report
cat >> "$PERFORMANCE_REPORT" << EOF

## Performance Charts

The following charts show the performance of each component over time.

EOF

# Generate charts (placeholder - in a real implementation, this would use a charting library)
echo "Generating performance charts..."
echo "Note: Chart generation is a placeholder. In a real implementation, this would use a charting library."

# Check for performance regressions
echo "Checking for performance regressions..."

# Compare with previous benchmark results if they exist
PREV_RESULTS=$(find "$METRICS_DIR" -name "benchmark_results_*.csv" -not -name "benchmark_results_$DATE.csv" | sort -r | head -n 1)

if [ -n "$PREV_RESULTS" ]; then
    echo "Comparing with previous results: $PREV_RESULTS"
    
    # Generate regression report header
    cat > "$REGRESSION_REPORT" << EOF
# IntelliRouter Performance Regression Report

Generated: $(date)
Comparing with: $(basename "$PREV_RESULTS")

## Performance Changes

| Component | Benchmark | Previous (ms) | Current (ms) | Change (%) | Status |
|-----------|-----------|--------------|-------------|------------|--------|
EOF
    
    # Parse current and previous results and detect regressions
    while IFS=, read -r COMPONENT BENCHMARK MEDIAN MEAN STD_DEV MIN MAX TIMESTAMP; do
        # Skip header if present
        if [ "$COMPONENT" = "Component" ]; then
            continue
        fi
        
        # Find corresponding benchmark in previous results
        PREV_LINE=$(grep "^$COMPONENT,$BENCHMARK," "$PREV_RESULTS")
        
        if [ -n "$PREV_LINE" ]; then
            PREV_MEAN=$(echo "$PREV_LINE" | cut -d, -f4)
            
            # Calculate change percentage
            CHANGE=$(echo "scale=2; ($MEAN - $PREV_MEAN) / $PREV_MEAN * 100" | bc)
            
            # Determine status
            if (( $(echo "$CHANGE > 5" | bc -l) )); then
                STATUS="⚠️ Regression"
            elif (( $(echo "$CHANGE < -5" | bc -l) )); then
                STATUS="✅ Improvement"
            else
                STATUS="✓ Stable"
            fi
            
            # Add to report
            echo "| $COMPONENT | $BENCHMARK | $PREV_MEAN | $MEAN | $CHANGE% | $STATUS |" >> "$REGRESSION_REPORT"
        fi
    done < "$BENCHMARK_RESULT_FILE"
    
    # Add summary to regression report
    REGRESSION_COUNT=$(grep -c "⚠️ Regression" "$REGRESSION_REPORT")
    IMPROVEMENT_COUNT=$(grep -c "✅ Improvement" "$REGRESSION_REPORT")
    STABLE_COUNT=$(grep -c "✓ Stable" "$REGRESSION_REPORT")
    
    cat >> "$REGRESSION_REPORT" << EOF

## Summary

- Regressions: $REGRESSION_COUNT
- Improvements: $IMPROVEMENT_COUNT
- Stable: $STABLE_COUNT

EOF
    
    # Add recommendations if there are regressions
    if [ "$REGRESSION_COUNT" -gt 0 ]; then
        cat >> "$REGRESSION_REPORT" << EOF
## Recommendations

The following benchmarks show significant performance regressions:

$(grep "⚠️ Regression" "$REGRESSION_REPORT" | sed 's/^/- /')

These should be investigated to determine the cause and potential fixes.

EOF
    fi
else
    echo "No previous results found for comparison."
    
    # Generate a simple regression report
    cat > "$REGRESSION_REPORT" << EOF
# IntelliRouter Performance Regression Report

Generated: $(date)

No previous benchmark results found for comparison. This will serve as the baseline for future regression detection.

EOF
fi

echo "Done!"
echo "Performance report: $PERFORMANCE_REPORT"
echo "Regression report: $REGRESSION_REPORT"
echo "Benchmark results: $BENCHMARK_RESULT_FILE"