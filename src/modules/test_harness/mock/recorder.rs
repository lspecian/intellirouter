//! Mock Recorder Module
//!
//! This module provides functionality for recording and verifying interactions with mocks.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::modules::test_harness::types::TestHarnessError;

/// Interaction with a mock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    /// Interaction ID
    pub id: String,
    /// Interaction timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Request data
    pub request: serde_json::Value,
    /// Response data
    pub response: Option<serde_json::Value>,
}

impl Interaction {
    /// Create a new interaction
    pub fn new(id: impl Into<String>, request: serde_json::Value) -> Self {
        Self {
            id: id.into(),
            timestamp: chrono::Utc::now(),
            request,
            response: None,
        }
    }

    /// Set the response
    pub fn with_response(mut self, response: serde_json::Value) -> Self {
        self.response = Some(response);
        self
    }

    /// Get a request field
    pub fn request_field<T: for<'de> Deserialize<'de>>(
        &self,
        field: &str,
    ) -> Result<T, TestHarnessError> {
        if let Some(value) = self.request.get(field) {
            serde_json::from_value(value.clone()).map_err(|e| {
                TestHarnessError::SerializationError(format!(
                    "Failed to deserialize request field '{}': {}",
                    field, e
                ))
            })
        } else {
            Err(TestHarnessError::MockError(format!(
                "Request field '{}' not found",
                field
            )))
        }
    }

    /// Get a response field
    pub fn response_field<T: for<'de> Deserialize<'de>>(
        &self,
        field: &str,
    ) -> Result<T, TestHarnessError> {
        if let Some(response) = &self.response {
            if let Some(value) = response.get(field) {
                serde_json::from_value(value.clone()).map_err(|e| {
                    TestHarnessError::SerializationError(format!(
                        "Failed to deserialize response field '{}': {}",
                        field, e
                    ))
                })
            } else {
                Err(TestHarnessError::MockError(format!(
                    "Response field '{}' not found",
                    field
                )))
            }
        } else {
            Err(TestHarnessError::MockError(
                "No response available".to_string(),
            ))
        }
    }
}

/// Recorded interaction with a mock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedInteraction {
    /// Interaction
    pub interaction: Interaction,
    /// Behavior name
    pub behavior: Option<String>,
    /// Verification status
    pub verified: bool,
}

impl RecordedInteraction {
    /// Create a new recorded interaction
    pub fn new(interaction: Interaction) -> Self {
        Self {
            interaction,
            behavior: None,
            verified: false,
        }
    }

    /// Set the behavior name
    pub fn with_behavior(mut self, behavior: impl Into<String>) -> Self {
        self.behavior = Some(behavior.into());
        self
    }

    /// Mark the interaction as verified
    pub fn verify(&mut self) {
        self.verified = true;
    }

    /// Check if the interaction is verified
    pub fn is_verified(&self) -> bool {
        self.verified
    }
}

/// Interaction matcher for matching interactions
pub trait InteractionMatcher: Send + Sync {
    /// Check if the matcher matches an interaction
    fn matches(&self, interaction: &Interaction) -> bool;
}

/// Mock recorder for recording and verifying interactions
pub struct MockRecorder {
    /// Recorded interactions
    interactions: RwLock<Vec<RecordedInteraction>>,
    /// Expected interactions
    expected: RwLock<Vec<(Box<dyn InteractionMatcher>, usize)>>,
}

impl MockRecorder {
    /// Create a new mock recorder
    pub fn new() -> Self {
        Self {
            interactions: RwLock::new(Vec::new()),
            expected: RwLock::new(Vec::new()),
        }
    }

    /// Record an interaction
    pub async fn record(&self, interaction: Interaction) -> RecordedInteraction {
        let recorded = RecordedInteraction::new(interaction);
        let mut interactions = self.interactions.write().await;
        interactions.push(recorded.clone());
        recorded
    }

    /// Record an interaction with a behavior
    pub async fn record_with_behavior(
        &self,
        interaction: Interaction,
        behavior: impl Into<String>,
    ) -> RecordedInteraction {
        let recorded = RecordedInteraction::new(interaction).with_behavior(behavior);
        let mut interactions = self.interactions.write().await;
        interactions.push(recorded.clone());
        recorded
    }

    /// Get all recorded interactions
    pub async fn get_interactions(&self) -> Vec<RecordedInteraction> {
        let interactions = self.interactions.read().await;
        interactions.clone()
    }

