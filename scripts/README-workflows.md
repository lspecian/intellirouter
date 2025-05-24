# IntelliRouter CI/CD Workflows

This document explains the CI/CD workflow structure of the IntelliRouter project and how to test workflows locally.

## Workflow Structure

The project has 7 different workflow files, each handling a specific aspect of testing and deployment:

1. **cd.yml** - Continuous Deployment (handles deployment to production)
2. **ci.yml** - Continuous Integration (general build verification, compilation checks, testing, linting, security scanning)
3. **codeql-analysis.yml** - Code Quality Analysis (security scanning)
4. **dependency-review.yml** - Dependency Review (checks for vulnerable dependencies)
5. **e2e-tests.yml** - End-to-End Tests (integration testing)
6. **security.yml** - Security Checks (additional security scanning)
7. **test.yml** - Unit Tests, Coverage, and Linting

This separation of concerns is a common practice in large projects because it:
- Makes the CI/CD pipeline more maintainable
- Allows for parallel execution of different types of tests
- Enables selective execution (e.g., run only security checks when security-related code changes)
- Provides clearer failure points when issues arise

## Testing Workflows Locally

You can test GitHub Actions workflows locally using [act](https://github.com/nektos/act), a tool that allows you to run GitHub Actions locally without pushing to GitHub.

### Prerequisites

1. Install Docker (required by act)
2. Install act:
   ```bash
   # macOS
   brew install act
   
   # Linux
   curl -s https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash
   ```

### Using the Workflow Testing Script

We've provided a script to simplify testing workflows locally:

```bash
# Make the script executable (if not already)
chmod +x scripts/test-workflows-locally.sh

# Run the script
./scripts/test-workflows-locally.sh
```

The script provides a menu to:
- Run all workflows
- Run specific workflows (ci, test, e2e-tests, codeql-analysis)
- Run the compilation check job specifically
- Run specific jobs from any workflow

### Manual Usage of act

If you prefer to use act directly:

1. Run all workflows triggered by push:
   ```bash
   act push
   ```

2. Run a specific workflow:
   ```bash
   act -W .github/workflows/test.yml push
   ```

3. Run a specific job from a workflow:
   ```bash
   act -W .github/workflows/test.yml -j test
   ```

### Configuration

The `.actrc` file in the project root contains recommended settings for act:
```
-P ubuntu-latest=nektos/act-environments-ubuntu:18.04
--env FONTCONFIG_NO_PKG_CONFIG=1
```

This configuration:
- Uses the Ubuntu 18.04 Docker image for running workflows
- Sets the `FONTCONFIG_NO_PKG_CONFIG=1` environment variable to bypass pkg-config requirements for fontconfig

## Common Issues and Solutions

1. **System Dependencies**: Some workflows require system dependencies like `libfontconfig1-dev`. These are installed in the workflow files, but you may need to install them locally if testing with act.

2. **Docker Image Size**: The default Docker images used by act can be large. Use the `-P` flag to specify a smaller image if needed.

3. **Secrets**: If your workflows use secrets, you can provide them using the `--secret` flag or by creating a `.secrets` file.

4. **GitHub API Rate Limiting**: Some workflows may hit GitHub API rate limits. You can provide a GitHub token using the `--token` flag.

## CI Workflow Details

The CI workflow (`ci.yml`) includes several jobs that run in sequence:

1. **check**: Compilation status check that runs `cargo check` on:
   - Library code (`--lib`)
   - Binary code (`--bins`)
   - All targets (`--all-targets`)
   
   This job runs first and helps catch compilation errors early in the development process.

2. **build**: Builds the project on multiple operating systems and Rust versions.

3. **test**: Runs unit tests.

4. **lint**: Checks code formatting and runs clippy for static analysis.

5. **security**: Runs security audits on dependencies.

6. **coverage**: Generates code coverage reports.

7. **warnings**: Analyzes compilation warnings and provides suggestions for fixing them.
   - Runs `cargo check --message-format=json` to get warnings in JSON format
   - Categorizes warnings by type (e.g., unused variables, unused imports)
   - Identifies files with the most warnings
   - Provides suggestions for fixing common warnings
   - Generates a markdown report that is uploaded as an artifact
   
   This job is configured to be non-blocking, meaning it won't fail the build if warnings are found.

The compilation check job is particularly important as it helps prevent compilation issues from being merged into the codebase. It's faster than a full build and provides quick feedback on code correctness.

## Conclusion

Testing workflows locally with act can save time and reduce the number of commits needed to fix issues. It's a valuable tool for CI/CD pipeline development and maintenance.

Running the compilation check job locally before pushing changes is highly recommended to catch compilation errors early:

```bash
./scripts/test-workflows-locally.sh
# Select option 6 to run the compilation check job
```

Additionally, running the warning analyzer locally can help identify and fix potential issues before they accumulate:

```bash
./scripts/analyze_warnings.sh
```

If you want to analyze warnings even when there are compilation errors:

```bash
./scripts/analyze_warnings.sh --allow-errors
```

This generates a `warning_report.md` file with detailed information about warnings in the codebase, including suggestions for fixing common warning types.

These proactive steps help maintain code quality and prevent both compilation issues and warnings from accumulating in the codebase.

## Automated Tests for Compilation Tools

The IntelliRouter project includes automated tests for the compilation tools to ensure they continue to work correctly as the codebase evolves. These tests are run as part of the CI pipeline in the `compilation_tool_tests` job.

### Compilation Status Check Tests

The compilation status check tests verify that the compilation status check correctly identifies various types of compilation errors:

```bash
# Run compilation check tests
cargo test -p intellirouter --test test_compilation_check
```

These tests create temporary Rust projects with intentional compilation errors and verify that the compilation status check correctly identifies them. The tests cover:

- Library code errors
- Binary code errors
- Module visibility errors
- Trait implementation errors
- Import path errors

### Warning Analyzer Tests

The warning analyzer tests verify that the warning analyzer correctly identifies and categorizes different types of warnings:

```bash
# Run warning analyzer tests
./tests/compilation_tools/test_warning_analyzer.sh
```

These tests create temporary Rust files with intentional warnings and verify that the warning analyzer correctly identifies and categorizes them. The tests cover:

- Unused variable warnings
- Unused import warnings
- Dead code warnings
- Multiple warning types
- Naming convention warnings
- Unused field warnings

### Running Tests in CI

The `compilation_tool_tests` job in the CI pipeline runs both the compilation check tests and the warning analyzer tests:

```yaml
compilation_tool_tests:
  name: Compilation Tool Tests
  runs-on: ubuntu-latest
  needs: [check, warnings]
  steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@master
      with:
        toolchain: 1.82.0
        components: rustfmt, clippy
    
    - name: Rust Cache
      uses: Swatinem/rust-cache@v2
    
    - name: Install jq
      run: sudo apt-get install -y jq
    
    - name: Run compilation check tests
      run: cargo test --test test_compilation_check -- --test-threads=1
    
    - name: Run warning analyzer tests
      run: ./tests/compilation_tools/test_warning_analyzer.sh
```

You can also run these tests locally using the workflow testing script:

```bash
./scripts/test-workflows-locally.sh
# Select option to run the compilation_tool_tests job
```

These tests help ensure that our compilation tools continue to work correctly as the codebase evolves, allowing us to catch regressions early in the development process.