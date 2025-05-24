//! Comparators for assertions
//!
//! This module provides comparators for comparing values in assertions.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareResult {
    /// Whether the values are equal
    pub is_equal: bool,
    /// Expected value
    pub expected: Option<String>,
    /// Actual value
    pub actual: Option<String>,
    /// Comparison details
    pub details: Option<String>,
}

impl CompareResult {
    /// Create a new comparison result
    pub fn new(is_equal: bool) -> Self {
        Self {
            is_equal,
            expected: None,
            actual: None,
            details: None,
        }
    }

    /// Set the expected value
    pub fn with_expected(mut self, expected: impl fmt::Display) -> Self {
        self.expected = Some(expected.to_string());
        self
    }

    /// Set the actual value
    pub fn with_actual(mut self, actual: impl fmt::Display) -> Self {
        self.actual = Some(actual.to_string());
        self
    }

    /// Set the comparison details
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

/// Comparison options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareOptions {
    /// Whether to ignore case when comparing strings
    pub ignore_case: bool,
    /// Whether to ignore whitespace when comparing strings
    pub ignore_whitespace: bool,
    /// Whether to ignore order when comparing arrays
    pub ignore_order: bool,
    /// Whether to ignore extra fields when comparing objects
    pub ignore_extra_fields: bool,
    /// Whether to ignore missing fields when comparing objects
    pub ignore_missing_fields: bool,
    /// Custom options
    pub custom_options: std::collections::HashMap<String, serde_json::Value>,
}

impl Default for CompareOptions {
    fn default() -> Self {
        Self {
            ignore_case: false,
            ignore_whitespace: false,
            ignore_order: false,
            ignore_extra_fields: false,
            ignore_missing_fields: false,
            custom_options: std::collections::HashMap::new(),
        }
    }
}

/// Comparator trait for comparing values
pub trait Comparator: Send + Sync {
    /// Compare two values
    fn compare(&self, actual: &dyn std::any::Any, expected: &dyn std::any::Any) -> CompareResult;
}

/// Compare trait for types that can be compared
pub trait Compare {
    /// Compare with another value
    fn compare_with(&self, other: &Self, options: &CompareOptions) -> CompareResult;
}

/// Default comparator
pub struct DefaultComparator {
    /// Comparison options
    options: CompareOptions,
}

impl DefaultComparator {
    /// Create a new default comparator
    pub fn new() -> Self {
        Self {
            options: CompareOptions::default(),
        }
    }

    /// Create a new default comparator with options
    pub fn with_options(options: CompareOptions) -> Self {
        Self { options }
    }
}

impl Comparator for DefaultComparator {
    fn compare(&self, actual: &dyn std::any::Any, expected: &dyn std::any::Any) -> CompareResult {
        // Try to downcast to common types
        if let (Some(actual), Some(expected)) = (
            actual.downcast_ref::<String>(),
            expected.downcast_ref::<String>(),
        ) {
            return self.compare_strings(actual, expected);
        }

        if let (Some(actual), Some(expected)) =
            (actual.downcast_ref::<i32>(), expected.downcast_ref::<i32>())
        {
            return self.compare_i32(actual, expected);
        }

        if let (Some(actual), Some(expected)) =
            (actual.downcast_ref::<i64>(), expected.downcast_ref::<i64>())
        {
            return self.compare_i64(actual, expected);
        }

        if let (Some(actual), Some(expected)) =
            (actual.downcast_ref::<f64>(), expected.downcast_ref::<f64>())
        {
            return self.compare_f64(actual, expected);
        }

        if let (Some(actual), Some(expected)) = (
            actual.downcast_ref::<bool>(),
            expected.downcast_ref::<bool>(),
        ) {
            return self.compare_bool(actual, expected);
        }

        if let (Some(actual), Some(expected)) = (
            actual.downcast_ref::<Vec<String>>(),
            expected.downcast_ref::<Vec<String>>(),
        ) {
            return self.compare_string_vec(actual, expected);
        }

        if let (Some(actual), Some(expected)) = (
            actual.downcast_ref::<Vec<i32>>(),
            expected.downcast_ref::<Vec<i32>>(),
        ) {
            return self.compare_i32_vec(actual, expected);
        }

        if let (Some(actual), Some(expected)) = (
            actual.downcast_ref::<serde_json::Value>(),
            expected.downcast_ref::<serde_json::Value>(),
        ) {
            return self.compare_json(actual, expected);
        }

        // Fall back to string representation
        let actual_str = format!("{:?}", actual);
        let expected_str = format!("{:?}", expected);
        let is_equal = actual_str == expected_str;

        CompareResult::new(is_equal)
            .with_actual(actual_str)
            .with_expected(expected_str)
    }
}

