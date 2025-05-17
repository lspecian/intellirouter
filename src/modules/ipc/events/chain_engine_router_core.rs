//! Chain Engine to Router Core event definitions and publishers/subscribers
//!
//! This module defines the events that are published by the Chain Engine
//! and subscribed to by the Router Core.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::modules::ipc::chain_engine::{ChainExecutionEventData, ErrorDetails, Status};
use crate::modules::ipc::redis_pubsub::{
    ChannelName, EventPayload, Message, RedisClient, Subscription,
};
use crate::modules::ipc::IpcResult;

/// Chain execution completed event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainExecutionCompletedEvent {
    /// Execution ID
    pub execution_id: String,

    /// Output of the chain
    pub output: String,

    /// Total tokens used
    pub total_tokens: u32,

    /// Execution time in milliseconds
    pub execution_time_ms: u64,

    /// When the execution completed
    pub timestamp: DateTime<Utc>,

    /// Additional metadata about the execution
    pub metadata: HashMap<String, String>,
}

/// Chain execution failed event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainExecutionFailedEvent {
    /// Execution ID
    pub execution_id: String,

    /// Error details
    pub error: ErrorDetails,

    /// Execution time in milliseconds
    pub execution_time_ms: u64,

    /// When the execution failed
    pub timestamp: DateTime<Utc>,

    /// Additional metadata about the execution
    pub metadata: HashMap<String, String>,
}

/// Chain step completed event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStepCompletedEvent {
    /// Execution ID
    pub execution_id: String,

    /// Step ID
    pub step_id: String,

    /// Step index
    pub step_index: u32,

    /// Output from the step
    pub output: String,

    /// Tokens used by this step
    pub tokens: u32,

    /// When the step completed
    pub timestamp: DateTime<Utc>,

    /// Additional metadata about the step
    pub metadata: HashMap<String, String>,
}

/// Chain Engine event publisher
pub struct ChainEngineEventPublisher {
    redis_client: Arc<dyn RedisClient>,
}

impl ChainEngineEventPublisher {
    /// Create a new Chain Engine event publisher
    pub fn new(redis_client: Arc<dyn RedisClient>) -> Self {
        Self { redis_client }
    }

    /// Publish a chain execution completed event
    pub async fn publish_chain_execution_completed(
        &self,
        event: ChainExecutionCompletedEvent,
    ) -> IpcResult<()> {
        let channel = ChannelName::new("chain_engine", "router_core", "chain_execution_completed");
        let payload = EventPayload::serialize(&event)?;
        self.redis_client
            .publish(&channel.to_string(), &payload)
            .await
    }

    /// Publish a chain execution failed event
    pub async fn publish_chain_execution_failed(
        &self,
        event: ChainExecutionFailedEvent,
    ) -> IpcResult<()> {
        let channel = ChannelName::new("chain_engine", "router_core", "chain_execution_failed");
        let payload = EventPayload::serialize(&event)?;
        self.redis_client
            .publish(&channel.to_string(), &payload)
            .await
    }

    /// Publish a chain step completed event
    pub async fn publish_chain_step_completed(
        &self,
        event: ChainStepCompletedEvent,
    ) -> IpcResult<()> {
        let channel = ChannelName::new("chain_engine", "router_core", "chain_step_completed");
        let payload = EventPayload::serialize(&event)?;
        self.redis_client
            .publish(&channel.to_string(), &payload)
            .await
    }

    /// Helper method to publish a chain execution event from the Chain Engine's event data
    pub async fn publish_chain_execution_event(
        &self,
        execution_id: &str,
        event_data: ChainExecutionEventData,
        timestamp: DateTime<Utc>,
        metadata: HashMap<String, String>,
    ) -> IpcResult<()> {
        match event_data {
            ChainExecutionEventData::ChainCompleted {
                output,
                total_tokens,
                execution_time_ms,
            } => {
                let event = ChainExecutionCompletedEvent {
                    execution_id: execution_id.to_string(),
                    output,
                    total_tokens,
                    execution_time_ms,
                    timestamp,
                    metadata,
                };
                self.publish_chain_execution_completed(event).await
            }
            ChainExecutionEventData::ChainFailed {
                error,
                execution_time_ms,
            } => {
                let event = ChainExecutionFailedEvent {
                    execution_id: execution_id.to_string(),
                    error,
                    execution_time_ms,
                    timestamp,
                    metadata,
                };
                self.publish_chain_execution_failed(event).await
            }
            ChainExecutionEventData::StepCompleted {
                step_id,
                step_index,
                output,
                tokens,
            } => {
                let event = ChainStepCompletedEvent {
                    execution_id: execution_id.to_string(),
                    step_id,
                    step_index,
                    output,
                    tokens,
                    timestamp,
                    metadata,
                };
                self.publish_chain_step_completed(event).await
            }
            _ => Ok(()), // Other event types are not published to Router Core
        }
    }
}

/// Chain Engine event subscriber
pub struct RouterCoreEventSubscriber {
    redis_client: Arc<dyn RedisClient>,
}

impl RouterCoreEventSubscriber {
    /// Create a new Router Core event subscriber
    pub fn new(redis_client: Arc<dyn RedisClient>) -> Self {
        Self { redis_client }
    }

    /// Subscribe to chain execution completed events
    pub async fn subscribe_to_chain_execution_completed(
        &self,
    ) -> IpcResult<ChainExecutionCompletedSubscription> {
        let channel = ChannelName::new("chain_engine", "router_core", "chain_execution_completed");
        let subscription = self.redis_client.subscribe(&channel.to_string()).await?;
        Ok(ChainExecutionCompletedSubscription { subscription })
    }

