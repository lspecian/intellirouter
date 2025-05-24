# IntelliRouter

IntelliRouter is a programmable LLM gateway that provides an OpenAI-compatible API endpoint for chat completions, supporting both streaming and non-streaming responses. It's designed to be highly extensible and configurable, with support for various deployment scenarios.

## Features

- **Programmable Routing**: Route requests to different LLM backends based on customizable strategies
- **Extensibility**: Plugin system for custom routing strategies, model connectors, and telemetry exporters
- **Multi-Role Deployment**: Support for deploying as separate services with secure IPC
- **Client SDKs**: Python, TypeScript, and Rust libraries for easy integration
- **Deployment Options**: Configurations for various environments from edge to cloud
- **Simple Examples**: Easy-to-follow examples to help you get started quickly

## Getting Started

For a comprehensive guide on installing, configuring, and using IntelliRouter, see the [Getting Started Guide](docs/getting_started.md).

### Prerequisites

- Rust 1.70 or later
- Docker (for containerized deployment)
- Kubernetes (for production deployment)

### Installation

#### From Source

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/intellirouter.git
   cd intellirouter
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Run the router:
   ```bash
   ./target/release/intellirouter run
   ```

   You can also specify a specific role:
   ```bash
   ./target/release/intellirouter run --role router
   ```

   Available roles:
   - `router`: Runs the Router service
   - `orchestrator`: Runs the Orchestrator service
   - `rag-injector`: Runs the RAG Manager service
   - `summarizer`: Runs the Persona Layer service
   - `all`: Runs all services (default)

#### Using Docker

1. Build the Docker image:
   ```bash
   docker build -t intellirouter .
   ```

2. Run the container:
   ```bash
   docker run -p 8000:8000 intellirouter run
   ```

   You can also specify a specific role:
   ```bash
   docker run -p 8000:8000 intellirouter run --role router
   ```

#### Using Docker Compose

1. Start all services:
   ```bash
   docker-compose up -d
   ```

### Configuration

IntelliRouter can be configured using a TOML file. Create a `config.toml` file in the `config` directory:

```toml
[server]
host = "0.0.0.0"
port = 8000

[logging]
level = "info"

[redis]
host = "localhost"
port = 6379

[chromadb]
host = "localhost"
port = 8001
```

### Deployment Options

#### Local Development

For local development, you can use Docker Compose:

```bash
docker-compose -f docker-compose.dev.yml up -d
```

#### Edge Deployment

For edge deployment, use the edge-specific Docker Compose file:

```bash
cd deployment/edge
docker-compose up -d
```

#### Kubernetes Deployment

For Kubernetes deployment, use Helm:

```bash
# MicroK8s
cd deployment/microk8s
helm install intellirouter ../../helm/intellirouter -f values.yaml

# EKS
cd deployment/eks
helm install intellirouter ../../helm/intellirouter -f values.yaml

# GKE
cd deployment/gke
helm install intellirouter ../../helm/intellirouter -f values.yaml
```

## Architecture

IntelliRouter consists of several modules:

- **LLM Proxy**: OpenAI-compatible API endpoint
- **Model Registry**: Tracks available LLM backends
- **Router Core**: Routes requests to appropriate model backends
- **Persona Layer**: Injects system prompts and guardrails
- **Chain Engine**: Orchestrates multi-step inference flows
- **Memory**: Provides short-term and long-term memory capabilities
- **RAG Manager**: Manages Retrieval Augmented Generation
- **Authentication**: Handles API key validation and RBAC
- **Telemetry**: Collects logs, costs, and usage metrics
- **Plugin SDK**: Provides extensibility for custom components

## SDKs

IntelliRouter provides SDKs for easy integration:

- [Python SDK](sdk/python/README.md)
- [TypeScript SDK](sdk/typescript/README.md)
- [Rust SDK](sdk/rust/README.md)

## Examples

IntelliRouter comes with several examples to help you get started:

- **[Basic Usage Shell Script](examples/basic_usage.sh)**: Demonstrates how to start IntelliRouter and send requests using curl
- **[Simple Rust Client](examples/simple_client.rs)**: Shows how to connect to IntelliRouter from Rust code
- **[Configuration Example](examples/config/simple.toml)**: Example configuration file with detailed comments
- **[Examples README](examples/README.md)**: Detailed instructions for running the examples

These examples provide a starting point for understanding how to use IntelliRouter. See the [Examples README](examples/README.md) for more information.

## HTTP API

IntelliRouter provides an OpenAI-compatible HTTP API:

### Chat Completions

```
POST /v1/chat/completions
```

