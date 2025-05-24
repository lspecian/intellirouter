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
PR_NUMBER=""
OUTPUT_FILE="code_review_report.json"
CONFIG_FILE=".github/code-review-config.yml"

for arg in "$@"; do
    case $arg in
        --pr=*)
            PR_NUMBER="${arg#*=}"
            shift
            ;;
        --output=*)
            OUTPUT_FILE="${arg#*=}"
            shift
            ;;
        --config=*)
            CONFIG_FILE="${arg#*=}"
            shift
            ;;
    esac
done

echo -e "${BOLD}${BLUE}IntelliRouter Automated Code Review${RESET}"
echo -e "${CYAN}====================================${RESET}\n"

# Check if PR number is provided
if [ -z "$PR_NUMBER" ]; then
    echo -e "${YELLOW}No PR number provided. Running analysis on all changed files in the current branch.${RESET}"
    DIFF_FILES=$(git diff --name-only HEAD~1 HEAD | grep -E '\.rs$|\.py$|\.ts$|\.proto$')
else
    echo -e "${YELLOW}Running analysis on PR #$PR_NUMBER${RESET}"
    DIFF_FILES=$(git diff --name-only origin/main | grep -E '\.rs$|\.py$|\.ts$|\.proto$')
fi

# Create temporary files
TEMP_REPORT=$(mktemp)
TEMP_CLIPPY=$(mktemp)
TEMP_RUSTFMT=$(mktemp)
TEMP_DOC=$(mktemp)

# Initialize counters
TOTAL_ISSUES=0
COMPILATION_ISSUES=0
STYLE_ISSUES=0
PERFORMANCE_ISSUES=0
DOCUMENTATION_ISSUES=0

# Initialize arrays for issues
COMPILATION_ISSUES_ARRAY=()
STYLE_ISSUES_ARRAY=()
PERFORMANCE_ISSUES_ARRAY=()
DOCUMENTATION_ISSUES_ARRAY=()
FILE_COMMENTS=()

# Load configuration if available
IGNORE_PATTERNS=""
if [ -f "$CONFIG_FILE" ]; then
    echo -e "${YELLOW}Loading configuration from $CONFIG_FILE${RESET}"
    # Extract ignore patterns
    IGNORE_PATTERNS=$(grep "ignore_patterns:" "$CONFIG_FILE" -A 10 | grep -v "ignore_patterns:" | grep "^  - " | sed 's/^  - //')
fi

# Filter out ignored files
if [ -n "$IGNORE_PATTERNS" ]; then
    FILTERED_FILES=""
    for file in $DIFF_FILES; do
        IGNORE=false
        for pattern in $IGNORE_PATTERNS; do
            if [[ $file =~ $pattern ]]; then
                IGNORE=true
                break
            fi
        done
        if [ "$IGNORE" = false ]; then
            FILTERED_FILES="$FILTERED_FILES $file"
        fi
    done
    DIFF_FILES=$FILTERED_FILES
fi

echo -e "${YELLOW}Analyzing ${#DIFF_FILES[@]} changed files...${RESET}"

# Run compilation check
echo -e "${YELLOW}Running compilation check...${RESET}"
cargo check --message-format=json > "$TEMP_CLIPPY" 2>&1 || {
    echo -e "${RED}Compilation failed. This will be reported in the review.${RESET}"
    COMPILATION_ISSUES=$((COMPILATION_ISSUES + 1))
    COMPILATION_ISSUES_ARRAY+=("Compilation failed. Please fix compilation errors before merging.")
}

# Extract compilation errors and warnings for changed files
for file in $DIFF_FILES; do
    if [[ $file == *.rs ]]; then
        # Extract errors and warnings for this file
        FILE_ERRORS=$(grep -A 2 "\"level\":\"error\"" "$TEMP_CLIPPY" | grep -A 1 "\"file_name\":\"$file\"" || true)
        FILE_WARNINGS=$(grep -A 2 "\"level\":\"warning\"" "$TEMP_CLIPPY" | grep -A 1 "\"file_name\":\"$file\"" || true)
        
        if [ -n "$FILE_ERRORS" ]; then
            COMPILATION_ISSUES=$((COMPILATION_ISSUES + 1))
            # Extract line numbers and messages
            while IFS= read -r line; do
                if [[ $line == *"line_start"* ]]; then
                    LINE_NUM=$(echo "$line" | grep -o "\"line_start\":[0-9]*" | cut -d':' -f2)
                    MESSAGE=$(echo "$FILE_ERRORS" | grep -A 1 "$line" | grep "\"message\":" | cut -d':' -f2- | sed 's/^[ \t]*//;s/[ \t]*$//' | sed 's/^"//' | sed 's/"$//' | sed 's/",$//')
                    
                    if [ -n "$LINE_NUM" ] && [ -n "$MESSAGE" ]; then
                        FILE_COMMENTS+=("{\"path\":\"$file\",\"line\":$LINE_NUM,\"body\":\"🔴 **Compilation Error:** $MESSAGE\"}")
                    fi
                fi
            done <<< "$FILE_ERRORS"
        fi
        
        if [ -n "$FILE_WARNINGS" ]; then
            # Extract line numbers and messages
            while IFS= read -r line; do
                if [[ $line == *"line_start"* ]]; then
                    LINE_NUM=$(echo "$line" | grep -o "\"line_start\":[0-9]*" | cut -d':' -f2)
                    MESSAGE=$(echo "$FILE_WARNINGS" | grep -A 1 "$line" | grep "\"message\":" | cut -d':' -f2- | sed 's/^[ \t]*//;s/[ \t]*$//' | sed 's/^"//' | sed 's/"$//' | sed 's/",$//')
                    
                    if [ -n "$LINE_NUM" ] && [ -n "$MESSAGE" ]; then
                        FILE_COMMENTS+=("{\"path\":\"$file\",\"line\":$LINE_NUM,\"body\":\"⚠️ **Warning:** $MESSAGE\"}")
                        STYLE_ISSUES=$((STYLE_ISSUES + 1))
                    fi
                fi
            done <<< "$FILE_WARNINGS"
        fi
    fi
