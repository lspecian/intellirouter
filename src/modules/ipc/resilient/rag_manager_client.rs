//! Resilient RAG Manager Client
//!
//! This module provides a resilient client for the RAG manager service.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;

use crate::modules::ipc::rag_manager::{
    AugmentRequestResponse, Document, IndexDocumentResponse, ListDocumentsResponse,
    RAGManagerClient, RetrieveDocumentsResponse,
};
use crate::modules::ipc::{IpcError, IpcResult};
use crate::modules::router_core::retry::{CircuitBreakerConfig, RetryPolicy};
use std::collections::HashMap;

use super::circuit_breaker::{CircuitBreaker, CircuitState};
use super::config::ResilientClientConfig;
use super::retry::RetryHandler;
use super::traits::{GracefulShutdown, HealthCheckable, ResilientClient};

/// Resilient RAG Manager Client
pub struct ResilientRAGManagerClient {
    /// Inner client
    inner: Arc<dyn RAGManagerClient>,
    /// Retry handler
    retry_handler: RetryHandler,
    /// Circuit breaker
    circuit_breaker: CircuitBreaker,
    /// Last successful connection time
    last_successful_connection: Arc<Mutex<Option<Instant>>>,
    /// Configuration
    config: ResilientClientConfig,
}

impl std::fmt::Debug for ResilientRAGManagerClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResilientRAGManagerClient")
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

impl ResilientRAGManagerClient {
    /// Create a new resilient RAG manager client
    pub fn new(inner: impl RAGManagerClient + 'static, config: ResilientClientConfig) -> Self {
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

    /// Create a new resilient RAG manager client with default configuration
    pub fn new_with_defaults(inner: impl RAGManagerClient + 'static) -> Self {
        Self::new(inner, ResilientClientConfig::default())
    }

    /// Execute an operation with retry and circuit breaker logic
    async fn execute<F, Fut, T>(&self, operation: F) -> IpcResult<T>
    where
        F: Fn(Arc<dyn RAGManagerClient>) -> Fut + Send + Sync,
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
impl ResilientClient for ResilientRAGManagerClient {
    fn service_name(&self) -> &str {
        "rag_manager"
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
impl HealthCheckable for ResilientRAGManagerClient {
    async fn health_check(&self) -> IpcResult<()> {
        // Since RAGManagerClient doesn't have a health_check method,
        // we'll implement a simple check by trying to list documents
        self.execute(|inner| async move {
            // Try to list documents with a limit of 1 just to check if the service is responsive
            inner.list_documents(Some(1), Some(0), None).await?;
            Ok(())
        })
        .await
    }
}

#[async_trait]
impl GracefulShutdown for ResilientRAGManagerClient {
    async fn shutdown(&self) -> IpcResult<()> {
        // No specific shutdown needed for this client
        Ok(())
    }
}

// Implement the RAGManagerClient trait for ResilientRAGManagerClient
#[async_trait]
impl RAGManagerClient for ResilientRAGManagerClient {
    async fn index_document(
        &self,
        document: Document,
        chunk_size: Option<u32>,
        chunk_overlap: Option<u32>,
        compute_embeddings: bool,
        embedding_model: Option<&str>,
    ) -> IpcResult<IndexDocumentResponse> {
        let document_clone = document.clone();

        self.execute(|inner| {
            let document_clone2 = document_clone.clone();

            async move {
                inner
                    .index_document(
                        document_clone2,
                        chunk_size,
                        chunk_overlap,
                        compute_embeddings,
                        embedding_model,
                    )
                    .await
            }
        })
        .await
    }

    async fn retrieve_documents(
        &self,
        query: &str,
        top_k: Option<u32>,
        min_score: Option<f32>,
        metadata_filter: Option<HashMap<String, String>>,
        include_content: bool,
        rerank: bool,
        rerank_model: Option<&str>,
    ) -> IpcResult<RetrieveDocumentsResponse> {
        let metadata_filter_clone = metadata_filter.clone();

        self.execute(|inner| {
            let metadata_filter_clone2 = metadata_filter_clone.clone();

            async move {
                inner
                    .retrieve_documents(
                        query,
                        top_k,
                        min_score,
                        metadata_filter_clone2,
                        include_content,
                        rerank,
                        rerank_model,
                    )
                    .await
            }
        })
        .await
    }

    async fn augment_request(
        &self,
        request: &str,
        top_k: Option<u32>,
        min_score: Option<f32>,
        metadata_filter: Option<HashMap<String, String>>,
        include_citations: bool,
        max_context_length: Option<u32>,
        context_template: Option<&str>,
    ) -> IpcResult<AugmentRequestResponse> {
        let metadata_filter_clone = metadata_filter.clone();

        self.execute(|inner| {
            let metadata_filter_clone2 = metadata_filter_clone.clone();

            async move {
                inner
                    .augment_request(
                        request,
                        top_k,
                        min_score,
                        metadata_filter_clone2,
                        include_citations,
                        max_context_length,
                        context_template,
                    )
                    .await
            }
        })
        .await
    }

    async fn get_document_by_id(
        &self,
        document_id: &str,
        include_chunks: bool,
    ) -> IpcResult<Document> {
        self.execute(
            |inner| async move { inner.get_document_by_id(document_id, include_chunks).await },
        )
        .await
    }

    async fn delete_document(&self, document_id: &str) -> IpcResult<()> {
        self.execute(|inner| async move { inner.delete_document(document_id).await })
            .await
    }

    async fn list_documents(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
        metadata_filter: Option<HashMap<String, String>>,
    ) -> IpcResult<ListDocumentsResponse> {
        self.execute(|inner| {
            let metadata_filter_clone = metadata_filter.clone();
            async move {
                inner
                    .list_documents(limit, offset, metadata_filter_clone)
                    .await
            }
        })
        .await
    }
}
