//! Core assertion types and functions for the assertion framework.

use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::modules::test_harness::types::TestHarnessError;

use super::matchers::Matcher;

/// Represents the outcome of an assertion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssertionOutcome {
    /// The assertion passed.
    Passed,
    /// The assertion failed.
    Failed,
    /// The assertion failed but is treated as a warning.
    Warning,
    /// The assertion was skipped.
    Skipped,
}

/// Represents an error that occurred during an assertion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionError {
    /// The error message.
    pub message: String,
    /// The expected value.
    pub expected: String,
    /// The actual value.
    pub actual: String,
    /// The location where the assertion was made.
    pub location: Option<String>,
    /// The stack trace.
    pub stack_trace: Option<String>,
}

impl AssertionError {
    /// Creates a new assertion error.
    pub fn new<T: Display, U: Display>(message: &str, expected: T, actual: U) -> Self {
        Self {
            message: message.to_string(),
            expected: expected.to_string(),
            actual: actual.to_string(),
            location: None,
            stack_trace: None,
        }
    }

    /// Sets the location where the assertion was made.
    pub fn with_location(mut self, location: &str) -> Self {
        self.location = Some(location.to_string());
        self
    }

    /// Sets the stack trace.
    pub fn with_stack_trace(mut self, stack_trace: &str) -> Self {
        self.stack_trace = Some(stack_trace.to_string());
        self
    }
}

/// Represents the result of an assertion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionResult {
    /// The name of the assertion.
    pub name: String,
    /// The outcome of the assertion.
    pub outcome: AssertionOutcome,
    /// The error that occurred, if any.
    pub error: Option<AssertionError>,
    /// The time it took to execute the assertion.
    pub duration: Duration,
    /// Whether the assertion should fail the test.
    pub fail_test: bool,
    /// Additional metadata about the assertion.
    pub metadata: serde_json::Value,
}

impl AssertionResult {
    /// Creates a new assertion result.
    pub fn new(name: &str, outcome: AssertionOutcome) -> Self {
        Self {
            name: name.to_string(),
            outcome,
            error: None,
            duration: Duration::default(),
            fail_test: true,
            metadata: serde_json::Value::Null,
        }
    }

    /// Sets the error that occurred.
    pub fn with_error(mut self, error: AssertionError) -> Self {
        self.error = Some(error);
        self
    }

    /// Sets the duration of the assertion.
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Sets whether the assertion should fail the test.
    pub fn with_fail_test(mut self, fail_test: bool) -> Self {
        self.fail_test = fail_test;
        self
    }

    /// Sets additional metadata about the assertion.
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Returns whether the assertion passed.
    pub fn passed(&self) -> bool {
        self.outcome == AssertionOutcome::Passed
    }

    /// Returns whether the assertion failed.
    pub fn failed(&self) -> bool {
        self.outcome == AssertionOutcome::Failed
    }

    /// Returns whether the assertion is a warning.
    pub fn is_warning(&self) -> bool {
        self.outcome == AssertionOutcome::Warning
    }

    /// Returns whether the assertion was skipped.
    pub fn skipped(&self) -> bool {
        self.outcome == AssertionOutcome::Skipped
    }

    /// Returns the error that occurred, if any.
    pub fn error(&self) -> Option<&AssertionError> {
        self.error.as_ref()
    }
}

/// Trait for types that can be asserted.
pub trait Assert<T> {
    /// Asserts that the value is equal to the expected value.
    fn is_equal_to(self, expected: T) -> AssertionResult;

    /// Asserts that the value is not equal to the expected value.
    fn is_not_equal_to(self, expected: T) -> AssertionResult;

    /// Asserts that the value matches the given matcher.
    fn matches<M: Matcher<T>>(self, matcher: M) -> AssertionResult;

    /// Sets whether the assertion should fail the test.
    fn with_fail_test(self, fail_test: bool) -> Self;

    /// Sets the name of the assertion.
    fn with_name(self, name: &str) -> Self;
}

/// A wrapper around a value that can be asserted.
#[derive(Debug)]
pub struct AssertThat<T> {
    /// The value to assert.
    value: T,
    /// The name of the assertion.
    name: Option<String>,
    /// Whether the assertion should fail the test.
    fail_test: bool,
}

