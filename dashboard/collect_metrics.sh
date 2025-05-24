#!/bin/bash
# Script to collect metrics from various sources and update the dashboard data

set -e

# Define paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DATA_DIR="$SCRIPT_DIR/data"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# Create data directories if they don't exist
mkdir -p "$DATA_DIR/code_quality"
mkdir -p "$DATA_DIR/performance"
mkdir -p "$DATA_DIR/security"
mkdir -p "$DATA_DIR/documentation"

# Print header
echo "IntelliRouter Dashboard Metrics Collector"
echo "========================================"
echo "Date: $(date)"
echo "Project root: $PROJECT_ROOT"
echo "Data directory: $DATA_DIR"
echo ""

# Collect code quality metrics
echo "Collecting code quality metrics..."
if [ -f "$PROJECT_ROOT/scripts/generate_code_quality_report.sh" ]; then
    # Run the code quality report script
    "$PROJECT_ROOT/scripts/generate_code_quality_report.sh" --output="$DATA_DIR/code_quality/code_quality_$TIMESTAMP.json"
    echo "Code quality metrics collected."
else
    echo "Warning: Code quality report script not found. Skipping code quality metrics collection."
fi

# Collect performance metrics
echo "Collecting performance metrics..."
if [ -f "$PROJECT_ROOT/scripts/run_benchmarks.sh" ]; then
    # Run the benchmarks script
    "$PROJECT_ROOT/scripts/run_benchmarks.sh"
    
    # Copy the latest benchmark results to the dashboard data directory
    LATEST_BENCHMARK=$(find "$PROJECT_ROOT/metrics/performance" -name "benchmark_results_*.csv" -type f -exec ls -t {} \; | head -n 1)
    if [ -n "$LATEST_BENCHMARK" ]; then
        cp "$LATEST_BENCHMARK" "$DATA_DIR/performance/benchmark_results_$TIMESTAMP.csv"
        echo "Performance metrics collected."
    else
        echo "Warning: No benchmark results found. Skipping performance metrics collection."
    fi
else
    echo "Warning: Benchmarks script not found. Skipping performance metrics collection."
fi

# Collect security metrics
echo "Collecting security metrics..."
if [ -f "$PROJECT_ROOT/scripts/security/run_security_audit.sh" ]; then
    # Run the security audit script
    "$PROJECT_ROOT/scripts/security/run_security_audit.sh" --output="$DATA_DIR/security" --report-only
    
    # Copy the latest security metrics to the dashboard data directory
    LATEST_SECURITY=$(find "$PROJECT_ROOT/metrics/security" -name "security_metrics.csv" -type f -exec ls -t {} \; | head -n 1)
    if [ -n "$LATEST_SECURITY" ]; then
        cp "$LATEST_SECURITY" "$DATA_DIR/security/security_metrics_$TIMESTAMP.csv"
        echo "Security metrics collected."
    else
        echo "Warning: No security metrics found. Skipping security metrics collection."
    fi
else
    echo "Warning: Security audit script not found. Skipping security metrics collection."
fi

# Collect documentation metrics
echo "Collecting documentation metrics..."
if [ -f "$PROJECT_ROOT/scripts/docs/check_doc_coverage.sh" ]; then
    # Run the documentation coverage script
    "$PROJECT_ROOT/scripts/docs/check_doc_coverage.sh" --output="$DATA_DIR/documentation/doc_coverage_$TIMESTAMP.json"
    echo "Documentation metrics collected."
else
    echo "Warning: Documentation coverage script not found. Skipping documentation metrics collection."
fi

# Generate dashboard data file
echo "Generating dashboard data file..."
DASHBOARD_DATA="$DATA_DIR/dashboard_data_$TIMESTAMP.json"

# Create a basic dashboard data structure
cat > "$DASHBOARD_DATA" << EOF
{
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "code_quality": {
    "total_warnings": 0,
    "warning_density": 0.0,
    "test_coverage": 0.0,
    "doc_coverage": 0.0,
    "lines_of_code": 0
  },
  "performance": {
    "benchmark_results": [],
    "regressions": []
  },
  "security": {
    "total_issues": 0,
    "critical_issues": 0,
    "high_issues": 0,
    "medium_issues": 0,
    "low_issues": 0,
    "vulnerabilities": []
  },
  "documentation": {
    "overall_coverage": 0.0,
    "api_coverage": 0.0,
    "user_guides_coverage": 0.0,
    "architecture_coverage": 0.0,
    "examples_coverage": 0.0
  },
  "project_health": {
    "overall_health": 0.0,
    "code_quality_score": 0.0,
    "performance_score": 0.0,
    "security_score": 0.0,
    "documentation_score": 0.0
  }
}
EOF

