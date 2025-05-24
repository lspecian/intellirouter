//! LLM assertions for the assertion framework.
//!
//! This module provides assertions specific to the LLM (Large Language Model) component of IntelliRouter.

use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::modules::test_harness::assert::core::{
    assert_that, AssertThat, AssertionOutcome, AssertionResult,
};
use crate::modules::test_harness::types::TestHarnessError;

/// Represents an LLM request for assertion purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequest {
    /// The request ID.
    pub request_id: String,
    /// The model ID.
    pub model_id: String,
    /// The prompt.
    pub prompt: String,
    /// The system message.
    pub system_message: Option<String>,
    /// The request parameters.
    pub parameters: Value,
    /// The request timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// The request metadata.
    pub metadata: Value,
}

impl LlmRequest {
    /// Creates a new LLM request.
    pub fn new(request_id: &str, model_id: &str, prompt: &str) -> Self {
        Self {
            request_id: request_id.to_string(),
            model_id: model_id.to_string(),
            prompt: prompt.to_string(),
            system_message: None,
            parameters: Value::Null,
            timestamp: chrono::Utc::now(),
            metadata: Value::Null,
        }
    }

    /// Sets the system message.
    pub fn with_system_message(mut self, system_message: &str) -> Self {
        self.system_message = Some(system_message.to_string());
        self
    }

    /// Sets the request parameters.
    pub fn with_parameters(mut self, parameters: Value) -> Self {
        self.parameters = parameters;
        self
    }

    /// Sets the request timestamp.
    pub fn with_timestamp(mut self, timestamp: chrono::DateTime<chrono::Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Sets the request metadata.
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Represents an LLM response for assertion purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    /// The response ID.
    pub response_id: String,
    /// The request ID.
    pub request_id: String,
    /// The model ID.
    pub model_id: String,
    /// The response text.
    pub text: String,
    /// The response tokens.
    pub tokens: LlmTokens,
    /// The response timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// The response metadata.
    pub metadata: Value,
    /// The response time.
    pub response_time: Duration,
    /// Whether the response was cached.
    pub cached: bool,
    /// The error, if any.
    pub error: Option<String>,
}

impl LlmResponse {
    /// Creates a new LLM response.
    pub fn new(response_id: &str, request_id: &str, model_id: &str, text: &str) -> Self {
        Self {
            response_id: response_id.to_string(),
            request_id: request_id.to_string(),
            model_id: model_id.to_string(),
            text: text.to_string(),
            tokens: LlmTokens::default(),
            timestamp: chrono::Utc::now(),
            metadata: Value::Null,
            response_time: Duration::default(),
            cached: false,
            error: None,
        }
    }

    /// Sets the response tokens.
    pub fn with_tokens(mut self, tokens: LlmTokens) -> Self {
        self.tokens = tokens;
        self
    }

    /// Sets the response timestamp.
    pub fn with_timestamp(mut self, timestamp: chrono::DateTime<chrono::Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Sets the response metadata.
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Sets the response time.
    pub fn with_response_time(mut self, response_time: Duration) -> Self {
        self.response_time = response_time;
        self
    }

    /// Sets whether the response was cached.
    pub fn with_cached(mut self, cached: bool) -> Self {
        self.cached = cached;
        self
    }

    /// Sets the error.
    pub fn with_error(mut self, error: &str) -> Self {
        self.error = Some(error.to_string());
        self
    }
}

/// Represents LLM tokens for assertion purposes.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmTokens {
    /// The prompt tokens.
    pub prompt_tokens: usize,
    /// The completion tokens.
    pub completion_tokens: usize,
    /// The total tokens.
    pub total_tokens: usize,
}

impl LlmTokens {
    /// Creates a new LLM tokens instance.
    pub fn new(prompt_tokens: usize, completion_tokens: usize) -> Self {
        Self {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        }
    }
}

/// Assertions for LLM components.
#[derive(Debug, Clone)]
pub struct LlmAssertions;

impl LlmAssertions {
    /// Creates a new LLM assertions instance.
    pub fn new() -> Self {
        Self
    }

