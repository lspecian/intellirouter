//! Response Behavior Module
//!
//! This module provides functionality for configuring response behaviors
//! such as latency, errors, and other dynamic response characteristics.

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use super::http::{HttpRequest, HttpResponse};
use crate::modules::test_harness::types::TestHarnessError;

/// Latency configuration for simulating network delays
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyConfig {
    /// Whether to simulate latency
    pub enabled: bool,
    /// Fixed latency in milliseconds
    pub fixed_ms: Option<u64>,
    /// Minimum latency in milliseconds (for random latency)
    pub min_ms: u64,
    /// Maximum latency in milliseconds (for random latency)
    pub max_ms: u64,
    /// Distribution type for random latency
    pub distribution: LatencyDistribution,
}

impl Default for LatencyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            fixed_ms: None,
            min_ms: 10,
            max_ms: 100,
            distribution: LatencyDistribution::Uniform,
        }
    }
}

/// Latency distribution type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LatencyDistribution {
    /// Uniform distribution
    Uniform,
    /// Normal distribution
    Normal,
    /// Exponential distribution
    Exponential,
}

impl fmt::Display for LatencyDistribution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LatencyDistribution::Uniform => write!(f, "Uniform"),
            LatencyDistribution::Normal => write!(f, "Normal"),
            LatencyDistribution::Exponential => write!(f, "Exponential"),
        }
    }
}

/// Error simulation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorConfig {
    /// Whether to simulate errors
    pub enabled: bool,
    /// Probability of an error occurring (0.0 - 1.0)
    pub probability: f64,
    /// Error types to simulate with their probabilities
    pub error_types: HashMap<ErrorType, f64>,
    /// Custom error responses
    pub custom_errors: Vec<CustomError>,
}

impl Default for ErrorConfig {
    fn default() -> Self {
        let mut error_types = HashMap::new();
        error_types.insert(ErrorType::ServerError, 0.5);
        error_types.insert(ErrorType::Timeout, 0.3);
        error_types.insert(ErrorType::RateLimited, 0.2);

        Self {
            enabled: false,
            probability: 0.1,
            error_types,
            custom_errors: Vec::new(),
        }
    }
}

/// Error type for simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorType {
    /// Server error (500)
    ServerError,
    /// Not found error (404)
    NotFound,
    /// Bad request error (400)
    BadRequest,
    /// Unauthorized error (401)
    Unauthorized,
    /// Forbidden error (403)
    Forbidden,
    /// Timeout error (504)
    Timeout,
    /// Rate limited error (429)
    RateLimited,
    /// Service unavailable error (503)
    ServiceUnavailable,
    /// Custom error
    Custom,
}

impl ErrorType {
    /// Get the status code for the error type
    pub fn status_code(&self) -> u16 {
        match self {
            ErrorType::ServerError => 500,
            ErrorType::NotFound => 404,
            ErrorType::BadRequest => 400,
            ErrorType::Unauthorized => 401,
            ErrorType::Forbidden => 403,
            ErrorType::Timeout => 504,
            ErrorType::RateLimited => 429,
            ErrorType::ServiceUnavailable => 503,
            ErrorType::Custom => 500, // Default for custom errors
        }
    }

    /// Get the default error message
    pub fn default_message(&self) -> &'static str {
        match self {
            ErrorType::ServerError => "Internal Server Error",
            ErrorType::NotFound => "Not Found",
            ErrorType::BadRequest => "Bad Request",
            ErrorType::Unauthorized => "Unauthorized",
            ErrorType::Forbidden => "Forbidden",
            ErrorType::Timeout => "Gateway Timeout",
            ErrorType::RateLimited => "Too Many Requests",
            ErrorType::ServiceUnavailable => "Service Unavailable",
            ErrorType::Custom => "Error",
        }
    }
}

impl fmt::Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorType::ServerError => write!(f, "ServerError"),
            ErrorType::NotFound => write!(f, "NotFound"),
            ErrorType::BadRequest => write!(f, "BadRequest"),
            ErrorType::Unauthorized => write!(f, "Unauthorized"),
            ErrorType::Forbidden => write!(f, "Forbidden"),
            ErrorType::Timeout => write!(f, "Timeout"),
            ErrorType::RateLimited => write!(f, "RateLimited"),
            ErrorType::ServiceUnavailable => write!(f, "ServiceUnavailable"),
            ErrorType::Custom => write!(f, "Custom"),
        }
    }
}

