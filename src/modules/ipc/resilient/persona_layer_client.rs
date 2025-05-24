//! Resilient Persona Layer Client
//!
//! This module provides a resilient client for the persona layer service.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;

use crate::modules::ipc::persona_layer::{
    ApplyPersonaResponse, ListPersonasResponse, Persona, PersonaLayerClient,
};
use crate::modules::ipc::{IpcError, IpcResult};
use crate::modules::router_core::retry::{CircuitBreakerConfig, RetryPolicy};
use std::collections::HashMap;

use super::circuit_breaker::{CircuitBreaker, CircuitState};
use super::config::ResilientClientConfig;
use super::retry::RetryHandler;
use super::traits::{GracefulShutdown, HealthCheckable, ResilientClient};

/// Resilient Persona Layer Client
pub struct ResilientPersonaLayerClient {
    /// Inner client
    inner: Arc<dyn PersonaLayerClient>,
    /// Retry handler
    retry_handler: RetryHandler,
    /// Circuit breaker
    circuit_breaker: CircuitBreaker,
    /// Last successful connection time
    last_successful_connection: Arc<Mutex<Option<Instant>>>,
    /// Configuration
    config: ResilientClientConfig,
}

impl std::fmt::Debug for ResilientPersonaLayerClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResilientPersonaLayerClient")
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

impl ResilientPersonaLayerClient {
    /// Create a new resilient persona layer client
    pub fn new(inner: impl PersonaLayerClient + 'static, config: ResilientClientConfig) -> Self {
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

    /// Create a new resilient persona layer client with default configuration
    pub fn new_with_defaults(inner: impl PersonaLayerClient + 'static) -> Self {
        Self::new(inner, ResilientClientConfig::default())
    }

    /// Execute an operation with retry and circuit breaker logic
    async fn execute<F, Fut, T>(&self, operation: F) -> IpcResult<T>
    where
        F: Fn(Arc<dyn PersonaLayerClient>) -> Fut + Send + Sync,
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
impl ResilientClient for ResilientPersonaLayerClient {
    fn service_name(&self) -> &str {
        "persona_layer"
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
impl HealthCheckable for ResilientPersonaLayerClient {
    async fn health_check(&self) -> IpcResult<()> {
        // Since PersonaLayerClient doesn't have a health_check method,
        // we'll implement a simple check by trying to list personas
        self.execute(|inner| async move {
            // Try to list personas with a limit of 1 just to check if the service is responsive
            inner.list_personas(Some(1), Some(0), None).await?;
            Ok(())
        })
        .await
    }
}

#[async_trait]
impl GracefulShutdown for ResilientPersonaLayerClient {
    async fn shutdown(&self) -> IpcResult<()> {
        // No specific shutdown needed for this client
        Ok(())
    }
}

// Implement the PersonaLayerClient trait for ResilientPersonaLayerClient
#[async_trait]
impl PersonaLayerClient for ResilientPersonaLayerClient {
    async fn create_persona(
        &self,
        name: &str,
        description: &str,
        system_prompt: &str,
        response_format: Option<&str>,
        metadata: HashMap<String, String>,
        tags: Vec<String>,
    ) -> IpcResult<Persona> {
        let metadata_clone = metadata.clone();
        let tags_clone = tags.clone();

        self.execute(|inner| {
            let metadata_clone2 = metadata_clone.clone();
            let tags_clone2 = tags_clone.clone();

            async move {
                inner
                    .create_persona(
                        name,
                        description,
                        system_prompt,
                        response_format,
                        metadata_clone2,
                        tags_clone2,
                    )
                    .await
            }
        })
        .await
    }

    async fn get_persona(&self, persona_id: &str) -> IpcResult<Persona> {
        self.execute(|inner| async move { inner.get_persona(persona_id).await })
            .await
    }

    async fn update_persona(
        &self,
        persona_id: &str,
        name: Option<&str>,
        description: Option<&str>,
        system_prompt: Option<&str>,
        response_format: Option<&str>,
        metadata: Option<HashMap<String, String>>,
        tags: Option<Vec<String>>,
    ) -> IpcResult<Persona> {
        let metadata_clone = metadata.clone();
        let tags_clone = tags.clone();

        self.execute(|inner| {
            let metadata_clone2 = metadata_clone.clone();
            let tags_clone2 = tags_clone.clone();

            async move {
                inner
                    .update_persona(
                        persona_id,
                        name,
                        description,
                        system_prompt,
                        response_format,
                        metadata_clone2,
                        tags_clone2,
                    )
                    .await
            }
        })
        .await
    }

    async fn delete_persona(&self, persona_id: &str) -> IpcResult<()> {
        self.execute(|inner| async move { inner.delete_persona(persona_id).await })
            .await
    }

    async fn list_personas(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
        tag_filter: Option<Vec<String>>,
    ) -> IpcResult<ListPersonasResponse> {
        let tag_filter_clone = tag_filter.clone();
        self.execute(|inner| {
            let tag_filter_clone2 = tag_filter_clone.clone();
            async move { inner.list_personas(limit, offset, tag_filter_clone2).await }
        })
        .await
    }

    async fn apply_persona(
        &self,
        persona_id: Option<&str>,
        persona: Option<Persona>,
        request: &str,
        additional_context: Option<&str>,
        include_description: bool,
    ) -> IpcResult<ApplyPersonaResponse> {
        let persona_clone = persona.clone();

        self.execute(|inner| {
            let persona_clone2 = persona_clone.clone();

            async move {
                inner
                    .apply_persona(
                        persona_id,
                        persona_clone2,
                        request,
                        additional_context,
                        include_description,
                    )
                    .await
            }
        })
        .await
    }
}
