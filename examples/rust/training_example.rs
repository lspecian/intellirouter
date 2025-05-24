//! Training Materials Generator Example
//!
//! This example demonstrates how to use the IntelliRouter training materials generator.

use intellirouter::modules::test_harness::{
    docs::{DocumentationFormat, DocumentationSection},
    training::{
        TrainingConfig, TrainingCourse, TrainingDifficulty, TrainingGenerator, TrainingMaterial,
        TrainingMaterialType,
    },
    types::TestHarnessError,
};
use std::collections::HashMap;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("IntelliRouter Training Materials Generator Example");
    println!("=================================================");

    // Create a training configuration
    let training_config = TrainingConfig {
        title: "IntelliRouter Test Harness Training".to_string(),
        description: Some("Training materials for the IntelliRouter test harness".to_string()),
        version: "1.0.0".to_string(),
        author: Some("IntelliRouter Team".to_string()),
        output_dir: PathBuf::from("training"),
        formats: vec![DocumentationFormat::Markdown, DocumentationFormat::Html],
        template_dir: None,
        assets_dir: None,
        metadata: HashMap::new(),
    };
    println!("Created training configuration");

    // Create a training generator
    let training_generator = TrainingGenerator::new(training_config);
    println!("Created training generator");

    // Create training materials
    println!("\nCreating training materials...");

    // Create getting started tutorial
    let getting_started = TrainingMaterial::new(
        "getting-started",
        "Getting Started with the Test Harness",
        "Learn how to use the test harness for basic testing",
        TrainingMaterialType::Tutorial,
        TrainingDifficulty::Beginner,
    )
    .with_duration(30)
    .with_prerequisite("Basic Rust knowledge")
    .with_prerequisite("IntelliRouter installed")
    .with_content(r#"
## Introduction

Welcome to the IntelliRouter test harness! This tutorial will guide you through the basics of using the test harness to write and run tests for IntelliRouter components.

## Installation

To install the test harness, add the following to your `Cargo.toml`:

```toml
[dependencies]
intellirouter-test-harness = "1.0.0"
```

## Basic Usage

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
"#)
    .with_section(DocumentationSection::new(
        "installation",
        "Installation",
        r#"
### Prerequisites

Before installing the test harness, ensure you have:

- Rust 1.65 or later
- Cargo
- IntelliRouter installed

### Installation Steps

1. Add the test harness to your `Cargo.toml`:

```toml
[dependencies]
intellirouter-test-harness = "1.0.0"
```

2. Run `cargo build` to download and compile the test harness.

3. Import the test harness in your code:

```rust
use intellirouter::modules::test_harness;
```
"#,
    ))
    .with_section(DocumentationSection::new(
        "configuration",
        "Configuration",
        r#"
### Test Harness Configuration

The test harness can be configured using a TOML file. Create a file named `test_harness.toml` in your project root:

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

### Configuration Options

- `test.parallel`: Run tests in parallel (default: `false`)
- `test.timeout`: Test timeout in seconds (default: `60`)
- `environment.type`: Environment type (default: `"local"`)
- `environment.image`: Docker image for containerized tests
- `reporting.format`: Report formats (default: `["html"]`)
- `reporting.output_dir`: Report output directory (default: `"reports"`)
"#,
    ));

    println!("Created getting started tutorial");

    // Create assertion workshop
    let assertions_workshop = TrainingMaterial::new(
        "assertions-workshop",
        "Assertion Workshop",
        "Learn how to use the assertion library for effective testing",
        TrainingMaterialType::Workshop,
        TrainingDifficulty::Intermediate,
    )
    .with_duration(60)
    .with_prerequisite("Getting Started with the Test Harness")
    .with_prerequisite("Intermediate Rust knowledge")
    .with_content(r#"
## Introduction

The assertion library is a powerful tool for writing expressive and readable tests. This workshop will guide you through the various assertion types and how to use them effectively.

## Basic Assertions

Let's start with some basic assertions:

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
```

## Collection Assertions

Now let's look at collection assertions:

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

## Workshop Exercises

1. Write assertions to verify a user object has the correct properties.
2. Write assertions to verify a list of users is sorted by age.
3. Write assertions to verify a map contains specific entries.
"#)
    .with_section(DocumentationSection::new(
        "custom-assertions",
        "Custom Assertions",
        r#"
### Creating Custom Assertions

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

### Exercise: Custom Assertions

Create a custom assertion that verifies a string is a valid email address.
"#,
    ))
    .with_section(DocumentationSection::new(
        "assertion-contexts",
        "Assertion Contexts",
        r#"
### Using Assertion Contexts

Assertion contexts allow you to group related assertions:

```rust
use intellirouter::modules::test_harness::assert::{assert_that, context};

let user = User { name: "Alice", age: 30 };

context("User validation", |ctx| {
    ctx.assert(assert_that(user.name).is_equal_to("Alice"));
    ctx.assert(assert_that(user.age).is_greater_than(18));
});
```

### Exercise: Assertion Contexts

Create an assertion context to validate a complex object with multiple properties.
"#,
    ));

    println!("Created assertion workshop");

    // Create benchmarking exercise
    let benchmarking_exercise = TrainingMaterial::new(
        "benchmarking-exercise",
        "Benchmarking Exercise",
        "Practice using the benchmarking framework",
        TrainingMaterialType::Exercise,
        TrainingDifficulty::Advanced,
    )
    .with_duration(45)
    .with_prerequisite("Getting Started with the Test Harness")
    .with_prerequisite("Assertion Workshop")
    .with_content(r#"
## Introduction

The benchmarking framework allows you to measure the performance of your code. This exercise will guide you through creating and running benchmarks.

## Basic Benchmarking

Here's a simple benchmark:

```rust
use intellirouter::modules::test_harness::benchmark::{
    BenchmarkConfig, BenchmarkRunner, BenchmarkType,
};
use std::time::Duration;

#[tokio::main]
async fn main() {
    // Create a benchmark configuration
    let config = BenchmarkConfig::new(
        "string-concat",
        "String Concatenation Benchmark",
        BenchmarkType::Throughput,
    )
    .with_duration(Duration::from_secs(5))
    .with_warmup_duration(Duration::from_secs(1));
    
    // Create a benchmark function
    let benchmark_fn = || {
        let mut s = String::new();
        for i in 0..100 {
            s.push_str(&i.to_string());
        }
        Ok(Duration::from_nanos(s.len() as u64))
    };
    
    // Create a benchmark runner
    let runner = BenchmarkRunner::new(config, benchmark_fn);
    
    // Run the benchmark
    let result = runner.run().await.unwrap();
    
    // Print the results
    println!("Throughput: {:.2} ops/sec", result.throughput);
    println!("Latency (avg): {:.2} ms", result.latency.avg_duration.as_secs_f64() * 1000.0);
}
```

## Exercise Tasks

1. Create a benchmark to measure the performance of sorting a vector of integers.
2. Create a benchmark to measure the performance of a hash map lookup.
3. Create a benchmark to measure the performance of a regex match.
4. Compare the performance of different algorithms for the same task.
"#);

    println!("Created benchmarking exercise");

    // Create testing cheat sheet
    let testing_cheatsheet = TrainingMaterial::new(
        "testing-cheatsheet",
        "Testing Cheat Sheet",
        "Quick reference for the test harness",
        TrainingMaterialType::CheatSheet,
        TrainingDifficulty::Beginner,
    )
    .with_duration(15)
    .with_content(
        r#"
## Test Harness Cheat Sheet

### Basic Test Structure

```rust
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

### Common Assertions

```rust
// Value assertions
assert_that(value).is_equal_to(expected);
assert_that(value).is_not_equal_to(unexpected);
assert_that(value).is_greater_than(min);
assert_that(value).is_less_than(max);
assert_that(value).is_between(min, max);

// Boolean assertions
assert_that(condition).is_true();
assert_that(condition).is_false();

// String assertions
assert_that(string).contains(substring);
assert_that(string).starts_with(prefix);
assert_that(string).ends_with(suffix);
assert_that(string).matches(regex);

// Option assertions
assert_that(option).is_some();
assert_that(option).is_none();
assert_that(option).contains(value);

// Result assertions
assert_that(result).is_ok();
assert_that(result).is_err();
assert_that(result).contains(value);

// Collection assertions
assert_that(collection).has_length(length);
assert_that(collection).contains(element);
assert_that(collection).contains_all_of(&elements);
assert_that(collection).contains_any_of(&elements);
assert_that(collection).contains_none_of(&elements);
assert_that(collection).is_sorted();

// Map assertions
assert_that(map).has_size(size);
assert_that(map).contains_key(key);
assert_that(map).contains_entry(key, value);
```

### Running Tests

```bash
# Run all tests
cargo test

# Run a specific test
cargo test test_example

# Run tests with a specific tag
cargo test -- --include-tag integration

# Run tests with a specific category
cargo test -- --include-category unit
```

### Benchmarking

```rust
// Create a benchmark configuration
let config = BenchmarkConfig::new(
    "benchmark-id",
    "Benchmark Name",
    BenchmarkType::Throughput,
)
.with_duration(Duration::from_secs(5));

// Create a benchmark function
let benchmark_fn = || {
    // Code to benchmark
    Ok(Duration::from_nanos(42))
};

// Create a benchmark runner
let runner = BenchmarkRunner::new(config, benchmark_fn);

// Run the benchmark
let result = runner.run().await.unwrap();
```

### Security Testing

```rust
// Create a security test configuration
let config = SecurityTestConfig::new(
    "security-test-id",
    "Security Test Name",
    SecurityTestType::DependencyScanning,
    "target",
)
.with_severity_threshold(VulnerabilitySeverity::High);

// Create a security test function
let security_test_fn = || {
    // Security test code
    Ok(vec![])
};

// Create a security test runner
let runner = SecurityTestRunner::new(config, security_test_fn);

// Run the security test
let result = runner.run().await.unwrap();
```
"#,
    );

    println!("Created testing cheat sheet");

    // Create courses
    println!("\nCreating training courses...");

    // Create beginner course
    let beginner_course = TrainingCourse::new(
        "beginner",
        "IntelliRouter Test Harness for Beginners",
        "A beginner's guide to using the IntelliRouter test harness",
    )
    .with_material(getting_started.clone())
    .with_material(testing_cheatsheet.clone());

    println!("Created beginner course");

    // Create advanced course
    let advanced_course = TrainingCourse::new(
        "advanced",
        "Advanced IntelliRouter Testing",
        "Advanced techniques for testing IntelliRouter components",
    )
    .with_material(assertions_workshop.clone())
    .with_material(benchmarking_exercise.clone());

    println!("Created advanced course");

    // Generate training materials
    println!("\nGenerating training materials...");
    training_generator
        .generate(vec![beginner_course, advanced_course])
        .await?;

    println!("Training materials generated successfully in the 'training' directory");
    println!("  - Beginner course: training/beginner/index.md");
    println!("  - Advanced course: training/advanced/index.md");
    println!("  - Getting started tutorial: training/beginner/getting-started.md");
    println!("  - Testing cheat sheet: training/beginner/testing-cheatsheet.md");
    println!("  - Assertion workshop: training/advanced/assertions-workshop.md");
    println!("  - Benchmarking exercise: training/advanced/benchmarking-exercise.md");

    println!("\nTraining materials generator example completed successfully!");
    Ok(())
}