impl DefaultComparator {
    /// Compare two strings
    fn compare_strings(&self, actual: &str, expected: &str) -> CompareResult {
        let mut actual_str = actual.to_string();
        let mut expected_str = expected.to_string();

        if self.options.ignore_case {
            actual_str = actual_str.to_lowercase();
            expected_str = expected_str.to_lowercase();
        }

        if self.options.ignore_whitespace {
            actual_str = actual_str.split_whitespace().collect::<String>();
            expected_str = expected_str.split_whitespace().collect::<String>();
        }

        let is_equal = actual_str == expected_str;

        CompareResult::new(is_equal)
            .with_actual(actual)
            .with_expected(expected)
    }

    /// Compare two i32 values
    fn compare_i32(&self, actual: &i32, expected: &i32) -> CompareResult {
        let is_equal = actual == expected;

        CompareResult::new(is_equal)
            .with_actual(actual)
            .with_expected(expected)
    }

    /// Compare two i64 values
    fn compare_i64(&self, actual: &i64, expected: &i64) -> CompareResult {
        let is_equal = actual == expected;

        CompareResult::new(is_equal)
            .with_actual(actual)
            .with_expected(expected)
    }

    /// Compare two f64 values
    fn compare_f64(&self, actual: &f64, expected: &f64) -> CompareResult {
        // Use approximate equality for floating point
        let epsilon = 1e-10;
        let is_equal = (actual - expected).abs() < epsilon;

        CompareResult::new(is_equal)
            .with_actual(actual)
            .with_expected(expected)
    }

    /// Compare two bool values
    fn compare_bool(&self, actual: &bool, expected: &bool) -> CompareResult {
        let is_equal = actual == expected;

        CompareResult::new(is_equal)
            .with_actual(actual)
            .with_expected(expected)
    }

    /// Compare two string vectors
    fn compare_string_vec(&self, actual: &[String], expected: &[String]) -> CompareResult {
        if self.options.ignore_order {
            // Compare as sets
            let actual_set: std::collections::HashSet<_> = actual.iter().collect();
            let expected_set: std::collections::HashSet<_> = expected.iter().collect();
            let is_equal = actual_set == expected_set;

            CompareResult::new(is_equal)
                .with_actual(format!("{:?}", actual))
                .with_expected(format!("{:?}", expected))
        } else {
            // Compare as ordered sequences
            let is_equal = actual == expected;

            CompareResult::new(is_equal)
                .with_actual(format!("{:?}", actual))
                .with_expected(format!("{:?}", expected))
        }
    }

    /// Compare two i32 vectors
    fn compare_i32_vec(&self, actual: &[i32], expected: &[i32]) -> CompareResult {
        if self.options.ignore_order {
            // Compare as sets
            let actual_set: std::collections::HashSet<_> = actual.iter().collect();
            let expected_set: std::collections::HashSet<_> = expected.iter().collect();
            let is_equal = actual_set == expected_set;

            CompareResult::new(is_equal)
                .with_actual(format!("{:?}", actual))
                .with_expected(format!("{:?}", expected))
        } else {
            // Compare as ordered sequences
            let is_equal = actual == expected;

            CompareResult::new(is_equal)
                .with_actual(format!("{:?}", actual))
                .with_expected(format!("{:?}", expected))
        }
    }

