//! Workshop Example
//!
//! This example demonstrates how to use the IntelliRouter workshop module.

use intellirouter::modules::test_harness::{
    docs::{DocumentationFormat, DocumentationSection},
    training::{TrainingDifficulty, TrainingMaterial, TrainingMaterialType},
    types::TestHarnessError,
    workshop::{
        Workshop, WorkshopActivity, WorkshopActivityType, WorkshopConfig, WorkshopGenerator,
        WorkshopSession,
    },
};
use std::collections::HashMap;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("IntelliRouter Workshop Example");
    println!("=============================");

    // Create a workshop configuration
    let workshop_config = WorkshopConfig {
        title: "IntelliRouter Test Harness Workshop".to_string(),
        description: Some("Workshop materials for the IntelliRouter test harness".to_string()),
        version: "1.0.0".to_string(),
        author: Some("IntelliRouter Team".to_string()),
        output_dir: PathBuf::from("workshops"),
        formats: vec![DocumentationFormat::Markdown, DocumentationFormat::Html],
        template_dir: None,
        assets_dir: None,
        metadata: HashMap::new(),
    };
    println!("Created workshop configuration");

    // Create a workshop generator
    let workshop_generator = WorkshopGenerator::new(workshop_config);
    println!("Created workshop generator");

    // Create workshop activities
    println!("\nCreating workshop activities...");

    // Create introduction activity
    let introduction = WorkshopActivity::new(
        "introduction",
        "Introduction to the Test Harness",
        "An introduction to the IntelliRouter test harness and its components",
        WorkshopActivityType::Presentation,
        30,
    )
    .with_content(r#"
## Welcome to the IntelliRouter Test Harness Workshop!

In this workshop, we'll explore the IntelliRouter test harness and learn how to use it effectively for testing IntelliRouter components.

## Agenda

1. Introduction to the test harness
2. Setting up the test environment
3. Writing and running tests
4. Using the assertion library
5. Benchmarking and performance testing
6. Security testing
7. CI/CD integration
8. Documentation and reporting

## Workshop Goals

By the end of this workshop, you'll be able to:

- Set up and configure the test harness
- Write effective tests for IntelliRouter components
- Use the assertion library for expressive tests
- Run benchmarks and performance tests
- Conduct security scans
- Integrate tests with CI/CD pipelines
- Generate documentation and reports
"#)
    .with_section(DocumentationSection::new(
        "test-harness-overview",
        "Test Harness Overview",
        r#"
The IntelliRouter test harness provides a comprehensive framework for testing all aspects of IntelliRouter, including:

- Unit tests
- Integration tests
- End-to-end tests
- Performance benchmarks
- Security scans

It's designed to be modular, extensible, and easy to use, with a consistent API across all test types.
"#,
    ));

    println!("Created introduction activity");

    // Create hands-on activity
    let hands_on = WorkshopActivity::new(
        "hands-on",
        "Writing Your First Test",
        "A hands-on exercise to write and run a test using the test harness",
        WorkshopActivityType::HandsOn,
        45,
    )
    .with_content(
        r#"
## Writing Your First Test

In this exercise, you'll write a simple test for the IntelliRouter router component.

### Step 1: Create a Test File

Create a new file called `router_test.rs` in the `tests` directory:

```rust
use intellirouter::modules::test_harness::{
    assert::assert_that,
    reporting::{TestResult, TestStatus},
};
use intellirouter::modules::router_core::Router;

#[test]
fn test_router_initialization() {
    // Arrange
    let config = RouterConfig::default();
    
    // Act
    let router = Router::new(config);
    
    // Assert
    assert_that(router).is_not_null();
    assert_that(router.is_initialized()).is_true();
}
```

### Step 2: Run the Test

Run the test using the test harness:

```bash
cargo test test_router_initialization
```

### Step 3: Add More Assertions

Extend the test with more assertions:

```rust
assert_that(router.config().timeout).is_equal_to(Duration::from_secs(30));
assert_that(router.routes()).has_length(0);
```

### Step 4: Run the Test Again

Run the test again to verify your changes:

```bash
cargo test test_router_initialization
```
"#,
    )
    .with_material(
        TrainingMaterial::new(
            "test-writing-guide",
            "Test Writing Guide",
            "A guide to writing effective tests",
            TrainingMaterialType::CheatSheet,
            TrainingDifficulty::Beginner,
        )
        .with_content(
            r#"
## Test Writing Cheat Sheet

### Test Structure

```rust
#[test]
fn test_name() {
    // Arrange
    // Set up the test environment and create test objects
    
    // Act
    // Perform the action being tested
    
    // Assert
    // Verify the results
}
```

### Common Assertions

```rust
// Value assertions
assert_that(value).is_equal_to(expected);
assert_that(value).is_not_equal_to(unexpected);
assert_that(value).is_greater_than(min);
assert_that(value).is_less_than(max);

// Boolean assertions
assert_that(condition).is_true();
assert_that(condition).is_false();

// String assertions
assert_that(string).contains(substring);
assert_that(string).starts_with(prefix);
assert_that(string).ends_with(suffix);

// Collection assertions
assert_that(collection).has_length(length);
assert_that(collection).contains(element);
assert_that(collection).contains_all_of(&elements);
```
"#,
        ),
    );

    println!("Created hands-on activity");

    // Create discussion activity
    let discussion = WorkshopActivity::new(
        "discussion",
        "Test Strategy Discussion",
        "A group discussion about test strategies for IntelliRouter",
        WorkshopActivityType::Discussion,
        30,
    )
    .with_content(
        r#"
## Test Strategy Discussion

In this discussion, we'll explore different test strategies for IntelliRouter components.

### Discussion Topics

1. What are the key components of IntelliRouter that need testing?
2. What are the most critical test cases for each component?
3. How should we balance unit tests, integration tests, and end-to-end tests?
4. What performance metrics should we measure?
5. What security aspects should we focus on?

### Discussion Format

1. Break into small groups of 3-4 people
2. Discuss the topics for 15 minutes
3. Each group presents their key findings (2-3 minutes per group)
4. Full group discussion and summary (5-10 minutes)
"#,
    );

    println!("Created discussion activity");

    // Create pair programming activity
    let pair_programming = WorkshopActivity::new(
        "pair-programming",
        "Test Harness Integration",
        "A pair programming exercise to integrate the test harness with a new component",
        WorkshopActivityType::PairProgramming,
        60,
    )
    .with_content(r#"
## Test Harness Integration

In this pair programming exercise, you'll work with a partner to integrate the test harness with a new IntelliRouter component.

### Exercise Goals

- Set up the test harness for a new component
- Write a comprehensive test suite
- Run the tests and analyze the results
- Generate a test report

### Pair Programming Format

1. Find a partner
2. Decide who will be the driver and who will be the navigator
3. Driver writes code, navigator reviews and provides guidance
4. Switch roles every 15 minutes
5. Complete the exercise together

### Exercise Steps

1. Create a new test file for the component
2. Set up the test environment
3. Write test cases for the component's API
4. Run the tests and fix any issues
5. Generate a test report
"#);

    println!("Created pair programming activity");

    // Create workshop sessions
    println!("\nCreating workshop sessions...");

    // Create morning session
    let morning_session = WorkshopSession::new(
        "morning",
        "Morning Session: Test Harness Basics",
        "Introduction to the test harness and basic testing",
        TrainingDifficulty::Beginner,
    )
    .with_activity(introduction.clone())
    .with_activity(hands_on.clone());

    println!("Created morning session");

    // Create afternoon session
    let afternoon_session = WorkshopSession::new(
        "afternoon",
        "Afternoon Session: Advanced Testing",
        "Advanced testing techniques and strategies",
        TrainingDifficulty::Intermediate,
    )
    .with_activity(discussion.clone())
    .with_activity(pair_programming.clone())
    .with_prerequisite("Morning Session: Test Harness Basics");

    println!("Created afternoon session");

    // Create workshop
    println!("\nCreating workshop...");

    let workshop = Workshop::new(
        "test-harness-workshop",
        "IntelliRouter Test Harness Workshop",
        "A comprehensive workshop on using the IntelliRouter test harness",
    )
    .with_session(morning_session)
    .with_session(afternoon_session)
    .with_metadata("location", "Conference Room A")
    .with_metadata("date", "2025-05-20");

    println!("Created workshop");
    println!("  Title: {}", workshop.title);
    println!("  Description: {}", workshop.description);
    println!("  Total Duration: {} minutes", workshop.total_duration());
    println!("  Sessions: {}", workshop.sessions.len());

    // Generate workshop materials
    println!("\nGenerating workshop materials...");
    workshop_generator.generate(vec![workshop]).await?;

    println!("Workshop materials generated successfully in the 'workshops' directory");
    println!("  - Workshop index: workshops/test-harness-workshop/index.md");
    println!("  - Morning session: workshops/test-harness-workshop/morning/index.md");
    println!("  - Afternoon session: workshops/test-harness-workshop/afternoon/index.md");

    println!("\nWorkshop example completed successfully!");
    Ok(())
}