impl<T: Debug + Clone> AssertThat<T> {
    /// Creates a new assertion wrapper.
    pub fn new(value: T) -> Self {
        Self {
            value,
            name: None,
            fail_test: true,
        }
    }

    /// Sets the name of the assertion.
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Sets whether the assertion should fail the test.
    pub fn with_fail_test(mut self, fail_test: bool) -> Self {
        self.fail_test = fail_test;
        self
    }

    /// Gets the name of the assertion.
    fn get_name(&self) -> String {
        self.name.clone().unwrap_or_else(|| "assertion".to_string())
    }
}

impl<T: Debug + Clone + PartialEq> Assert<T> for AssertThat<T> {
    fn is_equal_to(self, expected: T) -> AssertionResult {
        let start = Instant::now();
        let name = self.get_name();
        let actual = self.value.clone();

        if actual == expected {
            AssertionResult::new(&name, AssertionOutcome::Passed)
                .with_duration(start.elapsed())
                .with_fail_test(self.fail_test)
        } else {
            let error = AssertionError::new(
                &format!("{} is not equal to expected value", name),
                &format!("{:?}", expected),
                &format!("{:?}", actual),
            );

            AssertionResult::new(&name, AssertionOutcome::Failed)
                .with_error(error)
                .with_duration(start.elapsed())
                .with_fail_test(self.fail_test)
        }
    }

    fn is_not_equal_to(self, expected: T) -> AssertionResult {
        let start = Instant::now();
        let name = self.get_name();
        let actual = self.value.clone();

        if actual != expected {
            AssertionResult::new(&name, AssertionOutcome::Passed)
                .with_duration(start.elapsed())
                .with_fail_test(self.fail_test)
        } else {
            let error = AssertionError::new(
                &format!("{} is equal to the value it should not equal", name),
                &format!("not {:?}", expected),
                &format!("{:?}", actual),
            );

            AssertionResult::new(&name, AssertionOutcome::Failed)
                .with_error(error)
                .with_duration(start.elapsed())
                .with_fail_test(self.fail_test)
        }
    }

    fn matches<M: Matcher<T>>(self, matcher: M) -> AssertionResult {
        let start = Instant::now();
        let name = self.get_name();
        let actual = self.value.clone();

        match matcher.matches(&actual) {
            Ok(true) => AssertionResult::new(&name, AssertionOutcome::Passed)
                .with_duration(start.elapsed())
                .with_fail_test(self.fail_test),
            Ok(false) => {
                let error = AssertionError::new(
                    &format!("{} does not match the expected pattern", name),
                    matcher.description(),
                    &format!("{:?}", actual),
                );

                AssertionResult::new(&name, AssertionOutcome::Failed)
                    .with_error(error)
                    .with_duration(start.elapsed())
                    .with_fail_test(self.fail_test)
            }
            Err(e) => {
                let error = AssertionError::new(
                    &format!("Error matching {}: {}", name, e),
                    matcher.description(),
                    &format!("{:?}", actual),
                );

                AssertionResult::new(&name, AssertionOutcome::Failed)
                    .with_error(error)
                    .with_duration(start.elapsed())
                    .with_fail_test(self.fail_test)
            }
        }
    }

    fn with_fail_test(self, fail_test: bool) -> Self {
        self.with_fail_test(fail_test)
    }

    fn with_name(self, name: &str) -> Self {
        self.with_name(name)
    }
}

/// Extension traits for specific assertion types
impl<T: Debug + Clone + PartialEq> AssertThat<T> {
    /// Asserts that the value is true (for boolean values).
    pub fn is_true(self) -> AssertionResult
    where
        T: PartialEq<bool>,
    {
        let start = Instant::now();
        let name = self.get_name();
        let actual = self.value.clone();

        if actual == true {
            AssertionResult::new(&name, AssertionOutcome::Passed)
                .with_duration(start.elapsed())
                .with_fail_test(self.fail_test)
        } else {
            let error = AssertionError::new(
                &format!("{} is not true", name),
                "true",
                &format!("{:?}", actual),
            );

            AssertionResult::new(&name, AssertionOutcome::Failed)
                .with_error(error)
                .with_duration(start.elapsed())
                .with_fail_test(self.fail_test)
        }
    }