    /// Get recorded interactions by behavior
    pub async fn get_interactions_by_behavior(&self, behavior: &str) -> Vec<RecordedInteraction> {
        let interactions = self.interactions.read().await;
        interactions
            .iter()
            .filter(|i| i.behavior.as_deref() == Some(behavior))
            .cloned()
            .collect()
    }

    /// Get recorded interactions by matcher
    pub async fn get_interactions_by_matcher(
        &self,
        matcher: impl InteractionMatcher,
    ) -> Vec<RecordedInteraction> {
        let interactions = self.interactions.read().await;
        interactions
            .iter()
            .filter(|i| matcher.matches(&i.interaction))
            .cloned()
            .collect()
    }

    /// Clear all recorded interactions
    pub async fn clear(&self) {
        let mut interactions = self.interactions.write().await;
        interactions.clear();
    }

    /// Add an expected interaction
    pub async fn expect(&self, matcher: impl InteractionMatcher + 'static, count: usize) {
        let mut expected = self.expected.write().await;
        expected.push((Box::new(matcher), count));
    }

    /// Clear all expected interactions
    pub async fn clear_expectations(&self) {
        let mut expected = self.expected.write().await;
        expected.clear();
    }

    /// Verify that all expected interactions occurred
    pub async fn verify(&self) -> Result<(), TestHarnessError> {
        let interactions = self.interactions.read().await;
        let expected = self.expected.read().await;

        let mut errors = Vec::new();

        for (matcher, expected_count) in expected.iter() {
            let actual_count = interactions
                .iter()
                .filter(|i| matcher.matches(&i.interaction))
                .count();

            if actual_count != *expected_count {
                errors.push(format!(
                    "Expected {} interactions, but got {}",
                    expected_count, actual_count
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(TestHarnessError::MockError(format!(
                "Verification failed: {}",
                errors.join(", ")
            )))
        }
    }

    /// Mark all interactions as verified
    pub async fn mark_all_verified(&self) {
        let mut interactions = self.interactions.write().await;
        for interaction in interactions.iter_mut() {
            interaction.verify();
        }
    }

    /// Get the number of recorded interactions
    pub async fn count(&self) -> usize {
        let interactions = self.interactions.read().await;
        interactions.len()
    }

    /// Get the number of verified interactions
    pub async fn verified_count(&self) -> usize {
        let interactions = self.interactions.read().await;
        interactions.iter().filter(|i| i.verified).count()
    }

    /// Get the number of unverified interactions
    pub async fn unverified_count(&self) -> usize {
        let interactions = self.interactions.read().await;
        interactions.iter().filter(|i| !i.verified).count()
    }
}

impl Default for MockRecorder {
    fn default() -> Self {
        Self::new()
    }
}

/// Function matcher for matching interactions
pub struct FunctionMatcher {
    /// Matcher function
    matcher: Box<dyn Fn(&Interaction) -> bool + Send + Sync>,
}

impl FunctionMatcher {
    /// Create a new function matcher
    pub fn new(matcher: impl Fn(&Interaction) -> bool + Send + Sync + 'static) -> Self {
        Self {
            matcher: Box::new(matcher),
        }
    }
}

impl InteractionMatcher for FunctionMatcher {
    fn matches(&self, interaction: &Interaction) -> bool {
        (self.matcher)(interaction)
    }
}

/// Path matcher for matching interactions by path
pub struct PathMatcher {
    /// Path to match
    path: String,
}

impl PathMatcher {
    /// Create a new path matcher
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }
}

impl InteractionMatcher for PathMatcher {
    fn matches(&self, interaction: &Interaction) -> bool {
        if let Some(path) = interaction.request.get("path") {
            if let Some(path_str) = path.as_str() {
                return path_str == self.path;
            }
        }
        false
    }
}

/// Method matcher for matching interactions by HTTP method
pub struct MethodMatcher {
    /// Method to match
    method: String,
}

impl MethodMatcher {
    /// Create a new method matcher
    pub fn new(method: impl Into<String>) -> Self {
        Self {
            method: method.into(),
        }
    }
}

impl InteractionMatcher for MethodMatcher {
    fn matches(&self, interaction: &Interaction) -> bool {
        if let Some(method) = interaction.request.get("method") {
            if let Some(method_str) = method.as_str() {
                return method_str.eq_ignore_ascii_case(&self.method);
            }
        }
        false
    }
}

/// Path and method matcher for matching interactions by path and method
pub struct PathAndMethodMatcher {
    /// Path to match
    path: String,
    /// Method to match
    method: String,
}

impl PathAndMethodMatcher {
    /// Create a new path and method matcher
    pub fn new(path: impl Into<String>, method: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            method: method.into(),
        }
    }
}

impl InteractionMatcher for PathAndMethodMatcher {
    fn matches(&self, interaction: &Interaction) -> bool {
        let path_matches = if let Some(path) = interaction.request.get("path") {
            if let Some(path_str) = path.as_str() {
                path_str == self.path
            } else {
                false
            }
        } else {
            false
        };

        let method_matches = if let Some(method) = interaction.request.get("method") {
            if let Some(method_str) = method.as_str() {
                method_str.eq_ignore_ascii_case(&self.method)
            } else {
                false
            }
        } else {
            false
        };

        path_matches && method_matches
    }
}

/// Header matcher for matching interactions by header
pub struct HeaderMatcher {
    /// Header name
    name: String,
    /// Header value
    value: Option<String>,
}

impl HeaderMatcher {
    /// Create a new header matcher
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: None,
        }
    }

    /// Create a new header matcher with a value
    pub fn with_value(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: Some(value.into()),
        }
    }
}