    /// Compare two JSON values
    fn compare_json(
        &self,
        actual: &serde_json::Value,
        expected: &serde_json::Value,
    ) -> CompareResult {
        match (actual, expected) {
            (serde_json::Value::Object(actual_obj), serde_json::Value::Object(expected_obj)) => {
                // Compare objects
                let mut is_equal = true;
                let mut details = Vec::new();

                // Check expected fields in actual
                for (key, expected_value) in expected_obj {
                    if let Some(actual_value) = actual_obj.get(key) {
                        let result = self.compare_json(actual_value, expected_value);
                        if !result.is_equal {
                            is_equal = false;
                            details.push(format!(
                                "Field '{}' values differ: {}",
                                key,
                                result.details.unwrap_or_default()
                            ));
                        }
                    } else if !self.options.ignore_missing_fields {
                        is_equal = false;
                        details.push(format!("Field '{}' missing in actual", key));
                    }
                }

                // Check for extra fields in actual
                if !self.options.ignore_extra_fields {
                    for key in actual_obj.keys() {
                        if !expected_obj.contains_key(key) {
                            is_equal = false;
                            details.push(format!("Extra field '{}' in actual", key));
                        }
                    }
                }

                CompareResult::new(is_equal)
                    .with_actual(actual.to_string())
                    .with_expected(expected.to_string())
                    .with_details(details.join(", "))
            }
            (serde_json::Value::Array(actual_arr), serde_json::Value::Array(expected_arr)) => {
                // Compare arrays
                if self.options.ignore_order {
                    // This is a simplified comparison that works for primitive values
                    // For complex objects, a more sophisticated comparison would be needed
                    let mut is_equal = true;
                    let mut details = Vec::new();

                    // Check if all expected items are in actual
                    for expected_item in expected_arr {
                        let found = actual_arr.iter().any(|actual_item| {
                            let result = self.compare_json(actual_item, expected_item);
                            result.is_equal
                        });

                        if !found {
                            is_equal = false;
                            details.push(format!(
                                "Expected item {} not found in actual",
                                expected_item
                            ));
                        }
                    }

                    // Check if actual has extra items
                    if !self.options.ignore_extra_fields && actual_arr.len() > expected_arr.len() {
                        is_equal = false;
                        details.push(format!(
                            "Actual has {} items, expected {}",
                            actual_arr.len(),
                            expected_arr.len()
                        ));
                    }

                    CompareResult::new(is_equal)
                        .with_actual(actual.to_string())
                        .with_expected(expected.to_string())
                        .with_details(details.join(", "))
                } else {
                    // Compare as ordered sequences
                    if actual_arr.len() != expected_arr.len() {
                        return CompareResult::new(false)
                            .with_actual(actual.to_string())
                            .with_expected(expected.to_string())
                            .with_details(format!(
                                "Array length mismatch: actual {} vs expected {}",
                                actual_arr.len(),
                                expected_arr.len()
                            ));
                    }

                    let mut is_equal = true;
                    let mut details = Vec::new();

                    for (i, (actual_item, expected_item)) in
                        actual_arr.iter().zip(expected_arr.iter()).enumerate()
                    {
                        let result = self.compare_json(actual_item, expected_item);
                        if !result.is_equal {
                            is_equal = false;
                            details.push(format!(
                                "Item at index {} differs: {}",
                                i,
                                result.details.unwrap_or_default()
                            ));
                        }
                    }

                    CompareResult::new(is_equal)
                        .with_actual(actual.to_string())
                        .with_expected(expected.to_string())
                        .with_details(details.join(", "))
                }
            }
            // For primitive values, use direct comparison
            _ => {
                let is_equal = actual == expected;
                CompareResult::new(is_equal)
                    .with_actual(actual.to_string())
                    .with_expected(expected.to_string())
            }
        }
    }
}

/// String comparator
pub struct StringComparator {
    /// Comparison options
    options: CompareOptions,
}

impl StringComparator {
    /// Create a new string comparator
    pub fn new() -> Self {
        Self {
            options: CompareOptions::default(),
        }
    }

    /// Create a new string comparator with options
    pub fn with_options(options: CompareOptions) -> Self {
        Self { options }
    }
}

impl Comparator for StringComparator {
    fn compare(&self, actual: &dyn std::any::Any, expected: &dyn std::any::Any) -> CompareResult {
        if let (Some(actual), Some(expected)) = (
            actual.downcast_ref::<String>(),
            expected.downcast_ref::<String>(),
        ) {
            let mut actual_str = actual.clone();
            let mut expected_str = expected.clone();

            if self.options.ignore_case {
                actual_str = actual_str.to_lowercase();
                expected_str = expected_str.to_lowercase();
            }

            if self.options.ignore_whitespace {
                actual_str = actual_str.split_whitespace().collect::<String>();
                expected_str = expected_str.split_whitespace().collect::<String>();
            }

            let is_equal = actual_str == expected_str;

            CompareResult::new(is_equal)
                .with_actual(actual)
                .with_expected(expected)
        } else {
            CompareResult::new(false)
                .with_actual(format!("{:?}", actual))
                .with_expected(format!("{:?}", expected))
                .with_details("Values are not strings".to_string())
        }
    }
}

/// JSON comparator
pub struct JsonComparator {
    /// Comparison options
    options: CompareOptions,
}

impl JsonComparator {
    /// Create a new JSON comparator
    pub fn new() -> Self {
        Self {
            options: CompareOptions::default(),
        }
    }

    /// Create a new JSON comparator with options
    pub fn with_options(options: CompareOptions) -> Self {
        Self { options }
    }
}

