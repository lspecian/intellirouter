//! Test Harness Utilities
//!
//! This module provides utility functions for the test harness.

use super::types::{AssertionError, AssertionResult, TestHarnessError};

/// Assertion helper for test cases
pub struct AssertionHelper;

impl AssertionHelper {
    /// Assert that two values are equal
    pub fn assert_eq<T: PartialEq + std::fmt::Debug>(
        actual: T,
        expected: T,
        message: &str,
    ) -> Result<AssertionResult, TestHarnessError> {
        let passed = actual == expected;
        let result = AssertionResult::new(message, passed);

        if !passed {
            let error = AssertionError::new(message.to_string())
                .with_expected(format!("{:?}", expected))
                .with_actual(format!("{:?}", actual));

            return Err(TestHarnessError::AssertionError(error.message));
        }

        Ok(result)
    }

    /// Assert that a condition is true
    pub fn assert_true(
        condition: bool,
        message: &str,
    ) -> Result<AssertionResult, TestHarnessError> {
        let result = AssertionResult::new(message, condition);

        if !condition {
            let error = AssertionError::new(message.to_string())
                .with_expected("true".to_string())
                .with_actual("false".to_string());

            return Err(TestHarnessError::AssertionError(error.message));
        }

        Ok(result)
    }

    /// Assert that a condition is false
    pub fn assert_false(
        condition: bool,
        message: &str,
    ) -> Result<AssertionResult, TestHarnessError> {
        let result = AssertionResult::new(message, !condition);

        if condition {
            let error = AssertionError::new(message.to_string())
                .with_expected("false".to_string())
                .with_actual("true".to_string());

            return Err(TestHarnessError::AssertionError(error.message));
        }

        Ok(result)
    }

    /// Assert that a string contains a substring
    pub fn assert_contains(
        haystack: &str,
        needle: &str,
        message: &str,
    ) -> Result<AssertionResult, TestHarnessError> {
        let passed = haystack.contains(needle);
        let result = AssertionResult::new(message, passed);

        if !passed {
            let error = AssertionError::new(message.to_string())
                .with_expected(format!("string containing '{}'", needle))
                .with_actual(format!("'{}'", haystack));

            return Err(TestHarnessError::AssertionError(error.message));
        }

        Ok(result)
    }

    /// Assert that a value is None
    pub fn assert_none<T: std::fmt::Debug>(
        option: Option<T>,
        message: &str,
    ) -> Result<AssertionResult, TestHarnessError> {
        let passed = option.is_none();
        let result = AssertionResult::new(message, passed);

        if !passed {
            let error = AssertionError::new(message.to_string())
                .with_expected("None".to_string())
                .with_actual(format!("Some({:?})", option.unwrap()));

            return Err(TestHarnessError::AssertionError(error.message));
        }

        Ok(result)
    }

    /// Assert that a value is Some
    pub fn assert_some<T: std::fmt::Debug>(
        option: Option<T>,
        message: &str,
    ) -> Result<AssertionResult, TestHarnessError> {
        let passed = option.is_some();
        let result = AssertionResult::new(message, passed);

        if !passed {
            let error = AssertionError::new(message.to_string())
                .with_expected("Some(...)".to_string())
                .with_actual("None".to_string());

            return Err(TestHarnessError::AssertionError(error.message));
        }

        Ok(result)
    }

    /// Assert that a result is Ok
    pub fn assert_ok<T: std::fmt::Debug, E: std::fmt::Debug>(
        result: Result<T, E>,
        message: &str,
    ) -> Result<AssertionResult, TestHarnessError> {
        let passed = result.is_ok();
        let assertion_result = AssertionResult::new(message, passed);

        if !passed {
            let error = AssertionError::new(message.to_string())
                .with_expected("Ok(...)".to_string())
                .with_actual(format!("Err({:?})", result.unwrap_err()));

            return Err(TestHarnessError::AssertionError(error.message));
        }

        Ok(assertion_result)
    }

    /// Assert that a result is Err
    pub fn assert_err<T: std::fmt::Debug, E: std::fmt::Debug>(
        result: Result<T, E>,
        message: &str,
    ) -> Result<AssertionResult, TestHarnessError> {
        let passed = result.is_err();
        let assertion_result = AssertionResult::new(message, passed);

        if !passed {
            let error = AssertionError::new(message.to_string())
                .with_expected("Err(...)".to_string())
                .with_actual(format!("Ok({:?})", result.unwrap()));

            return Err(TestHarnessError::AssertionError(error.message));
        }

        Ok(assertion_result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_eq() {
        let result = AssertionHelper::assert_eq(2 + 2, 4, "2 + 2 should equal 4");
        assert!(result.is_ok());

        let result = AssertionHelper::assert_eq(2 + 2, 5, "2 + 2 should equal 5");
        assert!(result.is_err());
    }

    #[test]
    fn test_assert_true() {
        let result = AssertionHelper::assert_true(true, "true should be true");
        assert!(result.is_ok());

        let result = AssertionHelper::assert_true(false, "false should be true");
        assert!(result.is_err());
    }

    #[test]
    fn test_assert_false() {
        let result = AssertionHelper::assert_false(false, "false should be false");
        assert!(result.is_ok());

        let result = AssertionHelper::assert_false(true, "true should be false");
        assert!(result.is_err());
    }

    #[test]
    fn test_assert_contains() {
        let result =
            AssertionHelper::assert_contains("hello world", "world", "should contain world");
        assert!(result.is_ok());

        let result =
            AssertionHelper::assert_contains("hello world", "universe", "should contain universe");
        assert!(result.is_err());
    }

    #[test]
    fn test_assert_none() {
        let option: Option<i32> = None;
        let result = AssertionHelper::assert_none(option, "should be None");
        assert!(result.is_ok());

        let option: Option<i32> = Some(42);
        let result = AssertionHelper::assert_none(option, "should be None");
        assert!(result.is_err());
    }

    #[test]
    fn test_assert_some() {
        let option: Option<i32> = Some(42);
        let result = AssertionHelper::assert_some(option, "should be Some");
        assert!(result.is_ok());

        let option: Option<i32> = None;
        let result = AssertionHelper::assert_some(option, "should be Some");
        assert!(result.is_err());
    }

    #[test]
    fn test_assert_ok() {
        let result: Result<i32, &str> = Ok(42);
        let assertion_result = AssertionHelper::assert_ok(result, "should be Ok");
        assert!(assertion_result.is_ok());

        let result: Result<i32, &str> = Err("error");
        let assertion_result = AssertionHelper::assert_ok(result, "should be Ok");
        assert!(assertion_result.is_err());
    }

    #[test]
    fn test_assert_err() {
        let result: Result<i32, &str> = Err("error");
        let assertion_result = AssertionHelper::assert_err(result, "should be Err");
        assert!(assertion_result.is_ok());

        let result: Result<i32, &str> = Ok(42);
        let assertion_result = AssertionHelper::assert_err(result, "should be Err");
        assert!(assertion_result.is_err());
    }
}
