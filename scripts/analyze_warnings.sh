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
ALLOW_ERRORS=false
for arg in "$@"; do
    case $arg in
        --allow-errors)
            ALLOW_ERRORS=true
            shift
            ;;
    esac
done

echo -e "${BOLD}${BLUE}IntelliRouter Compilation Warning Analyzer${RESET}"
echo -e "${CYAN}========================================${RESET}\n"

# Create a temporary file for the JSON output
TEMP_FILE=$(mktemp)
REPORT_FILE="warning_report.md"

# Run cargo check with JSON output
echo -e "${YELLOW}Running cargo check to collect warnings...${RESET}"
cargo check --message-format=json > "$TEMP_FILE" || {
    if [ "$ALLOW_ERRORS" = true ]; then
        echo -e "${YELLOW}Warning: Compilation errors detected, but continuing with warning analysis due to --allow-errors flag.${RESET}"
        echo -e "${YELLOW}Note: Some warnings may be related to compilation errors and might be resolved when errors are fixed.${RESET}"
    else
        echo -e "${RED}Error: Compilation failed. Please fix compilation errors before analyzing warnings.${RESET}"
        echo -e "${YELLOW}Tip: Run 'cargo check' to see detailed error messages.${RESET}"
        echo -e "${YELLOW}     Or use --allow-errors flag to analyze warnings despite compilation errors.${RESET}"
        rm "$TEMP_FILE"
        exit 1
    fi
}

# Extract warnings from the JSON output
echo -e "${YELLOW}Analyzing warnings...${RESET}"

# Count total warnings
TOTAL_WARNINGS=$(grep -c "\"level\":\"warning\"" "$TEMP_FILE")
echo -e "${BOLD}Total warnings: ${RED}$TOTAL_WARNINGS${RESET}\n"

# Create arrays to store warning data
declare -A WARNING_TYPES
declare -A WARNING_FILES
declare -A WARNING_MESSAGES
declare -A COMMON_WARNINGS

# Process each warning
while IFS= read -r line; do
    # Extract warning information
    MESSAGE=$(echo "$line" | jq -r '.message.message // empty')
    if [ -n "$MESSAGE" ]; then
        # Extract file path and line number
        FILE=$(echo "$line" | jq -r '.message.spans[0].file_name // empty')
        LINE=$(echo "$line" | jq -r '.message.spans[0].line_start // empty')
        
        # Extract warning code
        CODE=$(echo "$line" | jq -r '.message.code.code // empty')
        
        # Categorize warning
        if [[ "$MESSAGE" == *"unused variable"* ]]; then
            TYPE="unused_variable"
            WARNING_TYPES["$TYPE"]=$((${WARNING_TYPES["$TYPE"]:-0} + 1))
            COMMON_WARNINGS["$TYPE"]="${MESSAGE}"
        elif [[ "$MESSAGE" == *"unused import"* || "$MESSAGE" == *"unused extern crate"* ]]; then
            TYPE="unused_import"
            WARNING_TYPES["$TYPE"]=$((${WARNING_TYPES["$TYPE"]:-0} + 1))
            COMMON_WARNINGS["$TYPE"]="${MESSAGE}"
        elif [[ "$MESSAGE" == *"dead code"* ]]; then
            TYPE="dead_code"
            WARNING_TYPES["$TYPE"]=$((${WARNING_TYPES["$TYPE"]:-0} + 1))
            COMMON_WARNINGS["$TYPE"]="${MESSAGE}"
        elif [[ "$MESSAGE" == *"function is never used"* ]]; then
            TYPE="unused_function"
            WARNING_TYPES["$TYPE"]=$((${WARNING_TYPES["$TYPE"]:-0} + 1))
            COMMON_WARNINGS["$TYPE"]="${MESSAGE}"
        elif [[ "$MESSAGE" == *"field is never read"* ]]; then
            TYPE="unused_field"
            WARNING_TYPES["$TYPE"]=$((${WARNING_TYPES["$TYPE"]:-0} + 1))
            COMMON_WARNINGS["$TYPE"]="${MESSAGE}"
        elif [[ "$MESSAGE" == *"lint level"* ]]; then
            TYPE="lint_level"
            WARNING_TYPES["$TYPE"]=$((${WARNING_TYPES["$TYPE"]:-0} + 1))
            COMMON_WARNINGS["$TYPE"]="${MESSAGE}"
        elif [[ "$MESSAGE" == *"should have a snake case"* ]]; then
            TYPE="naming_convention"
            WARNING_TYPES["$TYPE"]=$((${WARNING_TYPES["$TYPE"]:-0} + 1))
            COMMON_WARNINGS["$TYPE"]="${MESSAGE}"
        elif [[ "$MESSAGE" == *"deprecated"* ]]; then
            TYPE="deprecated"
            WARNING_TYPES["$TYPE"]=$((${WARNING_TYPES["$TYPE"]:-0} + 1))
            COMMON_WARNINGS["$TYPE"]="${MESSAGE}"
        else
            TYPE="other"
            WARNING_TYPES["$TYPE"]=$((${WARNING_TYPES["$TYPE"]:-0} + 1))
        fi
        
        # Count warnings per file
        if [ -n "$FILE" ]; then
            WARNING_FILES["$FILE"]=$((${WARNING_FILES["$FILE"]:-0} + 1))
        fi
        
        # Store warning message with file and line
        if [ -n "$FILE" ] && [ -n "$LINE" ]; then
            WARNING_MESSAGES["$FILE:$LINE"]="$MESSAGE"
        fi
    fi
