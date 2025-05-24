//! HTTP assertions for the assertion framework.
//!
//! This module provides assertions for HTTP requests and responses.

use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use http::{HeaderMap, Method, StatusCode, Uri, Version};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::modules::test_harness::assert::core::{
    assert_that, AssertThat, AssertionOutcome, AssertionResult,
};
use crate::modules::test_harness::assert::matchers::{
    ContainsMatcher, EqualsMatcher, HeaderMatcher, JsonMatcher, JsonSchemaMatcher, Matcher,
    RegexMatcher, ResponseTimeMatcher, StatusCodeMatcher,
};
use crate::modules::test_harness::types::TestHarnessError;

/// Represents an HTTP request for assertion purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    /// The HTTP method.
    pub method: Method,
    /// The request URI.
    pub uri: Uri,
    /// The HTTP version.
    pub version: Version,
    /// The request headers.
    pub headers: HeaderMap,
    /// The request body.
    pub body: Option<Value>,
}

impl HttpRequest {
    /// Creates a new HTTP request.
    pub fn new(method: Method, uri: Uri) -> Self {
        Self {
            method,
            uri,
            version: Version::HTTP_11,
            headers: HeaderMap::new(),
            body: None,
        }
    }

    /// Sets the HTTP version.
    pub fn with_version(mut self, version: Version) -> Self {
        self.version = version;
        self
    }

    /// Adds a header to the request.
    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        if let Ok(name) = name.parse() {
            if let Ok(value) = value.parse() {
                self.headers.insert(name, value);
            }
        }
        self
    }

    /// Sets the request body.
    pub fn with_body(mut self, body: Value) -> Self {
        self.body = Some(body);
        self
    }
}

/// Represents an HTTP response for assertion purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    /// The HTTP status code.
    pub status: StatusCode,
    /// The HTTP version.
    pub version: Version,
    /// The response headers.
    pub headers: HeaderMap,
    /// The response body.
    pub body: Option<Value>,
    /// The response time.
    pub response_time: Duration,
}

impl HttpResponse {
    /// Creates a new HTTP response.
    pub fn new(status: StatusCode) -> Self {
        Self {
            status,
            version: Version::HTTP_11,
            headers: HeaderMap::new(),
            body: None,
            response_time: Duration::default(),
        }
    }

    /// Sets the HTTP version.
    pub fn with_version(mut self, version: Version) -> Self {
        self.version = version;
        self
    }

    /// Adds a header to the response.
    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        if let Ok(name) = name.parse() {
            if let Ok(value) = value.parse() {
                self.headers.insert(name, value);
            }
        }
        self
    }

    /// Sets the response body.
    pub fn with_body(mut self, body: Value) -> Self {
        self.body = Some(body);
        self
    }

    /// Sets the response time.
    pub fn with_response_time(mut self, response_time: Duration) -> Self {
        self.response_time = response_time;
        self
    }
}

/// Assertions for HTTP requests and responses.
#[derive(Debug, Clone)]
pub struct HttpAssertions;

impl HttpAssertions {
    /// Creates a new HTTP assertions instance.
    pub fn new() -> Self {
        Self
    }

    /// Asserts that a response has a specific status code.
    pub fn assert_status_code(
        &self,
        response: &HttpResponse,
        expected: StatusCode,
    ) -> AssertionResult {
        assert_that(response.status)
            .with_name(&format!("Status code is {}", expected))
            .is_equal_to(expected)
    }

    /// Asserts that a response has a status code in the 2xx range.
    pub fn assert_success(&self, response: &HttpResponse) -> AssertionResult {
        let status = response.status;
        let is_success = status.is_success();

        if is_success {
            AssertionResult::new("Status code is success (2xx)", AssertionOutcome::Passed)
        } else {
            AssertionResult::new("Status code is success (2xx)", AssertionOutcome::Failed)
                .with_error(
                    crate::modules::test_harness::assert::core::AssertionError::new(
                        "Status code is not in the 2xx range",
                        "2xx status code",
                        status.as_u16(),
                    ),
                )
        }
    }

