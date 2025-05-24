# IntelliRouter Documentation System

This document provides an overview of the IntelliRouter documentation generation system, which automatically generates comprehensive documentation from the codebase, tracks documentation coverage over time, and provides guidance on improving documentation.

## Overview

The IntelliRouter documentation system consists of several components:

1. **Documentation Generation Scripts**: A set of scripts that generate different types of documentation from the codebase.
2. **Documentation Coverage Tracking**: A system that tracks documentation coverage over time and identifies areas that need improvement.
3. **Documentation Reports**: Reports that provide insights into documentation coverage and trends.
4. **CI Integration**: Integration with the CI pipeline to automatically generate documentation and check for regressions.

## Documentation Types

The documentation system generates several types of documentation:

### API Documentation

API documentation is generated from the source code using tools like `rustdoc` for Rust code, Sphinx for Python code, and TypeDoc for TypeScript code. This documentation provides detailed information about the API, including:

- Classes, structs, and enums
- Functions and methods
- Traits and interfaces
- Constants and variables
- Modules and namespaces

### User Guides

User guides provide step-by-step instructions for using IntelliRouter. They are written in Markdown and converted to HTML. User guides cover topics such as:

- Installation and setup
- Configuration
- Basic usage
- Advanced features
- Troubleshooting

### Architecture Documentation

Architecture documentation provides an overview of the IntelliRouter architecture, including:

- System architecture
- Component architecture
- Data flow
- Deployment architecture
- Security architecture
- Scalability

Architecture documentation includes diagrams generated using PlantUML.

### Examples and Tutorials

Examples and tutorials provide practical examples of using IntelliRouter. They are extracted from example code files and converted to HTML. Examples and tutorials cover topics such as:

- Basic usage examples
- Advanced usage examples
- Integration examples
- Step-by-step tutorials

## Documentation Generation Process

The documentation generation process consists of the following steps:

1. **Preparation**: The documentation generation scripts are executed, which prepare the necessary directories and files.
2. **Generation**: Each documentation generator script is executed, which generates the corresponding documentation.
3. **Coverage Check**: The documentation coverage is checked, and metrics are collected.
4. **Report Generation**: A report is generated based on the documentation metrics.

### Documentation Generation Scripts

The documentation generation scripts are located in the `scripts/docs/` directory:

- `generate_docs.sh`: The main documentation generation script that coordinates the generation of different types of documentation and collects metrics.
- `generate_api_docs.sh`: Generates API documentation using rustdoc, Sphinx, and TypeDoc.
- `generate_user_guides.sh`: Generates user guides documentation from Markdown files.
- `generate_architecture_docs.sh`: Generates architecture documentation using diagrams and Markdown.
- `generate_examples_docs.sh`: Generates examples and tutorials documentation from example code files.
- `check_doc_coverage.sh`: Checks documentation coverage and collects metrics.
- `generate_doc_report.sh`: Generates a report based on the documentation metrics.

### Documentation Coverage Metrics

The documentation system tracks several coverage metrics:

- **API Documentation Coverage**: The percentage of public API items that are documented.
- **User Guides Coverage**: The percentage of modules that have corresponding user guides.
- **Architecture Documentation Coverage**: The percentage of expected architecture documents that exist.
- **Examples and Tutorials Coverage**: The percentage of examples that have corresponding documentation.

These metrics are stored in JSON files in the `metrics/docs/` directory.

### Documentation Reports

The documentation system generates reports based on the documentation metrics. These reports provide insights into documentation coverage and trends, and include:

- Overall documentation coverage
- Documentation coverage by type
- Code documentation coverage by language
- Documentation coverage trends
- Recommendations for improving documentation

## CI Integration

The documentation system is integrated with the CI pipeline to automatically generate documentation and check for regressions. The CI workflow is defined in `.github/workflows/documentation.yml` and consists of the following jobs:

1. **Generate Documentation**: Generates documentation and collects metrics.
2. **Check Documentation Regression**: Checks for documentation coverage regressions.
3. **Deploy Documentation**: Deploys the documentation to GitHub Pages.

The CI workflow runs on push to the main branch, on pull requests to the main branch, and on a weekly schedule.

## Improving Documentation

To improve documentation coverage, follow these guidelines:

### API Documentation

- Add documentation comments to all public items in the codebase.
- Use `///` for documentation comments in Rust code.
- Use docstrings for documentation in Python code.
- Use JSDoc comments for documentation in TypeScript code.

### User Guides

- Create user guides for all modules and features.
- Organize user guides by topic.
- Include step-by-step instructions and examples.

### Architecture Documentation

- Create architecture documents for all components and modules.
- Include diagrams to illustrate the architecture.
- Explain the design decisions and trade-offs.

### Examples and Tutorials

- Create examples for all key features and use cases.
- Include comments in example code to explain the code.
- Create step-by-step tutorials for common tasks.

## Running the Documentation System

To run the documentation system manually, execute the following commands:

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