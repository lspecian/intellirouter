//! Stub Generator Module
//!
//! This module provides functionality for generating stubs from recorded interactions.

use std::collections::HashMap;
use std::fmt;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::http::{HttpMock, HttpRequest, HttpResponse, HttpStub};
use super::recorder::{Interaction, MockRecorder, RecordedInteraction};
use crate::modules::test_harness::types::TestHarnessError;

/// Stub generation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StubGenerationConfig {
    /// Whether to generate stubs automatically
    pub auto_generate: bool,
    /// Output directory for generated stubs
    pub output_dir: PathBuf,
    /// Whether to overwrite existing stubs
    pub overwrite_existing: bool,
    /// Whether to include headers in stubs
    pub include_headers: bool,
    /// Headers to exclude from stubs
    pub excluded_headers: Vec<String>,
    /// Whether to include request body in stubs
    pub include_request_body: bool,
    /// Whether to include response body in stubs
    pub include_response_body: bool,
    /// Whether to generate strict matchers
    pub strict_matching: bool,
    /// Fields to match strictly
    pub strict_match_fields: Vec<String>,
}

impl Default for StubGenerationConfig {
    fn default() -> Self {
        Self {
            auto_generate: true,
            output_dir: PathBuf::from("stubs"),
            overwrite_existing: false,
            include_headers: true,
            excluded_headers: vec![
                "authorization".to_string(),
                "cookie".to_string(),
                "set-cookie".to_string(),
            ],
            include_request_body: true,
            include_response_body: true,
            strict_matching: false,
            strict_match_fields: Vec::new(),
        }
    }
}

/// Stub generator for creating stubs from recorded interactions
pub struct StubGenerator {
    /// Configuration
    config: RwLock<StubGenerationConfig>,
}

impl StubGenerator {
    /// Create a new stub generator
    pub fn new(config: StubGenerationConfig) -> Self {
        Self {
            config: RwLock::new(config),
        }
    }

    /// Get the configuration
    pub async fn config(&self) -> StubGenerationConfig {
        self.config.read().await.clone()
    }

    /// Update the configuration
    pub async fn update_config(&self, config: StubGenerationConfig) {
        let mut cfg = self.config.write().await;
        *cfg = config;
    }

    /// Generate stubs from recorded interactions
    pub async fn generate_stubs(&self, interactions: &[RecordedInteraction]) -> Vec<HttpStub> {
        let config = self.config.read().await.clone();
        let mut stubs = Vec::new();

        for interaction in interactions {
            if let Some(stub) = self.generate_stub(interaction, &config).await {
                stubs.push(stub);
            }
        }

        // Save stubs to files if configured
        if !config.output_dir.as_os_str().is_empty() {
            self.save_stubs_to_files(&stubs, &config.output_dir)
                .await
                .ok();
        }

        stubs
    }

    /// Generate a stub from a recorded interaction
    async fn generate_stub(
        &self,
        interaction: &RecordedInteraction,
        config: &StubGenerationConfig,
    ) -> Option<HttpStub> {
        // Extract request and response from the interaction
        let request = self.extract_request(interaction)?;
        let response = self.extract_response(interaction)?;

        // Create a matcher function based on the configuration
        let matcher = self.create_matcher(&request, config);

        // Create a response generator function
        let response_generator = self.create_response_generator(response);

        // Create the stub
        Some(HttpStub::new(matcher, response_generator))
    }

    /// Extract a request from a recorded interaction
    fn extract_request(&self, interaction: &RecordedInteraction) -> Option<HttpRequest> {
        let method = interaction
            .interaction
            .request_field::<String>("method")
            .ok()?;
        let path = interaction
            .interaction
            .request_field::<String>("path")
            .ok()?;

        let mut request = HttpRequest::new(method, path);

        // Add query parameters
        if let Ok(query) = interaction
            .interaction
            .request_field::<HashMap<String, String>>("query")
        {
            for (key, value) in query {
                request = request.with_query(key, value);
            }
        }

        // Add headers
        if let Ok(headers) = interaction
            .interaction
            .request_field::<HashMap<String, String>>("headers")
        {
            for (key, value) in headers {
                request = request.with_header(key, value);
            }
        }

        // Add body
        if let Ok(body) = interaction
            .interaction
            .request_field::<serde_json::Value>("body")
        {
            if let Ok(req) = request.with_body(body) {
                request = req;
            }
        }

        Some(request)
    }

    /// Extract a response from a recorded interaction
    fn extract_response(&self, interaction: &RecordedInteraction) -> Option<HttpResponse> {
        if interaction.interaction.response.is_none() {
            return None;
        }

        let status = interaction
            .interaction
            .response_field::<u16>("status")
            .unwrap_or(200);
        let mut response = HttpResponse::new(status);

        // Add headers
        if let Ok(headers) = interaction
            .interaction
            .response_field::<HashMap<String, String>>("headers")
        {
            for (key, value) in headers {
                response = response.with_header(key, value);
            }
        }

        // Add body
        if let Ok(body) = interaction
            .interaction
            .response_field::<serde_json::Value>("body")
        {
            if let Ok(resp) = response.with_body(body) {
                response = resp;
            }
        }

        Some(response)
    }

