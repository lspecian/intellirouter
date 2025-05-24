//! Tests for resilient IPC clients
//!
//! This module contains tests for the resilient IPC clients.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tokio::sync::Mutex;

use crate::modules::ipc::chain_engine::{
    CancelChainResponse, Chain, ChainEngineClient, ChainExecutionEvent, ChainExecutionResponse,
    ChainStatusResponse, Status,
};
use crate::modules::ipc::resilient_client::{
    default_circuit_breaker_config, default_retryable_error_categories, ResilientChainEngineClient,
};
use crate::modules::ipc::{IpcError, IpcResult};
use crate::modules::router_core::retry::RetryPolicy;

// Mock Chain Engine Client for testing
struct MockChainEngineClient {
    fail_count: Arc<Mutex<usize>>,
    max_fails: usize,
    delay_ms: u64,
}

impl MockChainEngineClient {
    fn new(max_fails: usize, delay_ms: u64) -> Self {
        Self {
            fail_count: Arc::new(Mutex::new(0)),
            max_fails,
            delay_ms,
        }
    }
}

#[async_trait::async_trait]
impl ChainEngineClient for MockChainEngineClient {
    async fn execute_chain(
        &self,
        _chain_id: Option<String>,
        _chain: Option<Chain>,
        input: String,
        _variables: HashMap<String, String>,
        _stream: bool,
        _timeout_seconds: Option<u32>,
    ) -> IpcResult<ChainExecutionResponse> {
        // Simulate network delay
        tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;

        // Increment fail count and check if we should fail
        let mut fail_count = self.fail_count.lock().await;
        if *fail_count < self.max_fails {
            *fail_count += 1;
            return Err(IpcError::Connection("Simulated network error".to_string()));
        }

        // After max_fails attempts, succeed
        Ok(ChainExecutionResponse {
            execution_id: "test-execution-id".to_string(),
            status: Status::Success,
            output: format!("Processed: {}", input),
            error: None,
            start_time: Utc::now(),
            end_time: Utc::now(),
            step_results: Vec::new(),
            total_tokens: 100,
            metadata: HashMap::new(),
        })
    }

    async fn get_chain_status(&self, _execution_id: &str) -> IpcResult<ChainStatusResponse> {
        // Simulate network delay
        tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;

        // Increment fail count and check if we should fail
        let mut fail_count = self.fail_count.lock().await;
        if *fail_count < self.max_fails {
            *fail_count += 1;
            return Err(IpcError::Timeout("Simulated timeout error".to_string()));
        }

        // After max_fails attempts, succeed
        Ok(ChainStatusResponse {
            execution_id: "test-execution-id".to_string(),
            status: Status::Success,
            current_step_id: Some("test-step-1".to_string()),
            completed_steps: 1,
            total_steps: 3,
            error: None,
            start_time: Utc::now(),
            update_time: Utc::now(),
        })
    }

    async fn cancel_chain_execution(&self, _execution_id: &str) -> IpcResult<CancelChainResponse> {
        // Simulate network delay
        tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;

        // Increment fail count and check if we should fail
        let mut fail_count = self.fail_count.lock().await;
        if *fail_count < self.max_fails {
            *fail_count += 1;
            return Err(IpcError::Connection("Simulated network error".to_string()));
        }

        // After max_fails attempts, succeed
        Ok(CancelChainResponse {
            execution_id: "test-execution-id".to_string(),
            success: true,
            error: None,
        })
    }

    async fn stream_chain_execution(
        &self,
        _chain_id: Option<String>,
        _chain: Option<Chain>,
        _input: String,
        _variables: HashMap<String, String>,
        _timeout_seconds: Option<u32>,
    ) -> IpcResult<
        Pin<Box<dyn futures::Stream<Item = Result<ChainExecutionEvent, tonic::Status>> + Send>>,
    > {
        // Return an empty stream for testing
        let stream = futures::stream::empty();
        Ok(Box::pin(stream))
    }
}