done

# Run clippy for Rust files
echo -e "${YELLOW}Running clippy for Rust files...${RESET}"
cargo clippy --message-format=json > "$TEMP_CLIPPY" 2>&1 || {
    echo -e "${YELLOW}Clippy reported issues. These will be included in the review.${RESET}"
}

# Extract clippy warnings for changed files
for file in $DIFF_FILES; do
    if [[ $file == *.rs ]]; then
        # Extract clippy warnings for this file
        FILE_CLIPPY=$(grep -A 5 "\"level\":\"warning\"" "$TEMP_CLIPPY" | grep -A 4 "\"file_name\":\"$file\"" || true)
        
        if [ -n "$FILE_CLIPPY" ]; then
            # Extract line numbers and messages
            while IFS= read -r line; do
                if [[ $line == *"line_start"* ]]; then
                    LINE_NUM=$(echo "$line" | grep -o "\"line_start\":[0-9]*" | cut -d':' -f2)
                    MESSAGE=$(echo "$FILE_CLIPPY" | grep -A 2 "$line" | grep "\"message\":" | cut -d':' -f2- | sed 's/^[ \t]*//;s/[ \t]*$//' | sed 's/^"//' | sed 's/"$//' | sed 's/",$//')
                    CODE=$(echo "$FILE_CLIPPY" | grep -A 3 "$line" | grep "\"code\":" | grep -o "\"code\":\"[^\"]*\"" | cut -d':' -f2 | sed 's/^"//' | sed 's/"$//')
                    
                    if [ -n "$LINE_NUM" ] && [ -n "$MESSAGE" ]; then
                        # Determine if it's a style or performance issue
                        if [[ $CODE == *"clippy::perf"* || $MESSAGE == *"performance"* ]]; then
                            FILE_COMMENTS+=("{\"path\":\"$file\",\"line\":$LINE_NUM,\"body\":\"🐢 **Performance Issue:** $MESSAGE\"}")
                            PERFORMANCE_ISSUES=$((PERFORMANCE_ISSUES + 1))
                        else
                            FILE_COMMENTS+=("{\"path\":\"$file\",\"line\":$LINE_NUM,\"body\":\"🔧 **Style Issue:** $MESSAGE\"}")
                            STYLE_ISSUES=$((STYLE_ISSUES + 1))
                        fi
                    fi
                fi
            done <<< "$FILE_CLIPPY"
        fi
    fi
done

# Check rustfmt for Rust files
echo -e "${YELLOW}Checking code formatting...${RESET}"
for file in $DIFF_FILES; do
    if [[ $file == *.rs ]]; then
        rustfmt --check "$file" > "$TEMP_RUSTFMT" 2>&1 || {
            STYLE_ISSUES=$((STYLE_ISSUES + 1))
            STYLE_ISSUES_ARRAY+=("$file: Formatting issues detected. Run 'rustfmt $file' to fix.")
            FILE_COMMENTS+=("{\"path\":\"$file\",\"line\":1,\"body\":\"📝 **Formatting Issue:** This file has formatting issues. Please run 'rustfmt $file' to fix.\"}")
        }
    fi
done

# Check documentation for Rust files
echo -e "${YELLOW}Checking documentation...${RESET}"
cargo doc --no-deps > "$TEMP_DOC" 2>&1
for file in $DIFF_FILES; do
    if [[ $file == *.rs ]]; then
        # Check for missing documentation warnings
        DOC_WARNINGS=$(grep -A 1 "warning: missing documentation" "$TEMP_DOC" | grep "$file" || true)
        
        if [ -n "$DOC_WARNINGS" ]; then
            # Extract line numbers and items
            while IFS= read -r line; do
                if [[ $line == *"$file"* ]]; then
                    LINE_INFO=$(echo "$line" | grep -o "$file:[0-9]*:[0-9]*")
                    LINE_NUM=$(echo "$LINE_INFO" | cut -d':' -f2)
                    ITEM=$(echo "$line" | sed "s/.*$file:[0-9]*:[0-9]*: //" | sed 's/`//g')
                    
                    if [ -n "$LINE_NUM" ] && [ -n "$ITEM" ]; then
                        FILE_COMMENTS+=("{\"path\":\"$file\",\"line\":$LINE_NUM,\"body\":\"📚 **Documentation Issue:** Missing documentation for $ITEM\"}")
                        DOCUMENTATION_ISSUES=$((DOCUMENTATION_ISSUES + 1))
                    fi
                fi
            done <<< "$DOC_WARNINGS"
        fi
    fi
