//! Assertion Framework Example
//!
//! This example demonstrates how to use the IntelliRouter assertion framework.

use intellirouter::modules::test_harness::{
    assert::{
        assert_context, assert_that, Assert, Assertions, ContainsMatcher, EqualsMatcher,
        JsonMatcher, Matcher, RegexMatcher, TypeMatcher,
    },
    types::TestHarnessError,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("IntelliRouter Assertion Framework Example");
    println!("========================================");

    // Create an assertions instance
    let assertions = Assertions::new().with_fail_fast(false);
    println!("Created assertions instance");

    // Basic assertions
    println!("\nBasic Assertions:");

    // Equality assertion
    match assertions.assert_equals(42, 42, Some("Numbers should be equal".to_string())) {
        Ok(_) => println!("✅ Equality assertion passed"),
        Err(e) => println!("❌ Equality assertion failed: {}", e),
    }

    // Inequality assertion
    match assertions.assert_not_equals(
        "hello",
        "world",
        Some("Strings should not be equal".to_string()),
    ) {
        Ok(_) => println!("✅ Inequality assertion passed"),
        Err(e) => println!("❌ Inequality assertion failed: {}", e),
    }

    // Boolean assertions
    match assertions.assert_true(true, Some("Value should be true".to_string())) {
        Ok(_) => println!("✅ True assertion passed"),
        Err(e) => println!("❌ True assertion failed: {}", e),
    }

    match assertions.assert_false(false, Some("Value should be false".to_string())) {
        Ok(_) => println!("✅ False assertion passed"),
        Err(e) => println!("❌ False assertion failed: {}", e),
    }

    // Contains assertion
    match assertions.assert_contains(
        "hello world",
        "world",
        Some("String should contain substring".to_string()),
    ) {
        Ok(_) => println!("✅ Contains assertion passed"),
        Err(e) => println!("❌ Contains assertion failed: {}", e),
    }

    // Pattern matching
    match assertions.assert_matches(
        "hello world",
        r"hello \w+",
        Some("String should match pattern".to_string()),
    ) {
        Ok(_) => println!("✅ Pattern matching assertion passed"),
        Err(e) => println!("❌ Pattern matching assertion failed: {}", e),
    }

    // Fluent assertions
    println!("\nFluent Assertions:");

    // Equality assertion
    let result = assert_that(42).is_equal_to(42);
    println!(
        "Equality assertion: {}",
        if result.passed() {
            "✅ Passed"
        } else {
            "❌ Failed"
        }
    );

    // String contains assertion
    let result = assert_that("hello world").contains("world");
    println!(
        "Contains assertion: {}",
        if result.passed() {
            "✅ Passed"
        } else {
            "❌ Failed"
        }
    );

    // Regex pattern matching
    let result = assert_that("hello world").matches_pattern(r"hello \w+");
    println!(
        "Pattern matching assertion: {}",
        if result.passed() {
            "✅ Passed"
        } else {
            "❌ Failed"
        }
    );

    // Type assertion
    let result = assert_that("hello").is_type("string");
    println!(
        "Type assertion: {}",
        if result.passed() {
            "✅ Passed"
        } else {
            "❌ Failed"
        }
    );

    // Matchers
    println!("\nMatchers:");

    // Equals matcher
    let equals_matcher = EqualsMatcher::new(42);
    let result = assert_that(42).matches(equals_matcher);
    println!(
        "Equals matcher: {}",
        if result.passed() {
            "✅ Passed"
        } else {
            "❌ Failed"
        }
    );

    // Contains matcher
    let contains_matcher = ContainsMatcher::new("world");
    let result = assert_that("hello world").matches(contains_matcher);
    println!(
        "Contains matcher: {}",
        if result.passed() {
            "✅ Passed"
        } else {
            "❌ Failed"
        }
    );

    // Regex matcher
    let regex_matcher = RegexMatcher::new(r"hello \w+").unwrap();
    let result = assert_that("hello world").matches(regex_matcher);
    println!(
        "Regex matcher: {}",
        if result.passed() {
            "✅ Passed"
        } else {
            "❌ Failed"
        }
    );

    // Type matcher
    let type_matcher = TypeMatcher::new("string");
    let result = assert_that("hello").matches(type_matcher);
    println!(
        "Type matcher: {}",
        if result.passed() {
            "✅ Passed"
        } else {
            "❌ Failed"
        }
    );

    // JSON matcher
    let expected_json = serde_json::json!({
        "name": "John",
        "age": 30,
        "address": {
            "city": "New York",
            "country": "USA"
        }
    });

    let json_matcher = JsonMatcher::new(expected_json.clone());
    let result = assert_that(expected_json.clone()).matches(json_matcher);
    println!(
        "JSON matcher: {}",
        if result.passed() {
            "✅ Passed"
        } else {
            "❌ Failed"
        }
    );

    // Assertion context
    println!("\nAssertion Context:");

    let mut context = assert_context("User validation");

    // Add assertions to the context
    context.assert(|| assert_that("John").is_type("string"));
    context.assert(|| assert_that(30).is_type("integer"));
    context.assert(|| assert_that(vec![1, 2, 3]).contains(2));
    context.assert(|| assert_that("john.doe@example.com").matches_pattern(r".+@.+\..+"));

    // Check the context results
    println!("Context: {}", context.name());
    println!("Total assertions: {}", context.assertion_count());
    println!("Passed assertions: {}", context.passed_count());
    println!("Failed assertions: {}", context.failed_count());
    println!("Warning assertions: {}", context.warning_count());
    println!("All passed: {}", context.all_passed());

    // Failing assertions
    println!("\nFailing Assertions:");

    // Equality assertion that fails
    let result = assert_that(42).is_equal_to(43);
    println!(
        "Failing equality assertion: {}",
        if result.passed() {
            "✅ Passed"
        } else {
            "❌ Failed"
        }
    );
    if let Some(error) = result.error() {
        println!("  Error: {}", error.message);
        println!("  Expected: {}", error.expected);
        println!("  Actual: {}", error.actual);
    }

    // Contains assertion that fails
    let result = assert_that("hello").contains("world");
    println!(
        "Failing contains assertion: {}",
        if result.passed() {
            "✅ Passed"
        } else {
            "❌ Failed"
        }
    );
    if let Some(error) = result.error() {
        println!("  Error: {}", error.message);
        println!("  Expected: {}", error.expected);
        println!("  Actual: {}", error.actual);
    }

    // Pattern matching assertion that fails
    let result = assert_that("hello").matches_pattern(r"world \w+");
    println!(
        "Failing pattern matching assertion: {}",
        if result.passed() {
            "✅ Passed"
        } else {
            "❌ Failed"
        }
    );
    if let Some(error) = result.error() {
        println!("  Error: {}", error.message);
        println!("  Expected: {}", error.expected);
        println!("  Actual: {}", error.actual);
    }

    // Warning assertions
    println!("\nWarning Assertions:");

    // Equality assertion with warning
    let result = assert_that(42).is_equal_to(43).with_fail_test(false);
    println!(
        "Warning equality assertion: {}",
        if result.is_warning() {
            "⚠️ Warning"
        } else if result.passed() {
            "✅ Passed"
        } else {
            "❌ Failed"
        }
    );
    if let Some(error) = result.error() {
        println!("  Warning: {}", error.message);
        println!("  Expected: {}", error.expected);
        println!("  Actual: {}", error.actual);
    }

    println!("\nAssertion framework example completed successfully!");
    Ok(())
}
