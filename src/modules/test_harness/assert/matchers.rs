//! Matchers for the assertion framework.
//!
//! This module provides a set of matchers that can be used with the assertion framework.
//! Matchers are used to check if a value matches a specific pattern or condition.

use std::fmt::Debug;
use std::marker::PhantomData;

use regex::Regex;
use serde_json::Value;

use crate::modules::test_harness::types::TestHarnessError;

/// Trait for types that can match values.
pub trait Matcher<T> {
    /// Checks if the value matches the pattern.
    fn matches(&self, value: &T) -> Result<bool, TestHarnessError>;

    /// Returns a description of the matcher.
    fn description(&self) -> String;
}

/// A matcher that checks if a value is equal to another value.
#[derive(Debug, Clone)]
pub struct EqualsMatcher<T> {
    /// The expected value.
    expected: T,
}

impl<T> EqualsMatcher<T> {
    /// Creates a new equals matcher.
    pub fn new(expected: T) -> Self {
        Self { expected }
    }
}

impl<T: Debug + PartialEq> Matcher<T> for EqualsMatcher<T> {
    fn matches(&self, value: &T) -> Result<bool, TestHarnessError> {
        Ok(value == &self.expected)
    }

    fn description(&self) -> String {
        format!("equals {:?}", self.expected)
    }
}

/// A matcher that checks if a string contains a substring.
#[derive(Debug, Clone)]
pub struct ContainsMatcher<T> {
    /// The expected substring.
    expected: T,
}

impl<T> ContainsMatcher<T> {
    /// Creates a new contains matcher.
    pub fn new(expected: T) -> Self {
        Self { expected }
    }
}

impl Matcher<&str> for ContainsMatcher<&str> {
    fn matches(&self, value: &&str) -> Result<bool, TestHarnessError> {
        Ok(value.contains(self.expected))
    }

    fn description(&self) -> String {
        format!("contains '{}'", self.expected)
    }
}

impl Matcher<String> for ContainsMatcher<&str> {
    fn matches(&self, value: &String) -> Result<bool, TestHarnessError> {
        Ok(value.contains(self.expected))
    }

    fn description(&self) -> String {
        format!("contains '{}'", self.expected)
    }
}

/// A matcher that checks if a string matches a regex pattern.
#[derive(Debug, Clone)]
pub struct RegexMatcher {
    /// The regex pattern.
    pattern: String,
    /// The compiled regex.
    regex: Regex,
}

impl RegexMatcher {
    /// Creates a new regex matcher.
    pub fn new(pattern: &str) -> Result<Self, TestHarnessError> {
        match Regex::new(pattern) {
            Ok(regex) => Ok(Self {
                pattern: pattern.to_string(),
                regex,
            }),
            Err(e) => Err(TestHarnessError::ValidationError(format!(
                "Invalid regex pattern: {}",
                e
            ))),
        }
    }
}

impl Matcher<&str> for RegexMatcher {
    fn matches(&self, value: &&str) -> Result<bool, TestHarnessError> {
        Ok(self.regex.is_match(value))
    }

    fn description(&self) -> String {
        format!("matches pattern '{}'", self.pattern)
    }
}

impl Matcher<String> for RegexMatcher {
    fn matches(&self, value: &String) -> Result<bool, TestHarnessError> {
        Ok(self.regex.is_match(value))
    }

    fn description(&self) -> String {
        format!("matches pattern '{}'", self.pattern)
    }
}

/// A matcher that checks if a value is of a specific type.
#[derive(Debug, Clone)]
pub struct TypeMatcher {
    /// The expected type.
    expected_type: String,
}

impl TypeMatcher {
    /// Creates a new type matcher.
    pub fn new(expected_type: &str) -> Self {
        Self {
            expected_type: expected_type.to_string(),
        }
    }
}

impl<T: Debug> Matcher<T> for TypeMatcher {
    fn matches(&self, value: &T) -> Result<bool, TestHarnessError> {
        let type_name = std::any::type_name::<T>();
        let expected_type = self.expected_type.as_str();

        // Check if the type name contains the expected type
        // This is a simple check, but it works for most cases
        Ok(match expected_type {
            "string" => type_name.contains("str") || type_name.contains("String"),
            "number" => {
                type_name.contains("i8")
                    || type_name.contains("i16")
                    || type_name.contains("i32")
                    || type_name.contains("i64")
                    || type_name.contains("u8")
                    || type_name.contains("u16")
                    || type_name.contains("u32")
                    || type_name.contains("u64")
                    || type_name.contains("f32")
                    || type_name.contains("f64")
            }
            "integer" => {
                type_name.contains("i8")
                    || type_name.contains("i16")
                    || type_name.contains("i32")
                    || type_name.contains("i64")
                    || type_name.contains("u8")
                    || type_name.contains("u16")
                    || type_name.contains("u32")
                    || type_name.contains("u64")
            }
            "float" => type_name.contains("f32") || type_name.contains("f64"),
            "boolean" => type_name.contains("bool"),
            "array" => {
                type_name.contains("Vec")
                    || type_name.contains("Array")
                    || type_name.contains("Slice")
            }
            "object" => {
                type_name.contains("Map")
                    || type_name.contains("HashMap")
                    || type_name.contains("BTreeMap")
            }
            _ => type_name.contains(expected_type),
        })
    }