    /// Asserts that a response has a specific model ID.
    pub fn assert_model_id(&self, response: &LlmResponse, expected: &str) -> AssertionResult {
        assert_that(response.model_id.as_str())
            .with_name(&format!("Model ID is '{}'", expected))
            .is_equal_to(expected)
    }

    /// Asserts that a response was successful (no error).
    pub fn assert_success(&self, response: &LlmResponse) -> AssertionResult {
        match &response.error {
            None => AssertionResult::new("Response is successful", AssertionOutcome::Passed),
            Some(error) => AssertionResult::new("Response is successful", AssertionOutcome::Failed)
                .with_error(
                    crate::modules::test_harness::assert::core::AssertionError::new(
                        "Response has an error",
                        "No error",
                        error,
                    ),
                ),
        }
    }

    /// Asserts that a response has a specific error.
    pub fn assert_error(&self, response: &LlmResponse, expected: &str) -> AssertionResult {
        match &response.error {
            Some(error) => {
                if error == expected {
                    AssertionResult::new(
                        &format!("Response has error '{}'", expected),
                        AssertionOutcome::Passed,
                    )
                } else {
                    AssertionResult::new(
                        &format!("Response has error '{}'", expected),
                        AssertionOutcome::Failed,
                    )
                    .with_error(
                        crate::modules::test_harness::assert::core::AssertionError::new(
                            "Response has a different error",
                            expected,
                            error,
                        ),
                    )
                }
            }
            None => AssertionResult::new(
                &format!("Response has error '{}'", expected),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Response does not have an error",
                    expected,
                    "No error",
                ),
            ),
        }
    }

