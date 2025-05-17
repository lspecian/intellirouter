//! Router Core to Model Registry event definitions and publishers/subscribers
//!
//! This module defines the events that are published by the Router Core
//! and subscribed to by the Model Registry.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::modules::ipc::redis_pubsub::{
    ChannelName, EventPayload, Message, RedisClient, Subscription,
};
use crate::modules::ipc::IpcResult;

/// Model usage event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsageEvent {
    /// Model ID
    pub model_id: String,

    /// Request ID
    pub request_id: String,

    /// User ID (if available)
    pub user_id: Option<String>,

    /// Organization ID (if available)
    pub org_id: Option<String>,

    /// Input tokens used
    pub input_tokens: u32,

    /// Output tokens generated
    pub output_tokens: u32,

    /// Latency in milliseconds
    pub latency_ms: u64,

    /// When the model was used
    pub timestamp: DateTime<Utc>,

    /// Whether the request was successful
    pub success: bool,

    /// Error message (if not successful)
    pub error_message: Option<String>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Model health check event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelHealthCheckEvent {
    /// Model ID
    pub model_id: String,

    /// Whether the model is healthy
    pub healthy: bool,

    /// Latency in milliseconds
    pub latency_ms: u64,

    /// Error message (if not healthy)
    pub error_message: Option<String>,

    /// When the health check was performed
    pub timestamp: DateTime<Utc>,

    /// Additional details about the health check
    pub details: HashMap<String, String>,
}

/// Model routing decision event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRoutingDecisionEvent {
    /// Request ID
    pub request_id: String,

    /// User ID (if available)
    pub user_id: Option<String>,

    /// Organization ID (if available)
    pub org_id: Option<String>,

    /// Selected model ID
    pub selected_model_id: String,

    /// Routing strategy used
    pub routing_strategy: String,

    /// Candidate model IDs that were considered
    pub candidate_model_ids: Vec<String>,

    /// Reason for the selection
    pub selection_reason: String,

    /// When the routing decision was made
    pub timestamp: DateTime<Utc>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Router Core event publisher
pub struct RouterCoreEventPublisher {
    redis_client: Arc<dyn RedisClient>,
}

impl RouterCoreEventPublisher {
    /// Create a new Router Core event publisher
    pub fn new(redis_client: Arc<dyn RedisClient>) -> Self {
        Self { redis_client }
    }

    /// Publish a model usage event
    pub async fn publish_model_usage(&self, event: ModelUsageEvent) -> IpcResult<()> {
        let channel = ChannelName::new("router_core", "model_registry", "model_usage");
        let payload = event.serialize()?;
        self.redis_client
            .publish(&channel.to_string(), &payload)
            .await
    }

    /// Publish a model health check event
    pub async fn publish_model_health_check(&self, event: ModelHealthCheckEvent) -> IpcResult<()> {
        let channel = ChannelName::new("router_core", "model_registry", "model_health_check");
        let payload = event.serialize()?;
        self.redis_client
            .publish(&channel.to_string(), &payload)
            .await
    }

    /// Publish a model routing decision event
    pub async fn publish_model_routing_decision(
        &self,
        event: ModelRoutingDecisionEvent,
    ) -> IpcResult<()> {
        let channel = ChannelName::new("router_core", "model_registry", "model_routing_decision");
        let payload = event.serialize()?;
        self.redis_client
            .publish(&channel.to_string(), &payload)
            .await
    }
}

/// Model Registry event subscriber
pub struct ModelRegistryEventSubscriber {
    redis_client: Arc<dyn RedisClient>,
}

impl ModelRegistryEventSubscriber {
    /// Create a new Model Registry event subscriber
    pub fn new(redis_client: Arc<dyn RedisClient>) -> Self {
        Self { redis_client }
    }

    /// Subscribe to model usage events
    pub async fn subscribe_to_model_usage(&self) -> IpcResult<ModelUsageSubscription> {
        let channel = ChannelName::new("router_core", "model_registry", "model_usage");
        let subscription = self.redis_client.subscribe(&channel.to_string()).await?;
        Ok(ModelUsageSubscription { subscription })
    }

    /// Subscribe to model health check events
    pub async fn subscribe_to_model_health_check(&self) -> IpcResult<ModelHealthCheckSubscription> {
        let channel = ChannelName::new("router_core", "model_registry", "model_health_check");
        let subscription = self.redis_client.subscribe(&channel.to_string()).await?;
        Ok(ModelHealthCheckSubscription { subscription })
    }

    /// Subscribe to model routing decision events
    pub async fn subscribe_to_model_routing_decision(
        &self,
    ) -> IpcResult<ModelRoutingDecisionSubscription> {
        let channel = ChannelName::new("router_core", "model_registry", "model_routing_decision");
        let subscription = self.redis_client.subscribe(&channel.to_string()).await?;
        Ok(ModelRoutingDecisionSubscription { subscription })
    }

