//! Retry and Fallback Mechanisms
//!
//! This module contains the implementation of retry policies, circuit breakers,
//! backoff algorithms, and degraded service modes for the IntelliRouter.

use std::collections::{HashMap, HashSet};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant};

use futures::Future;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::modules::model_registry::{
    connectors::{
        ChatCompletionChoice, ChatCompletionResponse, ChatMessage,
        MessageRole,
    },
    storage::ModelRegistry,
};

use super::{RouterError, RoutingMetadata, RoutingRequest, RoutingResponse};

/// Error category for determining retry behavior
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Network error
    Network,
    /// Authentication error
    Authentication,
    /// Rate limiting error
    RateLimit,
    /// Invalid request error
    InvalidRequest,
    /// Server error
    Server,
    /// Timeout error
    Timeout,
    /// Model not found error
    ModelNotFound,
    /// Other error
    Other,
}

impl RouterError {
    /// Get the error category
    pub fn category(&self) -> ErrorCategory {
        match self {
            RouterError::NoSuitableModel(_) => ErrorCategory::ModelNotFound,
            RouterError::RegistryError(_) => ErrorCategory::Server,
            RouterError::ConnectorError(msg) => {
                if msg.contains("timeout") || msg.contains("timed out") {
                    ErrorCategory::Timeout
                } else if msg.contains("network") || msg.contains("connection") {
                    ErrorCategory::Network
                } else if msg.contains("authentication") || msg.contains("unauthorized") {
                    ErrorCategory::Authentication
                } else if msg.contains("rate limit") || msg.contains("too many requests") {
                    ErrorCategory::RateLimit
                } else if msg.contains("invalid") || msg.contains("bad request") {
                    ErrorCategory::InvalidRequest
                } else if msg.contains("server error") || msg.contains("internal error") {
                    ErrorCategory::Server
                } else {
                    ErrorCategory::Other
                }
            }
            RouterError::StrategyConfigError(_) => ErrorCategory::InvalidRequest,
            RouterError::InvalidRequest(_) => ErrorCategory::InvalidRequest,
            RouterError::Timeout(_) => ErrorCategory::Timeout,
            RouterError::FallbackError(_) => ErrorCategory::Other,
            RouterError::Other(_) => ErrorCategory::Other,
            RouterError::SerializationError(_) => ErrorCategory::Other,
        }
    }

    /// Check if the error is retryable
    pub fn is_retryable(&self, retryable_categories: &HashSet<ErrorCategory>) -> bool {
        retryable_categories.contains(&self.category())
    }
}

/// Retry policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryPolicy {
    /// No retries
    None,
    /// Fixed interval retries
    Fixed {
        /// Retry interval in milliseconds
        interval_ms: u64,
        /// Maximum number of retries
        max_retries: u32,
    },
    /// Exponential backoff retries
    ExponentialBackoff {
        /// Initial retry interval in milliseconds
        initial_interval_ms: u64,
        /// Backoff factor
        backoff_factor: f64,
        /// Maximum number of retries
        max_retries: u32,
        /// Maximum interval in milliseconds
        max_interval_ms: u64,
    },
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::ExponentialBackoff {
            initial_interval_ms: 100,
            backoff_factor: 2.0,
            max_retries: 3,
            max_interval_ms: 5000,
        }
    }
}

/// Circuit breaker state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CircuitBreakerState {
    /// Circuit is closed (normal operation)
    Closed,
    /// Circuit is open (failing fast)
    Open,
    /// Circuit is half-open (testing if it can be closed)
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Failure threshold to open the circuit
    pub failure_threshold: u32,
    /// Success threshold to close the circuit
    pub success_threshold: u32,
    /// Reset timeout in milliseconds
    pub reset_timeout_ms: u64,
    /// Whether to enable the circuit breaker
    pub enabled: bool,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            reset_timeout_ms: 30000, // 30 seconds
            enabled: true,
        }
    }
}

/// Degraded service mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DegradedServiceMode {
    /// Fail fast
    FailFast,
    /// Use a default model
    DefaultModel(String),
    /// Return a static response
    StaticResponse(String),
}

impl Default for DegradedServiceMode {
    fn default() -> Self {
        Self::FailFast
    }
}

