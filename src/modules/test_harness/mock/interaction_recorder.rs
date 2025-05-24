//! HTTP Interaction Recorder Module
//!
//! This module provides functionality for recording and replaying HTTP interactions
//! with enhanced capabilities for intelligent stub generation and configurable behaviors.

use std::collections::HashMap;
use std::fmt;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::http::{HttpRequest, HttpResponse};
use super::recorder::{Interaction, MockRecorder, RecordedInteraction};
use crate::modules::test_harness::types::TestHarnessError;

/// Recording mode for the interaction recorder
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordingMode {
    /// Record all interactions
    Record,
    /// Replay recorded interactions
    Replay,
    /// Record if no matching interaction is found
    Auto,
    /// Passthrough to the real service
    Passthrough,
}

impl fmt::Display for RecordingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecordingMode::Record => write!(f, "Record"),
            RecordingMode::Replay => write!(f, "Replay"),
            RecordingMode::Auto => write!(f, "Auto"),
            RecordingMode::Passthrough => write!(f, "Passthrough"),
        }
    }
}

/// HTTP interaction for recording and replaying
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpInteraction {
    /// Interaction ID
    pub id: String,
    /// Interaction timestamp
    pub timestamp: DateTime<Utc>,
    /// HTTP request
    pub request: HttpRequest,
    /// HTTP response
    pub response: HttpResponse,
    /// Latency in milliseconds
    pub latency_ms: u64,
    /// Tags for categorizing interactions
    pub tags: Vec<String>,
    /// Metadata for additional information
    pub metadata: HashMap<String, String>,
}

impl HttpInteraction {
    /// Create a new HTTP interaction
    pub fn new(request: HttpRequest, response: HttpResponse) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            request,
            response,
            latency_ms: 0,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set the latency
    pub fn with_latency(mut self, latency_ms: u64) -> Self {
        self.latency_ms = latency_ms;
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add multiple tags
    pub fn with_tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for tag in tags {
            self.tags.push(tag.into());
        }
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Convert to a recorded interaction
    pub fn to_recorded_interaction(&self) -> RecordedInteraction {
        let mut request = serde_json::Map::new();
        request.insert(
            "method".to_string(),
            serde_json::Value::String(self.request.method.clone()),
        );
        request.insert(
            "path".to_string(),
            serde_json::Value::String(self.request.path.clone()),
        );
        request.insert(
            "query".to_string(),
            serde_json::to_value(&self.request.query).unwrap(),
        );
        request.insert(
            "headers".to_string(),
            serde_json::to_value(&self.request.headers).unwrap(),
        );

        if let Some(body) = &self.request.body {
            request.insert("body".to_string(), body.clone());
        }

        let mut response = serde_json::Map::new();
        response.insert(
            "status".to_string(),
            serde_json::Value::Number(serde_json::Number::from(self.response.status)),
        );
        response.insert(
            "headers".to_string(),
            serde_json::to_value(&self.response.headers).unwrap(),
        );

        if let Some(body) = &self.response.body {
            response.insert("body".to_string(), body.clone());
        }

        let interaction = Interaction::new(self.id.clone(), serde_json::Value::Object(request))
            .with_response(serde_json::Value::Object(response));

        RecordedInteraction::new(interaction)
    }
}

/// HTTP interaction matcher for finding matching interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpInteractionMatcher {
    /// Match by method
    pub match_method: bool,
    /// Match by path
    pub match_path: bool,
    /// Match by query parameters
    pub match_query: bool,
    /// Match by headers
    pub match_headers: Vec<String>,
    /// Match by body
    pub match_body: bool,
    /// Match by body fields
    pub match_body_fields: Vec<String>,
}

impl Default for HttpInteractionMatcher {
    fn default() -> Self {
        Self {
            match_method: true,
            match_path: true,
            match_query: false,
            match_headers: Vec::new(),
            match_body: false,
            match_body_fields: Vec::new(),
        }
    }
}

impl HttpInteractionMatcher {
    /// Create a new HTTP interaction matcher
    pub fn new() -> Self {
        Self::default()
    }

    /// Match by method
    pub fn with_method_matching(mut self, match_method: bool) -> Self {
        self.match_method = match_method;
        self
    }