#[tokio::test]
async fn test_resilient_client_retries_and_succeeds() {
    // Create a mock client that will fail twice and then succeed
    let mock_client = MockChainEngineClient::new(2, 50);

    // Create a resilient client with exponential backoff retry policy
    let retry_policy = RetryPolicy::ExponentialBackoff {
        initial_interval_ms: 100,
        backoff_factor: 2.0,
        max_retries: 3,
        max_interval_ms: 5000,
    };

    let resilient_client = ResilientChainEngineClient::new(
        mock_client,
        retry_policy,
        default_circuit_breaker_config(),
        default_retryable_error_categories(),
    );

    // Execute a chain and expect it to succeed after retries
    let result = resilient_client
        .execute_chain(
            Some("test-chain".to_string()),
            None,
            "Test input".to_string(),
            HashMap::new(),
            false,
            None,
        )
        .await;

    // Assert that the operation succeeded
    assert!(result.is_ok(), "Expected success, got: {:?}", result);
    let response = result.unwrap();
    assert_eq!(response.status, Status::Success);
    assert_eq!(response.output, "Processed: Test input");
}

#[tokio::test]
async fn test_resilient_client_fails_after_max_retries() {
    // Create a mock client that will always fail
    let mock_client = MockChainEngineClient::new(10, 50); // More than our retry limit

    // Create a resilient client with a limited retry policy
    let retry_policy = RetryPolicy::Fixed {
        interval_ms: 100,
        max_retries: 2, // Only retry twice
    };

    let resilient_client = ResilientChainEngineClient::new(
        mock_client,
        retry_policy,
        default_circuit_breaker_config(),
        default_retryable_error_categories(),
    );

    // Execute a chain and expect it to fail after max retries
    let result = resilient_client
        .execute_chain(
            Some("test-chain".to_string()),
            None,
            "Test input".to_string(),
            HashMap::new(),
            false,
            None,
        )
        .await;

    // Assert that the operation failed
    assert!(result.is_err(), "Expected failure, got: {:?}", result);
    match result {
        Err(IpcError::Connection(msg)) => {
            assert!(msg.contains("Simulated network error"));
        }
        _ => panic!("Expected Connection error, got: {:?}", result),
    }
}

#[tokio::test]
async fn test_circuit_breaker_opens_after_failures() {
    // Create a mock client that will always fail
    let mock_client = MockChainEngineClient::new(10, 50);

    // Create a circuit breaker config with a low failure threshold
    let circuit_breaker_config = crate::modules::router_core::retry::CircuitBreakerConfig {
        failure_threshold: 2, // Open after 2 failures
        success_threshold: 1,
        reset_timeout_ms: 500, // 500ms reset timeout
        enabled: true,
    };

    // Create a resilient client with the circuit breaker config
    let resilient_client = ResilientChainEngineClient::new(
        mock_client,
        RetryPolicy::None, // No retries, just circuit breaking
        circuit_breaker_config,
        default_retryable_error_categories(),
    );

    // First request should fail but not trip the circuit breaker
    let result1 = resilient_client
        .execute_chain(
            Some("test-chain".to_string()),
            None,
            "Test input 1".to_string(),
            HashMap::new(),
            false,
            None,
        )
        .await;
    assert!(result1.is_err());

    // Second request should fail and trip the circuit breaker
    let result2 = resilient_client
        .execute_chain(
            Some("test-chain".to_string()),
            None,
            "Test input 2".to_string(),
            HashMap::new(),
            false,
            None,
        )
        .await;
    assert!(result2.is_err());

    // Third request should fail immediately due to open circuit breaker
    let result3 = resilient_client
        .execute_chain(
            Some("test-chain".to_string()),
            None,
            "Test input 3".to_string(),
            HashMap::new(),
            false,
            None,
        )
        .await;
    assert!(result3.is_err());
    match result3 {
        Err(IpcError::Internal(msg)) => {
            assert!(msg.contains("Circuit breaker is open"));
        }
        _ => panic!(
            "Expected Internal error with circuit breaker message, got: {:?}",
            result3
        ),
    }

    // Wait for the circuit breaker to reset
    tokio::time::sleep(Duration::from_millis(600)).await;

    // Now the circuit breaker should be half-open and allow a test request
    // But our mock client will still fail, so the circuit breaker should open again
    let result4 = resilient_client
        .execute_chain(
            Some("test-chain".to_string()),
            None,
            "Test input 4".to_string(),
            HashMap::new(),
            false,
            None,
        )
        .await;
    assert!(result4.is_err());

    // Circuit breaker should be open again
    let result5 = resilient_client
        .execute_chain(
            Some("test-chain".to_string()),
            None,
            "Test input 5".to_string(),
            HashMap::new(),
            false,
            None,
        )
        .await;
    assert!(result5.is_err());
    match result5 {
        Err(IpcError::Internal(msg)) => {
            assert!(msg.contains("Circuit breaker is open"));
        }
        _ => panic!(
            "Expected Internal error with circuit breaker message, got: {:?}",
            result5
        ),
    }
}