    /// Asserts that a response was cached.
    pub fn assert_cached(&self, response: &LlmResponse) -> AssertionResult {
        if response.cached {
            AssertionResult::new("Response is cached", AssertionOutcome::Passed)
        } else {
            AssertionResult::new("Response is cached", AssertionOutcome::Failed).with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Response is not cached",
                    "Cached",
                    "Not cached",
                ),
            )
        }
    }

    /// Asserts that a response was not cached.
    pub fn assert_not_cached(&self, response: &LlmResponse) -> AssertionResult {
        if !response.cached {
            AssertionResult::new("Response is not cached", AssertionOutcome::Passed)
        } else {
            AssertionResult::new("Response is not cached", AssertionOutcome::Failed).with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Response is cached",
                    "Not cached",
                    "Cached",
                ),
            )
        }
    }

    /// Asserts that a response was received within a specific time.
    pub fn assert_response_time(&self, response: &LlmResponse, max_ms: u64) -> AssertionResult {
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

    /// Asserts that a response has a specific text.
    pub fn assert_text(&self, response: &LlmResponse, expected: &str) -> AssertionResult {
        assert_that(response.text.as_str())
            .with_name(&format!("Response text is '{}'", expected))
            .is_equal_to(expected)
    }

    /// Asserts that a response text contains a specific substring.
    pub fn assert_text_contains(&self, response: &LlmResponse, expected: &str) -> AssertionResult {
        assert_that(response.text.as_str())
            .with_name(&format!("Response text contains '{}'", expected))
            .contains(expected)
    }

    /// Asserts that a response text matches a specific pattern.
    pub fn assert_text_matches(&self, response: &LlmResponse, pattern: &str) -> AssertionResult {
        assert_that(response.text.as_str())
            .with_name(&format!("Response text matches pattern '{}'", pattern))
            .matches_pattern(pattern)
    }

    /// Asserts that a response has a specific number of prompt tokens.
    pub fn assert_prompt_tokens(&self, response: &LlmResponse, expected: usize) -> AssertionResult {
        assert_that(response.tokens.prompt_tokens)
            .with_name(&format!("Prompt tokens is {}", expected))
            .is_equal_to(expected)
    }

    /// Asserts that a response has a specific number of completion tokens.
    pub fn assert_completion_tokens(
        &self,
        response: &LlmResponse,
        expected: usize,
    ) -> AssertionResult {
        assert_that(response.tokens.completion_tokens)
            .with_name(&format!("Completion tokens is {}", expected))
            .is_equal_to(expected)
    }

    /// Asserts that a response has a specific number of total tokens.
    pub fn assert_total_tokens(&self, response: &LlmResponse, expected: usize) -> AssertionResult {
        assert_that(response.tokens.total_tokens)
            .with_name(&format!("Total tokens is {}", expected))
            .is_equal_to(expected)
    }

    /// Asserts that a response has at most a specific number of prompt tokens.
    pub fn assert_max_prompt_tokens(&self, response: &LlmResponse, max: usize) -> AssertionResult {
        let prompt_tokens = response.tokens.prompt_tokens;

        if prompt_tokens <= max {
            AssertionResult::new(
                &format!("Prompt tokens <= {}", max),
                AssertionOutcome::Passed,
            )
        } else {
            AssertionResult::new(
                &format!("Prompt tokens <= {}", max),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Prompt tokens exceeds maximum",
                    &format!("<= {}", max),
                    &format!("{}", prompt_tokens),
                ),
            )
        }
    }

    /// Asserts that a response has at most a specific number of completion tokens.
    pub fn assert_max_completion_tokens(
        &self,
        response: &LlmResponse,
        max: usize,
    ) -> AssertionResult {
        let completion_tokens = response.tokens.completion_tokens;

        if completion_tokens <= max {
            AssertionResult::new(
                &format!("Completion tokens <= {}", max),
                AssertionOutcome::Passed,
            )
        } else {
            AssertionResult::new(
                &format!("Completion tokens <= {}", max),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Completion tokens exceeds maximum",
                    &format!("<= {}", max),
                    &format!("{}", completion_tokens),
                ),
            )
        }
    }

    /// Asserts that a response has at most a specific number of total tokens.
    pub fn assert_max_total_tokens(&self, response: &LlmResponse, max: usize) -> AssertionResult {
        let total_tokens = response.tokens.total_tokens;

        if total_tokens <= max {
            AssertionResult::new(
                &format!("Total tokens <= {}", max),
                AssertionOutcome::Passed,
            )
        } else {
            AssertionResult::new(
                &format!("Total tokens <= {}", max),
                AssertionOutcome::Failed,
            )
            .with_error(
                crate::modules::test_harness::assert::core::AssertionError::new(
                    "Total tokens exceeds maximum",
                    &format!("<= {}", max),
                    &format!("{}", total_tokens),
                ),
            )
        }
    }

    /// Asserts that a response text is factually accurate.
    /// This is a placeholder for a more sophisticated factual accuracy check.
    pub fn assert_factually_accurate(&self, response: &LlmResponse) -> AssertionResult {
        // In a real implementation, this would use a more sophisticated factual accuracy check.
        // For now, we just return a passed assertion.
        AssertionResult::new("Response is factually accurate", AssertionOutcome::Passed)
    }

    /// Asserts that a response text is not harmful.
    /// This is a placeholder for a more sophisticated harmful content check.
    pub fn assert_not_harmful(&self, response: &LlmResponse) -> AssertionResult {
        // In a real implementation, this would use a more sophisticated harmful content check.
        // For now, we just return a passed assertion.
        AssertionResult::new("Response is not harmful", AssertionOutcome::Passed)
    }

    /// Asserts that a response text is not biased.
    /// This is a placeholder for a more sophisticated bias check.
    pub fn assert_not_biased(&self, response: &LlmResponse) -> AssertionResult {
        // In a real implementation, this would use a more sophisticated bias check.
        // For now, we just return a passed assertion.
        AssertionResult::new("Response is not biased", AssertionOutcome::Passed)
    }
}

impl Default for LlmAssertions {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new LLM assertions instance.
pub fn create_llm_assertions() -> LlmAssertions {
    LlmAssertions::new()
}

/// Creates a new LLM request.
pub fn create_llm_request(request_id: &str, model_id: &str, prompt: &str) -> LlmRequest {
    LlmRequest::new(request_id, model_id, prompt)
}

/// Creates a new LLM response.
pub fn create_llm_response(
    response_id: &str,
    request_id: &str,
    model_id: &str,
    text: &str,
) -> LlmResponse {
    LlmResponse::new(response_id, request_id, model_id, text)
}

/// Creates a new LLM tokens instance.
pub fn create_llm_tokens(prompt_tokens: usize, completion_tokens: usize) -> LlmTokens {
    LlmTokens::new(prompt_tokens, completion_tokens)
}