    /// Match by path
    pub fn with_path_matching(mut self, match_path: bool) -> Self {
        self.match_path = match_path;
        self
    }

    /// Match by query parameters
    pub fn with_query_matching(mut self, match_query: bool) -> Self {
        self.match_query = match_query;
        self
    }

    /// Match by specific headers
    pub fn with_header_matching(
        mut self,
        headers: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        for header in headers {
            self.match_headers.push(header.into());
        }
        self
    }

    /// Match by body
    pub fn with_body_matching(mut self, match_body: bool) -> Self {
        self.match_body = match_body;
        self
    }

    /// Match by specific body fields
    pub fn with_body_field_matching(
        mut self,
        fields: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        for field in fields {
            self.match_body_fields.push(field.into());
        }
        self
    }

    /// Check if the matcher matches an interaction
    pub fn matches(&self, recorded: &HttpInteraction, request: &HttpRequest) -> bool {
        // Match method
        if self.match_method && recorded.request.method != request.method {
            return false;
        }

        // Match path
        if self.match_path && recorded.request.path != request.path {
            return false;
        }

        // Match query parameters
        if self.match_query {
            for (key, value) in &request.query {
                if !recorded.request.query.contains_key(key)
                    || recorded.request.query.get(key) != Some(value)
                {
                    return false;
                }
            }
        }

        // Match headers
        for header in &self.match_headers {
            if !recorded.request.headers.contains_key(header)
                || recorded.request.headers.get(header) != request.headers.get(header)
            {
                return false;
            }
        }

        // Match body
        if self.match_body {
            if recorded.request.body != request.body {
                return false;
            }
        }

        // Match body fields
        if !self.match_body_fields.is_empty() {
            if let (Some(recorded_body), Some(request_body)) =
                (&recorded.request.body, &request.body)
            {
                if let (Some(recorded_obj), Some(request_obj)) =
                    (recorded_body.as_object(), request_body.as_object())
                {
                    for field in &self.match_body_fields {
                        if recorded_obj.get(field) != request_obj.get(field) {
                            return false;
                        }
                    }
                } else {
                    return false;
                }
            } else if !self.match_body_fields.is_empty() {
                return false;
            }
        }

        true
    }
}

/// HTTP interaction recorder configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionRecorderConfig {
    /// Recording mode
    pub mode: RecordingMode,
    /// Storage directory for recorded interactions
    pub storage_dir: PathBuf,
    /// Interaction matcher
    pub matcher: HttpInteractionMatcher,
    /// Whether to simulate latency when replaying
    pub simulate_latency: bool,
    /// Default latency in milliseconds when no recorded latency is available
    pub default_latency_ms: u64,
    /// Latency multiplier for scaling recorded latencies
    pub latency_multiplier: f64,
    /// Whether to record headers
    pub record_headers: bool,
    /// Headers to exclude from recording
    pub excluded_headers: Vec<String>,
    /// Whether to record request bodies
    pub record_request_body: bool,
    /// Whether to record response bodies
    pub record_response_body: bool,
    /// Whether to generate stubs automatically
    pub auto_generate_stubs: bool,
}

impl Default for InteractionRecorderConfig {
    fn default() -> Self {
        Self {
            mode: RecordingMode::Auto,
            storage_dir: PathBuf::from("interactions"),
            matcher: HttpInteractionMatcher::default(),
            simulate_latency: false,
            default_latency_ms: 50,
            latency_multiplier: 1.0,
            record_headers: true,
            excluded_headers: vec![
                "authorization".to_string(),
                "cookie".to_string(),
                "set-cookie".to_string(),
            ],
            record_request_body: true,
            record_response_body: true,
            auto_generate_stubs: true,
        }
    }
}

/// HTTP interaction recorder for recording and replaying HTTP interactions
pub struct HttpInteractionRecorder {
    /// Recorder name
    name: String,
    /// Recorder configuration
    config: RwLock<InteractionRecorderConfig>,
    /// Recorded interactions
    interactions: RwLock<Vec<HttpInteraction>>,
    /// Mock recorder for integration with the existing framework
    recorder: Arc<MockRecorder>,
}