    /// Subscribe to all Router Core events
    pub async fn subscribe_to_all_router_core_events(
        &self,
    ) -> IpcResult<AllRouterCoreEventsSubscription> {
        let pattern = "intellirouter:router_core:model_registry:*";
        let subscription = self.redis_client.psubscribe(pattern).await?;
        Ok(AllRouterCoreEventsSubscription { subscription })
    }
}

/// Model usage subscription
pub struct ModelUsageSubscription {
    subscription: Subscription,
}

impl ModelUsageSubscription {
    /// Get the next event from the subscription
    pub async fn next_event(&self) -> IpcResult<Option<ModelUsageEvent>> {
        if let Some(message) = self.subscription.next_message().await? {
            let event = ModelUsageEvent::deserialize(&message.payload)?;
            Ok(Some(event))
        } else {
            Ok(None)
        }
    }
}

/// Model health check subscription
pub struct ModelHealthCheckSubscription {
    subscription: Subscription,
}

impl ModelHealthCheckSubscription {
    /// Get the next event from the subscription
    pub async fn next_event(&self) -> IpcResult<Option<ModelHealthCheckEvent>> {
        if let Some(message) = self.subscription.next_message().await? {
            let event = ModelHealthCheckEvent::deserialize(&message.payload)?;
            Ok(Some(event))
        } else {
            Ok(None)
        }
    }
}

/// Model routing decision subscription
pub struct ModelRoutingDecisionSubscription {
    subscription: Subscription,
}

impl ModelRoutingDecisionSubscription {
    /// Get the next event from the subscription
    pub async fn next_event(&self) -> IpcResult<Option<ModelRoutingDecisionEvent>> {
        if let Some(message) = self.subscription.next_message().await? {
            let event = ModelRoutingDecisionEvent::deserialize(&message.payload)?;
            Ok(Some(event))
        } else {
            Ok(None)
        }
    }
}

/// Router Core event
#[derive(Debug, Clone)]
pub enum RouterCoreEvent {
    /// Model usage
    ModelUsage(ModelUsageEvent),

    /// Model health check
    ModelHealthCheck(ModelHealthCheckEvent),

    /// Model routing decision
    ModelRoutingDecision(ModelRoutingDecisionEvent),
}

/// All Router Core events subscription
pub struct AllRouterCoreEventsSubscription {
    subscription: Subscription,
}

impl AllRouterCoreEventsSubscription {
    /// Get the next event from the subscription
    pub async fn next_event(&self) -> IpcResult<Option<RouterCoreEvent>> {
        if let Some(message) = self.subscription.next_message().await? {
            let channel_name = ChannelName::from_string(&message.channel).ok_or_else(|| {
                crate::modules::ipc::IpcError::InvalidArgument(format!(
                    "Invalid channel name: {}",
                    message.channel
                ))
            })?;

            match channel_name.event_type() {
                "model_usage" => {
                    let event = ModelUsageEvent::deserialize(&message.payload)?;
                    Ok(Some(RouterCoreEvent::ModelUsage(event)))
                }
                "model_health_check" => {
                    let event = ModelHealthCheckEvent::deserialize(&message.payload)?;
                    Ok(Some(RouterCoreEvent::ModelHealthCheck(event)))
                }
                "model_routing_decision" => {
                    let event = ModelRoutingDecisionEvent::deserialize(&message.payload)?;
                    Ok(Some(RouterCoreEvent::ModelRoutingDecision(event)))
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
    // async fn test_router_core_events() {
    //     let redis_url = "redis://localhost:6379";
    //     let redis_client = Arc::new(RedisClientImpl::new(redis_url).await.unwrap());
    //
    //     let publisher = RouterCoreEventPublisher::new(redis_client.clone());
    //     let subscriber = ModelRegistryEventSubscriber::new(redis_client.clone());
    //
    //     // Subscribe to model usage events
    //     let subscription = subscriber.subscribe_to_model_usage().await.unwrap();
    //
    //     // Publish a model usage event
    //     let event = ModelUsageEvent {
    //         model_id: "test-model-id".to_string(),
    //         request_id: "test-request-id".to_string(),
    //         user_id: Some("test-user-id".to_string()),
    //         org_id: None,
    //         input_tokens: 10,
    //         output_tokens: 20,
    //         latency_ms: 100,
    //         timestamp: Utc::now(),
    //         success: true,
    //         error_message: None,
    //         metadata: HashMap::new(),
    //     };
    //
    //     publisher.publish_model_usage(event.clone()).await.unwrap();
    //
    //     // Wait for the event to be published
    //     sleep(Duration::from_millis(100)).await;
    //
    //     // Get the event
    //     let received_event = subscription.next_event().await.unwrap().unwrap();
    //
    //     assert_eq!(received_event.model_id, event.model_id);
    //     assert_eq!(received_event.request_id, event.request_id);
    //     assert_eq!(received_event.user_id, event.user_id);
    //     assert_eq!(received_event.input_tokens, event.input_tokens);
    //     assert_eq!(received_event.output_tokens, event.output_tokens);
    //     assert_eq!(received_event.latency_ms, event.latency_ms);
    //     assert_eq!(received_event.success, event.success);
    // }
}
