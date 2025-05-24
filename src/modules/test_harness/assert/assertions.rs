//! Assertions Module
//!
//! This module provides the core assertion functionality.

use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use crate::modules::test_harness::types::TestHarnessError;

/// Assertion error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionError {
    /// Error message
    pub message: String,
    /// Actual value
    pub actual: String,
    /// Expected value
    pub expected: String,
    /// Error details
    pub details: Option<String>,
    /// Error location
    pub location: Option<String>,
}

impl AssertionError {
    /// Create a new assertion error
    pub fn new(
        message: impl Into<String>,
        actual: impl Into<String>,
        expected: impl Into<String>,
    ) -> Self {
        Self {
            message: message.into(),
            actual: actual.into(),
            expected: expected.into(),
            details: None,
            location: None,
        }
    }

    /// Add details to the error
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Add location to the error
    pub fn with_location(mut self, location: impl Into<String>) -> Self {
        self.location = Some(location.into());
        self
    }
}

impl fmt::Display for AssertionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;

        if let Some(location) = &self.location {
            write!(f, " at {}", location)?;
        }

        write!(f, "\n  Expected: {}", self.expected)?;
        write!(f, "\n  Actual:   {}", self.actual)?;

        if let Some(details) = &self.details {
            write!(f, "\n  Details:  {}", details)?;
        }

        Ok(())
    }
}

/// Assertion result
#[derive(Debug, Clone)]
pub struct AssertionResult {
    /// Whether the assertion passed
    passed: bool,
    /// Whether the assertion is a warning
    warning: bool,
    /// Assertion message
    message: String,
    /// Assertion error
    error: Option<AssertionError>,
}

impl AssertionResult {
    /// Create a new assertion result
    pub fn new(passed: bool, message: impl Into<String>, error: Option<AssertionError>) -> Self {
        Self {
            passed,
            warning: false,
            message: message.into(),
            error,
        }
    }

    /// Create a successful assertion result
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            passed: true,
            warning: false,
            message: message.into(),
            error: None,
        }
    }

    /// Create a failed assertion result
    pub fn failure(error: AssertionError) -> Self {
        Self {
            passed: false,
            warning: false,
            message: error.message.clone(),
            error: Some(error),
        }
    }

    /// Create a warning assertion result
    pub fn warning(error: AssertionError) -> Self {
        Self {
            passed: true,
            warning: true,
            message: error.message.clone(),
            error: Some(error),
        }
    }

    /// Check if the assertion passed
    pub fn passed(&self) -> bool {
        self.passed
    }

    /// Check if the assertion failed
    pub fn failed(&self) -> bool {
        !self.passed
    }

    /// Check if the assertion is a warning
    pub fn is_warning(&self) -> bool {
        self.warning
    }

    /// Get the assertion message
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Get the assertion error
    pub fn error(&self) -> Option<&AssertionError> {
        self.error.as_ref()
    }

    /// Convert to a result
    pub fn to_result(self) -> Result<(), AssertionError> {
        if self.passed {
            Ok(())
        } else {
            Err(self.error.unwrap_or_else(|| {
                AssertionError::new(
                    self.message.clone(),
                    "unknown".to_string(),
                    "unknown".to_string(),
                )
            }))
        }
    }
}

impl fmt::Display for AssertionResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.passed {
            if self.warning {
                write!(f, "WARNING: {}", self.message)
            } else {
                write!(f, "PASSED: {}", self.message)
            }
        } else {
            write!(f, "FAILED: {}", self.message)?;

            if let Some(error) = &self.error {
                write!(f, "\n  Expected: {}", error.expected)?;
                write!(f, "\n  Actual:   {}", error.actual)?;

                if let Some(details) = &error.details {
                    write!(f, "\n  Details:  {}", details)?;
                }

                if let Some(location) = &error.location {
                    write!(f, "\n  Location: {}", location)?;
                }
            }

            Ok(())
        }
    }
}

/// Assertion trait
pub trait Assert {
    /// Make an assertion
    fn assert(&self, result: AssertionResult) -> Result<(), TestHarnessError>;