impl InteractionMatcher for HeaderMatcher {
    fn matches(&self, interaction: &Interaction) -> bool {
        if let Some(headers) = interaction.request.get("headers") {
            if let Some(headers_obj) = headers.as_object() {
                if let Some(header) = headers_obj.get(&self.name) {
                    if let Some(value) = &self.value {
                        if let Some(header_str) = header.as_str() {
                            return header_str == value;
                        }
                    } else {
                        return true;
                    }
                }
            }
        }
        false
    }
}

/// Body matcher for matching interactions by body
pub struct BodyMatcher {
    /// Body matcher function
    matcher: Box<dyn Fn(&serde_json::Value) -> bool + Send + Sync>,
}

impl BodyMatcher {
    /// Create a new body matcher
    pub fn new(matcher: impl Fn(&serde_json::Value) -> bool + Send + Sync + 'static) -> Self {
        Self {
            matcher: Box::new(matcher),
        }
    }

    /// Create a new body matcher that checks for a field
    pub fn has_field(field: impl Into<String>) -> Self {
        let field = field.into();
        Self::new(move |body| body.get(&field).is_some())
    }

    /// Create a new body matcher that checks for a field with a value
    pub fn field_equals<T: Serialize>(
        field: impl Into<String>,
        value: T,
    ) -> Result<Self, TestHarnessError> {
        let field = field.into();
        let value = serde_json::to_value(value).map_err(|e| {
            TestHarnessError::SerializationError(format!("Failed to serialize value: {}", e))
        })?;

        Ok(Self::new(move |body| {
            if let Some(field_value) = body.get(&field) {
                field_value == &value
            } else {
                false
            }
        }))
    }
}

impl InteractionMatcher for BodyMatcher {
    fn matches(&self, interaction: &Interaction) -> bool {
        if let Some(body) = interaction.request.get("body") {
            (self.matcher)(body)
        } else {
            false
        }
    }
}

/// Composite matcher for combining multiple matchers
pub struct CompositeMatcher {
    /// Matchers
    matchers: Vec<Box<dyn InteractionMatcher>>,
    /// Whether all matchers must match
    all: bool,
}

impl CompositeMatcher {
    /// Create a new composite matcher that requires all matchers to match
    pub fn all(matchers: Vec<Box<dyn InteractionMatcher>>) -> Self {
        Self {
            matchers,
            all: true,
        }
    }

    /// Create a new composite matcher that requires any matcher to match
    pub fn any(matchers: Vec<Box<dyn InteractionMatcher>>) -> Self {
        Self {
            matchers,
            all: false,
        }
    }

    /// Add a matcher
    pub fn add_matcher(&mut self, matcher: impl InteractionMatcher + 'static) {
        self.matchers.push(Box::new(matcher));
    }
}