echo "Dashboard data file generated: $DASHBOARD_DATA"

# Create a symlink to the latest dashboard data file
ln -sf "$DASHBOARD_DATA" "$DATA_DIR/dashboard_data_latest.json"
echo "Symlink to latest dashboard data created: $DATA_DIR/dashboard_data_latest.json"

# Update the dashboard data with the collected metrics
echo "Updating dashboard data with collected metrics..."

# Update code quality metrics
LATEST_CODE_QUALITY=$(find "$DATA_DIR/code_quality" -name "code_quality_*.json" -type f -exec ls -t {} \; | head -n 1)
if [ -n "$LATEST_CODE_QUALITY" ]; then
    # Extract metrics from the code quality report
    TOTAL_WARNINGS=$(jq -r '.total_warnings // 0' "$LATEST_CODE_QUALITY")
    WARNING_DENSITY=$(jq -r '.warning_density // 0' "$LATEST_CODE_QUALITY")
    TEST_COVERAGE=$(jq -r '.test_coverage // 0' "$LATEST_CODE_QUALITY")
    DOC_COVERAGE=$(jq -r '.documentation_coverage // 0' "$LATEST_CODE_QUALITY")
    LINES_OF_CODE=$(jq -r '.total // 0' "$LATEST_CODE_QUALITY")
    
    # Update the dashboard data
    jq --arg tw "$TOTAL_WARNINGS" \
       --arg wd "$WARNING_DENSITY" \
       --arg tc "$TEST_COVERAGE" \
       --arg dc "$DOC_COVERAGE" \
       --arg loc "$LINES_OF_CODE" \
       '.code_quality.total_warnings = ($tw | tonumber) | 
        .code_quality.warning_density = ($wd | tonumber) | 
        .code_quality.test_coverage = ($tc | tonumber) | 
        .code_quality.doc_coverage = ($dc | tonumber) | 
        .code_quality.lines_of_code = ($loc | tonumber)' \
       "$DASHBOARD_DATA" > "$DASHBOARD_DATA.tmp" && mv "$DASHBOARD_DATA.tmp" "$DASHBOARD_DATA"
    
    echo "Code quality metrics updated."
fi

# Update security metrics
LATEST_SECURITY_METRICS=$(find "$DATA_DIR/security" -name "security_metrics_*.csv" -type f -exec ls -t {} \; | head -n 1)
if [ -n "$LATEST_SECURITY_METRICS" ]; then
    # Extract metrics from the security metrics file
    SECURITY_DATA=$(tail -n 1 "$LATEST_SECURITY_METRICS")
    IFS=',' read -r DATE TOTAL_ISSUES CRITICAL_ISSUES HIGH_ISSUES MEDIUM_ISSUES LOW_ISSUES <<< "$SECURITY_DATA"
    
    # Update the dashboard data
    jq --arg ti "$TOTAL_ISSUES" \
       --arg ci "$CRITICAL_ISSUES" \
       --arg hi "$HIGH_ISSUES" \
       --arg mi "$MEDIUM_ISSUES" \
       --arg li "$LOW_ISSUES" \
       '.security.total_issues = ($ti | tonumber) | 
        .security.critical_issues = ($ci | tonumber) | 
        .security.high_issues = ($hi | tonumber) | 
        .security.medium_issues = ($mi | tonumber) | 
        .security.low_issues = ($li | tonumber)' \
       "$DASHBOARD_DATA" > "$DASHBOARD_DATA.tmp" && mv "$DASHBOARD_DATA.tmp" "$DASHBOARD_DATA"
    
    echo "Security metrics updated."
fi