    fn description(&self) -> String {
        format!("is of type '{}'", self.expected_type)
    }
}

/// A matcher that checks if a JSON value matches a specific pattern.
#[derive(Debug, Clone)]
pub struct JsonMatcher {
    /// The expected JSON value.
    expected: Value,
}

impl JsonMatcher {
    /// Creates a new JSON matcher.
    pub fn new(expected: Value) -> Self {
        Self { expected }
    }
}

impl Matcher<Value> for JsonMatcher {
    fn matches(&self, value: &Value) -> Result<bool, TestHarnessError> {
        Ok(value == &self.expected)
    }

    fn description(&self) -> String {
        format!("equals JSON {}", self.expected)
    }
}

/// A matcher that checks if a JSON value matches a JSON schema.
#[derive(Debug, Clone)]
pub struct JsonSchemaMatcher {
    /// The JSON schema.
    schema: Value,
    /// The compiled schema.
    #[allow(dead_code)]
    compiled_schema: jsonschema::JSONSchema,
}

impl JsonSchemaMatcher {
    /// Creates a new JSON schema matcher.
    pub fn new(schema: Value) -> Result<Self, TestHarnessError> {
        match jsonschema::JSONSchema::compile(&schema) {
            Ok(compiled_schema) => Ok(Self {
                schema,
                compiled_schema,
            }),
            Err(e) => Err(TestHarnessError::ValidationError(format!(
                "Invalid JSON schema: {}",
                e
            ))),
        }
    }
}

