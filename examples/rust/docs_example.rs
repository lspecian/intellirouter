//! Documentation Generator Example
//!
//! This example demonstrates how to use the IntelliRouter documentation generator.

use intellirouter::modules::test_harness::{
    docs::{
        DocumentationConfig, DocumentationFormat, DocumentationGenerator, DocumentationSection,
    },
    types::TestHarnessError,
};
use std::collections::HashMap;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("IntelliRouter Documentation Generator Example");
    println!("============================================");

    // Create a documentation configuration
    let doc_config = DocumentationConfig {
        title: "IntelliRouter Test Harness Documentation".to_string(),
        description: Some(
            "Comprehensive documentation for the IntelliRouter test harness".to_string(),
        ),
        version: "1.0.0".to_string(),
        author: Some("IntelliRouter Team".to_string()),
        output_dir: PathBuf::from("docs"),
        formats: vec![DocumentationFormat::Markdown, DocumentationFormat::Html],
        template_dir: None,
        assets_dir: None,
        metadata: HashMap::new(),
    };
    println!("Created documentation configuration");

    // Create a documentation generator
    let doc_generator = DocumentationGenerator::new(doc_config);
    println!("Created documentation generator");

    // Create custom documentation sections
    println!("\nCreating custom documentation sections...");

    // Create getting started section
    let getting_started = DocumentationSection::new(
        "getting-started",
        "Getting Started",
        r#"
## Introduction

The IntelliRouter test harness provides a comprehensive framework for testing, benchmarking, and security scanning of IntelliRouter components.

## Installation

To install the test harness, add the following to your `Cargo.toml`:

```toml
[dependencies]
intellirouter-test-harness = "1.0.0"
```

## Quick Start

Here's a simple example of how to use the test harness:

```rust
use intellirouter::modules::test_harness::{
    assert::assert_that,
    reporting::{TestResult, TestStatus},
};

#[test]
fn test_example() {
    // Arrange
    let value = 42;
    
    // Act
    let result = value * 2;
    
    // Assert
    assert_that(result).is_equal_to(84);
}
```
"#,
    )
    .with_subsection(DocumentationSection::new(
        "getting-started-prerequisites",
        "Prerequisites",
        r#"
Before using the test harness, ensure you have the following prerequisites:

- Rust 1.65 or later
- Cargo
- Docker (for containerized tests)
- OpenSSL development libraries
"#,
    ))
    .with_subsection(DocumentationSection::new(
        "getting-started-configuration",
        "Configuration",
        r#"
The test harness can be configured using a TOML file. Here's an example configuration:

```toml
[test]
parallel = true
timeout = 30

[environment]
type = "docker"
image = "intellirouter/test-env:latest"

[reporting]
format = ["html", "json"]
output_dir = "reports"
```
"#,
    ));

    println!("Created getting started section");

    // Create test execution section
    let test_execution = DocumentationSection::new(
        "test-execution",
        "Test Execution",
        r#"
The test execution engine is responsible for discovering, filtering, and executing tests. It provides a plugin-based architecture for extending the test harness with new test types.

## Running Tests

To run tests, use the `TestRunner` class:

```rust
use intellirouter::modules::test_harness::{
    TestRunner, TestConfig,
};

#[tokio::main]
async fn main() {
    let config = TestConfig::default();
    let runner = TestRunner::new(config);
    
    let results = runner.run().await.unwrap();
    println!("Tests: {}, Passed: {}, Failed: {}", 
        results.total, results.passed, results.failed);
}
```

## Filtering Tests

You can filter tests by name, tag, or category:

```rust
let config = TestConfig::default()
    .with_filter("api")  // Run tests with "api" in the name
    .with_tag_filter("integration")  // Run tests with the "integration" tag
    .with_category_filter("unit");  // Run tests in the "unit" category
```
"#,
    )
    .with_subsection(DocumentationSection::new(
        "test-execution-parallel",
        "Parallel Execution",
        r#"
The test harness supports parallel test execution:

```rust
let config = TestConfig::default()
    .with_parallel(true)  // Enable parallel execution
    .with_max_threads(4);  // Use up to 4 threads
```
"#,
    ))
    .with_subsection(DocumentationSection::new(
        "test-execution-timeouts",
        "Timeouts",
        r#"
You can configure timeouts for tests:

```rust
let config = TestConfig::default()
    .with_timeout(Duration::from_secs(30));  // 30 second timeout
```
"#,
    ));

    println!("Created test execution section");

    // Create assertions section
    let assertions = DocumentationSection::new(
        "assertions",
        "Assertions",
        r#"
The test harness provides a fluent assertion API for making assertions in tests.

## Basic Assertions

```rust
use intellirouter::modules::test_harness::assert::assert_that;

// Value assertions
assert_that(42).is_equal_to(42);
assert_that(42).is_not_equal_to(43);
assert_that(42).is_greater_than(41);
assert_that(42).is_less_than(43);

// Boolean assertions
assert_that(true).is_true();
assert_that(false).is_false();

// String assertions
assert_that("hello").contains("ell");
assert_that("hello").starts_with("he");
assert_that("hello").ends_with("lo");
assert_that("hello").matches(regex::Regex::new("h.*o").unwrap());

// Option assertions
assert_that(Some(42)).is_some();
assert_that(None::<i32>).is_none();
assert_that(Some(42)).contains(42);

// Result assertions
assert_that(Ok(42) as Result<_, ()>).is_ok();
assert_that(Err(()) as Result<i32, _>).is_err();
assert_that(Ok(42) as Result<_, ()>).contains(42);
```

## Collection Assertions

```rust
// Vector assertions
assert_that(vec![1, 2, 3]).has_length(3);
assert_that(vec![1, 2, 3]).contains(2);
assert_that(vec![1, 2, 3]).contains_all_of(&[1, 3]);
assert_that(vec![1, 2, 3]).contains_any_of(&[0, 1, 4]);
assert_that(vec![1, 2, 3]).contains_none_of(&[0, 4]);
assert_that(vec![1, 2, 3]).is_sorted();

// Map assertions
let map = std::collections::HashMap::from([("a", 1), ("b", 2)]);
assert_that(&map).has_size(2);
assert_that(&map).contains_key("a");
assert_that(&map).contains_entry("a", 1);
```
"#,
    )
    .with_subsection(DocumentationSection::new(
        "assertions-custom",
        "Custom Assertions",
        r#"
You can create custom assertions by implementing the `Matcher` trait:

```rust
use intellirouter::modules::test_harness::assert::{Matcher, MatchResult};

struct IsEven;

impl Matcher<i32> for IsEven {
    fn matches(&self, actual: &i32) -> MatchResult {
        if actual % 2 == 0 {
            MatchResult::Match
        } else {
            MatchResult::Mismatch(format!("Expected {} to be even", actual))
        }
    }
}

// Use the custom matcher
assert_that(42).matches(IsEven);
```
"#,
    ));

    println!("Created assertions section");

    // Generate documentation
    println!("\nGenerating documentation...");
    let documentation = doc_generator.generate().await?;

    // Add custom sections
    let mut custom_doc = documentation;
    custom_doc.add_section(getting_started);
    custom_doc.add_section(test_execution);
    custom_doc.add_section(assertions);

    // Generate files for custom documentation
    let custom_config = DocumentationConfig {
        title: "IntelliRouter Test Harness Documentation".to_string(),
        description: Some(
            "Comprehensive documentation for the IntelliRouter test harness".to_string(),
        ),
        version: "1.0.0".to_string(),
        author: Some("IntelliRouter Team".to_string()),
        output_dir: PathBuf::from("docs-custom"),
        formats: vec![DocumentationFormat::Markdown, DocumentationFormat::Html],
        template_dir: None,
        assets_dir: None,
        metadata: HashMap::new(),
    };

    let custom_generator = DocumentationGenerator::new(custom_config);
    custom_generator.generate_files(&custom_doc).await?;

    println!("Documentation generated successfully in the 'docs' and 'docs-custom' directories");
    println!("  - Markdown: docs/index.md and docs-custom/index.md");
    println!("  - HTML: docs/index.html and docs-custom/index.html");

    println!("\nDocumentation generator example completed successfully!");
    Ok(())
}