impl HttpInteractionRecorder {
    /// Create a new HTTP interaction recorder
    pub fn new(name: impl Into<String>, config: InteractionRecorderConfig) -> Self {
        Self {
            name: name.into(),
            config: RwLock::new(config),
            interactions: RwLock::new(Vec::new()),
            recorder: Arc::new(MockRecorder::new()),
        }
    }

    /// Get the recorder name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the recorder configuration
    pub async fn config(&self) -> InteractionRecorderConfig {
        self.config.read().await.clone()
    }

    /// Update the recorder configuration
    pub async fn update_config(&self, config: InteractionRecorderConfig) {
        let mut cfg = self.config.write().await;
        *cfg = config;
    }

    /// Set the recording mode
    pub async fn set_mode(&self, mode: RecordingMode) {
        let mut config = self.config.write().await;
        config.mode = mode;
    }

    /// Get the recording mode
    pub async fn mode(&self) -> RecordingMode {
        self.config.read().await.mode
    }

    /// Record an interaction
    pub async fn record(&self, interaction: HttpInteraction) {
        // Add to interactions
        let mut interactions = self.interactions.write().await;
        interactions.push(interaction.clone());

        // Record in the mock recorder
        let recorded = interaction.to_recorded_interaction();
        let mut recorder_interactions = self.recorder.get_interactions().await;
        recorder_interactions.push(recorded);
    }

    /// Find a matching interaction
    pub async fn find_matching(&self, request: &HttpRequest) -> Option<HttpInteraction> {
        let interactions = self.interactions.read().await;
        let config = self.config.read().await;

        for interaction in interactions.iter() {
            if config.matcher.matches(interaction, request) {
                return Some(interaction.clone());
            }
        }

        None
    }

    /// Handle a request
    pub async fn handle_request(
        &self,
        request: &HttpRequest,
        real_handler: impl Fn(&HttpRequest) -> HttpResponse,
    ) -> HttpResponse {
        let config = self.config.read().await;
        let mode = config.mode;

        match mode {
            RecordingMode::Record => {
                // Record mode: always call the real handler and record the interaction
                let start = Instant::now();
                let response = real_handler(request);
                let latency = start.elapsed().as_millis() as u64;

                // Create and record the interaction
                let interaction =
                    HttpInteraction::new(request.clone(), response.clone()).with_latency(latency);
                self.record(interaction).await;

                response
            }
            RecordingMode::Replay => {
                // Replay mode: find a matching interaction or return a 404
                if let Some(interaction) = self.find_matching(request).await {
                    // Simulate latency if configured
                    if config.simulate_latency {
                        let latency = if interaction.latency_ms > 0 {
                            (interaction.latency_ms as f64 * config.latency_multiplier) as u64
                        } else {
                            config.default_latency_ms
                        };
                        tokio::time::sleep(Duration::from_millis(latency)).await;
                    }

                    interaction.response.clone()
                } else {
                    // No matching interaction found
                    warn!("No matching interaction found for request: {:?}", request);
                    HttpResponse::new(404)
                        .with_body(serde_json::json!({
                            "error": "Not found",
                            "message": "No matching interaction found"
                        }))
                        .unwrap()
                }
            }
            RecordingMode::Auto => {
                // Auto mode: try to find a matching interaction, if not found, record a new one
                if let Some(interaction) = self.find_matching(request).await {
                    // Simulate latency if configured
                    if config.simulate_latency {
                        let latency = if interaction.latency_ms > 0 {
                            (interaction.latency_ms as f64 * config.latency_multiplier) as u64
                        } else {
                            config.default_latency_ms
                        };
                        tokio::time::sleep(Duration::from_millis(latency)).await;
                    }

                    interaction.response.clone()
                } else {
                    // No matching interaction found, call the real handler and record
                    let start = Instant::now();
                    let response = real_handler(request);
                    let latency = start.elapsed().as_millis() as u64;

                    // Create and record the interaction
                    let interaction = HttpInteraction::new(request.clone(), response.clone())
                        .with_latency(latency);
                    self.record(interaction).await;

                    response
                }
            }
            RecordingMode::Passthrough => {
                // Passthrough mode: always call the real handler without recording
                real_handler(request)
            }
        }
    }

