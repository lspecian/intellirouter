//! Resilient Model Registry Client
//!
//! This module provides a resilient client for the model registry service.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;

use crate::modules::ipc::model_registry::{
    HealthCheckResult, ModelFilter, ModelMetadata, ModelRegistryClient,
    ModelStatus,
};
use crate::modules::ipc::{IpcError, IpcResult};
use crate::modules::router_core::retry::{CircuitBreakerConfig, RetryPolicy};

use super::circuit_breaker::{CircuitBreaker, CircuitState};
use super::config::ResilientClientConfig;
use super::retry::RetryHandler;
use super::traits::{GracefulShutdown, HealthCheckable, ResilientClient};

/// Resilient Model Registry Client
pub struct ResilientModelRegistryClient {
    /// Inner client
    inner: Arc<dyn ModelRegistryClient>,
    /// Retry handler
    retry_handler: RetryHandler,
    /// Circuit breaker
    circuit_breaker: CircuitBreaker,
    /// Last successful connection time
    last_successful_connection: Arc<Mutex<Option<Instant>>>,
    /// Configuration
    config: ResilientClientConfig,
}

impl std::fmt::Debug for ResilientModelRegistryClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResilientModelRegistryClient")
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

impl ResilientModelRegistryClient {
    /// Create a new resilient model registry client
    pub fn new(inner: impl ModelRegistryClient + 'static, config: ResilientClientConfig) -> Self {
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

    /// Create a new resilient model registry client with default configuration
    pub fn new_with_defaults(inner: impl ModelRegistryClient + 'static) -> Self {
        Self::new(inner, ResilientClientConfig::default())
    }

    /// Execute an operation with retry and circuit breaker logic
    async fn execute<F, Fut, T>(&self, operation: F) -> IpcResult<T>
    where
        F: Fn(Arc<dyn ModelRegistryClient>) -> Fut + Send + Sync,
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
impl ResilientClient for ResilientModelRegistryClient {
    fn service_name(&self) -> &str {
        "model_registry"
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
impl HealthCheckable for ResilientModelRegistryClient {
    async fn health_check(&self) -> IpcResult<()> {
        // Since ModelRegistryClient doesn't have a health_check method,
        // we'll implement a simple check by trying to list models
        self.execute(|inner| async move {
            // Try to list models to check if the service is responsive
            inner.list_models().await?;
            Ok(())
        })
        .await
    }
}

#[async_trait]
impl GracefulShutdown for ResilientModelRegistryClient {
    async fn shutdown(&self) -> IpcResult<()> {
        // No specific shutdown needed for this client
        Ok(())
    }
}

// Implement the ModelRegistryClient trait for ResilientModelRegistryClient
#[async_trait]
impl ModelRegistryClient for ResilientModelRegistryClient {
    async fn register_model(&self, metadata: ModelMetadata) -> IpcResult<ModelMetadata> {
        let metadata_clone = metadata.clone();
        self.execute(|inner| {
            let metadata_clone2 = metadata_clone.clone();
            async move { inner.register_model(metadata_clone2).await }
        })
        .await
    }

    async fn get_model(&self, model_id: &str) -> IpcResult<ModelMetadata> {
        self.execute(|inner| async move { inner.get_model(model_id).await })
            .await
    }

    async fn update_model(&self, metadata: ModelMetadata) -> IpcResult<ModelMetadata> {
        let metadata_clone = metadata.clone();
        self.execute(|inner| {
            let metadata_clone2 = metadata_clone.clone();
            async move { inner.update_model(metadata_clone2).await }
        })
        .await
    }

    async fn remove_model(&self, model_id: &str) -> IpcResult<ModelMetadata> {
        self.execute(|inner| async move { inner.remove_model(model_id).await })
            .await
    }

    async fn list_models(&self) -> IpcResult<Vec<ModelMetadata>> {
        self.execute(|inner| async move { inner.list_models().await })
            .await
    }

    async fn find_models(&self, filter: ModelFilter) -> IpcResult<Vec<ModelMetadata>> {
        let filter_clone = filter.clone();
        self.execute(|inner| {
            let filter_clone2 = filter_clone.clone();
            async move { inner.find_models(filter_clone2).await }
        })
        .await
    }

    async fn update_model_status(
        &self,
        model_id: &str,
        status: ModelStatus,
        reason: &str,
    ) -> IpcResult<ModelMetadata> {
        self.execute(
            |inner| async move { inner.update_model_status(model_id, status, reason).await },
        )
        .await
    }

    async fn check_model_health(
        &self,
        model_id: &str,
        timeout_ms: Option<u32>,
    ) -> IpcResult<HealthCheckResult> {
        self.execute(|inner| async move { inner.check_model_health(model_id, timeout_ms).await })
            .await
    }
}
