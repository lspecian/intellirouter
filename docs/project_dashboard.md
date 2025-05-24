# IntelliRouter Project Dashboard

The IntelliRouter Project Dashboard provides a unified interface for monitoring the health and quality of the IntelliRouter project. It integrates metrics from various systems including code quality, performance benchmarking, security audit, and documentation generation.

## Features

- **Unified Metrics View**: Combines metrics from multiple systems into a single dashboard
- **Project Health Monitoring**: Calculates overall project health based on various metrics
- **Real-time Updates**: Automatically refreshes metrics at configurable intervals
- **Interactive Charts**: Visualizes trends and patterns in project metrics
- **Recommendations**: Provides actionable recommendations for improving project health

## Components

The dashboard integrates the following components:

### 1. Code Quality Metrics

- **Warning Count**: Total number of compiler warnings
- **Warning Density**: Warnings per 1000 lines of code
- **Test Coverage**: Percentage of code covered by tests
- **Documentation Coverage**: Percentage of code with documentation
- **Lines of Code**: Total lines of code in the project

### 2. Performance Metrics

- **Benchmark Results**: Performance benchmarks for various components
- **Performance Trends**: Changes in performance over time
- **Regressions**: Detected performance regressions

### 3. Security Metrics

- **Security Issues**: Total number of security issues
- **Issue Severity**: Breakdown of issues by severity (Critical, High, Medium, Low)
- **Vulnerabilities**: Detailed information about detected vulnerabilities

### 4. Documentation Metrics

- **Overall Coverage**: Percentage of project with documentation
- **API Documentation**: Coverage of API documentation
- **User Guides**: Coverage of user guides
- **Architecture Documentation**: Coverage of architecture documentation
- **Examples & Tutorials**: Coverage of examples and tutorials

### 5. Project Health

- **Overall Health Score**: Combined score based on all metrics
- **Component Scores**: Individual scores for code quality, performance, security, and documentation
- **Health Trends**: Changes in project health over time

## Architecture

The dashboard is built using the following technologies:

- **Backend**: Rust with Rocket web framework
- **Frontend**: HTML, CSS, JavaScript with Bootstrap
- **Data Storage**: JSON files for metrics data
- **Visualization**: Chart.js for interactive charts

The dashboard follows a modular architecture:

1. **Data Collection**: Scripts collect metrics from various sources
2. **Data Processing**: Metrics are processed and combined into a unified format
3. **Data Storage**: Processed metrics are stored in JSON files
4. **Web Server**: Rocket serves the dashboard web interface
5. **Web Interface**: HTML templates render the dashboard UI

## Setup and Usage

### Prerequisites

- Rust 1.65 or later
- Cargo
- jq (for metrics processing)

### Installation

1. Clone the IntelliRouter repository
2. Navigate to the dashboard directory: `cd dashboard`
3. Build the dashboard: `cargo build --release`

### Running the Dashboard

1. Run the dashboard server: `./run_dashboard.sh`
2. Open a web browser and navigate to: `http://localhost:8080`

### Collecting Metrics

Metrics are collected automatically when the dashboard server starts. You can also collect metrics manually:

```bash
./collect_metrics.sh
```

### Configuration

The dashboard can be configured by editing the `Cargo.toml` file. The following options are available:

- **Host**: The host to bind to (default: 127.0.0.1)
- **Port**: The port to bind to (default: 8080)
- **Refresh Interval**: The interval in seconds to refresh metrics (default: 60)
- **Theme**: The dashboard theme (default: default)

## Integration with CI/CD

The dashboard can be integrated with CI/CD pipelines to automatically collect and display metrics. The following steps are required:

1. Add the `collect_metrics.sh` script to your CI/CD pipeline
2. Configure the CI/CD pipeline to store metrics in a persistent location
3. Deploy the dashboard server to a publicly accessible location
4. Configure the dashboard server to read metrics from the persistent location

## Contributing

Contributions to the dashboard are welcome! Please follow these steps:

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Run tests: `cargo test`
5. Submit a pull request

## License

The IntelliRouter Project Dashboard is licensed under the same license as the IntelliRouter project.