/// Custom error configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomError {
    /// Error name
    pub name: String,
    /// Status code
    pub status: u16,
    /// Error message
    pub message: String,
    /// Error details
    pub details: Option<serde_json::Value>,
    /// Error headers
    pub headers: HashMap<String, String>,
    /// Probability of this error occurring (0.0 - 1.0)
    pub probability: f64,
    /// Conditions for when this error should occur
    pub conditions: Vec<ErrorCondition>,
}

/// Error condition for determining when to trigger an error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCondition {
    /// Condition type
    pub condition_type: ErrorConditionType,
    /// Condition value
    pub value: String,
}

/// Error condition type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorConditionType {
    /// Match path
    Path,
    /// Match method
    Method,
    /// Match header
    Header,
    /// Match query parameter
    Query,
    /// Match body field
    BodyField,
}

impl fmt::Display for ErrorConditionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorConditionType::Path => write!(f, "Path"),
            ErrorConditionType::Method => write!(f, "Method"),
            ErrorConditionType::Header => write!(f, "Header"),
            ErrorConditionType::Query => write!(f, "Query"),
            ErrorConditionType::BodyField => write!(f, "BodyField"),
        }
    }
}

/// Response transformation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationConfig {
    /// Whether to transform responses
    pub enabled: bool,
    /// Transformations to apply
    pub transformations: Vec<ResponseTransformation>,
}

impl Default for TransformationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            transformations: Vec::new(),
        }
    }
}

/// Response transformation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTransformation {
    /// Transformation name
    pub name: String,
    /// Transformation type
    pub transformation_type: TransformationType,
    /// Field to transform (for field-specific transformations)
    pub field: Option<String>,
    /// Transformation value
    pub value: Option<String>,
    /// Conditions for when to apply the transformation
    pub conditions: Vec<TransformationCondition>,
}

/// Transformation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransformationType {
    /// Add a header
    AddHeader,
    /// Remove a header
    RemoveHeader,
    /// Modify a header
    ModifyHeader,
    /// Add a field to the response body
    AddBodyField,
    /// Remove a field from the response body
    RemoveBodyField,
    /// Modify a field in the response body
    ModifyBodyField,
    /// Replace the entire response body
    ReplaceBody,
    /// Change the status code
    ChangeStatus,
}

impl fmt::Display for TransformationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransformationType::AddHeader => write!(f, "AddHeader"),
            TransformationType::RemoveHeader => write!(f, "RemoveHeader"),
            TransformationType::ModifyHeader => write!(f, "ModifyHeader"),
            TransformationType::AddBodyField => write!(f, "AddBodyField"),
            TransformationType::RemoveBodyField => write!(f, "RemoveBodyField"),
            TransformationType::ModifyBodyField => write!(f, "ModifyBodyField"),
            TransformationType::ReplaceBody => write!(f, "ReplaceBody"),
            TransformationType::ChangeStatus => write!(f, "ChangeStatus"),
        }
    }
}

/// Transformation condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationCondition {
    /// Condition type
    pub condition_type: TransformationConditionType,
    /// Condition value
    pub value: String,
}

/// Transformation condition type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransformationConditionType {
    /// Match path
    Path,
    /// Match method
    Method,
    /// Match status code
    Status,
    /// Match header
    Header,
    /// Match body field
    BodyField,
}

impl fmt::Display for TransformationConditionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransformationConditionType::Path => write!(f, "Path"),
            TransformationConditionType::Method => write!(f, "Method"),
            TransformationConditionType::Status => write!(f, "Status"),
            TransformationConditionType::Header => write!(f, "Header"),
            TransformationConditionType::BodyField => write!(f, "BodyField"),
        }
    }
}

/// Response behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseBehaviorConfig {
    /// Latency configuration
    pub latency: LatencyConfig,
    /// Error configuration
    pub error: ErrorConfig,
    /// Transformation configuration
    pub transformation: TransformationConfig,
}

