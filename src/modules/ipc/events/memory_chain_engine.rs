//! Memory to Chain Engine event definitions and publishers/subscribers
//!
//! This module defines the events that are published by the Memory module
//! and subscribed to by the Chain Engine.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::modules::ipc::redis_pubsub::{
    ChannelName, EventPayload, Message, RedisClient, Subscription,
};
use crate::modules::ipc::IpcResult;

/// Conversation message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    /// Message ID
    pub id: String,

    /// Role of the message sender (e.g., "user", "assistant", "system")
    pub role: String,

    /// Content of the message
    pub content: String,

    /// When the message was created
    pub timestamp: DateTime<Utc>,

    /// Additional metadata about the message
    pub metadata: HashMap<String, String>,

    /// Parent message ID (for threaded conversations)
    pub parent_id: Option<String>,

    /// Token count (if available)
    pub token_count: Option<u32>,
}

/// Conversation updated event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationUpdatedEvent {
    /// Conversation ID
    pub conversation_id: String,

    /// New message added to the conversation
    pub new_message: ConversationMessage,

    /// Total number of messages in the conversation
    pub message_count: u32,

    /// When the conversation was updated
    pub timestamp: DateTime<Utc>,

    /// User ID associated with the conversation
    pub user_id: Option<String>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Conversation history retrieved event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationHistoryRetrievedEvent {
    /// Conversation ID
    pub conversation_id: String,

    /// Messages in the conversation history
    pub messages: Vec<ConversationMessage>,

    /// Total token count
    pub total_tokens: u32,

    /// When the history was retrieved
    pub timestamp: DateTime<Utc>,

    /// User ID associated with the conversation
    pub user_id: Option<String>,

    /// Format of the history (e.g., "openai", "anthropic", "raw")
    pub format: String,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Memory event publisher
pub struct MemoryEventPublisher {
    redis_client: Arc<dyn RedisClient>,
}

impl MemoryEventPublisher {
    /// Create a new Memory event publisher
    pub fn new(redis_client: Arc<dyn RedisClient>) -> Self {
        Self { redis_client }
    }

    /// Publish a conversation updated event
    pub async fn publish_conversation_updated(
        &self,
        event: ConversationUpdatedEvent,
    ) -> IpcResult<()> {
        let channel = ChannelName::new("memory", "chain_engine", "conversation_updated");
        let payload = event.serialize()?;
        self.redis_client
            .publish(&channel.to_string(), &payload)
            .await
    }

    /// Publish a conversation history retrieved event
    pub async fn publish_conversation_history_retrieved(
        &self,
        event: ConversationHistoryRetrievedEvent,
    ) -> IpcResult<()> {
        let channel = ChannelName::new("memory", "chain_engine", "conversation_history_retrieved");
        let payload = event.serialize()?;
        self.redis_client
            .publish(&channel.to_string(), &payload)
            .await
    }
}

/// Chain Engine event subscriber
pub struct ChainEngineMemorySubscriber {
    redis_client: Arc<dyn RedisClient>,
}

impl ChainEngineMemorySubscriber {
    /// Create a new Chain Engine memory subscriber
    pub fn new(redis_client: Arc<dyn RedisClient>) -> Self {
        Self { redis_client }
    }

    /// Subscribe to conversation updated events
    pub async fn subscribe_to_conversation_updated(
        &self,
    ) -> IpcResult<ConversationUpdatedSubscription> {
        let channel = ChannelName::new("memory", "chain_engine", "conversation_updated");
        let subscription = self.redis_client.subscribe(&channel.to_string()).await?;
        Ok(ConversationUpdatedSubscription { subscription })
    }

    /// Subscribe to conversation history retrieved events
    pub async fn subscribe_to_conversation_history_retrieved(
        &self,
    ) -> IpcResult<ConversationHistoryRetrievedSubscription> {
        let channel = ChannelName::new("memory", "chain_engine", "conversation_history_retrieved");
        let subscription = self.redis_client.subscribe(&channel.to_string()).await?;
        Ok(ConversationHistoryRetrievedSubscription { subscription })
    }

    /// Subscribe to all Memory events
    pub async fn subscribe_to_all_memory_events(&self) -> IpcResult<AllMemoryEventsSubscription> {
        let pattern = "intellirouter:memory:chain_engine:*";
        let subscription = self.redis_client.psubscribe(pattern).await?;
        Ok(AllMemoryEventsSubscription { subscription })
    }
}

/// Conversation updated subscription
pub struct ConversationUpdatedSubscription {
    subscription: Subscription,
}

impl ConversationUpdatedSubscription {
    /// Get the next event from the subscription
    pub async fn next_event(&self) -> IpcResult<Option<ConversationUpdatedEvent>> {
        if let Some(message) = self.subscription.next_message().await? {
            let event = ConversationUpdatedEvent::deserialize(&message.payload)?;
            Ok(Some(event))
        } else {
            Ok(None)
        }
    }
}

/// Conversation history retrieved subscription
pub struct ConversationHistoryRetrievedSubscription {
    subscription: Subscription,
}

impl ConversationHistoryRetrievedSubscription {
    /// Get the next event from the subscription
    pub async fn next_event(&self) -> IpcResult<Option<ConversationHistoryRetrievedEvent>> {
        if let Some(message) = self.subscription.next_message().await? {
            let event = ConversationHistoryRetrievedEvent::deserialize(&message.payload)?;
            Ok(Some(event))
        } else {
            Ok(None)
        }
    }
}

/// Memory event
#[derive(Debug, Clone)]
pub enum MemoryEvent {
    /// Conversation updated
    ConversationUpdated(ConversationUpdatedEvent),

    /// Conversation history retrieved
    ConversationHistoryRetrieved(ConversationHistoryRetrievedEvent),
}

/// All Memory events subscription
pub struct AllMemoryEventsSubscription {
    subscription: Subscription,
}

impl AllMemoryEventsSubscription {
    /// Get the next event from the subscription
    pub async fn next_event(&self) -> IpcResult<Option<MemoryEvent>> {
        if let Some(message) = self.subscription.next_message().await? {
            let channel_name = ChannelName::from_string(&message.channel).ok_or_else(|| {
                crate::modules::ipc::IpcError::InvalidArgument(format!(
                    "Invalid channel name: {}",
                    message.channel
                ))
            })?;

            match channel_name.event_type() {
                "conversation_updated" => {
                    let event = ConversationUpdatedEvent::deserialize(&message.payload)?;
                    Ok(Some(MemoryEvent::ConversationUpdated(event)))
                }
                "conversation_history_retrieved" => {
                    let event = ConversationHistoryRetrievedEvent::deserialize(&message.payload)?;
                    Ok(Some(MemoryEvent::ConversationHistoryRetrieved(event)))
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
    // async fn test_memory_events() {
    //     let redis_url = "redis://localhost:6379";
    //     let redis_client = Arc::new(RedisClientImpl::new(redis_url).await.unwrap());
    //
    //     let publisher = MemoryEventPublisher::new(redis_client.clone());
    //     let subscriber = ChainEngineMemorySubscriber::new(redis_client.clone());
    //
    //     // Subscribe to conversation updated events
    //     let subscription = subscriber.subscribe_to_conversation_updated().await.unwrap();
    //
    //     // Create a test message
    //     let message = ConversationMessage {
    //         id: "test-message-id".to_string(),
    //         role: "user".to_string(),
    //         content: "Hello, world!".to_string(),
    //         timestamp: Utc::now(),
    //         metadata: HashMap::new(),
    //         parent_id: None,
    //         token_count: Some(3),
    //     };
    //
    //     // Publish a conversation updated event
    //     let event = ConversationUpdatedEvent {
    //         conversation_id: "test-conversation-id".to_string(),
    //         new_message: message.clone(),
    //         message_count: 1,
    //         timestamp: Utc::now(),
    //         user_id: Some("test-user-id".to_string()),
    //         metadata: HashMap::new(),
    //     };
    //
    //     publisher.publish_conversation_updated(event.clone()).await.unwrap();
    //
    //     // Wait for the event to be published
    //     sleep(Duration::from_millis(100)).await;
    //
    //     // Get the event
    //     let received_event = subscription.next_event().await.unwrap().unwrap();
    //
    //     assert_eq!(received_event.conversation_id, event.conversation_id);
    //     assert_eq!(received_event.new_message.id, event.new_message.id);
    //     assert_eq!(received_event.new_message.role, event.new_message.role);
    //     assert_eq!(received_event.new_message.content, event.new_message.content);
    //     assert_eq!(received_event.message_count, event.message_count);
    // }
}