impl InteractionMatcher for CompositeMatcher {
    fn matches(&self, interaction: &Interaction) -> bool {
        if self.all {
            self.matchers.iter().all(|m| m.matches(interaction))
        } else {
            self.matchers.iter().any(|m| m.matches(interaction))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_recorder() {
        // Create a recorder
        let recorder = MockRecorder::new();

        // Record some interactions
        let interaction1 = Interaction::new(
            "interaction1",
            serde_json::json!({
                "path": "/api/test",
                "method": "GET"
            }),
        );

        let interaction2 = Interaction::new(
            "interaction2",
            serde_json::json!({
                "path": "/api/test",
                "method": "POST",
                "body": {
                    "name": "test"
                }
            }),
        );

        recorder.record(interaction1).await;
        recorder
            .record_with_behavior(interaction2, "test-behavior")
            .await;

        // Check the interaction count
        assert_eq!(recorder.count().await, 2);

        // Get interactions by behavior
        let behavior_interactions = recorder.get_interactions_by_behavior("test-behavior").await;
        assert_eq!(behavior_interactions.len(), 1);
        assert_eq!(behavior_interactions[0].interaction.id, "interaction2");

        // Get interactions by matcher
        let path_matcher = PathMatcher::new("/api/test");
        let path_interactions = recorder.get_interactions_by_matcher(path_matcher).await;
        assert_eq!(path_interactions.len(), 2);

        let method_matcher = MethodMatcher::new("POST");
        let method_interactions = recorder.get_interactions_by_matcher(method_matcher).await;
        assert_eq!(method_interactions.len(), 1);
        assert_eq!(method_interactions[0].interaction.id, "interaction2");

        // Clear the recorder
        recorder.clear().await;
        assert_eq!(recorder.count().await, 0);
    }

    #[tokio::test]
    async fn test_interaction_matchers() {
        // Create an interaction
        let interaction = Interaction::new(
            "test",
            serde_json::json!({
                "path": "/api/test",
                "method": "POST",
                "headers": {
                    "Content-Type": "application/json",
                    "Authorization": "Bearer token"
                },
                "body": {
                    "name": "test",
                    "value": 42
                }
            }),
        );

        // Test path matcher
        let path_matcher = PathMatcher::new("/api/test");
        assert!(path_matcher.matches(&interaction));

        let wrong_path_matcher = PathMatcher::new("/api/wrong");
        assert!(!wrong_path_matcher.matches(&interaction));

        // Test method matcher
        let method_matcher = MethodMatcher::new("POST");
        assert!(method_matcher.matches(&interaction));

        let wrong_method_matcher = MethodMatcher::new("GET");
        assert!(!wrong_method_matcher.matches(&interaction));

        // Test path and method matcher
        let path_method_matcher = PathAndMethodMatcher::new("/api/test", "POST");
        assert!(path_method_matcher.matches(&interaction));

        let wrong_path_method_matcher = PathAndMethodMatcher::new("/api/test", "GET");
        assert!(!wrong_path_method_matcher.matches(&interaction));

        // Test header matcher
        let header_matcher = HeaderMatcher::with_value("Content-Type", "application/json");
        assert!(header_matcher.matches(&interaction));

        let wrong_header_matcher = HeaderMatcher::with_value("Content-Type", "text/plain");
        assert!(!wrong_header_matcher.matches(&interaction));

        // Test body matcher
        let body_matcher = BodyMatcher::field_equals("name", "test").unwrap();
        assert!(body_matcher.matches(&interaction));

        let wrong_body_matcher = BodyMatcher::field_equals("name", "wrong").unwrap();
        assert!(!wrong_body_matcher.matches(&interaction));

        // Test composite matcher
        let composite_matcher = CompositeMatcher::all(vec![
            Box::new(PathMatcher::new("/api/test")),
            Box::new(MethodMatcher::new("POST")),
            Box::new(HeaderMatcher::with_value(
                "Content-Type",
                "application/json",
            )),
            Box::new(BodyMatcher::field_equals("name", "test").unwrap()),
        ]);
        assert!(composite_matcher.matches(&interaction));

        let wrong_composite_matcher = CompositeMatcher::all(vec![
            Box::new(PathMatcher::new("/api/test")),
            Box::new(MethodMatcher::new("GET")),
        ]);
        assert!(!wrong_composite_matcher.matches(&interaction));

        let any_composite_matcher = CompositeMatcher::any(vec![
            Box::new(PathMatcher::new("/api/wrong")),
            Box::new(MethodMatcher::new("GET")),
            Box::new(HeaderMatcher::with_value(
                "Content-Type",
                "application/json",
            )),
        ]);
        assert!(any_composite_matcher.matches(&interaction));
    }
}