impl Default for ResponseBehaviorConfig {
    fn default() -> Self {
        Self {
            latency: LatencyConfig::default(),
            error: ErrorConfig::default(),
            transformation: TransformationConfig::default(),
        }
    }
}

/// Response behavior handler
pub struct ResponseBehaviorHandler {
    /// Configuration
    config: RwLock<ResponseBehaviorConfig>,
}

impl ResponseBehaviorHandler {
    /// Create a new response behavior handler
    pub fn new(config: ResponseBehaviorConfig) -> Self {
        Self {
            config: RwLock::new(config),
        }
    }

    /// Get the configuration
    pub async fn config(&self) -> ResponseBehaviorConfig {
        self.config.read().await.clone()
    }

    /// Update the configuration
    pub async fn update_config(&self, config: ResponseBehaviorConfig) {
        let mut cfg = self.config.write().await;
        *cfg = config;
    }

    /// Apply behaviors to a response
    pub async fn apply(&self, request: &HttpRequest, response: HttpResponse) -> HttpResponse {
        let config = self.config.read().await.clone();
        let mut modified_response = response;

        // Apply error simulation
        if config.error.enabled && self.should_simulate_error(&config.error) {
            return self.generate_error_response(&config.error, request).await;
        }

        // Apply transformations
        if config.transformation.enabled {
            modified_response = self
                .apply_transformations(&config.transformation, request, modified_response)
                .await;
        }

        // Apply latency simulation
        if config.latency.enabled {
            self.simulate_latency(&config.latency).await;
        }

        modified_response
    }

    /// Check if an error should be simulated
    fn should_simulate_error(&self, config: &ErrorConfig) -> bool {
        let mut rng = rand::thread_rng();
        rng.gen::<f64>() < config.probability
    }

    /// Generate an error response
    async fn generate_error_response(
        &self,
        config: &ErrorConfig,
        request: &HttpRequest,
    ) -> HttpResponse {
        // Check for custom errors first
        for custom_error in &config.custom_errors {
            if self.should_apply_custom_error(custom_error, request) {
                return self.create_custom_error_response(custom_error);
            }
        }

        // Otherwise, use a standard error type
        let error_type = self.select_error_type(&config.error_types);
        self.create_error_response(error_type)
    }

