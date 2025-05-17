//! RAG Manager to Persona Layer event definitions and publishers/subscribers
//!
//! This module defines the events that are published by the RAG Manager
//! and subscribed to by the Persona Layer.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::modules::ipc::redis_pubsub::{
    ChannelName, EventPayload, Message, RedisClient, Subscription,
};
use crate::modules::ipc::IpcResult;

/// Document indexed event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentIndexedEvent {
    /// Document ID
    pub document_id: String,

    /// Document title or name
    pub document_name: String,

    /// Number of chunks created
    pub chunk_count: u32,

    /// When the document was indexed
    pub timestamp: DateTime<Utc>,

    /// Document metadata
    pub metadata: HashMap<String, String>,
}

/// Document retrieval event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRetrievalEvent {
    /// Query text
    pub query: String,

    /// IDs of retrieved documents
    pub document_ids: Vec<String>,

    /// Similarity scores for each document
    pub scores: Vec<f32>,

    /// When the retrieval was performed
    pub timestamp: DateTime<Utc>,

    /// User ID (if available)
    pub user_id: Option<String>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Context augmentation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAugmentationEvent {
    /// Original request
    pub original_request: String,

    /// Augmented request
    pub augmented_request: String,

    /// IDs of documents used for augmentation
    pub document_ids: Vec<String>,

    /// When the augmentation was performed
    pub timestamp: DateTime<Utc>,

    /// User ID (if available)
    pub user_id: Option<String>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// RAG Manager event publisher
pub struct RagManagerEventPublisher {
    redis_client: Arc<dyn RedisClient>,
}

impl RagManagerEventPublisher {
    /// Create a new RAG Manager event publisher
    pub fn new(redis_client: Arc<dyn RedisClient>) -> Self {
        Self { redis_client }
    }

    /// Publish a document indexed event
    pub async fn publish_document_indexed(&self, event: DocumentIndexedEvent) -> IpcResult<()> {
        let channel = ChannelName::new("rag_manager", "persona_layer", "document_indexed");
        let payload = event.serialize()?;
        self.redis_client
            .publish(&channel.to_string(), &payload)
            .await
    }

    /// Publish a document retrieval event
    pub async fn publish_document_retrieval(&self, event: DocumentRetrievalEvent) -> IpcResult<()> {
        let channel = ChannelName::new("rag_manager", "persona_layer", "document_retrieval");
        let payload = event.serialize()?;
        self.redis_client
            .publish(&channel.to_string(), &payload)
            .await
    }

    /// Publish a context augmentation event
    pub async fn publish_context_augmentation(
        &self,
        event: ContextAugmentationEvent,
    ) -> IpcResult<()> {
        let channel = ChannelName::new("rag_manager", "persona_layer", "context_augmentation");
        let payload = event.serialize()?;
        self.redis_client
            .publish(&channel.to_string(), &payload)
            .await
    }
}

/// Persona Layer event subscriber
pub struct PersonaLayerEventSubscriber {
    redis_client: Arc<dyn RedisClient>,
}

impl PersonaLayerEventSubscriber {
    /// Create a new Persona Layer event subscriber
    pub fn new(redis_client: Arc<dyn RedisClient>) -> Self {
        Self { redis_client }
    }

    /// Subscribe to document indexed events
    pub async fn subscribe_to_document_indexed(&self) -> IpcResult<DocumentIndexedSubscription> {
        let channel = ChannelName::new("rag_manager", "persona_layer", "document_indexed");
        let subscription = self.redis_client.subscribe(&channel.to_string()).await?;
        Ok(DocumentIndexedSubscription { subscription })
    }

    /// Subscribe to document retrieval events
    pub async fn subscribe_to_document_retrieval(
        &self,
    ) -> IpcResult<DocumentRetrievalSubscription> {
        let channel = ChannelName::new("rag_manager", "persona_layer", "document_retrieval");
        let subscription = self.redis_client.subscribe(&channel.to_string()).await?;
        Ok(DocumentRetrievalSubscription { subscription })
    }

    /// Subscribe to context augmentation events
    pub async fn subscribe_to_context_augmentation(
        &self,
    ) -> IpcResult<ContextAugmentationSubscription> {
        let channel = ChannelName::new("rag_manager", "persona_layer", "context_augmentation");
        let subscription = self.redis_client.subscribe(&channel.to_string()).await?;
        Ok(ContextAugmentationSubscription { subscription })
    }

    /// Subscribe to all RAG Manager events
    pub async fn subscribe_to_all_rag_manager_events(
        &self,
    ) -> IpcResult<AllRagManagerEventsSubscription> {
        let pattern = "intellirouter:rag_manager:persona_layer:*";
        let subscription = self.redis_client.psubscribe(pattern).await?;
        Ok(AllRagManagerEventsSubscription { subscription })
    }
}

/// Document indexed subscription
pub struct DocumentIndexedSubscription {
    subscription: Subscription,
}

impl DocumentIndexedSubscription {
    /// Get the next event from the subscription
    pub async fn next_event(&self) -> IpcResult<Option<DocumentIndexedEvent>> {
        if let Some(message) = self.subscription.next_message().await? {
            let event = DocumentIndexedEvent::deserialize(&message.payload)?;
            Ok(Some(event))
        } else {
            Ok(None)
        }
    }
}

/// Document retrieval subscription
pub struct DocumentRetrievalSubscription {
    subscription: Subscription,
}

impl DocumentRetrievalSubscription {
    /// Get the next event from the subscription
    pub async fn next_event(&self) -> IpcResult<Option<DocumentRetrievalEvent>> {
        if let Some(message) = self.subscription.next_message().await? {
            let event = DocumentRetrievalEvent::deserialize(&message.payload)?;
            Ok(Some(event))
        } else {
            Ok(None)
        }
    }
}

/// Context augmentation subscription
pub struct ContextAugmentationSubscription {
    subscription: Subscription,
}

impl ContextAugmentationSubscription {
    /// Get the next event from the subscription
    pub async fn next_event(&self) -> IpcResult<Option<ContextAugmentationEvent>> {
        if let Some(message) = self.subscription.next_message().await? {
            let event = ContextAugmentationEvent::deserialize(&message.payload)?;
            Ok(Some(event))
        } else {
            Ok(None)
        }
    }
}

/// RAG Manager event
#[derive(Debug, Clone)]
pub enum RagManagerEvent {
    /// Document indexed
    DocumentIndexed(DocumentIndexedEvent),

    /// Document retrieval
    DocumentRetrieval(DocumentRetrievalEvent),

    /// Context augmentation
    ContextAugmentation(ContextAugmentationEvent),
}

/// All RAG Manager events subscription
pub struct AllRagManagerEventsSubscription {
    subscription: Subscription,
}

impl AllRagManagerEventsSubscription {
    /// Get the next event from the subscription
    pub async fn next_event(&self) -> IpcResult<Option<RagManagerEvent>> {
        if let Some(message) = self.subscription.next_message().await? {
            let channel_name = ChannelName::from_string(&message.channel).ok_or_else(|| {
                crate::modules::ipc::IpcError::InvalidArgument(format!(
                    "Invalid channel name: {}",
                    message.channel
                ))
            })?;

            match channel_name.event_type() {
                "document_indexed" => {
                    let event = DocumentIndexedEvent::deserialize(&message.payload)?;
                    Ok(Some(RagManagerEvent::DocumentIndexed(event)))
                }
                "document_retrieval" => {
                    let event = DocumentRetrievalEvent::deserialize(&message.payload)?;
                    Ok(Some(RagManagerEvent::DocumentRetrieval(event)))
                }
                "context_augmentation" => {
                    let event = ContextAugmentationEvent::deserialize(&message.payload)?;
                    Ok(Some(RagManagerEvent::ContextAugmentation(event)))
                }
                _ => {
                    // Unknown event type
                    Ok(None)
                }
            }
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::ipc::redis_pubsub::RedisClientImpl;
    use std::collections::HashMap;
    use tokio::time::{sleep, Duration};

    // This test requires a running Redis instance
    // #[tokio::test]
    // async fn test_rag_manager_events() {
    //     let redis_url = "redis://localhost:6379";
    //     let redis_client = Arc::new(RedisClientImpl::new(redis_url).await.unwrap());
    //
    //     let publisher = RagManagerEventPublisher::new(redis_client.clone());
    //     let subscriber = PersonaLayerEventSubscriber::new(redis_client.clone());
    //
    //     // Subscribe to document indexed events
    //     let subscription = subscriber.subscribe_to_document_indexed().await.unwrap();
    //
    //     // Publish a document indexed event
    //     let event = DocumentIndexedEvent {
    //         document_id: "test-document-id".to_string(),
    //         document_name: "test-document-name".to_string(),
    //         chunk_count: 10,
    //         timestamp: Utc::now(),
    //         metadata: HashMap::new(),
    //     };
    //
    //     publisher.publish_document_indexed(event.clone()).await.unwrap();
    //
    //     // Wait for the event to be published
    //     sleep(Duration::from_millis(100)).await;
    //
    //     // Get the event
    //     let received_event = subscription.next_event().await.unwrap().unwrap();
    //
    //     assert_eq!(received_event.document_id, event.document_id);
    //     assert_eq!(received_event.document_name, event.document_name);
    //     assert_eq!(received_event.chunk_count, event.chunk_count);
    // }
}