    /// Make multiple assertions
    fn assert_all(&self, results: Vec<AssertionResult>) -> Result<(), TestHarnessError>;

    /// Make an assertion with a custom message
    fn assert_with_message(
        &self,
        condition: bool,
        message: impl Into<String>,
    ) -> Result<(), TestHarnessError>;

    /// Make an assertion with a custom error
    fn assert_with_error(
        &self,
        condition: bool,
        error: AssertionError,
    ) -> Result<(), TestHarnessError>;

    /// Assert that a value equals an expected value
    fn assert_equals<T, U>(
        &self,
        actual: T,
        expected: U,
        message: Option<String>,
    ) -> Result<(), TestHarnessError>
    where
        T: PartialEq<U> + fmt::Debug,
        U: fmt::Debug;

    /// Assert that a value does not equal an expected value
    fn assert_not_equals<T, U>(
        &self,
        actual: T,
        expected: U,
        message: Option<String>,
    ) -> Result<(), TestHarnessError>
    where
        T: PartialEq<U> + fmt::Debug,
        U: fmt::Debug;

    /// Assert that a value is true
    fn assert_true(&self, condition: bool, message: Option<String>)
        -> Result<(), TestHarnessError>;

    /// Assert that a value is false
    fn assert_false(
        &self,
        condition: bool,
        message: Option<String>,
    ) -> Result<(), TestHarnessError>;

    /// Assert that a value is null
    fn assert_null<T: fmt::Debug>(
        &self,
        value: T,
        message: Option<String>,
    ) -> Result<(), TestHarnessError>;

    /// Assert that a value is not null
    fn assert_not_null<T: fmt::Debug>(
        &self,
        value: T,
        message: Option<String>,
    ) -> Result<(), TestHarnessError>;

    /// Assert that a collection contains a value
    fn assert_contains<T, U>(
        &self,
        collection: T,
        value: U,
        message: Option<String>,
    ) -> Result<(), TestHarnessError>
    where
        T: fmt::Debug,
        U: fmt::Debug;

    /// Assert that a collection does not contain a value
    fn assert_not_contains<T, U>(
        &self,
        collection: T,
        value: U,
        message: Option<String>,
    ) -> Result<(), TestHarnessError>
    where
        T: fmt::Debug,
        U: fmt::Debug;

    /// Assert that a value matches a pattern
    fn assert_matches<T: fmt::Display + fmt::Debug>(
        &self,
        value: T,
        pattern: &str,
        message: Option<String>,
    ) -> Result<(), TestHarnessError>;

    /// Assert that a value does not match a pattern
    fn assert_not_matches<T: fmt::Display + fmt::Debug>(
        &self,
        value: T,
        pattern: &str,
        message: Option<String>,
    ) -> Result<(), TestHarnessError>;
}

/// Assertions implementation
pub struct Assertions {
    /// Whether to fail on the first assertion failure
    fail_fast: bool,
    /// Whether to log assertions
    log_assertions: bool,
}

impl Assertions {
    /// Create a new assertions instance
    pub fn new() -> Self {
        Self {
            fail_fast: true,
            log_assertions: true,
        }
    }

    /// Set whether to fail on the first assertion failure
    pub fn with_fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }

    /// Set whether to log assertions
    pub fn with_log_assertions(mut self, log_assertions: bool) -> Self {
        self.log_assertions = log_assertions;
        self
    }
}

impl Default for Assertions {
    fn default() -> Self {
        Self::new()
    }
}

impl Assert for Assertions {
    fn assert(&self, result: AssertionResult) -> Result<(), TestHarnessError> {
        if self.log_assertions {
            if result.passed() {
                if result.is_warning() {
                    warn!("{}", result);
                } else {
                    info!("{}", result);
                }
            } else {
                error!("{}", result);
            }
        }

        if result.failed() && self.fail_fast {
            if let Some(error) = result.error() {
                return Err(TestHarnessError::AssertionError(error.clone()));
            } else {
                return Err(TestHarnessError::AssertionError(AssertionError::new(
                    result.message().to_string(),
                    "unknown".to_string(),
                    "unknown".to_string(),
                )));
            }
        }

        Ok(())
    }