done < <(grep "\"level\":\"warning\"" "$TEMP_FILE")

# Display warning types
echo -e "${BOLD}${BLUE}Warning Types:${RESET}"
for type in "${!WARNING_TYPES[@]}"; do
    echo -e "${CYAN}$type${RESET}: ${WARNING_TYPES[$type]}"
done
echo ""

# Display top 10 files with most warnings
echo -e "${BOLD}${BLUE}Top 10 Files with Most Warnings:${RESET}"
for file in $(for k in "${!WARNING_FILES[@]}"; do echo "$k ${WARNING_FILES[$k]}"; done | sort -rn -k2 | head -10 | cut -d' ' -f1); do
    echo -e "${CYAN}$file${RESET}: ${WARNING_FILES[$file]}"
done
echo ""

# Generate suggestions for common warning types
echo -e "${BOLD}${BLUE}Suggestions for Common Warning Types:${RESET}"

if [ ${WARNING_TYPES["unused_variable"]:-0} -gt 0 ]; then
    echo -e "${BOLD}${CYAN}Unused Variables (${WARNING_TYPES["unused_variable"]} warnings):${RESET}"
    echo -e "  - Prefix unused variables with underscore (_variable_name)"
    echo -e "  - Remove unused variables if they're not needed"
    echo -e "  - Example fix: \`let unused = 5;\` → \`let _unused = 5;\`"
    echo ""
fi

if [ ${WARNING_TYPES["unused_import"]:-0} -gt 0 ]; then
    echo -e "${BOLD}${CYAN}Unused Imports (${WARNING_TYPES["unused_import"]} warnings):${RESET}"
    echo -e "  - Remove unused imports"
    echo -e "  - Use cargo-fix with \`cargo fix --allow-dirty\` to automatically remove unused imports"
    echo -e "  - Consider using an IDE with auto-import cleanup"
    echo ""
fi

if [ ${WARNING_TYPES["dead_code"]:-0} -gt 0 ]; then
    echo -e "${BOLD}${CYAN}Dead Code (${WARNING_TYPES["dead_code"]} warnings):${RESET}"
    echo -e "  - Remove unused functions, methods, or structs"
    echo -e "  - If the code is for future use, add #[allow(dead_code)] attribute"
    echo -e "  - Example: \`#[allow(dead_code)]\nfn unused_function() {}\`"
    echo ""
fi

if [ ${WARNING_TYPES["unused_function"]:-0} -gt 0 ]; then
    echo -e "${BOLD}${CYAN}Unused Functions (${WARNING_TYPES["unused_function"]} warnings):${RESET}"
    echo -e "  - Remove functions that are never called"
    echo -e "  - If the function is for testing or future use, add #[allow(dead_code)] attribute"
    echo -e "  - Example: \`#[allow(dead_code)]\nfn test_helper() {}\`"
    echo ""
fi

if [ ${WARNING_TYPES["unused_field"]:-0} -gt 0 ]; then
    echo -e "${BOLD}${CYAN}Unused Fields (${WARNING_TYPES["unused_field"]} warnings):${RESET}"
    echo -e "  - Remove fields that are never read"
    echo -e "  - If the field is for future use, add #[allow(dead_code)] attribute to the struct"
    echo -e "  - Example: \`#[allow(dead_code)]\nstruct Config { unused: String }\`"
    echo ""
fi

if [ ${WARNING_TYPES["naming_convention"]:-0} -gt 0 ]; then
    echo -e "${BOLD}${CYAN}Naming Convention Issues (${WARNING_TYPES["naming_convention"]} warnings):${RESET}"
    echo -e "  - Use snake_case for variables, functions, and modules"
    echo -e "  - Use CamelCase for types, traits, and enums"
    echo -e "  - Example fix: \`fn badName()\` → \`fn bad_name()\`"
    echo ""