    /// Subscribe to chain execution failed events
    pub async fn subscribe_to_chain_execution_failed(
        &self,
    ) -> IpcResult<ChainExecutionFailedSubscription> {
        let channel = ChannelName::new("chain_engine", "router_core", "chain_execution_failed");
        let subscription = self.redis_client.subscribe(&channel.to_string()).await?;
        Ok(ChainExecutionFailedSubscription { subscription })
    }

    /// Subscribe to chain step completed events
    pub async fn subscribe_to_chain_step_completed(
        &self,
    ) -> IpcResult<ChainStepCompletedSubscription> {
        let channel = ChannelName::new("chain_engine", "router_core", "chain_step_completed");
        let subscription = self.redis_client.subscribe(&channel.to_string()).await?;
        Ok(ChainStepCompletedSubscription { subscription })
    }

    /// Subscribe to all Chain Engine events
    pub async fn subscribe_to_all_chain_engine_events(
        &self,
    ) -> IpcResult<AllChainEngineEventsSubscription> {
        let pattern = "intellirouter:chain_engine:router_core:*";
        let subscription = self.redis_client.psubscribe(pattern).await?;
        Ok(AllChainEngineEventsSubscription { subscription })
    }
}

/// Chain execution completed subscription
pub struct ChainExecutionCompletedSubscription {
    subscription: Subscription,
}

impl ChainExecutionCompletedSubscription {
    /// Get the next event from the subscription
    pub async fn next_event(&self) -> IpcResult<Option<ChainExecutionCompletedEvent>> {
        if let Some(message) = self.subscription.next_message().await? {
            let event = EventPayload::deserialize(&message.payload)?;
            Ok(Some(event))
        } else {
            Ok(None)
        }
    }
}

/// Chain execution failed subscription
pub struct ChainExecutionFailedSubscription {
    subscription: Subscription,
}

impl ChainExecutionFailedSubscription {
    /// Get the next event from the subscription
    pub async fn next_event(&self) -> IpcResult<Option<ChainExecutionFailedEvent>> {
        if let Some(message) = self.subscription.next_message().await? {
            let event = EventPayload::deserialize(&message.payload)?;
            Ok(Some(event))
        } else {
            Ok(None)
        }
    }
}

/// Chain step completed subscription
pub struct ChainStepCompletedSubscription {
    subscription: Subscription,
}

impl ChainStepCompletedSubscription {
    /// Get the next event from the subscription
    pub async fn next_event(&self) -> IpcResult<Option<ChainStepCompletedEvent>> {
        if let Some(message) = self.subscription.next_message().await? {
            let event = EventPayload::deserialize(&message.payload)?;
            Ok(Some(event))
        } else {
            Ok(None)
        }
    }
}

/// Chain Engine event
#[derive(Debug, Clone)]
pub enum ChainEngineEvent {
    /// Chain execution completed
    ChainExecutionCompleted(ChainExecutionCompletedEvent),

    /// Chain execution failed
    ChainExecutionFailed(ChainExecutionFailedEvent),

    /// Chain step completed
    ChainStepCompleted(ChainStepCompletedEvent),
}

/// All Chain Engine events subscription
pub struct AllChainEngineEventsSubscription {
    subscription: Subscription,
}

impl AllChainEngineEventsSubscription {
    /// Get the next event from the subscription
    pub async fn next_event(&self) -> IpcResult<Option<ChainEngineEvent>> {
        if let Some(message) = self.subscription.next_message().await? {
            let channel_name = ChannelName::from_string(&message.channel).ok_or_else(|| {
                crate::modules::ipc::IpcError::InvalidArgument(format!(
                    "Invalid channel name: {}",
                    message.channel
                ))
            })?;

            match channel_name.event_type() {
                "chain_execution_completed" => {
                    let event = EventPayload::deserialize(&message.payload)?;
                    Ok(Some(ChainEngineEvent::ChainExecutionCompleted(event)))
                }
                "chain_execution_failed" => {
                    let event = EventPayload::deserialize(&message.payload)?;
                    Ok(Some(ChainEngineEvent::ChainExecutionFailed(event)))
                }
                "chain_step_completed" => {
                    let event = EventPayload::deserialize(&message.payload)?;
                    Ok(Some(ChainEngineEvent::ChainStepCompleted(event)))
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
    // async fn test_chain_engine_events() {
    //     let redis_url = "redis://localhost:6379";
    //     let redis_client = Arc::new(RedisClientImpl::new(redis_url).await.unwrap());
    //
    //     let publisher = ChainEngineEventPublisher::new(redis_client.clone());
    //     let subscriber = RouterCoreEventSubscriber::new(redis_client.clone());
    //
    //     // Subscribe to chain execution completed events
    //     let subscription = subscriber.subscribe_to_chain_execution_completed().await.unwrap();
    //
    //     // Publish a chain execution completed event
    //     let event = ChainExecutionCompletedEvent {
    //         execution_id: "test-execution-id".to_string(),
    //         output: "test-output".to_string(),
    //         total_tokens: 100,
    //         execution_time_ms: 1000,
    //         timestamp: Utc::now(),
    //         metadata: HashMap::new(),
    //     };
    //
    //     publisher.publish_chain_execution_completed(event.clone()).await.unwrap();
    //
    //     // Wait for the event to be published
    //     sleep(Duration::from_millis(100)).await;
    //
    //     // Get the event
    //     let received_event = subscription.next_event().await.unwrap().unwrap();
    //
    //     assert_eq!(received_event.execution_id, event.execution_id);
    //     assert_eq!(received_event.output, event.output);
    //     assert_eq!(received_event.total_tokens, event.total_tokens);
    //     assert_eq!(received_event.execution_time_ms, event.execution_time_ms);
    // }
}