    /// Create a matcher function based on the configuration
    fn create_matcher(
        &self,
        request: &HttpRequest,
        config: &StubGenerationConfig,
    ) -> impl Fn(&HttpRequest) -> bool + Send + Sync + 'static {
        // Clone the values we need to move into the closure
        let method = request.method.clone();
        let path = request.path.clone();
        let query = if config.strict_matching {
            request.query.clone()
        } else {
            HashMap::new()
        };

        let headers = if config.include_headers {
            let mut filtered_headers = HashMap::new();
            for (key, value) in &request.headers {
                if !config.excluded_headers.contains(key) {
                    filtered_headers.insert(key.clone(), value.clone());
                }
            }
            filtered_headers
        } else {
            HashMap::new()
        };

        let body = if config.include_request_body && config.strict_matching {
            request.body.clone()
        } else {
            None
        };

        let strict_match_fields = config.strict_match_fields.clone();

        move |req: &HttpRequest| {
            // Always match method and path
            if req.method != method || req.path != path {
                return false;
            }

            // Match query parameters if strict matching is enabled
            if !query.is_empty() {
                for (key, value) in &query {
                    if req.query.get(key) != Some(value) {
                        return false;
                    }
                }
            }

            // Match headers if included
            if !headers.is_empty() {
                for (key, value) in &headers {
                    if req.headers.get(key) != Some(value) {
                        return false;
                    }
                }
            }

            // Match body if strict matching is enabled and body is included
            if let Some(ref expected_body) = body {
                if let Some(ref actual_body) = req.body {
                    if !strict_match_fields.is_empty() {
                        // Match only specific fields
                        if let (Some(expected_obj), Some(actual_obj)) =
                            (expected_body.as_object(), actual_body.as_object())
                        {
                            for field in &strict_match_fields {
                                if expected_obj.get(field) != actual_obj.get(field) {
                                    return false;
                                }
                            }
                        } else {
                            return false;
                        }
                    } else if expected_body != actual_body {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            true
        }
    }

    /// Create a response generator function
    fn create_response_generator(
        &self,
        response: HttpResponse,
    ) -> impl Fn(&HttpRequest) -> HttpResponse + Send + Sync + 'static {
        // Clone the response to move into the closure
        let response_clone = response.clone();

        move |_: &HttpRequest| response_clone.clone()
    }

    /// Save stubs to files
    async fn save_stubs_to_files(
        &self,
        stubs: &[HttpStub],
        output_dir: &Path,
    ) -> Result<(), TestHarnessError> {
        // Create the output directory if it doesn't exist
        fs::create_dir_all(output_dir).map_err(|e| {
            TestHarnessError::IoError(format!("Failed to create output directory: {}", e))
        })?;

        // TODO: Implement stub serialization and saving
        // This would require additional serialization support for HttpStub

        Ok(())
    }

    /// Load stubs from files
    pub async fn load_stubs_from_files(
        &self,
        input_dir: &Path,
    ) -> Result<Vec<HttpStub>, TestHarnessError> {
        // Check if the input directory exists
        if !input_dir.exists() || !input_dir.is_dir() {
            return Err(TestHarnessError::IoError(format!(
                "Input directory does not exist or is not a directory: {:?}",
                input_dir
            )));
        }

        // TODO: Implement stub deserialization and loading
        // This would require additional deserialization support for HttpStub

        Ok(Vec::new())
    }

    /// Apply stubs to a mock
    pub async fn apply_stubs_to_mock(
        &self,
        mock: &HttpMock,
        stubs: &[HttpStub],
    ) -> Result<(), TestHarnessError> {
        for stub in stubs {
            // TODO: Add stubs to the mock
            // This would require HttpMock to be clonable or have a method to add stubs
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::recorder::Interaction;
    use super::*;

    #[tokio::test]
    async fn test_stub_generation() {
        // Create a stub generator
        let generator = StubGenerator::new(StubGenerationConfig::default());

        // Create a recorded interaction
        let request = serde_json::json!({
            "method": "GET",
            "path": "/api/test",
            "query": {},
            "headers": {
                "Content-Type": "application/json"
            }
        });

        let response = serde_json::json!({
            "status": 200,
            "headers": {
                "Content-Type": "application/json"
            },
            "body": {
                "message": "success"
            }
        });

        let interaction = Interaction::new("test", request).with_response(response);

        let recorded = RecordedInteraction::new(interaction);

        // Generate stubs
        let stubs = generator.generate_stubs(&[recorded]).await;

        // Check that a stub was generated
        assert_eq!(stubs.len(), 1);
    }
}