    /// Asserts that the value is false (for boolean values).
    pub fn is_false(self) -> AssertionResult
    where
        T: PartialEq<bool>,
    {
        let start = Instant::now();
        let name = self.get_name();
        let actual = self.value.clone();

        if actual == false {
            AssertionResult::new(&name, AssertionOutcome::Passed)
                .with_duration(start.elapsed())
                .with_fail_test(self.fail_test)
        } else {
            let error = AssertionError::new(
                &format!("{} is not false", name),
                "false",
                &format!("{:?}", actual),
            );

            AssertionResult::new(&name, AssertionOutcome::Failed)
                .with_error(error)
                .with_duration(start.elapsed())
                .with_fail_test(self.fail_test)
        }
    }
}

/// A context for grouping assertions.
#[derive(Debug)]
pub struct AssertionContext {
    /// The name of the context.
    name: String,
    /// The assertions in the context.
    assertions: Vec<AssertionResult>,
}

impl AssertionContext {
    /// Creates a new assertion context.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            assertions: Vec::new(),
        }
    }

    /// Adds an assertion to the context.
    pub fn assert<F>(&mut self, f: F)
    where
        F: FnOnce() -> AssertionResult,
    {
        let result = f();
        self.assertions.push(result);
    }

    /// Returns the name of the context.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the number of assertions in the context.
    pub fn assertion_count(&self) -> usize {
        self.assertions.len()
    }

    /// Returns the number of passed assertions in the context.
    pub fn passed_count(&self) -> usize {
        self.assertions
            .iter()
            .filter(|a| a.outcome == AssertionOutcome::Passed)
            .count()
    }

    /// Returns the number of failed assertions in the context.
    pub fn failed_count(&self) -> usize {
        self.assertions
            .iter()
            .filter(|a| a.outcome == AssertionOutcome::Failed && a.fail_test)
            .count()
    }

    /// Returns the number of warning assertions in the context.
    pub fn warning_count(&self) -> usize {
        self.assertions
            .iter()
            .filter(|a| {
                a.outcome == AssertionOutcome::Warning
                    || (a.outcome == AssertionOutcome::Failed && !a.fail_test)
            })
            .count()
    }

    /// Returns whether all assertions in the context passed.
    pub fn all_passed(&self) -> bool {
        self.failed_count() == 0
    }

    /// Returns the assertions in the context.
    pub fn assertions(&self) -> &[AssertionResult] {
        &self.assertions
    }
}

/// A utility for making assertions.
#[derive(Debug, Clone)]
pub struct Assertions {
    /// Whether to fail fast.
    fail_fast: bool,
}

impl Assertions {
    /// Creates a new assertions utility.
    pub fn new() -> Self {
        Self { fail_fast: false }
    }