/// Circuit breaker
#[derive(Debug)]
pub struct CircuitBreaker {
    /// Configuration
    config: CircuitBreakerConfig,
    /// Current state
    state: Mutex<CircuitBreakerState>,
    /// Failure count
    failure_count: AtomicUsize,
    /// Success count
    success_count: AtomicUsize,
    /// Last failure time
    last_failure_time: Mutex<Instant>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Mutex::new(CircuitBreakerState::Closed),
            failure_count: AtomicUsize::new(0),
            success_count: AtomicUsize::new(0),
            last_failure_time: Mutex::new(Instant::now()),
        }
    }

    /// Check if a request is allowed
    pub fn allow_request(&self) -> bool {
        // If the circuit breaker is disabled, always allow
        if !self.config.enabled {
            return true;
        }

        let state = self.state.lock().unwrap();
        match *state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                // Check if the reset timeout has elapsed
                let last_failure = self.last_failure_time.lock().unwrap();
                let elapsed = last_failure.elapsed().as_millis() as u64;
                elapsed >= self.config.reset_timeout_ms
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }

    /// Record a successful request
    pub fn record_success(&self) {
        // If the circuit breaker is disabled, do nothing
        if !self.config.enabled {
            return;
        }

        let mut state = self.state.lock().unwrap();
        match *state {
            CircuitBreakerState::Closed => {
                // Reset failure count
                self.failure_count.store(0, Ordering::SeqCst);
            }
            CircuitBreakerState::Open => {
                // This shouldn't happen, but just in case
                *state = CircuitBreakerState::HalfOpen;
                self.success_count.store(1, Ordering::SeqCst);
            }
            CircuitBreakerState::HalfOpen => {
                // Increment success count
                let success_count = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;

                // If we've reached the success threshold, close the circuit
                if success_count >= self.config.success_threshold as usize {
                    *state = CircuitBreakerState::Closed;
                    self.success_count.store(0, Ordering::SeqCst);
                    self.failure_count.store(0, Ordering::SeqCst);
                }
            }
        }
    }

    /// Record a failed request
    pub fn record_failure(&self) {
        // If the circuit breaker is disabled, do nothing
        if !self.config.enabled {
            return;
        }

        // Update last failure time
        let mut last_failure = self.last_failure_time.lock().unwrap();
        *last_failure = Instant::now();

        let mut state = self.state.lock().unwrap();
        match *state {
            CircuitBreakerState::Closed => {
                // Increment failure count
                let failure_count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;

                // If we've reached the failure threshold, open the circuit
                if failure_count >= self.config.failure_threshold as usize {
                    *state = CircuitBreakerState::Open;
                    self.failure_count.store(0, Ordering::SeqCst);
                }
            }
            CircuitBreakerState::Open => {
                // Already open, do nothing
            }
            CircuitBreakerState::HalfOpen => {
                // Any failure in half-open state opens the circuit again
                *state = CircuitBreakerState::Open;
                self.success_count.store(0, Ordering::SeqCst);
            }
        }
    }

    /// Get the current state
    pub fn get_state(&self) -> CircuitBreakerState {
        self.state.lock().unwrap().clone()
    }
}

/// Retry manager
#[derive(Debug)]
pub struct RetryManager {
    /// Retry policy
    policy: RetryPolicy,
    /// Circuit breaker
    circuit_breaker: CircuitBreaker,
    /// Retryable error categories
    retryable_errors: HashSet<ErrorCategory>,
}

impl RetryManager {
    /// Create a new retry manager
    pub fn new(
        policy: RetryPolicy,
        circuit_breaker_config: CircuitBreakerConfig,
        retryable_errors: HashSet<ErrorCategory>,
    ) -> Self {
        Self {
            policy,
            circuit_breaker: CircuitBreaker::new(circuit_breaker_config),
            retryable_errors,
        }
    }