impl Matcher<Value> for JsonSchemaMatcher {
    fn matches(&self, value: &Value) -> Result<bool, TestHarnessError> {
        match self.compiled_schema.validate(value) {
            Ok(_) => Ok(true),
            Err(errors) => {
                if errors.is_empty() {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }

    fn description(&self) -> String {
        format!("matches JSON schema {}", self.schema)
    }
}

/// A matcher that checks if an XML document matches a specific pattern.
#[derive(Debug, Clone)]
pub struct XmlMatcher {
    /// The expected XML document.
    expected: String,
}

impl XmlMatcher {
    /// Creates a new XML matcher.
    pub fn new(expected: &str) -> Self {
        Self {
            expected: expected.to_string(),
        }
    }
}

impl Matcher<&str> for XmlMatcher {
    fn matches(&self, value: &&str) -> Result<bool, TestHarnessError> {
        // Parse the XML documents
        let expected_doc = match roxmltree::Document::parse(&self.expected) {
            Ok(doc) => doc,
            Err(e) => {
                return Err(TestHarnessError::ValidationError(format!(
                    "Invalid expected XML: {}",
                    e
                )))
            }
        };

        let actual_doc = match roxmltree::Document::parse(value) {
            Ok(doc) => doc,
            Err(e) => {
                return Err(TestHarnessError::ValidationError(format!(
                    "Invalid actual XML: {}",
                    e
                )))
            }
        };

        // Compare the root elements
        let expected_root = expected_doc.root_element();
        let actual_root = actual_doc.root_element();

        // Compare the elements recursively
        Ok(compare_xml_elements(expected_root, actual_root))
    }

    fn description(&self) -> String {
        format!("equals XML {}", self.expected)
    }
}

/// Compares two XML elements recursively.
fn compare_xml_elements(expected: roxmltree::Node, actual: roxmltree::Node) -> bool {
    // Compare tag names
    if expected.tag_name().name() != actual.tag_name().name() {
        return false;
    }

    // Compare attributes
    let expected_attrs: Vec<_> = expected.attributes().collect();
    let actual_attrs: Vec<_> = actual.attributes().collect();

    if expected_attrs.len() != actual_attrs.len() {
        return false;
    }

    for expected_attr in expected_attrs {
        let matching_attr = actual_attrs
            .iter()
            .find(|attr| attr.name() == expected_attr.name());
        match matching_attr {
            Some(attr) => {
                if attr.value() != expected_attr.value() {
                    return false;
                }
            }
            None => return false,
        }
    }

    // Compare children
    let expected_children: Vec<_> = expected.children().filter(|n| n.is_element()).collect();
    let actual_children: Vec<_> = actual.children().filter(|n| n.is_element()).collect();

    if expected_children.len() != actual_children.len() {
        return false;
    }

    for (expected_child, actual_child) in expected_children.iter().zip(actual_children.iter()) {
        if !compare_xml_elements(*expected_child, *actual_child) {
            return false;
        }
    }

    true
}

/// A matcher that checks if HTTP headers contain a specific header.
#[derive(Debug, Clone)]
pub struct HeaderMatcher {
    /// The expected header name.
    name: String,
    /// The expected header value.
    value: Option<String>,
}

impl HeaderMatcher {
    /// Creates a new header matcher.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            value: None,
        }
    }

    /// Sets the expected header value.
    pub fn with_value(mut self, value: &str) -> Self {
        self.value = Some(value.to_string());
        self
    }
}

impl<T: AsRef<http::HeaderMap>> Matcher<T> for HeaderMatcher {
    fn matches(&self, value: &T) -> Result<bool, TestHarnessError> {
        let headers = value.as_ref();

        if let Some(header_value) = headers.get(&self.name) {
            if let Some(expected_value) = &self.value {
                if let Ok(header_str) = header_value.to_str() {
                    Ok(header_str == expected_value)
                } else {
                    Ok(false)
                }
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }

    fn description(&self) -> String {
        if let Some(value) = &self.value {
            format!("has header '{}' with value '{}'", self.name, value)
        } else {
            format!("has header '{}'", self.name)
        }
    }
}

/// A matcher that checks if an HTTP status code matches a specific value.
#[derive(Debug, Clone)]
pub struct StatusCodeMatcher {
    /// The expected status code.
    expected: http::StatusCode,
}

impl StatusCodeMatcher {
    /// Creates a new status code matcher.
    pub fn new(expected: http::StatusCode) -> Self {
        Self { expected }
    }
}

impl Matcher<http::StatusCode> for StatusCodeMatcher {
    fn matches(&self, value: &http::StatusCode) -> Result<bool, TestHarnessError> {
        Ok(value == &self.expected)
    }

    fn description(&self) -> String {
        format!("has status code {}", self.expected)
    }
}

/// A matcher that checks if a response time is within a specific range.
#[derive(Debug, Clone)]
pub struct ResponseTimeMatcher {
    /// The maximum allowed response time in milliseconds.
    max_ms: u64,
}

impl ResponseTimeMatcher {
    /// Creates a new response time matcher.
    pub fn new(max_ms: u64) -> Self {
        Self { max_ms }
    }
}

impl Matcher<std::time::Duration> for ResponseTimeMatcher {
    fn matches(&self, value: &std::time::Duration) -> Result<bool, TestHarnessError> {
        let ms = value.as_millis() as u64;
        Ok(ms <= self.max_ms)
    }

    fn description(&self) -> String {
        format!("response time <= {} ms", self.max_ms)
    }
}

/// A matcher that checks if a latency value is within a specific range.
#[derive(Debug, Clone)]
pub struct LatencyMatcher {
    /// The maximum allowed latency in milliseconds.
    max_ms: f64,
}

impl LatencyMatcher {
    /// Creates a new latency matcher.
    pub fn new(max_ms: f64) -> Self {
        Self { max_ms }
    }
}

impl Matcher<f64> for LatencyMatcher {
    fn matches(&self, value: &f64) -> Result<bool, TestHarnessError> {
        Ok(*value <= self.max_ms)
    }

    fn description(&self) -> String {
        format!("latency <= {} ms", self.max_ms)
    }
}

/// A matcher that checks if a throughput value is within a specific range.
#[derive(Debug, Clone)]
pub struct ThroughputMatcher {
    /// The minimum required throughput in requests per second.
    min_rps: f64,
}

impl ThroughputMatcher {
    /// Creates a new throughput matcher.
    pub fn new(min_rps: f64) -> Self {
        Self { min_rps }
    }
}

impl Matcher<f64> for ThroughputMatcher {
    fn matches(&self, value: &f64) -> Result<bool, TestHarnessError> {
        Ok(*value >= self.min_rps)
    }

    fn description(&self) -> String {
        format!("throughput >= {} rps", self.min_rps)
    }
}

/// A matcher that checks if an error rate is within a specific range.
#[derive(Debug, Clone)]
pub struct ErrorRateMatcher {
    /// The maximum allowed error rate as a percentage.
    max_percent: f64,
}

impl ErrorRateMatcher {
    /// Creates a new error rate matcher.
    pub fn new(max_percent: f64) -> Self {
        Self { max_percent }
    }
}

impl Matcher<f64> for ErrorRateMatcher {
    fn matches(&self, value: &f64) -> Result<bool, TestHarnessError> {
        Ok(*value <= self.max_percent)
    }

    fn description(&self) -> String {
        format!("error rate <= {}%", self.max_percent)
    }
}