impl Comparator for JsonComparator {
    fn compare(&self, actual: &dyn std::any::Any, expected: &dyn std::any::Any) -> CompareResult {
        if let (Some(actual), Some(expected)) = (
            actual.downcast_ref::<serde_json::Value>(),
            expected.downcast_ref::<serde_json::Value>(),
        ) {
            let comparator = DefaultComparator::with_options(self.options.clone());
            comparator.compare_json(actual, expected)
        } else {
            // Try to convert to JSON
            let actual_json = match serde_json::to_value(actual) {
                Ok(json) => json,
                Err(_) => {
                    return CompareResult::new(false)
                        .with_actual(format!("{:?}", actual))
                        .with_expected(format!("{:?}", expected))
                        .with_details("Failed to convert actual value to JSON".to_string());
                }
            };

            let expected_json = match serde_json::to_value(expected) {
                Ok(json) => json,
                Err(_) => {
                    return CompareResult::new(false)
                        .with_actual(format!("{:?}", actual))
                        .with_expected(format!("{:?}", expected))
                        .with_details("Failed to convert expected value to JSON".to_string());
                }
            };

            let comparator = DefaultComparator::with_options(self.options.clone());
            comparator.compare_json(&actual_json, &expected_json)
        }
    }
}

/// Diff comparator
pub struct DiffComparator {
    /// Comparison options
    options: CompareOptions,
}

impl DiffComparator {
    /// Create a new diff comparator
    pub fn new() -> Self {
        Self {
            options: CompareOptions::default(),
        }
    }

    /// Create a new diff comparator with options
    pub fn with_options(options: CompareOptions) -> Self {
        Self { options }
    }
}

impl Comparator for DiffComparator {
    fn compare(&self, actual: &dyn std::any::Any, expected: &dyn std::any::Any) -> CompareResult {
        if let (Some(actual), Some(expected)) = (
            actual.downcast_ref::<String>(),
            expected.downcast_ref::<String>(),
        ) {
            let mut actual_str = actual.clone();
            let mut expected_str = expected.clone();

            if self.options.ignore_case {
                actual_str = actual_str.to_lowercase();
                expected_str = expected_str.to_lowercase();
            }

            if self.options.ignore_whitespace {
                actual_str = actual_str.split_whitespace().collect::<String>();
                expected_str = expected_str.split_whitespace().collect::<String>();
            }

            let is_equal = actual_str == expected_str;

            if is_equal {
                CompareResult::new(true)
                    .with_actual(actual)
                    .with_expected(expected)
            } else {
                // Generate a diff
                let diff = similar::TextDiff::from_lines(&expected_str, &actual_str);
                let mut diff_output = String::new();

                for change in diff.iter_all_changes() {
                    let sign = match change.tag() {
                        similar::ChangeTag::Delete => "-",
                        similar::ChangeTag::Insert => "+",
                        similar::ChangeTag::Equal => " ",
                    };
                    diff_output.push_str(&format!("{}{}", sign, change));
                }

                CompareResult::new(false)
                    .with_actual(actual)
                    .with_expected(expected)
                    .with_details(diff_output)
            }
        } else {
            // Fall back to default comparator
            let comparator = DefaultComparator::with_options(self.options.clone());
            comparator.compare(actual, expected)
        }
    }
}

/// Structural comparator
pub struct StructuralComparator {
    /// Comparison options
    options: CompareOptions,
}

impl StructuralComparator {
    /// Create a new structural comparator
    pub fn new() -> Self {
        Self {
            options: CompareOptions::default(),
        }
    }

    /// Create a new structural comparator with options
    pub fn with_options(options: CompareOptions) -> Self {
        Self { options }
    }
}

impl Comparator for StructuralComparator {
    fn compare(&self, actual: &dyn std::any::Any, expected: &dyn std::any::Any) -> CompareResult {
        // Try to convert to JSON for structural comparison
        let actual_json = match serde_json::to_value(actual) {
            Ok(json) => json,
            Err(_) => {
                return CompareResult::new(false)
                    .with_actual(format!("{:?}", actual))
                    .with_expected(format!("{:?}", expected))
                    .with_details("Failed to convert actual value to JSON".to_string());
            }
        };

        let expected_json = match serde_json::to_value(expected) {
            Ok(json) => json,
            Err(_) => {
                return CompareResult::new(false)
                    .with_actual(format!("{:?}", actual))
                    .with_expected(format!("{:?}", expected))
                    .with_details("Failed to convert expected value to JSON".to_string());
            }
        };

        let comparator = DefaultComparator::with_options(self.options.clone());
        comparator.compare_json(&actual_json, &expected_json)
    }
}