    /// Check if a custom error should be applied
    fn should_apply_custom_error(&self, custom_error: &CustomError, request: &HttpRequest) -> bool {
        // Check probability
        let mut rng = rand::thread_rng();
        if rng.gen::<f64>() >= custom_error.probability {
            return false;
        }

        // Check conditions
        for condition in &custom_error.conditions {
            match condition.condition_type {
                ErrorConditionType::Path => {
                    if request.path != condition.value {
                        return false;
                    }
                }
                ErrorConditionType::Method => {
                    if request.method != condition.value {
                        return false;
                    }
                }
                ErrorConditionType::Header => {
                    let parts: Vec<&str> = condition.value.splitn(2, ":").collect();
                    if parts.len() != 2 {
                        continue;
                    }
                    let header_name = parts[0].trim();
                    let header_value = parts[1].trim();
                    if request.headers.get(header_name) != Some(&header_value.to_string()) {
                        return false;
                    }
                }
                ErrorConditionType::Query => {
                    let parts: Vec<&str> = condition.value.splitn(2, "=").collect();
                    if parts.len() != 2 {
                        continue;
                    }
                    let param_name = parts[0].trim();
                    let param_value = parts[1].trim();
                    if request.query.get(param_name) != Some(&param_value.to_string()) {
                        return false;
                    }
                }
                ErrorConditionType::BodyField => {
                    let parts: Vec<&str> = condition.value.splitn(2, "=").collect();
                    if parts.len() != 2 {
                        continue;
                    }
                    let field_name = parts[0].trim();
                    let field_value = parts[1].trim();

                    if let Some(body) = &request.body {
                        if let Some(obj) = body.as_object() {
                            if let Some(value) = obj.get(field_name) {
                                if value.to_string() != format!("\"{}\"", field_value) {
                                    return false;
                                }
                            } else {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Create a custom error response
    fn create_custom_error_response(&self, custom_error: &CustomError) -> HttpResponse {
        let mut response = HttpResponse::new(custom_error.status);

        // Add headers
        for (key, value) in &custom_error.headers {
            response = response.with_header(key, value);
        }

        // Add body
        let mut body = serde_json::json!({
            "error": custom_error.name,
            "message": custom_error.message,
        });

        if let Some(details) = &custom_error.details {
            if let Some(obj) = body.as_object_mut() {
                obj.insert("details".to_string(), details.clone());
            }
        }

        match response.with_body(body) {
            Ok(resp) => resp,
            Err(_) => {
                // Fallback if serialization fails
                HttpResponse::new(custom_error.status)
                    .with_header("Content-Type", "text/plain")
                    .with_body(custom_error.message.clone())
                    .unwrap_or_else(|_| HttpResponse::new(custom_error.status))
            }
        }
    }

    /// Select an error type based on probabilities
    fn select_error_type(&self, error_types: &HashMap<ErrorType, f64>) -> ErrorType {
        let mut rng = rand::thread_rng();
        let random = rng.gen::<f64>();

        let mut cumulative = 0.0;
        for (error_type, probability) in error_types {
            cumulative += probability;
            if random < cumulative {
                return *error_type;
            }
        }

        // Default to server error if no match (should not happen with proper probabilities)
        ErrorType::ServerError
    }

    /// Create a standard error response
    fn create_error_response(&self, error_type: ErrorType) -> HttpResponse {
        let status = error_type.status_code();
        let message = error_type.default_message();

        HttpResponse::new(status)
            .with_header("Content-Type", "application/json")
            .with_body(serde_json::json!({
                "error": error_type.to_string(),
                "message": message,
            }))
            .unwrap_or_else(|_| {
                // Fallback if serialization fails
                HttpResponse::new(status)
                    .with_header("Content-Type", "text/plain")
                    .with_body(message)
                    .unwrap_or_else(|_| HttpResponse::new(status))
            })
    }

    /// Apply transformations to a response
    async fn apply_transformations(
        &self,
        config: &TransformationConfig,
        request: &HttpRequest,
        response: HttpResponse,
    ) -> HttpResponse {
        let mut modified_response = response;

        for transformation in &config.transformations {
            if self.should_apply_transformation(transformation, request, &modified_response) {
                modified_response = self
                    .apply_transformation(transformation, modified_response)
                    .await;
            }
        }

        modified_response
    }

    /// Check if a transformation should be applied
    fn should_apply_transformation(
        &self,
        transformation: &ResponseTransformation,
        request: &HttpRequest,
        response: &HttpResponse,
    ) -> bool {
        for condition in &transformation.conditions {
            match condition.condition_type {
                TransformationConditionType::Path => {
                    if request.path != condition.value {
                        return false;
                    }
                }
                TransformationConditionType::Method => {
                    if request.method != condition.value {
                        return false;
                    }
                }
                TransformationConditionType::Status => {
                    if response.status.to_string() != condition.value {
                        return false;
                    }
                }
                TransformationConditionType::Header => {
                    let parts: Vec<&str> = condition.value.splitn(2, ":").collect();
                    if parts.len() != 2 {
                        continue;
                    }
                    let header_name = parts[0].trim();
                    let header_value = parts[1].trim();
                    if response.headers.get(header_name) != Some(&header_value.to_string()) {
                        return false;
                    }
                }
                TransformationConditionType::BodyField => {
                    let parts: Vec<&str> = condition.value.splitn(2, "=").collect();
                    if parts.len() != 2 {
                        continue;
                    }
                    let field_name = parts[0].trim();
                    let field_value = parts[1].trim();

                    if let Some(body) = &response.body {
                        if let Some(obj) = body.as_object() {
                            if let Some(value) = obj.get(field_name) {
                                if value.to_string() != format!("\"{}\"", field_value) {
                                    return false;
                                }
                            } else {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Apply a transformation to a response
    async fn apply_transformation(
        &self,
        transformation: &ResponseTransformation,
        response: HttpResponse,
    ) -> HttpResponse {
        let mut modified_response = response;

        match transformation.transformation_type {
            TransformationType::AddHeader => {
                if let (Some(field), Some(value)) = (&transformation.field, &transformation.value) {
                    modified_response = modified_response.with_header(field, value);
                }
            }
            TransformationType::RemoveHeader => {
                if let Some(field) = &transformation.field {
                    modified_response.headers.remove(field);
                }
            }
            TransformationType::ModifyHeader => {
                if let (Some(field), Some(value)) = (&transformation.field, &transformation.value) {
                    if modified_response.headers.contains_key(field) {
                        modified_response
                            .headers
                            .insert(field.clone(), value.clone());
                    }
                }
            }
            TransformationType::AddBodyField => {
                if let (Some(field), Some(value)) = (&transformation.field, &transformation.value) {
                    if let Some(body) = &modified_response.body {
                        if let Some(mut obj) = body.as_object().cloned() {
                            // Try to parse the value as JSON first
                            let field_value = serde_json::from_str(value)
                                .unwrap_or_else(|_| serde_json::Value::String(value.clone()));

                            obj.insert(field.clone(), field_value);

                            // Update the response body
                            if let Ok(updated) = modified_response.with_body(obj) {
                                modified_response = updated;
                            }
                        }
                    } else {
                        // If there's no body, create a new one
                        let mut obj = serde_json::Map::new();

                        // Try to parse the value as JSON first
                        let field_value = serde_json::from_str(value)
                            .unwrap_or_else(|_| serde_json::Value::String(value.clone()));

                        obj.insert(field.clone(), field_value);

                        // Update the response body
                        if let Ok(updated) = modified_response.with_body(obj) {
                            modified_response = updated;
                        }
                    }
                }
            }
            TransformationType::RemoveBodyField => {
                if let Some(field) = &transformation.field {
                    if let Some(body) = &modified_response.body {
                        if let Some(mut obj) = body.as_object().cloned() {
                            obj.remove(field);

                            // Update the response body
                            if let Ok(updated) = modified_response.with_body(obj) {
                                modified_response = updated;
                            }
                        }
                    }
                }
            }
            TransformationType::ModifyBodyField => {
                if let (Some(field), Some(value)) = (&transformation.field, &transformation.value) {
                    if let Some(body) = &modified_response.body {
                        if let Some(mut obj) = body.as_object().cloned() {
                            if obj.contains_key(field) {
                                // Try to parse the value as JSON first
                                let field_value = serde_json::from_str(value)
                                    .unwrap_or_else(|_| serde_json::Value::String(value.clone()));

                                obj.insert(field.clone(), field_value);

                                // Update the response body
                                if let Ok(updated) = modified_response.with_body(obj) {
                                    modified_response = updated;
                                }
                            }
                        }
                    }
                }
            }
            TransformationType::ReplaceBody => {
                if let Some(value) = &transformation.value {
                    // Try to parse the value as JSON first
                    let new_body = serde_json::from_str(value)
                        .unwrap_or_else(|_| serde_json::Value::String(value.clone()));

                    // Update the response body
                    if let Ok(updated) = modified_response.with_body(new_body) {
                        modified_response = updated;
                    }
                }
            }
            TransformationType::ChangeStatus => {
                if let Some(value) = &transformation.value {
                    if let Ok(status) = value.parse::<u16>() {
                        modified_response.status = status;
                    }
                }
            }
        }

        modified_response
    }

    /// Simulate latency
    async fn simulate_latency(&self, config: &LatencyConfig) {
        let latency = if let Some(fixed) = config.fixed_ms {
            fixed
        } else {
            self.generate_random_latency(config)
        };

        tokio::time::sleep(Duration::from_millis(latency)).await;
    }

    /// Generate random latency based on the configuration
    fn generate_random_latency(&self, config: &LatencyConfig) -> u64 {
        let mut rng = rand::thread_rng();

        match config.distribution {
            LatencyDistribution::Uniform => rng.gen_range(config.min_ms..=config.max_ms),
            LatencyDistribution::Normal => {
                // Use a normal distribution centered between min and max
                let mean = (config.min_ms + config.max_ms) as f64 / 2.0;
                let std_dev = (config.max_ms - config.min_ms) as f64 / 6.0; // 3 std devs on each side

                // Generate a value from the normal distribution
                let normal = rand::distributions::Normal::new(mean, std_dev);
                let value = normal.sample(&mut rng);

                // Clamp to the min/max range
                value.max(config.min_ms as f64).min(config.max_ms as f64) as u64
            }
            LatencyDistribution::Exponential => {
                // Use an exponential distribution with lambda = 1 / (mean - min)
                let mean = (config.min_ms + config.max_ms) as f64 / 2.0;
                let lambda = 1.0 / (mean - config.min_ms as f64);

                // Generate a value from the exponential distribution
                let exp = rand::distributions::Exp::new(lambda);
                let value = config.min_ms as f64 + exp.sample(&mut rng);

                // Clamp to the min/max range
                value.max(config.min_ms as f64).min(config.max_ms as f64) as u64
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_latency_simulation() {
        // Create a latency config
        let config = LatencyConfig {
            enabled: true,
            fixed_ms: Some(50),
            min_ms: 10,
            max_ms: 100,
            distribution: LatencyDistribution::Uniform,
        };

        // Create a response behavior handler
        let handler = ResponseBehaviorHandler::new(ResponseBehaviorConfig {
            latency: config,
            ..Default::default()
        });

        // Measure the time it takes to simulate latency
        let start = std::time::Instant::now();
        handler
            .simulate_latency(&handler.config().await.latency)
            .await;
        let elapsed = start.elapsed();

        // Check that the latency was simulated
        assert!(elapsed.as_millis() >= 50);
    }

    #[tokio::test]
    async fn test_error_simulation() {
        // Create an error config
        let mut error_types = HashMap::new();
        error_types.insert(ErrorType::NotFound, 1.0); // Always use NotFound for testing

        let config = ErrorConfig {
            enabled: true,
            probability: 1.0, // Always generate an error
            error_types,
            custom_errors: Vec::new(),
        };

        // Create a response behavior handler
        let handler = ResponseBehaviorHandler::new(ResponseBehaviorConfig {
            error: config,
            ..Default::default()
        });

        // Create a request
        let request = HttpRequest::new("GET", "/api/test");

        // Apply behaviors
        let response = HttpResponse::new(200)
            .with_body(serde_json::json!({"message": "success"}))
            .unwrap();

        let modified = handler.apply(&request, response).await;

        // Check that an error was generated
        assert_eq!(modified.status, 404); // NotFound
        assert!(modified.body.is_some());
        if let Some(body) = modified.body {
            if let Some(obj) = body.as_object() {
                assert_eq!(obj.get("error").unwrap().as_str().unwrap(), "NotFound");
            }
        }
    }

    #[tokio::test]
    async fn test_response_transformation() {
        // Create a transformation config
        let transformations = vec![
            ResponseTransformation {
                name: "Add Header".to_string(),
                transformation_type: TransformationType::AddHeader,
                field: Some("X-Test".to_string()),
                value: Some("test-value".to_string()),
                conditions: Vec::new(),
            },
            ResponseTransformation {
                name: "Add Body Field".to_string(),
                transformation_type: TransformationType::AddBodyField,
                field: Some("added".to_string()),
                value: Some("true".to_string()),
                conditions: Vec::new(),
            },
        ];

        let config = TransformationConfig {
            enabled: true,
            transformations,
        };

        // Create a response behavior handler
        let handler = ResponseBehaviorHandler::new(ResponseBehaviorConfig {
            transformation: config,
            ..Default::default()
        });

        // Create a request and response
        let request = HttpRequest::new("GET", "/api/test");
        let response = HttpResponse::new(200)
            .with_body(serde_json::json!({"message": "success"}))
            .unwrap();

        // Apply transformations
        let modified = handler.apply(&request, response).await;

        // Check that transformations were applied
        assert_eq!(modified.headers.get("X-Test").unwrap(), "test-value");
        if let Some(body) = modified.body {
            if let Some(obj) = body.as_object() {
                assert_eq!(obj.get("added").unwrap().as_str().unwrap(), "true");
                assert_eq!(obj.get("message").unwrap().as_str().unwrap(), "success");
            }
        }
    }
}