fi

if [ ${WARNING_TYPES["deprecated"]:-0} -gt 0 ]; then
    echo -e "${BOLD}${CYAN}Deprecated Items (${WARNING_TYPES["deprecated"]} warnings):${RESET}"
    echo -e "  - Update code to use non-deprecated alternatives"
    echo -e "  - Check documentation for recommended replacements"
    echo ""
fi

# Generate a markdown report
echo -e "${YELLOW}Generating markdown report...${RESET}"

cat > "$REPORT_FILE" << EOF
# IntelliRouter Compilation Warning Report

**Total Warnings:** $TOTAL_WARNINGS

## Warning Types

| Type | Count |
|------|-------|
EOF

for type in "${!WARNING_TYPES[@]}"; do
    echo "| $type | ${WARNING_TYPES[$type]} |" >> "$REPORT_FILE"
done

cat >> "$REPORT_FILE" << EOF

## Top 10 Files with Most Warnings

| File | Count |
|------|-------|
EOF

for file in $(for k in "${!WARNING_FILES[@]}"; do echo "$k ${WARNING_FILES[$k]}"; done | sort -rn -k2 | head -10 | cut -d' ' -f1); do
    echo "| $file | ${WARNING_FILES[$file]} |" >> "$REPORT_FILE"
done

cat >> "$REPORT_FILE" << EOF

## Suggestions for Common Warning Types

EOF

if [ ${WARNING_TYPES["unused_variable"]:-0} -gt 0 ]; then
    cat >> "$REPORT_FILE" << EOF
### Unused Variables (${WARNING_TYPES["unused_variable"]} warnings)

- Prefix unused variables with underscore (_variable_name)
- Remove unused variables if they're not needed
- Example fix: \`let unused = 5;\` → \`let _unused = 5;\`

EOF
fi

if [ ${WARNING_TYPES["unused_import"]:-0} -gt 0 ]; then
    cat >> "$REPORT_FILE" << EOF
### Unused Imports (${WARNING_TYPES["unused_import"]} warnings)

- Remove unused imports
- Use cargo-fix with \`cargo fix --allow-dirty\` to automatically remove unused imports
- Consider using an IDE with auto-import cleanup

EOF
fi

if [ ${WARNING_TYPES["dead_code"]:-0} -gt 0 ]; then
    cat >> "$REPORT_FILE" << EOF
### Dead Code (${WARNING_TYPES["dead_code"]} warnings)

- Remove unused functions, methods, or structs
- If the code is for future use, add #[allow(dead_code)] attribute
- Example: \`#[allow(dead_code)]\nfn unused_function() {}\`

EOF
fi

if [ ${WARNING_TYPES["unused_function"]:-0} -gt 0 ]; then
    cat >> "$REPORT_FILE" << EOF
### Unused Functions (${WARNING_TYPES["unused_function"]} warnings)

- Remove functions that are never called
- If the function is for testing or future use, add #[allow(dead_code)] attribute
- Example: \`#[allow(dead_code)]\nfn test_helper() {}\`

EOF
fi

if [ ${WARNING_TYPES["unused_field"]:-0} -gt 0 ]; then
    cat >> "$REPORT_FILE" << EOF
### Unused Fields (${WARNING_TYPES["unused_field"]} warnings)

- Remove fields that are never read
- If the field is for future use, add #[allow(dead_code)] attribute to the struct
- Example: \`#[allow(dead_code)]\nstruct Config { unused: String }\`

EOF
fi

if [ ${WARNING_TYPES["naming_convention"]:-0} -gt 0 ]; then
    cat >> "$REPORT_FILE" << EOF
### Naming Convention Issues (${WARNING_TYPES["naming_convention"]} warnings)

- Use snake_case for variables, functions, and modules
- Use CamelCase for types, traits, and enums
- Example fix: \`fn badName()\` → \`fn bad_name()\`

EOF
fi

if [ ${WARNING_TYPES["deprecated"]:-0} -gt 0 ]; then
    cat >> "$REPORT_FILE" << EOF
### Deprecated Items (${WARNING_TYPES["deprecated"]} warnings)

- Update code to use non-deprecated alternatives
- Check documentation for recommended replacements

EOF
fi

echo -e "${GREEN}Warning analysis complete!${RESET}"
echo -e "Markdown report saved to ${BOLD}${REPORT_FILE}${RESET}"

# Clean up
rm "$TEMP_FILE"