done

# Run test coverage analysis
echo -e "${YELLOW}Running test coverage analysis...${RESET}"
COVERAGE_TEMP=$(mktemp)
cargo tarpaulin --out json --output-dir . > "$COVERAGE_TEMP" 2>&1 || {
    echo -e "${YELLOW}Tarpaulin reported issues. Continuing with analysis...${RESET}"
}

# Extract test coverage
if [ -f "tarpaulin-report.json" ]; then
    TEST_COVERAGE=$(grep -o '"line_coverage":[0-9.]*' tarpaulin-report.json | cut -d':' -f2)
    TEST_COVERAGE_FORMATTED=$(printf "%.2f" "$TEST_COVERAGE")
    
    # Check coverage for changed files
    for file in $DIFF_FILES; do
        if [[ $file == *.rs ]]; then
            FILE_COVERAGE=$(grep -A 10 "\"$file\"" tarpaulin-report.json | grep -o '"line_coverage":[0-9.]*' | head -1 | cut -d':' -f2 || echo "0")
            
            if [ -n "$FILE_COVERAGE" ] && (( $(echo "$FILE_COVERAGE < 70" | bc -l) )); then
                FILE_COMMENTS+=("{\"path\":\"$file\",\"line\":1,\"body\":\"🧪 **Test Coverage:** This file has only $FILE_COVERAGE% test coverage. Consider adding more tests.\"}")
                DOCUMENTATION_ISSUES=$((DOCUMENTATION_ISSUES + 1))
            fi
        fi
    done
else
    TEST_COVERAGE_FORMATTED="N/A"
fi

# Calculate total issues
TOTAL_ISSUES=$((COMPILATION_ISSUES + STYLE_ISSUES + PERFORMANCE_ISSUES + DOCUMENTATION_ISSUES))

# Generate summary
SUMMARY=""
if [ $TOTAL_ISSUES -eq 0 ]; then
    SUMMARY="✅ **Great job!** No issues were found in the changed files."
else
    SUMMARY="⚠️ **Issues Found:** The automated code review found $TOTAL_ISSUES issues that should be addressed before merging."
    
    if [ $COMPILATION_ISSUES -gt 0 ]; then
        SUMMARY="$SUMMARY\n\n**Compilation Issues:** $COMPILATION_ISSUES"
        for issue in "${COMPILATION_ISSUES_ARRAY[@]}"; do
            SUMMARY="$SUMMARY\n- $issue"
        done
    fi
    
    if [ $STYLE_ISSUES -gt 0 ]; then
        SUMMARY="$SUMMARY\n\n**Style Issues:** $STYLE_ISSUES"
        for issue in "${STYLE_ISSUES_ARRAY[@]}"; do
            SUMMARY="$SUMMARY\n- $issue"
        done
    fi
    
    if [ $PERFORMANCE_ISSUES -gt 0 ]; then
        SUMMARY="$SUMMARY\n\n**Performance Issues:** $PERFORMANCE_ISSUES"
        for issue in "${PERFORMANCE_ISSUES_ARRAY[@]}"; do
            SUMMARY="$SUMMARY\n- $issue"
        done
    fi
    
    if [ $DOCUMENTATION_ISSUES -gt 0 ]; then
        SUMMARY="$SUMMARY\n\n**Documentation Issues:** $DOCUMENTATION_ISSUES"
        for issue in "${DOCUMENTATION_ISSUES_ARRAY[@]}"; do
            SUMMARY="$SUMMARY\n- $issue"
        done
    fi
    
    SUMMARY="$SUMMARY\n\n**Test Coverage:** $TEST_COVERAGE_FORMATTED%"
fi

# Generate JSON report
echo -e "${YELLOW}Generating JSON report...${RESET}"
cat > "$OUTPUT_FILE" << EOF
{
  "total_issues": $TOTAL_ISSUES,
  "compilation_issues": ${COMPILATION_ISSUES},
  "style_issues": ${STYLE_ISSUES},
  "performance_issues": ${PERFORMANCE_ISSUES},
  "documentation_issues": ${DOCUMENTATION_ISSUES},
  "test_coverage": "$TEST_COVERAGE_FORMATTED",
  "summary": $(echo "$SUMMARY" | jq -R -s .),
  "file_comments": [
    $(IFS=,; echo "${FILE_COMMENTS[*]}")
  ]
}
EOF

echo -e "${GREEN}Code review completed!${RESET}"
echo -e "Report saved to: ${BOLD}$OUTPUT_FILE${RESET}"

# Clean up temporary files
rm -f "$TEMP_REPORT" "$TEMP_CLIPPY" "$TEMP_RUSTFMT" "$TEMP_DOC" "$COVERAGE_TEMP"

# Make the script executable
chmod +x "$0"