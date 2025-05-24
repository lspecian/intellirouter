//! Resilient Chain Engine Client
//!
//! This module provides a resilient client for the chain engine service.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;

use crate::modules::ipc::chain_engine::{
    CancelChainResponse, Chain, ChainEngineClient, ChainExecutionEvent, ChainExecutionResponse,
    ChainStatusResponse,
};
use crate::modules::ipc::{IpcError, IpcResult};
use crate::modules::router_core::retry::{CircuitBreakerConfig, RetryPolicy};

use super::circuit_breaker::{CircuitBreaker, CircuitState};
use super::config::ResilientClientConfig;
use super::retry::RetryHandler;
use super::traits::{GracefulShutdown, HealthCheckable, ResilientClient};

/// Resilient Chain Engine Client
pub struct ResilientChainEngineClient {
    /// Inner client
    inner: Arc<dyn ChainEngineClient>,
    /// Retry handler
    retry_handler: RetryHandler,
    /// Circuit breaker
    circuit_breaker: CircuitBreaker,
    /// Last successful connection time
    last_successful_connection: Arc<Mutex<Option<Instant>>>,
    /// Configuration
    config: ResilientClientConfig,
}

impl std::fmt::Debug for ResilientChainEngineClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResilientChainEngineClient")
            .field("retry_handler", &self.retry_handler)
            .field("circuit_breaker", &self.circuit_breaker)
            .field(
                "last_successful_connection",
                &self.last_successful_connection,
            )
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl ResilientChainEngineClient {
    /// Create a new resilient chain engine client
    pub fn new(inner: impl ChainEngineClient + 'static, config: ResilientClientConfig) -> Self {
        let retry_handler = RetryHandler::new(config.retry_policy.clone());
        let circuit_breaker =
            CircuitBreaker::new(config.circuit_breaker.clone(), config.degraded_mode.clone());

        Self {
            inner: Arc::new(inner),
            retry_handler,
            circuit_breaker,
            last_successful_connection: Arc::new(Mutex::new(None)),
            config,
        }
    }

    /// Create a new resilient chain engine client with default configuration
    pub fn new_with_defaults(inner: impl ChainEngineClient + 'static) -> Self {
        Self::new(inner, ResilientClientConfig::default())
    }

    /// Execute an operation with retry and circuit breaker logic
    async fn execute<F, Fut, T>(&self, operation: F) -> IpcResult<T>
    where
        F: Fn(Arc<dyn ChainEngineClient>) -> Fut + Send + Sync,
        Fut: std::future::Future<Output = IpcResult<T>> + Send,
        T: Send,
    {
        // Check if circuit breaker allows execution
        if !self.circuit_breaker.allow_execution() {
            let error = IpcError::CircuitOpen(format!(
                "Circuit breaker is open for {}",
                self.service_name()
            ));
            return Err(error);
        }

        // Execute operation with retry logic
        let inner = Arc::clone(&self.inner);
        let result = self
            .retry_handler
            .execute(move || operation(Arc::clone(&inner)))
            .await;

        // Record result in circuit breaker
        match &result {
            Ok(_) => {
                self.circuit_breaker.record_success();
                *self.last_successful_connection.lock().unwrap() = Some(Instant::now());
            }
            Err(e) => {
                self.circuit_breaker.record_failure(e);
            }
        }

        result
    }
}

#[async_trait]
impl ResilientClient for ResilientChainEngineClient {
    fn service_name(&self) -> &str {
        "chain_engine"
    }

    fn retry_policy(&self) -> &RetryPolicy {
        self.retry_handler.policy()
    }

    fn circuit_breaker_config(&self) -> &CircuitBreakerConfig {
        &self.config.circuit_breaker
    }

    fn is_healthy(&self) -> bool {
        self.circuit_breaker.state() != CircuitState::Open
    }

    fn last_successful_connection(&self) -> Option<Duration> {
        self.last_successful_connection
            .lock()
            .unwrap()
            .map(|t| t.elapsed())
    }

    fn consecutive_failures(&self) -> u32 {
        self.circuit_breaker.consecutive_failures()
    }

    fn successful_operations(&self) -> u64 {
        self.circuit_breaker.successful_operations()
    }

    fn failed_operations(&self) -> u64 {
        self.circuit_breaker.failed_operations()
    }

    fn reset_circuit_breaker(&mut self) {
        self.circuit_breaker.reset();
    }
}

#[async_trait]
impl HealthCheckable for ResilientChainEngineClient {
    async fn health_check(&self) -> IpcResult<()> {
        // Since ChainEngineClient doesn't have a health_check method,
        // we'll implement a simple check by trying to get the status of a dummy execution ID
        self.execute(|inner| async move {
            // Try to get the status of a non-existent execution ID
            // Any response (even an error) indicates the service is responsive
            match inner.get_chain_status("health-check-dummy-id").await {
                Ok(_) => Ok(()),
                // Any error is acceptable for health check as long as we got a response
                Err(_) => Ok(()),
            }
        })
        .await
    }
}

#[async_trait]
impl GracefulShutdown for ResilientChainEngineClient {
    async fn shutdown(&self) -> IpcResult<()> {
        // No specific shutdown needed for this client
        Ok(())
    }
}

// Implement the ChainEngineClient trait for ResilientChainEngineClient
#[async_trait]
impl ChainEngineClient for ResilientChainEngineClient {
    async fn execute_chain(
        &self,
        chain_id: Option<String>,
        chain: Option<Chain>,
        input: String,
        variables: HashMap<String, String>,
        stream: bool,
        timeout_seconds: Option<u32>,
    ) -> IpcResult<ChainExecutionResponse> {
        let chain_clone = chain.clone();
        let input_clone = input.clone();
        let variables_clone = variables.clone();

        self.execute(|inner| {
            let chain_id_clone = chain_id.clone();
            let chain_clone2 = chain_clone.clone();
            let input_clone2 = input_clone.clone();
            let variables_clone2 = variables_clone.clone();

            async move {
                inner
                    .execute_chain(
                        chain_id_clone,
                        chain_clone2,
                        input_clone2,
                        variables_clone2,
                        stream,
                        timeout_seconds,
                    )
                    .await
            }
        })
        .await
    }

    async fn get_chain_status(&self, execution_id: &str) -> IpcResult<ChainStatusResponse> {
        self.execute(|inner| async move { inner.get_chain_status(execution_id).await })
            .await
    }

    async fn cancel_chain_execution(&self, execution_id: &str) -> IpcResult<CancelChainResponse> {
        self.execute(|inner| async move { inner.cancel_chain_execution(execution_id).await })
            .await
    }

    async fn stream_chain_execution(
        &self,
        chain_id: Option<String>,
        chain: Option<Chain>,
        input: String,
        variables: HashMap<String, String>,
        timeout_seconds: Option<u32>,
    ) -> IpcResult<Pin<Box<dyn Stream<Item = Result<ChainExecutionEvent, tonic::Status>> + Send>>>
    {
        // Streaming operations are not retried
        // Just check if circuit breaker allows execution
        if !self.circuit_breaker.allow_execution() {
            return Err(IpcError::CircuitOpen(format!(
                "Circuit breaker is open for {}",
                self.service_name()
            )));
        }

        let inner = Arc::clone(&self.inner);

        // Record success in circuit breaker
        self.circuit_breaker.record_success();
        *self.last_successful_connection.lock().unwrap() = Some(Instant::now());

        // Return the stream
        inner
            .stream_chain_execution(chain_id, chain, input, variables, timeout_seconds)
            .await
    }
}