# Update documentation metrics
LATEST_DOC_COVERAGE=$(find "$DATA_DIR/documentation" -name "doc_coverage_*.json" -type f -exec ls -t {} \; | head -n 1)
if [ -n "$LATEST_DOC_COVERAGE" ]; then
    # Extract metrics from the documentation coverage report
    OVERALL_COVERAGE=$(jq -r '.overall_coverage // 0' "$LATEST_DOC_COVERAGE")
    API_COVERAGE=$(jq -r '.coverage.api_docs.coverage_percentage // 0' "$LATEST_DOC_COVERAGE")
    USER_GUIDES_COVERAGE=$(jq -r '.coverage.user_guides.coverage_percentage // 0' "$LATEST_DOC_COVERAGE")
    ARCHITECTURE_COVERAGE=$(jq -r '.coverage.architecture_docs.coverage_percentage // 0' "$LATEST_DOC_COVERAGE")
    EXAMPLES_COVERAGE=$(jq -r '.coverage.examples_docs.coverage_percentage // 0' "$LATEST_DOC_COVERAGE")
    
    # Update the dashboard data
    jq --arg oc "$OVERALL_COVERAGE" \
       --arg ac "$API_COVERAGE" \
       --arg uc "$USER_GUIDES_COVERAGE" \
       --arg arc "$ARCHITECTURE_COVERAGE" \
       --arg ec "$EXAMPLES_COVERAGE" \
       '.documentation.overall_coverage = ($oc | tonumber) | 
        .documentation.api_coverage = ($ac | tonumber) | 
        .documentation.user_guides_coverage = ($uc | tonumber) | 
        .documentation.architecture_coverage = ($arc | tonumber) | 
        .documentation.examples_coverage = ($ec | tonumber)' \
       "$DASHBOARD_DATA" > "$DASHBOARD_DATA.tmp" && mv "$DASHBOARD_DATA.tmp" "$DASHBOARD_DATA"
    
    echo "Documentation metrics updated."
fi

# Calculate project health scores
echo "Calculating project health scores..."

# Code quality score
CODE_QUALITY_SCORE=$(jq -r '
    # Calculate warning score (0-100, higher is better)
    ($warning_score = if .code_quality.warning_density > 0 then 
        100 * (1 - (.code_quality.warning_density / 20 | if . > 1 then 1 else . end)) 
    else 100 end) |
    
    # Calculate test coverage score (0-100)
    ($test_coverage_score = .code_quality.test_coverage) |
    
    # Calculate doc coverage score (0-100)
    ($doc_coverage_score = .code_quality.doc_coverage) |
    
    # Calculate overall code quality score
    ($warning_score * 0.3 + $test_coverage_score * 0.4 + $doc_coverage_score * 0.3) |
    
    # Ensure score is between 0 and 100
    if . > 100 then 100 elif . < 0 then 0 else . end |
    
    # Round to 1 decimal place
    round * 10 / 10
' "$DASHBOARD_DATA")

# Security score
SECURITY_SCORE=$(jq -r '
    # Calculate penalty for each issue type
    ($critical_penalty = .security.critical_issues * 20) |
    ($high_penalty = .security.high_issues * 10) |
    ($medium_penalty = .security.medium_issues * 5) |
    ($low_penalty = .security.low_issues * 1) |
    
    # Calculate security score
    (100 - $critical_penalty - $high_penalty - $medium_penalty - $low_penalty) |
    
    # Ensure score is between 0 and 100
    if . > 100 then 100 elif . < 0 then 0 else . end |
    
    # Round to 1 decimal place
    round * 10 / 10
' "$DASHBOARD_DATA")

# Documentation score
DOCUMENTATION_SCORE=$(jq -r '.documentation.overall_coverage | round * 10 / 10' "$DASHBOARD_DATA")

# Performance score (placeholder - in a real implementation, this would be calculated from benchmark results)
PERFORMANCE_SCORE=85.0

# Calculate overall health score
OVERALL_HEALTH=$(echo "scale=1; ($CODE_QUALITY_SCORE * 0.25 + $PERFORMANCE_SCORE * 0.25 + $SECURITY_SCORE * 0.3 + $DOCUMENTATION_SCORE * 0.2)" | bc)

# Update the dashboard data with health scores
jq --arg cqs "$CODE_QUALITY_SCORE" \
   --arg ps "$PERFORMANCE_SCORE" \
   --arg ss "$SECURITY_SCORE" \
   --arg ds "$DOCUMENTATION_SCORE" \
   --arg oh "$OVERALL_HEALTH" \
   '.project_health.code_quality_score = ($cqs | tonumber) | 
    .project_health.performance_score = ($ps | tonumber) | 
    .project_health.security_score = ($ss | tonumber) | 
    .project_health.documentation_score = ($ds | tonumber) | 
    .project_health.overall_health = ($oh | tonumber)' \
   "$DASHBOARD_DATA" > "$DASHBOARD_DATA.tmp" && mv "$DASHBOARD_DATA.tmp" "$DASHBOARD_DATA"

echo "Project health scores calculated and updated."

echo "Dashboard data collection complete."
echo "Dashboard data file: $DASHBOARD_DATA"
echo ""