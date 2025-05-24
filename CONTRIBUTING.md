# Contributing to IntelliRouter

Thank you for your interest in contributing to IntelliRouter! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Compilation Best Practices](#compilation-best-practices)
- [Code Quality Contributions](#code-quality-contributions)
- [Performance Benchmarking](#performance-benchmarking)
- [Security Considerations](#security-considerations)
- [Project Dashboard](#project-dashboard)
- [Pull Request Process](#pull-request-process)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Documentation](#documentation)

## Code of Conduct

Please be respectful and considerate of others when contributing to this project. We aim to foster an inclusive and welcoming community.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/yourusername/intellirouter.git`
3. Add the upstream repository: `git remote add upstream https://github.com/originalowner/intellirouter.git`
4. Create a new branch for your changes: `git checkout -b feature/your-feature-name`

## Development Workflow

1. Make your changes in your feature branch
2. Ensure your code compiles without errors: `cargo check --all-targets`
3. Run the tests: `cargo test`
4. Format your code: `cargo fmt`
5. Run clippy: `cargo clippy`
6. Analyze warnings: `./scripts/analyze_warnings.sh`
7. Commit your changes with a descriptive commit message
8. Push your changes to your fork
9. Submit a pull request

## Compilation Best Practices

IntelliRouter maintains strict compilation standards to ensure code quality and reliability. Before submitting a pull request, please ensure your code follows these best practices:

### Ensuring Code Compiles Without Errors

- Run `cargo check --all-targets` to verify your code compiles without errors
- Fix any compilation errors before proceeding
- Use the warning analyzer (`./scripts/analyze_warnings.sh`) to identify and fix warnings

### Module Organization and Visibility

- Follow consistent module structure
- Use proper re-exports for public items
- Be explicit about item visibility

### Import Management

- Use correct import paths
- Organize imports by source
- Remove unused imports

### Trait Implementations

- Complete all required trait implementations
- Ensure trait bounds are satisfied
- Use correct trait paths

### Type Consistency

- Avoid duplicate type definitions
- Use consistent naming
- Implement proper type conversions

### Borrowing and Ownership

- Avoid borrow conflicts
- Manage lifetimes properly
- Consider using owned types when appropriate

For detailed guidelines, refer to the [Compilation Best Practices](docs/compilation_best_practices.md) document.

## Code Quality Contributions

IntelliRouter maintains a continuous improvement process for code quality. We welcome contributions that help improve the overall quality of the codebase.

### Code Quality Metrics

We track several code quality metrics:

- **Compiler Warnings**: Number of warnings reported by the Rust compiler
- **Warning Density**: Number of warnings per 1000 lines of code
- **Test Coverage**: Percentage of code covered by tests
- **Documentation Coverage**: Percentage of public items with documentation

Current metrics and goals are available in the [Code Quality Goals](docs/code_quality_goals.md) document.

### How to Contribute to Code Quality

1. **Run the Code Quality Report**
   - Execute `./scripts/generate_code_quality_report.sh` to generate a report
   - Review the report to identify areas for improvement

2. **Address Warnings**
   - Fix compiler warnings, especially unused imports and variables
   - Use `./scripts/analyze_warnings.sh` to identify common warning patterns
   - Submit PRs that focus specifically on warning reduction

3. **Improve Test Coverage**
   - Add tests for untested code
   - Focus on core functionality and public APIs
   - Run `cargo tarpaulin` to identify modules with low coverage

4. **Enhance Documentation**
   - Add documentation for undocumented public items
   - Improve existing documentation with examples
   - Run `cargo doc --no-deps` to identify undocumented items

5. **Submit Focused PRs**
   - Create PRs that focus specifically on code quality improvements
   - Include before/after metrics in your PR description
   - Reference the specific code quality goal being addressed

### Recognition

Contributors who significantly improve code quality will be recognized in:

- Release notes
- The code quality leaderboard
- Project documentation

For more information on code quality goals and strategies, see the [Code Quality Goals](docs/code_quality_goals.md) document.

## Performance Benchmarking

IntelliRouter includes a comprehensive performance benchmarking system to measure and track the performance of key components. We encourage contributors to run benchmarks and consider performance implications when making changes.

### Running Benchmarks

To run the benchmarks:

```bash
# Run all benchmarks
./scripts/run_benchmarks.sh

# Run a specific benchmark
cargo bench --bench router_benchmarks

# Run a specific benchmark function
cargo bench --bench router_benchmarks -- bench_router_creation
```

### Adding New Benchmarks

If you're adding new functionality, consider adding benchmarks to measure its performance:

1. Create a new benchmark file in the `benches/` directory or add to an existing one
2. Implement the `Benchmarkable` trait for your benchmark
3. Add your benchmark to the `criterion_group!` macro
4. Update the `scripts/run_benchmarks.sh` script if necessary

### Performance Considerations

When making changes to the codebase:

1. Run benchmarks before and after your changes to measure impact
2. Consider both latency and throughput implications
3. Document performance improvements or regressions in your PR
4. Address any performance regressions before submitting your PR

### Performance Regression Detection

The CI pipeline automatically runs benchmarks and detects performance regressions. If your PR causes a regression:

1. Review the regression report to understand the impact
2. Determine if the regression is acceptable (e.g., trading performance for correctness)
3. If not acceptable, optimize your code to address the regression
4. Document any intentional performance trade-offs in your PR

For more information, see the [Performance Benchmarking](docs/performance_benchmarking.md) document.

## Security Considerations

IntelliRouter includes a comprehensive security audit system to identify and address security vulnerabilities. We encourage contributors to consider security implications when making changes and to run security checks before submitting PRs.

### Running Security Checks

To run the security checks:

```bash
# Run all security checks
./scripts/security/run_security_audit.sh

# Run a specific security check
./scripts/security/check_dependencies.sh
```

### Security Best Practices

When contributing to IntelliRouter, please follow these security best practices:

1. **Dependency Management**
   - Avoid adding dependencies with known vulnerabilities
   - Keep dependencies up-to-date
   - Run `cargo audit` to check for vulnerable dependencies

2. **Code Security**
   - Avoid using `unwrap()` or `expect()` in production code
   - Validate all user inputs
   - Use safe alternatives to unsafe code when possible
   - Document any unsafe code thoroughly

3. **Configuration Security**
   - Never hardcode secrets or credentials
   - Use environment variables for sensitive configuration
   - Provide secure defaults for configuration options

4. **Authentication and Authorization**
   - Follow the principle of least privilege
   - Implement proper authentication checks
   - Validate authorization for all sensitive operations

5. **Data Validation**
   - Validate all inputs from external sources
   - Sanitize data before using it in sensitive contexts
   - Use parameterized queries for database operations

### Security Review Process

All pull requests will undergo a security review as part of the code review process. The security review will:

1. Check for common security issues
2. Verify that security best practices are followed
3. Ensure that security checks pass
4. Identify potential security vulnerabilities

If security issues are identified, you will be asked to address them before your PR can be merged.

For more information, see the [Security Audit System](docs/security_audit.md) document.

## Project Dashboard

IntelliRouter includes a unified project dashboard that integrates metrics from code quality, performance benchmarking, security audit, and documentation generation systems. The dashboard provides a comprehensive view of the project's health and quality, making it easier to monitor and improve the project.

### Using the Dashboard

To run the dashboard:

```bash
# Make the scripts executable
chmod +x dashboard/run_dashboard.sh dashboard/collect_metrics.sh

# Build and run the dashboard
cd dashboard
./run_dashboard.sh
```

The dashboard will be available at `http://localhost:8080`.

### Dashboard Features

- **Unified Metrics View**: Combines metrics from multiple systems into a single dashboard
- **Project Health Monitoring**: Calculates overall project health based on various metrics
- **Real-time Updates**: Automatically refreshes metrics at configurable intervals
- **Interactive Charts**: Visualizes trends and patterns in project metrics
- **Recommendations**: Provides actionable recommendations for improving project health

### Contributing to the Dashboard

When contributing to the dashboard:

1. **Run the Dashboard Locally**
   - Test your changes with the dashboard to ensure they integrate properly
   - Verify that metrics are collected and displayed correctly

2. **Update Dashboard Components**
   - Add new metrics to the dashboard when implementing new features
   - Update existing dashboard components when changing metrics

3. **Document Dashboard Changes**
   - Update the dashboard documentation when making changes
   - Include screenshots of the dashboard in your PR if relevant

4. **Consider Dashboard Integration**
   - When adding new features, consider how they can be monitored via the dashboard
   - Add appropriate metrics collection for new components

For more information, see the [Project Dashboard](docs/project_dashboard.md) document.

## Pull Request Process

1. Ensure your code compiles without errors and passes all tests
2. Update documentation as necessary
3. Include a clear description of the changes in your pull request
4. Link any related issues in your pull request description
5. Wait for the automated code review bot to analyze your changes
6. Address any issues identified by the automated code review
7. Wait for a maintainer to review your pull request
8. Address any feedback from the review
9. Once approved, your pull request will be merged

### Automated Code Review

When you submit a pull request, our automated code review bot will analyze your changes and provide feedback. The bot:

1. **Checks for compilation errors and warnings**
   - Identifies any compilation issues in your code
   - Provides inline comments on specific lines with issues

2. **Analyzes code style and formatting**
   - Checks for adherence to Rust style guidelines
   - Identifies formatting inconsistencies

3. **Evaluates test coverage**
   - Measures test coverage for your changes
   - Flags files with insufficient test coverage

4. **Reviews documentation completeness**
   - Checks for missing documentation on public items
   - Ensures documentation is up-to-date with code changes

5. **Provides a summary report**
   - Generates an overview of all issues found
   - Includes actionable suggestions for improvement

The code review bot is configured through `.github/code-review-config.yml`. You can customize its behavior by modifying this file.

### Pull Request Checklist

Before submitting your pull request, make sure you can check off all of these items:

- [ ] Code compiles without errors (`cargo check --all-targets`)
- [ ] All tests pass (`cargo test`)
- [ ] Tests were written before implementation (test-first approach)
- [ ] PR includes evidence of test-first approach (e.g., commit history or test failure screenshots)
- [ ] Code is formatted properly (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Warning analyzer shows no critical warnings (`./scripts/analyze_warnings.sh`)
- [ ] Code quality report shows improvement or no regression (`./scripts/generate_code_quality_report.sh --compare`)
- [ ] Performance benchmarks show no significant regressions (`./scripts/run_benchmarks.sh`)
- [ ] Security checks pass (`./scripts/security/run_security_audit.sh`)
- [ ] No vulnerable dependencies are introduced (`./scripts/security/check_dependencies.sh`)
- [ ] Security best practices are followed
- [ ] Dashboard metrics are updated if relevant (`dashboard/collect_metrics.sh`)
- [ ] Dashboard components are updated if adding new metrics
- [ ] Documentation is updated
- [ ] Changes are tested
- [ ] Commit messages are clear and descriptive

## Coding Standards

### Rust Style Guidelines

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `snake_case` for variables, functions, and modules
- Use `CamelCase` for types, traits, and enums
- Use `SCREAMING_SNAKE_CASE` for constants
- Add documentation comments (`///`) for public items
- Keep functions small and focused
- Follow the principle of least privilege for visibility

### Documentation

- Document all public items with doc comments (`///`)
- Include examples in documentation where appropriate
- Keep documentation up-to-date with code changes

## Testing

### Test-First Approach

IntelliRouter follows a test-first development approach. This means:

- Write tests before implementing the functionality
- Verify that tests fail appropriately before implementation
- Implement the minimum code needed to make tests pass
- Refactor while maintaining passing tests

For detailed guidelines, see the [Test-First Development Rule](.roo/rules/test_first.md) and our comprehensive [Testing Policy](docs/testing_policy.md).

To install the test-first pre-commit hook that enforces this approach:

```bash
# Make the script executable
chmod +x scripts/install_test_first_hook.sh

# Install the hook
./scripts/install_test_first_hook.sh
```

### General Testing Guidelines

- Write unit tests for all new functionality
- Update existing tests when changing functionality
- Add integration tests for new features
- Consider adding property-based tests for invariants
- Mark longer tests with the `#[ignore]` attribute
- Add benchmarks for performance-critical code
- Run benchmarks to ensure no performance regressions

## Documentation

IntelliRouter maintains a comprehensive documentation generation system to ensure that the project remains well-documented as it evolves. When contributing to the project, please follow these documentation guidelines:

### General Documentation Guidelines

- Update documentation when adding or changing features
- Keep the README up-to-date
- Add examples for new features
- Document API changes
- Run the documentation generation system to check documentation coverage

### API Documentation

- Document all public items with doc comments:
  - Use `///` for Rust code
  - Use docstrings for Python code
  - Use JSDoc comments for TypeScript code
- Include examples in documentation where appropriate
- Document parameters, return values, and exceptions/errors
- Explain the purpose and behavior of each item
- Document any limitations or edge cases

### User Guides

- Update user guides when adding or changing features
- Create new user guides for new modules or significant features
- Include step-by-step instructions and examples
- Use clear and concise language

### Architecture Documentation

- Update architecture documentation when making architectural changes
- Include diagrams to illustrate the architecture
- Explain design decisions and trade-offs
- Document integration with other components

### Examples and Tutorials

- Add examples for new features
- Update existing examples when changing functionality
- Include comments in example code to explain the code
- Create step-by-step tutorials for common tasks

### Running the Documentation System

To check documentation coverage and generate documentation, run:

```bash
# Make the scripts executable
chmod +x scripts/docs/*.sh

# Generate documentation
./scripts/docs/generate_docs.sh

# Check documentation coverage
./scripts/docs/check_doc_coverage.sh
```

For more information on the documentation system, see [Documentation System](docs/documentation_system.md).

Thank you for contributing to IntelliRouter!