    /// Execute a function with retries
    pub async fn execute<F, Fut, T, E>(&self, f: F, context: &str) -> Result<T, RouterError>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: Into<RouterError>,
    {
        // Check circuit breaker
        if !self.circuit_breaker.allow_request() {
            debug!("Circuit breaker is open for {}", context);
            return Err(RouterError::Other(format!(
                "Circuit breaker is open for {}",
                context
            )));
        }

        // Execute with retries
        let result = match &self.policy {
            RetryPolicy::None => {
                // No retries, just execute once
                debug!("No retry policy, executing once for {}", context);
                f().await.map_err(|e| e.into())
            }
            RetryPolicy::Fixed {
                interval_ms,
                max_retries,
            } => {
                // Fixed interval retries
                debug!(
                    "Using fixed retry policy for {}: interval={}ms, max_retries={}",
                    context, interval_ms, max_retries
                );
                let mut attempts = 0;
                let mut last_error = None;

                while attempts <= *max_retries {
                    match f().await {
                        Ok(result) => {
                            // Success, record it and return
                            debug!("Attempt {} succeeded for {}", attempts, context);
                            self.circuit_breaker.record_success();
                            return Ok(result);
                        }
                        Err(e) => {
                            let error = e.into();
                            attempts += 1;

                            // Check if the error is retryable
                            if !error.is_retryable(&self.retryable_errors) {
                                // Not retryable, record failure and return
                                debug!(
                                    "Non-retryable error on attempt {} for {}: {:?}",
                                    attempts, context, error
                                );
                                self.circuit_breaker.record_failure();
                                return Err(error);
                            }

                            // Record the error and retry if we have attempts left
                            debug!(
                                "Retryable error on attempt {} for {}: {:?}",
                                attempts, context, error
                            );
                            last_error = Some(error);
                            if attempts <= *max_retries {
                                debug!("Retrying after {}ms for {}", interval_ms, context);
                                // Wait before retrying
                                tokio::time::sleep(Duration::from_millis(*interval_ms)).await;
                            }
                        }
                    }
                }

                // All retries failed
                debug!("All {} retries failed for {}", max_retries, context);
                self.circuit_breaker.record_failure();
                Err(last_error.unwrap_or_else(|| {
                    RouterError::Other(format!("All retries failed for {}", context))
                }))
            }
            RetryPolicy::ExponentialBackoff {
                initial_interval_ms,
                backoff_factor,
                max_retries,
                max_interval_ms,
            } => {
                // Exponential backoff retries
                debug!("Using exponential backoff retry policy for {}: initial={}ms, factor={}, max_retries={}, max_interval={}ms", 
                       context, initial_interval_ms, backoff_factor, max_retries, max_interval_ms);
                let mut attempts = 0;
                let mut interval_ms = *initial_interval_ms;
                let mut last_error = None;

                while attempts <= *max_retries {
                    match f().await {
                        Ok(result) => {
                            // Success, record it and return
                            debug!("Attempt {} succeeded for {}", attempts, context);
                            self.circuit_breaker.record_success();
                            return Ok(result);
                        }
                        Err(e) => {
                            let error = e.into();
                            attempts += 1;

                            // Check if the error is retryable
                            if !error.is_retryable(&self.retryable_errors) {
                                // Not retryable, record failure and return
                                debug!(
                                    "Non-retryable error on attempt {} for {}: {:?}",
                                    attempts, context, error
                                );
                                self.circuit_breaker.record_failure();
                                return Err(error);
                            }

                            // Record the error and retry if we have attempts left
                            debug!(
                                "Retryable error on attempt {} for {}: {:?}",
                                attempts, context, error
                            );
                            last_error = Some(error);
                            if attempts <= *max_retries {
                                debug!("Retrying after {}ms for {}", interval_ms, context);
                                // Wait before retrying
                                tokio::time::sleep(Duration::from_millis(interval_ms)).await;

                                // Calculate next interval with exponential backoff
                                interval_ms = (interval_ms as f64 * backoff_factor) as u64;
                                if interval_ms > *max_interval_ms {
                                    interval_ms = *max_interval_ms;
                                }
                            }
                        }
                    }
                }

                // All retries failed
                debug!("All {} retries failed for {}", max_retries, context);
                self.circuit_breaker.record_failure();
                Err(last_error.unwrap_or_else(|| {
                    RouterError::Other(format!("All retries failed for {}", context))
                }))
            }
        };

        // Record the result in the circuit breaker
        match &result {
            Ok(_) => self.circuit_breaker.record_success(),
            Err(_) => self.circuit_breaker.record_failure(),
        }

        result
    }

    /// Get the circuit breaker state
    pub fn get_circuit_breaker_state(&self) -> CircuitBreakerState {
        self.circuit_breaker.get_state()
    }

