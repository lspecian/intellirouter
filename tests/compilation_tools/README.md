# Compilation Tools Tests

This directory contains automated tests for the IntelliRouter compilation tools:

1. **Compilation Status Check Tests**: Tests that verify the compilation status check correctly identifies compilation errors in different parts of the codebase (library code, binary code, etc.).

2. **Warning Analyzer Tests**: Tests that verify the warning analyzer correctly identifies and categorizes different types of warnings.

These tests help ensure that our compilation tools continue to work correctly as the codebase evolves, allowing us to catch regressions early in the development process.

## Running the Tests

To run all compilation tool tests:

```bash
cargo test -p intellirouter --test test_compilation_check
./tests/compilation_tools/test_warning_analyzer.sh
```

To run only the compilation status check tests:

```bash
cargo test --test test_compilation_check -- --test-threads=1
```

To run only the warning analyzer tests:

```bash
./tests/compilation_tools/test_warning_analyzer.sh
```

## Adding New Tests

When adding new tests:

1. For compilation status check tests, add new test functions to `test_compilation_check.rs`.
2. For warning analyzer tests, add new test cases to `test_warning_analyzer.sh`.

Ensure that each test is focused on a specific aspect of the tool being tested and has clear assertions.