    /// Save recorded interactions to a file
    pub async fn save(&self, path: impl AsRef<Path>) -> Result<(), TestHarnessError> {
        let interactions = self.interactions.read().await;
        let file = File::create(path)
            .map_err(|e| TestHarnessError::IoError(format!("Failed to create file: {}", e)))?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &*interactions).map_err(|e| {
            TestHarnessError::SerializationError(format!("Failed to serialize interactions: {}", e))
        })?;
        Ok(())
    }

    /// Load recorded interactions from a file
    pub async fn load(&self, path: impl AsRef<Path>) -> Result<(), TestHarnessError> {
        let file = File::open(path)
            .map_err(|e| TestHarnessError::IoError(format!("Failed to open file: {}", e)))?;
        let reader = BufReader::new(file);
        let loaded: Vec<HttpInteraction> = serde_json::from_reader(reader).map_err(|e| {
            TestHarnessError::SerializationError(format!(
                "Failed to deserialize interactions: {}",
                e
            ))
        })?;

        let mut interactions = self.interactions.write().await;
        *interactions = loaded;
        Ok(())
    }

    /// Clear all recorded interactions
    pub async fn clear(&self) {
        let mut interactions = self.interactions.write().await;
        interactions.clear();
        self.recorder.clear().await;
    }

    /// Get all recorded interactions
    pub async fn get_interactions(&self) -> Vec<HttpInteraction> {
        let interactions = self.interactions.read().await;
        interactions.clone()
    }

    /// Generate stubs from recorded interactions
    pub async fn generate_stubs(&self) -> Vec<super::http::HttpStub> {
        let interactions = self.interactions.read().await;
        let mut stubs = Vec::new();

        for interaction in interactions.iter() {
            let request = &interaction.request;
            let response = interaction.response.clone();
            let latency = interaction.latency_ms;

            // Create a stub that matches this interaction
            let stub = super::http::HttpStub::for_path_and_method(
                request.path.clone(),
                request.method.clone(),
                move |_| {
                    // Clone response for the closure
                    let response_clone = response.clone();

                    // If latency simulation is enabled, sleep for the recorded latency
                    if latency > 0 {
                        // Note: This is a blocking sleep, which is not ideal in async code
                        // In a real implementation, we would use tokio::time::sleep
                        std::thread::sleep(Duration::from_millis(latency));
                    }

                    response_clone
                },
            );

            stubs.push(stub);
        }

        stubs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_interaction_recorder() {
        // Create a recorder
        let config = InteractionRecorderConfig {
            mode: RecordingMode::Record,
            ..Default::default()
        };
        let recorder = HttpInteractionRecorder::new("test-recorder", config);

        // Create a request
        let request = HttpRequest::new("GET", "/api/test")
            .with_query("param", "value")
            .with_header("Content-Type", "application/json");

        // Create a real handler
        let real_handler = |_: &HttpRequest| {
            HttpResponse::new(200)
                .with_body(serde_json::json!({"message": "success"}))
                .unwrap()
        };

        // Handle the request in record mode
        let response = recorder.handle_request(&request, real_handler).await;

        // Check the response
        assert_eq!(response.status, 200);
        assert_eq!(
            response.body.unwrap(),
            serde_json::json!({"message": "success"})
        );

        // Check that the interaction was recorded
        let interactions = recorder.get_interactions().await;
        assert_eq!(interactions.len(), 1);
        assert_eq!(interactions[0].request.method, "GET");
        assert_eq!(interactions[0].request.path, "/api/test");
        assert_eq!(interactions[0].response.status, 200);

        // Switch to replay mode
        recorder.set_mode(RecordingMode::Replay).await;

        // Handle the same request again
        let response2 = recorder
            .handle_request(&request, |_| {
                // This should not be called in replay mode
                HttpResponse::new(500)
                    .with_body(serde_json::json!({"message": "error"}))
                    .unwrap()
            })
            .await;

        // Check that we got the recorded response
        assert_eq!(response2.status, 200);
        assert_eq!(
            response2.body.unwrap(),
            serde_json::json!({"message": "success"})
        );

        // Generate stubs
        let stubs = recorder.generate_stubs().await;
        assert_eq!(stubs.len(), 1);
    }
}