    /// Sets whether to fail fast.
    pub fn with_fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }

    /// Asserts that two values are equal.
    pub fn assert_equals<T: Debug + PartialEq>(
        &self,
        actual: T,
        expected: T,
        message: Option<String>,
    ) -> Result<(), TestHarnessError> {
        let name = message.unwrap_or_else(|| "Equality assertion".to_string());
        let result = assert_that(actual).with_name(&name).is_equal_to(expected);

        if result.failed() && self.fail_fast {
            if let Some(error) = result.error() {
                return Err(TestHarnessError::AssertionError(error.message.clone()));
            }
        }

        Ok(())
    }

    /// Asserts that two values are not equal.
    pub fn assert_not_equals<T: Debug + PartialEq>(
        &self,
        actual: T,
        expected: T,
        message: Option<String>,
    ) -> Result<(), TestHarnessError> {
        let name = message.unwrap_or_else(|| "Inequality assertion".to_string());
        let result = assert_that(actual)
            .with_name(&name)
            .is_not_equal_to(expected);

        if result.failed() && self.fail_fast {
            if let Some(error) = result.error() {
                return Err(TestHarnessError::AssertionError(error.message.clone()));
            }
        }

        Ok(())
    }

    /// Asserts that a value is true.
    pub fn assert_true(
        &self,
        actual: bool,
        message: Option<String>,
    ) -> Result<(), TestHarnessError> {
        let name = message.unwrap_or_else(|| "True assertion".to_string());
        let result = assert_that(actual).with_name(&name).is_true();

        if result.failed() && self.fail_fast {
            if let Some(error) = result.error() {
                return Err(TestHarnessError::AssertionError(error.message.clone()));
            }
        }

        Ok(())
    }

    /// Asserts that a value is false.
    pub fn assert_false(
        &self,
        actual: bool,
        message: Option<String>,
    ) -> Result<(), TestHarnessError> {
        let name = message.unwrap_or_else(|| "False assertion".to_string());
        let result = assert_that(actual).with_name(&name).is_false();

        if result.failed() && self.fail_fast {
            if let Some(error) = result.error() {
                return Err(TestHarnessError::AssertionError(error.message.clone()));
            }
        }

        Ok(())
    }

    /// Asserts that a string contains a substring.
    pub fn assert_contains(
        &self,
        actual: &str,
        expected: &str,
        message: Option<String>,
    ) -> Result<(), TestHarnessError> {
        let name = message.unwrap_or_else(|| "Contains assertion".to_string());
        let result = assert_that(actual).with_name(&name).contains(expected);

        if result.failed() && self.fail_fast {
            if let Some(error) = result.error() {
                return Err(TestHarnessError::AssertionError(error.message.clone()));
            }
        }

        Ok(())
    }

    /// Asserts that a string matches a pattern.
    pub fn assert_matches(
        &self,
        actual: &str,
        pattern: &str,
        message: Option<String>,
    ) -> Result<(), TestHarnessError> {
        let name = message.unwrap_or_else(|| "Pattern matching assertion".to_string());
        let result = assert_that(actual)
            .with_name(&name)
            .matches_pattern(pattern);

        if result.failed() && self.fail_fast {
            if let Some(error) = result.error() {
                return Err(TestHarnessError::AssertionError(error.message.clone()));
            }
        }

        Ok(())
    }
}

impl Default for Assertions {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new assertion wrapper.
pub fn assert_that<T: Debug + Clone>(value: T) -> AssertThat<T> {
    AssertThat::new(value)
}

/// Creates a new assertion context.
pub fn assert_context(name: &str) -> AssertionContext {
    AssertionContext::new(name)
}

/// Extension trait for string assertions
impl AssertThat<&str> {
    /// Asserts that the string contains the expected substring.
    pub fn contains(self, expected: &str) -> AssertionResult {
        let start = Instant::now();
        let name = self.get_name();
        let actual = self.value;

        if actual.contains(expected) {
            AssertionResult::new(&name, AssertionOutcome::Passed)
                .with_duration(start.elapsed())
                .with_fail_test(self.fail_test)
        } else {
            let error = AssertionError::new(
                &format!("{} does not contain the expected substring", name),
                expected,
                actual,
            );

            AssertionResult::new(&name, AssertionOutcome::Failed)
                .with_error(error)
                .with_duration(start.elapsed())
                .with_fail_test(self.fail_test)
        }
    }

    /// Asserts that the string matches the expected pattern.
    pub fn matches_pattern(self, pattern: &str) -> AssertionResult {
        let start = Instant::now();
        let name = self.get_name();
        let actual = self.value;

        match regex::Regex::new(pattern) {
            Ok(regex) => {
                if regex.is_match(actual) {
                    AssertionResult::new(&name, AssertionOutcome::Passed)
                        .with_duration(start.elapsed())
                        .with_fail_test(self.fail_test)
                } else {
                    let error = AssertionError::new(
                        &format!("{} does not match the expected pattern", name),
                        pattern,
                        actual,
                    );

                    AssertionResult::new(&name, AssertionOutcome::Failed)
                        .with_error(error)
                        .with_duration(start.elapsed())
                        .with_fail_test(self.fail_test)
                }
            }
            Err(e) => {
                let error =
                    AssertionError::new(&format!("Invalid regex pattern: {}", e), pattern, actual);

                AssertionResult::new(&name, AssertionOutcome::Failed)
                    .with_error(error)
                    .with_duration(start.elapsed())
                    .with_fail_test(self.fail_test)
            }
        }
    }
}