    /// Asserts that a response has a status code in the 4xx range.
    pub fn assert_client_error(&self, response: &HttpResponse) -> AssertionResult {
        let status = response.status;
        let is_client_error = status.is_client_error();

        if is_client_error {
            AssertionResult::new(
                "Status code is client error (4xx)",
                AssertionOutcome::Passed,
            )
        } else {
            AssertionResult::new(
                "Status code is client error (4xx)",
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Status code is not in the 4xx range",
                    "4xx status code",
                    status.as_u16(),
                ),
            )
        }
    }

    /// Asserts that a response has a status code in the 5xx range.
    pub fn assert_server_error(&self, response: &HttpResponse) -> AssertionResult {
        let status = response.status;
        let is_server_error = status.is_server_error();

        if is_server_error {
            AssertionResult::new(
                "Status code is server error (5xx)",
                AssertionOutcome::Passed,
            )
        } else {
            AssertionResult::new(
                "Status code is server error (5xx)",
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Status code is not in the 5xx range",
                    "5xx status code",
                    status.as_u16(),
                ),
            )
        }
    }

    /// Asserts that a response has a specific header.
    pub fn assert_has_header(&self, response: &HttpResponse, name: &str) -> AssertionResult {
        let has_header = response.headers.contains_key(name);

        if has_header {
            AssertionResult::new(&format!("Has header '{}'", name), AssertionOutcome::Passed)
        } else {
            AssertionResult::new(&format!("Has header '{}'", name), AssertionOutcome::Failed)
                .with_error(
                    crate::modules::test_harness::assert::core::AssertionError::new(
                        &format!("Response does not have header '{}'", name),
                        &format!("Header '{}'", name),
                        "No such header",
                    ),
                )
        }
    }

    /// Asserts that a response has a header with a specific value.
    pub fn assert_header_value(
        &self,
        response: &HttpResponse,
        name: &str,
        expected: &str,
    ) -> AssertionResult {
        let header_value = response.headers.get(name).and_then(|v| v.to_str().ok());

        match header_value {
            Some(value) => {
                if value == expected {
                    AssertionResult::new(
                        &format!("Header '{}' has value '{}'", name, expected),
                        AssertionOutcome::Passed,
                    )
                } else {
                    AssertionResult::new(
                        &format!("Header '{}' has value '{}'", name, expected),
                        AssertionOutcome::Failed,
                    )
                    .with_error(
                        crate::modules::test_harness::assert::core::AssertionError::new(
                            &format!("Header '{}' has unexpected value", name),
                            expected,
                            value,
                        ),
                    )
                }
            }
            None => AssertionResult::new(
                &format!("Header '{}' has value '{}'", name, expected),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    &format!("Response does not have header '{}'", name),
                    &format!("Header '{}' with value '{}'", name, expected),
                    "No such header",
                ),
            ),
        }
    }

    /// Asserts that a response has a header that matches a pattern.
    pub fn assert_header_matches(
        &self,
        response: &HttpResponse,
        name: &str,
        pattern: &str,
    ) -> AssertionResult {
        let header_value = response.headers.get(name).and_then(|v| v.to_str().ok());

        match header_value {
            Some(value) => match regex::Regex::new(pattern) {
                Ok(regex) => {
                    if regex.is_match(value) {
                        AssertionResult::new(
                            &format!("Header '{}' matches pattern '{}'", name, pattern),
                            AssertionOutcome::Passed,
                        )
                    } else {
                        AssertionResult::new(
                            &format!("Header '{}' matches pattern '{}'", name, pattern),
                            AssertionOutcome::Failed,
                        )
                        .with_error(
                            crate::modules::test_harness::assert::core::AssertionError::new(
                                &format!("Header '{}' does not match pattern", name),
                                pattern,
                                value,
                            ),
                        )
                    }
                }
                Err(e) => AssertionResult::new(
                    &format!("Header '{}' matches pattern '{}'", name, pattern),
                    AssertionOutcome::Failed,
                )
                .with_error(
                    crate::modules::test_harness::assert::core::AssertionError::new(
                        &format!("Invalid regex pattern: {}", e),
                        pattern,
                        value,
                    ),
                ),
            },
            None => AssertionResult::new(
                &format!("Header '{}' matches pattern '{}'", name, pattern),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    &format!("Response does not have header '{}'", name),
                    &format!("Header '{}' matching pattern '{}'", name, pattern),
                    "No such header",
                ),
            ),
        }
    }

    /// Asserts that a response has a JSON body.
    pub fn assert_json_body(&self, response: &HttpResponse) -> AssertionResult {
        match &response.body {
            Some(body) => AssertionResult::new("Response has JSON body", AssertionOutcome::Passed),
            None => AssertionResult::new("Response has JSON body", AssertionOutcome::Failed)
                .with_error(
                    crate::modules::test_harness::assert::core::AssertionError::new(
                        "Response does not have a body",
                        "JSON body",
                        "No body",
                    ),
                ),
        }
    }

    /// Asserts that a response has a JSON body that matches a specific value.
    pub fn assert_json_body_equals(
        &self,
        response: &HttpResponse,
        expected: Value,
    ) -> AssertionResult {
        match &response.body {
            Some(body) => {
                if body == &expected {
                    AssertionResult::new(
                        "JSON body equals expected value",
                        AssertionOutcome::Passed,
                    )
                } else {
                    AssertionResult::new(
                        "JSON body equals expected value",
                        AssertionOutcome::Failed,
                    )
                    .with_error(
                        crate::modules::test_harness::assert::core::AssertionError::new(
                            "JSON body does not equal expected value",
                            &format!("{}", expected),
                            &format!("{}", body),
                        ),
                    )
                }
            }
            None => {
                AssertionResult::new("JSON body equals expected value", AssertionOutcome::Failed)
                    .with_error(
                        crate::modules::test_harness::assert::core::AssertionError::new(
                            "Response does not have a body",
                            &format!("{}", expected),
                            "No body",
                        ),
                    )
            }
        }
    }

    /// Asserts that a response has a JSON body that contains a specific field.
    pub fn assert_json_body_contains_field(
        &self,
        response: &HttpResponse,
        field: &str,
    ) -> AssertionResult {
        match &response.body {
            Some(body) => {
                let has_field = match body {
                    Value::Object(obj) => obj.contains_key(field),
                    _ => false,
                };

                if has_field {
                    AssertionResult::new(
                        &format!("JSON body contains field '{}'", field),
                        AssertionOutcome::Passed,
                    )
                } else {
                    AssertionResult::new(
                        &format!("JSON body contains field '{}'", field),
                        AssertionOutcome::Failed,
                    )
                    .with_error(
                        crate::modules::test_harness::assert::core::AssertionError::new(
                            &format!("JSON body does not contain field '{}'", field),
                            &format!("Field '{}'", field),
                            &format!("{}", body),
                        ),
                    )
                }
            }
            None => AssertionResult::new(
                &format!("JSON body contains field '{}'", field),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Response does not have a body",
                    &format!("Field '{}'", field),
                    "No body",
                ),
            ),
        }
    }

    /// Asserts that a response has a JSON body that contains a field with a specific value.
    pub fn assert_json_body_field_equals(
        &self,
        response: &HttpResponse,
        field: &str,
        expected: Value,
    ) -> AssertionResult {
        match &response.body {
            Some(body) => {
                let field_value = match body {
                    Value::Object(obj) => obj.get(field).cloned(),
                    _ => None,
                };

                match field_value {
                    Some(value) => {
                        if value == expected {
                            AssertionResult::new(
                                &format!("JSON body field '{}' equals expected value", field),
                                AssertionOutcome::Passed,
                            )
                        } else {
                            AssertionResult::new(
                                &format!("JSON body field '{}' equals expected value", field),
                                AssertionOutcome::Failed,
                            )
                            .with_error(
                                crate::modules::test_harness::assert::core::AssertionError::new(
                                    &format!(
                                        "JSON body field '{}' does not equal expected value",
                                        field
                                    ),
                                    &format!("{}", expected),
                                    &format!("{}", value),
                                ),
                            )
                        }
                    }
                    None => AssertionResult::new(
                        &format!("JSON body field '{}' equals expected value", field),
                        AssertionOutcome::Failed,
                    )
                    .with_error(
                        crate::modules::test_harness::assert::core::AssertionError::new(
                            &format!("JSON body does not contain field '{}'", field),
                            &format!("Field '{}' with value '{}'", field, expected),
                            &format!("{}", body),
                        ),
                    ),
                }
            }
            None => AssertionResult::new(
                &format!("JSON body field '{}' equals expected value", field),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Response does not have a body",
                    &format!("Field '{}' with value '{}'", field, expected),
                    "No body",
                ),
            ),
        }
    }

    /// Asserts that a response has a JSON body that matches a JSON schema.
    pub fn assert_json_body_matches_schema(
        &self,
        response: &HttpResponse,
        schema: Value,
    ) -> AssertionResult {
        match &response.body {
            Some(body) => match jsonschema::JSONSchema::compile(&schema) {
                Ok(compiled_schema) => match compiled_schema.validate(body) {
                    Ok(_) => {
                        AssertionResult::new("JSON body matches schema", AssertionOutcome::Passed)
                    }
                    Err(errors) => {
                        if errors.is_empty() {
                            AssertionResult::new(
                                "JSON body matches schema",
                                AssertionOutcome::Passed,
                            )
                        } else {
                            let error_messages: Vec<String> =
                                errors.iter().map(|e| format!("{}", e)).collect();
                            AssertionResult::new(
                                "JSON body matches schema",
                                AssertionOutcome::Failed,
                            )
                            .with_error(
                                crate::modules::test_harness::assert::core::AssertionError::new(
                                    &format!(
                                        "JSON body does not match schema: {}",
                                        error_messages.join(", ")
                                    ),
                                    &format!("{}", schema),
                                    &format!("{}", body),
                                ),
                            )
                        }
                    }
                },
                Err(e) => {
                    AssertionResult::new("JSON body matches schema", AssertionOutcome::Failed)
                        .with_error(
                            crate::modules::test_harness::assert::core::AssertionError::new(
                                &format!("Invalid JSON schema: {}", e),
                                &format!("{}", schema),
                                &format!("{}", body),
                            ),
                        )
                }
            },
            None => AssertionResult::new("JSON body matches schema", AssertionOutcome::Failed)
                .with_error(
                    crate::modules::test_harness::assert::core::AssertionError::new(
                        "Response does not have a body",
                        &format!("{}", schema),
                        "No body",
                    ),
                ),
        }
    }

    /// Asserts that a response was received within a specific time.
    pub fn assert_response_time(&self, response: &HttpResponse, max_ms: u64) -> AssertionResult {
        let response_time_ms = response.response_time.as_millis() as u64;

        if response_time_ms <= max_ms {
            AssertionResult::new(
                &format!("Response time <= {} ms", max_ms),
                AssertionOutcome::Passed,
            )
        } else {
            AssertionResult::new(
                &format!("Response time <= {} ms", max_ms),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Response time exceeds maximum",
                    &format!("<= {} ms", max_ms),
                    &format!("{} ms", response_time_ms),
                ),
            )
        }
    }
}

impl Default for HttpAssertions {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new HTTP assertions instance.
pub fn create_http_assertions() -> HttpAssertions {
    HttpAssertions::new()
}

/// Creates a new HTTP request.
pub fn create_http_request(method: Method, uri: Uri) -> HttpRequest {
    HttpRequest::new(method, uri)
}

/// Creates a new HTTP response.
pub fn create_http_response(status: StatusCode) -> HttpResponse {
    HttpResponse::new(status)
}
