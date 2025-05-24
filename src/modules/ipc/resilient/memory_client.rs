//! Resilient Memory Client
//!
//! This module provides a resilient client for the memory service.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::modules::ipc::memory::{
    Conversation as Memory, GetHistoryResponse, ListConversationsResponse, MemoryClient, Message, SearchMessagesResponse,
};
use crate::modules::ipc::{IpcError, IpcResult};
use crate::modules::router_core::retry::{CircuitBreakerConfig, RetryPolicy};

use super::circuit_breaker::{CircuitBreaker, CircuitState};
use super::config::ResilientClientConfig;
use super::retry::RetryHandler;
use super::traits::{GracefulShutdown, HealthCheckable, ResilientClient};

/// Resilient Memory Client
pub struct ResilientMemoryClient {
    /// Inner client
    inner: Arc<dyn MemoryClient>,
    /// Retry handler
    retry_handler: RetryHandler,
    /// Circuit breaker
    circuit_breaker: CircuitBreaker,
    /// Last successful connection time
    last_successful_connection: Arc<Mutex<Option<Instant>>>,
    /// Configuration
    config: ResilientClientConfig,
}

impl std::fmt::Debug for ResilientMemoryClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResilientMemoryClient")
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

impl ResilientMemoryClient {
    /// Create a new resilient memory client
    pub fn new(inner: impl MemoryClient + 'static, config: ResilientClientConfig) -> Self {
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

    /// Create a new resilient memory client with default configuration
    pub fn new_with_defaults(inner: impl MemoryClient + 'static) -> Self {
        Self::new(inner, ResilientClientConfig::default())
    }

    /// Execute an operation with retry and circuit breaker logic
    async fn execute<F, Fut, T>(&self, operation: F) -> IpcResult<T>
    where
        F: Fn(Arc<dyn MemoryClient>) -> Fut + Send + Sync,
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
impl ResilientClient for ResilientMemoryClient {
    fn service_name(&self) -> &str {
        "memory"
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
impl HealthCheckable for ResilientMemoryClient {
    async fn health_check(&self) -> IpcResult<()> {
        // Since MemoryClient doesn't have a health_check method,
        // we'll implement a simple check by trying to list conversations
        self.execute(|inner| async move {
            // Try to list conversations with a limit of 1 just to check if the service is responsive
            inner
                .list_conversations(Some(1), Some(0), None, vec![], None, None)
                .await?;
            Ok(())
        })
        .await
    }
}

#[async_trait]
impl GracefulShutdown for ResilientMemoryClient {
    async fn shutdown(&self) -> IpcResult<()> {
        // No specific shutdown needed for this client
        Ok(())
    }
}

// Implement the MemoryClient trait for ResilientMemoryClient
#[async_trait]
impl MemoryClient for ResilientMemoryClient {
    async fn create_conversation(
        &self,
        metadata: HashMap<String, String>,
        user_id: &str,
        title: Option<&str>,
        tags: Vec<String>,
        initial_messages: Vec<Message>,
    ) -> IpcResult<Memory> {
        let metadata_clone = metadata.clone();
        let tags_clone = tags.clone();
        let initial_messages_clone = initial_messages.clone();

        self.execute(|inner| {
            let metadata_clone2 = metadata_clone.clone();
            let tags_clone2 = tags_clone.clone();
            let initial_messages_clone2 = initial_messages_clone.clone();

            async move {
                inner
                    .create_conversation(
                        metadata_clone2,
                        user_id,
                        title,
                        tags_clone2,
                        initial_messages_clone2,
                    )
                    .await
            }
        })
        .await
    }

    async fn get_conversation(&self, conversation_id: &str) -> IpcResult<Memory> {
        self.execute(|inner| async move { inner.get_conversation(conversation_id).await })
            .await
    }

    async fn add_message(
        &self,
        conversation_id: &str,
        role: &str,
        content: &str,
        metadata: HashMap<String, String>,
        parent_id: Option<&str>,
    ) -> IpcResult<Message> {
        let metadata_clone = metadata.clone();

        self.execute(|inner| {
            let metadata_clone2 = metadata_clone.clone();

            async move {
                inner
                    .add_message(conversation_id, role, content, metadata_clone2, parent_id)
                    .await
            }
        })
        .await
    }

    async fn get_history(
        &self,
        conversation_id: &str,
        max_tokens: Option<u32>,
        max_messages: Option<u32>,
        include_system_messages: bool,
        format: Option<&str>,
    ) -> IpcResult<GetHistoryResponse> {
        self.execute(|inner| async move {
            inner
                .get_history(
                    conversation_id,
                    max_tokens,
                    max_messages,
                    include_system_messages,
                    format,
                )
                .await
        })
        .await
    }

    async fn save_conversation(&self, conversation_id: &str) -> IpcResult<()> {
        self.execute(|inner| async move { inner.save_conversation(conversation_id).await })
            .await
    }

    async fn load_conversation(&self, conversation_id: &str) -> IpcResult<Memory> {
        self.execute(|inner| async move { inner.load_conversation(conversation_id).await })
            .await
    }

    async fn delete_conversation(&self, conversation_id: &str) -> IpcResult<()> {
        self.execute(|inner| async move { inner.delete_conversation(conversation_id).await })
            .await
    }

    async fn list_conversations(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
        user_id: Option<&str>,
        tag_filter: Vec<String>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> IpcResult<ListConversationsResponse> {
        let tag_filter_clone = tag_filter.clone();

        self.execute(|inner| {
            let tag_filter_clone2 = tag_filter_clone.clone();

            async move {
                inner
                    .list_conversations(
                        limit,
                        offset,
                        user_id,
                        tag_filter_clone2,
                        start_date,
                        end_date,
                    )
                    .await
            }
        })
        .await
    }

    async fn search_messages(
        &self,
        query: &str,
        limit: Option<u32>,
        offset: Option<u32>,
        conversation_id: Option<&str>,
        user_id: Option<&str>,
        role: Option<&str>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> IpcResult<SearchMessagesResponse> {
        self.execute(|inner| async move {
            inner
                .search_messages(
                    query,
                    limit,
                    offset,
                    conversation_id,
                    user_id,
                    role,
                    start_date,
                    end_date,
                )
                .await
        })
        .await
    }
}