    /// Check if a request is allowed by the circuit breaker
    pub fn allow_request(&self, context: &str) -> bool {
        debug!("Checking if request is allowed for context: {}", context);

        match self.policy {
            RetryPolicy::None => true,
            _ => self.circuit_breaker.allow_request(),
        }
    }

    /// Check if the circuit breaker is open
    pub fn is_circuit_open(&self) -> bool {
        match self.circuit_breaker.get_state() {
            CircuitBreakerState::Open => true,
            _ => false,
        }
    }
}

/// Degraded service handler
#[derive(Debug)]
pub struct DegradedServiceHandler {
    /// Degraded service mode
    mode: DegradedServiceMode,
    /// Model registry
    registry: Arc<ModelRegistry>,
}

impl DegradedServiceHandler {
    /// Create a new degraded service handler
    pub fn new(mode: DegradedServiceMode, registry: Arc<ModelRegistry>) -> Self {
        Self { mode, registry }
    }

    /// Handle a request in degraded service mode
    pub async fn handle_request(
        &self,
        request: &RoutingRequest,
    ) -> Result<RoutingResponse, RouterError> {
        match &self.mode {
            DegradedServiceMode::FailFast => {
                // Just fail fast
                debug!("Degraded service mode: failing fast");
                Err(RouterError::Other(
                    "Service is in degraded mode, failing fast".to_string(),
                ))
            }
            DegradedServiceMode::DefaultModel(model_id) => {
                // Try to use the default model
                debug!("Degraded service mode: using default model {}", model_id);
                let _model = self.registry.get_model(model_id).map_err(|_| {
                    RouterError::NoSuitableModel(format!(
                        "Default model {} not found in degraded mode",
                        model_id
                    ))
                })?;

                // Get the connector
                let connector = self.registry.get_connector(model_id).ok_or_else(|| {
                    RouterError::NoSuitableModel(format!(
                        "No connector found for default model {} in degraded mode",
                        model_id
                    ))
                })?;

                // Create metadata
                let metadata = RoutingMetadata {
                    selected_model_id: model_id.clone(),
                    strategy_name: "degraded_mode".to_string(),
                    routing_start_time: chrono::Utc::now(),
                    routing_end_time: chrono::Utc::now(),
                    routing_time_ms: 0,
                    models_considered: 1,
                    attempts: 1,
                    is_fallback: true,
                    selection_criteria: Some("degraded_mode".to_string()),
                    additional_metadata: {
                        let mut map = HashMap::new();
                        map.insert("degraded_mode".to_string(), "true".to_string());
                        map
                    },
                };

                // Send the request to the model
                let response = connector
                    .generate(request.context.request.clone())
                    .await
                    .map_err(|e| RouterError::ConnectorError(e.to_string()))?;

                // Create routing response
                Ok(RoutingResponse { response, metadata })
            }
            DegradedServiceMode::StaticResponse(response_text) => {
                // Create a static response
                debug!("Degraded service mode: using static response");
                let response = ChatCompletionResponse {
                    id: "degraded-mode".to_string(),
                    model: "degraded-mode".to_string(),
                    created: chrono::Utc::now().timestamp() as u64,
                    choices: vec![ChatCompletionChoice {
                        index: 0,
                        message: ChatMessage {
                            role: MessageRole::Assistant,
                            content: response_text.clone(),
                            name: None,
                            function_call: None,
                            tool_calls: None,
                        },
                        finish_reason: Some("degraded_mode".to_string()),
                    }],
                    usage: None,
                };

                // Create metadata
                let metadata = RoutingMetadata {
                    selected_model_id: "degraded-mode".to_string(),
                    strategy_name: "degraded_mode".to_string(),
                    routing_start_time: chrono::Utc::now(),
                    routing_end_time: chrono::Utc::now(),
                    routing_time_ms: 0,
                    models_considered: 0,
                    attempts: 0,
                    is_fallback: true,
                    selection_criteria: Some("degraded_mode".to_string()),
                    additional_metadata: {
                        let mut map = HashMap::new();
                        map.insert("degraded_mode".to_string(), "true".to_string());
                        map
                    },
                };

                // Create routing response
                Ok(RoutingResponse { response, metadata })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_error_category() {
        // Test NoSuitableModel error
        let error = RouterError::NoSuitableModel("No model found".to_string());
        assert_eq!(error.category(), ErrorCategory::ModelNotFound);

        // Test ConnectorError with timeout
        let error = RouterError::ConnectorError("Request timed out".to_string());
        assert_eq!(error.category(), ErrorCategory::Timeout);

        // Test ConnectorError with network
        let error = RouterError::ConnectorError("Network error".to_string());
        assert_eq!(error.category(), ErrorCategory::Network);

        // Test ConnectorError with authentication
        let error = RouterError::ConnectorError("Authentication failed".to_string());
        assert_eq!(error.category(), ErrorCategory::Authentication);

        // Test ConnectorError with rate limit
        let error = RouterError::ConnectorError("Rate limit exceeded".to_string());
        assert_eq!(error.category(), ErrorCategory::RateLimit);

        // Test ConnectorError with invalid request
        let error = RouterError::ConnectorError("Invalid request".to_string());
        assert_eq!(error.category(), ErrorCategory::InvalidRequest);

        // Test ConnectorError with server error
        let error = RouterError::ConnectorError("Internal server error".to_string());
        assert_eq!(error.category(), ErrorCategory::Server);

        // Test ConnectorError with other error
        let error = RouterError::ConnectorError("Unknown error".to_string());
        assert_eq!(error.category(), ErrorCategory::Other);
    }

    #[test]
    fn test_is_retryable() {
        // Create a set of retryable error categories
        let mut retryable_categories = HashSet::new();
        retryable_categories.insert(ErrorCategory::Network);
        retryable_categories.insert(ErrorCategory::Timeout);
        retryable_categories.insert(ErrorCategory::RateLimit);
        retryable_categories.insert(ErrorCategory::Server);

        // Test retryable errors
        let error = RouterError::ConnectorError("Network error".to_string());
        assert!(error.is_retryable(&retryable_categories));

        let error = RouterError::ConnectorError("Request timed out".to_string());
        assert!(error.is_retryable(&retryable_categories));

        let error = RouterError::ConnectorError("Rate limit exceeded".to_string());
        assert!(error.is_retryable(&retryable_categories));

        let error = RouterError::ConnectorError("Internal server error".to_string());
        assert!(error.is_retryable(&retryable_categories));

        // Test non-retryable errors
        let error = RouterError::ConnectorError("Authentication failed".to_string());
        assert!(!error.is_retryable(&retryable_categories));

        let error = RouterError::ConnectorError("Invalid request".to_string());
        assert!(!error.is_retryable(&retryable_categories));

        let error = RouterError::NoSuitableModel("No model found".to_string());
        assert!(!error.is_retryable(&retryable_categories));
    }

    #[test]
    fn test_circuit_breaker() {
        // Create a circuit breaker with a low threshold for testing
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            reset_timeout_ms: 100,
            enabled: true,
        };

        let circuit_breaker = CircuitBreaker::new(config);

        // Initially, the circuit should be closed
        assert_eq!(circuit_breaker.get_state(), CircuitBreakerState::Closed);
        assert!(circuit_breaker.allow_request());

        // Record a failure
        circuit_breaker.record_failure();
        assert_eq!(circuit_breaker.get_state(), CircuitBreakerState::Closed);
        assert!(circuit_breaker.allow_request());

        // Record another failure to open the circuit
        circuit_breaker.record_failure();
        assert_eq!(circuit_breaker.get_state(), CircuitBreakerState::Open);

        // Wait for the reset timeout
        std::thread::sleep(Duration::from_millis(150));

        // The circuit should now allow a request (half-open)
        assert!(circuit_breaker.allow_request());

        // Record a success
        circuit_breaker.record_success();
        assert_eq!(circuit_breaker.get_state(), CircuitBreakerState::HalfOpen);

        // Record another success to close the circuit
        circuit_breaker.record_success();
        assert_eq!(circuit_breaker.get_state(), CircuitBreakerState::Closed);

        // Test disabled circuit breaker
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            reset_timeout_ms: 100,
            enabled: false,
        };

        let circuit_breaker = CircuitBreaker::new(config);

        // Record multiple failures, but the circuit should remain closed
        circuit_breaker.record_failure();
        circuit_breaker.record_failure();
        circuit_breaker.record_failure();
        assert!(circuit_breaker.allow_request());
    }
}