Request:
```json
{
  "model": "gpt-3.5-turbo",
  "messages": [
    {"role": "system", "content": "You are a helpful assistant."},
    {"role": "user", "content": "Hello!"}
  ]
}
```

Response:
```json
{
  "id": "chatcmpl-123",
  "object": "chat.completion",
  "created": 1677652288,
  "model": "gpt-3.5-turbo",
  "choices": [{
    "index": 0,
    "message": {
      "role": "assistant",
      "content": "Hello! How can I assist you today?"
    },
    "finish_reason": "stop"
  }]
}
```

## Testing

IntelliRouter follows a test-first development approach, where tests are written before implementing functionality. This ensures all code is testable and meets requirements from the start.

### Test-First Development

Our test-first approach requires:
- Writing tests before implementing functionality
- Verifying tests fail appropriately before implementation
- Implementing only what's needed to make tests pass
- Refactoring while maintaining passing tests

For more details, see [TESTING.md](TESTING.md), our [Test-First Development Rule](.roo/rules/test_first.md), and our comprehensive [Testing Policy](docs/testing_policy.md).

### Performance Benchmarking

IntelliRouter includes a comprehensive performance benchmarking system to measure the performance of key components, track metrics over time, and identify regressions:

- **Benchmarking Framework**: Reusable framework for creating and running benchmarks
- **Component Benchmarks**: Specific benchmarks for router core, model registry, chain engine, memory, and RAG manager
- **Performance Tracking**: Tools for storing and analyzing benchmark results over time
- **CI Integration**: Daily benchmark runs with regression detection
- **Reporting**: Performance reports and visualizations

To run the benchmarks:

```bash
# Run all benchmarks
./scripts/run_benchmarks.sh

# Run a specific benchmark
cargo bench --bench router_benchmarks
```

For more information, see [Performance Benchmarking](docs/performance_benchmarking.md).

### Security Auditing

IntelliRouter includes a comprehensive security audit system to identify security vulnerabilities, track security metrics over time, and provide guidance on fixing security issues:

- **Security Audit Framework**: Reusable framework for running security checks and collecting metrics
- **Security Checks**: Checks for dependency vulnerabilities, code vulnerabilities, configuration issues, authentication/authorization issues, and data validation issues
- **Security Metrics Tracking**: Tools for storing and analyzing security metrics over time
- **CI Integration**: Regular security audits with issue detection
- **Reporting**: Security reports and visualizations

To run the security audit:

```bash
# Run all security checks
./scripts/security/run_security_audit.sh

# Run a specific security check
./scripts/security/check_dependencies.sh
```

For more information, see [Security Audit System](docs/security_audit.md).

### Documentation Generation

IntelliRouter includes a comprehensive documentation generation system to automatically generate documentation from the codebase, track documentation coverage over time, and provide guidance on improving documentation:

- **Documentation Generation Framework**: Reusable framework for generating different types of documentation
- **Documentation Types**: API documentation, user guides, architecture documentation, examples and tutorials
- **Documentation Coverage Tracking**: Tools for tracking documentation coverage over time
- **CI Integration**: Regular documentation generation with regression detection
- **Reporting**: Documentation coverage reports and visualizations

To run the documentation generation:

```bash
# Make the scripts executable
chmod +x scripts/docs/*.sh

# Generate documentation
./scripts/docs/generate_docs.sh

# Check documentation coverage
./scripts/docs/check_doc_coverage.sh

# Generate documentation report
./scripts/docs/generate_doc_report.sh metrics/docs/doc_metrics_<timestamp>.json
```

The generated documentation will be available in the `docs/` directory, and the metrics and reports will be available in the `metrics/docs/` directory.

For more information, see [Documentation System](docs/documentation_system.md).

### Project Dashboard

IntelliRouter includes a unified project dashboard that integrates all the systems we've built (code quality, performance benchmarking, security audit, and documentation generation) into a single interface. This dashboard provides a comprehensive view of the project's health and quality, making it easier for developers to monitor and improve the project.

- **Unified Metrics View**: Combines metrics from multiple systems into a single dashboard
- **Project Health Monitoring**: Calculates overall project health based on various metrics
- **Real-time Updates**: Automatically refreshes metrics at configurable intervals
- **Interactive Charts**: Visualizes trends and patterns in project metrics
- **Recommendations**: Provides actionable recommendations for improving project health

To run the dashboard:

```bash
# Make the scripts executable
chmod +x dashboard/run_dashboard.sh dashboard/collect_metrics.sh

# Build and run the dashboard
cd dashboard
./run_dashboard.sh
```

The dashboard will be available at `http://localhost:8080`.

For more information, see [Project Dashboard](docs/project_dashboard.md).

### Router Role Testing

The router role can be tested using the provided integration tests and CLI script:

#### Running Integration Tests

```bash
# Run all tests
cargo test

# Run router-specific integration tests
cargo test --test router_integration_tests
```

#### Manual Testing with CLI Script

A CLI script is provided to test the router role manually:

```bash
# Start the router in one terminal
cargo run -- run --role router

# In another terminal, run the test script
./scripts/test-router.sh

# You can specify a different host, port, or model
./scripts/test-router.sh localhost 9000 mock-llama
```

The test script will:
1. Check if the router is running
2. Send a test request to the `/v1/chat/completions` endpoint
3. Validate the response structure
4. Display the results

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Automated Code Review

IntelliRouter uses an automated code review bot that analyzes pull requests for code quality issues and provides feedback. The bot:

1. Checks for compilation errors and warnings
2. Analyzes code style and formatting
3. Evaluates test coverage
4. Reviews documentation completeness
5. Provides inline comments on specific issues
6. Generates a summary report for each pull request

The code review bot is configured through `.github/code-review-config.yml` and can be customized to focus on specific aspects of code quality.

### Compilation Best Practices

IntelliRouter maintains strict compilation standards to ensure code quality and reliability. Before submitting a pull request, please:

1. Run `cargo check --all-targets` to verify your code compiles without errors
2. Use the warning analyzer (`./scripts/analyze_warnings.sh`) to identify and fix warnings
3. Follow the guidelines in [Compilation Best Practices](docs/compilation_best_practices.md)

For more information on our CI pipeline and compilation checks, see [CI Integration](docs/ci_integration.md).

### Continuous Improvement Process

IntelliRouter implements a continuous improvement process to ensure code quality continues to improve over time:

#### Code Quality Metrics

We track several code quality metrics:

- **Compiler Warnings**: Number of warnings reported by the Rust compiler
- **Warning Density**: Number of warnings per 1000 lines of code
- **Test Coverage**: Percentage of code covered by tests
- **Documentation Coverage**: Percentage of public items with documentation

#### Tools for Code Quality

The following tools are available to help improve code quality:

1. **Code Quality Report Generator**
   ```bash
   ./scripts/generate_code_quality_report.sh
   ```
   Generates a comprehensive report of code quality metrics.

2. **Metrics Charts Generator**
   ```bash
   ./scripts/generate_metrics_charts.sh
   ```
   Generates charts showing trends in code quality metrics over time.

3. **CI Code Quality Check**
   ```bash
   ./scripts/ci_code_quality.sh
   ```
   Runs code quality checks and compares against established goals.

#### Code Quality Goals

We have established specific goals for code quality metrics. See [Code Quality Goals](docs/code_quality_goals.md) for details.

#### Automated Code Review

Our automated code review bot analyzes pull requests and provides feedback on code quality issues. This helps maintain high standards and provides actionable feedback to contributors. See [Compilation Best Practices](docs/compilation_best_practices.md#automated-code-review) for more information.

#### Contributing to Code Quality

We welcome contributions specifically aimed at improving code quality. See [Contributing to Code Quality](CONTRIBUTING.md#code-quality-contributions) for more information.

#### Performance Optimization

We continuously monitor and optimize the performance of IntelliRouter. The performance benchmarking system helps identify areas for improvement and track progress over time. Contributions that improve performance are especially welcome.

When optimizing performance:

1. Use the benchmarking system to measure the impact of your changes
2. Focus on critical paths and bottlenecks
3. Consider both latency and throughput
4. Document performance improvements in your pull request

For more information on performance benchmarking, see [Performance Benchmarking](docs/performance_benchmarking.md).

## Recent Fixes

### Shutdown Coordinator Fix

The shutdown coordinator has been improved to handle graceful shutdown more reliably. The key changes include:

1. Added a new `wait_for_completion_shared` method to the `ShutdownCoordinator` that allows waiting for completion without requiring exclusive ownership of the coordinator.
2. Fixed the main application to use this new method instead of trying to unwrap the `Arc<ShutdownCoordinator>`.
3. Improved error handling during the shutdown process.

These changes ensure that all services can shut down gracefully, even when multiple references to the shutdown coordinator exist in different tasks.

### LLM Proxy Streaming Fix

The LLM Proxy module has been fixed to handle streaming responses correctly. The key changes include:

1. Added the missing `Infallible` import to the `routes.rs` file.
2. Modified the `chat_completions_stream` function to return a proper `Response` type that's compatible with Axum's `Handler` trait.
3. Fixed the type signature to ensure proper handling of streaming responses.

These changes ensure that the streaming chat completions endpoint works correctly and can be properly registered with the Axum router.

## License

This project is licensed under the MIT License - see the LICENSE file for details.