    fn assert_all(&self, results: Vec<AssertionResult>) -> Result<(), TestHarnessError> {
        let mut errors = Vec::new();

        for result in results {
            if self.log_assertions {
                if result.passed() {
                    if result.is_warning() {
                        warn!("{}", result);
                    } else {
                        info!("{}", result);
                    }
                } else {
                    error!("{}", result);
                }
            }

            if result.failed() {
                if let Some(error) = result.error() {
                    errors.push(error.clone());
                } else {
                    errors.push(AssertionError::new(
                        result.message().to_string(),
                        "unknown".to_string(),
                        "unknown".to_string(),
                    ));
                }

                if self.fail_fast {
                    break;
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else if errors.len() == 1 {
            Err(TestHarnessError::AssertionError(errors[0].clone()))
        } else {
            Err(TestHarnessError::MultipleAssertionErrors(errors))
        }
    }

    fn assert_with_message(
        &self,
        condition: bool,
        message: impl Into<String>,
    ) -> Result<(), TestHarnessError> {
        let message = message.into();

        if condition {
            self.assert(AssertionResult::success(message))
        } else {
            self.assert(AssertionResult::failure(AssertionError::new(
                message,
                "false".to_string(),
                "true".to_string(),
            )))
        }
    }

    fn assert_with_error(
        &self,
        condition: bool,
        error: AssertionError,
    ) -> Result<(), TestHarnessError> {
        if condition {
            self.assert(AssertionResult::success(error.message.clone()))
        } else {
            self.assert(AssertionResult::failure(error))
        }
    }

    fn assert_equals<T, U>(
        &self,
        actual: T,
        expected: U,
        message: Option<String>,
    ) -> Result<(), TestHarnessError>
    where
        T: PartialEq<U> + fmt::Debug,
        U: fmt::Debug,
    {
        let message =
            message.unwrap_or_else(|| format!("Expected {:?} to equal {:?}", actual, expected));

        if actual == expected {
            self.assert(AssertionResult::success(message))
        } else {
            self.assert(AssertionResult::failure(AssertionError::new(
                message,
                format!("{:?}", actual),
                format!("{:?}", expected),
            )))
        }
    }

    fn assert_not_equals<T, U>(
        &self,
        actual: T,
        expected: U,
        message: Option<String>,
    ) -> Result<(), TestHarnessError>
    where
        T: PartialEq<U> + fmt::Debug,
        U: fmt::Debug,
    {
        let message =
            message.unwrap_or_else(|| format!("Expected {:?} to not equal {:?}", actual, expected));

        if actual != expected {
            self.assert(AssertionResult::success(message))
        } else {
            self.assert(AssertionResult::failure(AssertionError::new(
                message,
                format!("{:?}", actual),
                format!("not {:?}", expected),
            )))
        }
    }

    fn assert_true(
        &self,
        condition: bool,
        message: Option<String>,
    ) -> Result<(), TestHarnessError> {
        let message = message.unwrap_or_else(|| "Expected condition to be true".to_string());

        if condition {
            self.assert(AssertionResult::success(message))
        } else {
            self.assert(AssertionResult::failure(AssertionError::new(
                message,
                "false".to_string(),
                "true".to_string(),
            )))
        }
    }

    fn assert_false(
        &self,
        condition: bool,
        message: Option<String>,
    ) -> Result<(), TestHarnessError> {
        let message = message.unwrap_or_else(|| "Expected condition to be false".to_string());

        if !condition {
            self.assert(AssertionResult::success(message))
        } else {
            self.assert(AssertionResult::failure(AssertionError::new(
                message,
                "true".to_string(),
                "false".to_string(),
            )))
        }
    }

    fn assert_null<T: fmt::Debug>(
        &self,
        value: T,
        message: Option<String>,
    ) -> Result<(), TestHarnessError> {
        let message = message.unwrap_or_else(|| format!("Expected {:?} to be null", value));

        let is_null = match serde_json::to_value(&value) {
            Ok(value) => value.is_null(),
            Err(_) => false,
        };

        if is_null {
            self.assert(AssertionResult::success(message))
        } else {
            self.assert(AssertionResult::failure(AssertionError::new(
                message,
                format!("{:?}", value),
                "null".to_string(),
            )))
        }
    }

    fn assert_not_null<T: fmt::Debug>(
        &self,
        value: T,
        message: Option<String>,
    ) -> Result<(), TestHarnessError> {
        let message = message.unwrap_or_else(|| format!("Expected {:?} to not be null", value));

        let is_null = match serde_json::to_value(&value) {
            Ok(value) => value.is_null(),
            Err(_) => false,
        };

        if !is_null {
            self.assert(AssertionResult::success(message))
        } else {
            self.assert(AssertionResult::failure(AssertionError::new(
                message,
                "null".to_string(),
                "not null".to_string(),
            )))
        }
    }

    fn assert_contains<T, U>(
        &self,
        collection: T,
        value: U,
        message: Option<String>,
    ) -> Result<(), TestHarnessError>
    where
        T: fmt::Debug,
        U: fmt::Debug,
    {
        let message =
            message.unwrap_or_else(|| format!("Expected {:?} to contain {:?}", collection, value));

        let contains = match (
            serde_json::to_value(&collection),
            serde_json::to_value(&value),
        ) {
            (Ok(collection), Ok(value)) => {
                if collection.is_array() {
                    if let Some(array) = collection.as_array() {
                        array.contains(&value)
                    } else {
                        false
                    }
                } else if collection.is_object() {
                    if let (Some(obj), Some(key)) = (collection.as_object(), value.as_str()) {
                        obj.contains_key(key)
                    } else {
                        false
                    }
                } else if collection.is_string() && value.is_string() {
                    if let (Some(s), Some(e)) = (collection.as_str(), value.as_str()) {
                        s.contains(e)
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => false,
        };

        if contains {
            self.assert(AssertionResult::success(message))
        } else {
            self.assert(AssertionResult::failure(AssertionError::new(
                message,
                format!("{:?}", collection),
                format!("to contain {:?}", value),
            )))
        }
    }

    fn assert_not_contains<T, U>(
        &self,
        collection: T,
        value: U,
        message: Option<String>,
    ) -> Result<(), TestHarnessError>
    where
        T: fmt::Debug,
        U: fmt::Debug,
    {
        let message = message
            .unwrap_or_else(|| format!("Expected {:?} to not contain {:?}", collection, value));

        let contains = match (
            serde_json::to_value(&collection),
            serde_json::to_value(&value),
        ) {
            (Ok(collection), Ok(value)) => {
                if collection.is_array() {
                    if let Some(array) = collection.as_array() {
                        array.contains(&value)
                    } else {
                        false
                    }
                } else if collection.is_object() {
                    if let (Some(obj), Some(key)) = (collection.as_object(), value.as_str()) {
                        obj.contains_key(key)
                    } else {
                        false
                    }
                } else if collection.is_string() && value.is_string() {
                    if let (Some(s), Some(e)) = (collection.as_str(), value.as_str()) {
                        s.contains(e)
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => false,
        };

        if !contains {
            self.assert(AssertionResult::success(message))
        } else {
            self.assert(AssertionResult::failure(AssertionError::new(
                message,
                format!("{:?}", collection),
                format!("to not contain {:?}", value),
            )))
        }
    }

    fn assert_matches<T: fmt::Display + fmt::Debug>(
        &self,
        value: T,
        pattern: &str,
        message: Option<String>,
    ) -> Result<(), TestHarnessError> {
        let message = message
            .unwrap_or_else(|| format!("Expected {:?} to match pattern '{}'", value, pattern));

        let regex = match regex::Regex::new(pattern) {
            Ok(regex) => regex,
            Err(e) => {
                return self.assert(AssertionResult::failure(AssertionError::new(
                    format!("Invalid regex pattern: {}", e),
                    format!("{:?}", value),
                    format!("to match pattern '{}'", pattern),
                )));
            }
        };

        let value_str = format!("{}", value);
        if regex.is_match(&value_str) {
            self.assert(AssertionResult::success(message))
        } else {
            self.assert(AssertionResult::failure(AssertionError::new(
                message,
                format!("{:?}", value),
                format!("to match pattern '{}'", pattern),
            )))
        }
    }

    fn assert_not_matches<T: fmt::Display + fmt::Debug>(
        &self,
        value: T,
        pattern: &str,
        message: Option<String>,
    ) -> Result<(), TestHarnessError> {
        let message = message
            .unwrap_or_else(|| format!("Expected {:?} to not match pattern '{}'", value, pattern));

        let regex = match regex::Regex::new(pattern) {
            Ok(regex) => regex,
            Err(e) => {
                return self.assert(AssertionResult::failure(AssertionError::new(
                    format!("Invalid regex pattern: {}", e),
                    format!("{:?}", value),
                    format!("to not match pattern '{}'", pattern),
                )));
            }
        };

        let value_str = format!("{}", value);
        if !regex.is_match(&value_str) {
            self.assert(AssertionResult::success(message))
        } else {
            self.assert(AssertionResult::failure(AssertionError::new(
                message,
                format!("{:?}", value),
                format!("to not match pattern '{}'", pattern),
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assertion_result() {
        // Test success
        let result = AssertionResult::success("Test passed");
        assert!(result.passed());
        assert!(!result.failed());
        assert!(!result.is_warning());
        assert_eq!(result.message(), "Test passed");
        assert!(result.error().is_none());

        // Test failure
        let error = AssertionError::new("Test failed", "actual", "expected");
        let result = AssertionResult::failure(error);
        assert!(!result.passed());
        assert!(result.failed());
        assert!(!result.is_warning());
        assert_eq!(result.message(), "Test failed");
        assert!(result.error().is_some());

        // Test warning
        let error = AssertionError::new("Test warning", "actual", "expected");
        let result = AssertionResult::warning(error);
        assert!(result.passed());
        assert!(!result.failed());
        assert!(result.is_warning());
        assert_eq!(result.message(), "Test warning");
        assert!(result.error().is_some());
    }

    #[test]
    fn test_assertions() {
        let assertions = Assertions::new();

        // Test assert_equals
        assert!(assertions.assert_equals(42, 42, None).is_ok());
        assert!(assertions.assert_equals(42, 43, None).is_err());

        // Test assert_not_equals
        assert!(assertions.assert_not_equals(42, 43, None).is_ok());
        assert!(assertions.assert_not_equals(42, 42, None).is_err());

        // Test assert_true
        assert!(assertions.assert_true(true, None).is_ok());
        assert!(assertions.assert_true(false, None).is_err());

        // Test assert_false
        assert!(assertions.assert_false(false, None).is_ok());
        assert!(assertions.assert_false(true, None).is_err());

        // Test assert_contains
        assert!(assertions.assert_contains(vec![1, 2, 3], 2, None).is_ok());
        assert!(assertions.assert_contains(vec![1, 2, 3], 4, None).is_err());

        // Test assert_not_contains
        assert!(assertions
            .assert_not_contains(vec![1, 2, 3], 4, None)
            .is_ok());
        assert!(assertions
            .assert_not_contains(vec![1, 2, 3], 2, None)
            .is_err());

        // Test assert_matches
        assert!(assertions
            .assert_matches("hello world", r"hello \w+", None)
            .is_ok());
        assert!(assertions
            .assert_matches("hello world", r"goodbye \w+", None)
            .is_err());

        // Test assert_not_matches
        assert!(assertions
            .assert_not_matches("hello world", r"goodbye \w+", None)
            .is_ok());
        assert!(assertions
            .assert_not_matches("hello world", r"hello \w+", None)
            .is_err());
    }
}
