//! Redis Pub/Sub infrastructure for asynchronous event propagation
//!
//! This module provides a Redis-based implementation of the pub/sub pattern
//! for asynchronous communication between IntelliRouter modules.

use async_trait::async_trait;
use futures::Stream;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_stream::wrappers::ReceiverStream;

use crate::modules::ipc::{IpcError, IpcResult};

/// Channel naming convention for Redis pub/sub channels
///
/// Format: `intellirouter:{source_module}:{destination_module}:{event_type}`
/// Example: `intellirouter:chain_engine:router_core:chain_execution_completed`
#[derive(Debug, Clone)]
pub struct ChannelName {
    source_module: String,
    destination_module: String,
    event_type: String,
}

impl ChannelName {
    /// Create a new channel name
    pub fn new(source_module: &str, destination_module: &str, event_type: &str) -> Self {
        Self {
            source_module: source_module.to_string(),
            destination_module: destination_module.to_string(),
            event_type: event_type.to_string(),
        }
    }

    /// Get the full channel name as a string
    pub fn to_string(&self) -> String {
        format!(
            "intellirouter:{}:{}:{}",
            self.source_module, self.destination_module, self.event_type
        )
    }

    /// Parse a channel name from a string
    pub fn from_string(channel: &str) -> Option<Self> {
        let parts: Vec<&str> = channel.split(':').collect();
        if parts.len() == 4 && parts[0] == "intellirouter" {
            Some(Self {
                source_module: parts[1].to_string(),
                destination_module: parts[2].to_string(),
                event_type: parts[3].to_string(),
            })
        } else {
            None
        }
    }

    /// Get the source module
    pub fn source_module(&self) -> &str {
        &self.source_module
    }

    /// Get the destination module
    pub fn destination_module(&self) -> &str {
        &self.destination_module
    }

    /// Get the event type
    pub fn event_type(&self) -> &str {
        &self.event_type
    }
}

impl fmt::Display for ChannelName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Trait for event payloads that can be serialized and deserialized
pub trait EventPayload: Sized + Send + Sync + 'static {
    /// Serialize the event payload to bytes
    fn serialize(&self) -> IpcResult<Vec<u8>>;

    /// Deserialize the event payload from bytes
    fn deserialize(bytes: &[u8]) -> IpcResult<Self>;
}

/// Message received from a Redis subscription
#[derive(Debug, Clone)]
pub struct Message {
    /// Channel the message was received on
    pub channel: String,

    /// Payload of the message
    pub payload: Vec<u8>,
}

/// Subscription to a Redis channel
pub struct Subscription {
    pubsub: Arc<Mutex<redis::aio::PubSub>>,
    channel: String,
}

impl Subscription {
    /// Create a new subscription
    fn new(pubsub: redis::aio::PubSub, channel: String) -> Self {
        Self {
            pubsub: Arc::new(Mutex::new(pubsub)),
            channel,
        }
    }

    /// Get the next message from the subscription
    pub async fn next_message(&self) -> IpcResult<Option<Message>> {
        let mut pubsub = self.pubsub.lock().await;
        match pubsub.get_message().await {
            Ok(msg) => {
                let channel = msg.get_channel_name().to_string();
                let payload = msg.get_payload_bytes().to_vec();
                Ok(Some(Message { channel, payload }))
            }
            Err(err) => Err(IpcError::Transport(err.into())),
        }
    }

    /// Convert the subscription into a stream of messages
    pub fn into_stream(self) -> impl Stream<Item = IpcResult<Message>> {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let pubsub = self.pubsub.clone();
        let channel = self.channel.clone();

        tokio::spawn(async move {
            loop {
                let result = {
                    let mut pubsub_guard = pubsub.lock().await;
                    pubsub_guard.get_message().await
                };

                match result {
                    Ok(msg) => {
                        let channel = msg.get_channel_name().to_string();
                        let payload = msg.get_payload_bytes().to_vec();
                        let message = Message { channel, payload };
                        if tx.send(Ok(message)).await.is_err() {
                            break;
                        }
                    }
                    Err(err) => {
                        let _ = tx.send(Err(IpcError::Transport(err.into()))).await;
                        break;
                    }
                }
            }
        });

        ReceiverStream::new(rx)
    }

    /// Get the channel name
    pub fn channel(&self) -> &str {
        &self.channel
    }
}

/// Trait for Redis client
#[async_trait]
pub trait RedisClient: Send + Sync {
    /// Publish a message to a channel
    async fn publish(&self, channel: &str, message: &[u8]) -> IpcResult<()>;

    /// Subscribe to a channel
    async fn subscribe(&self, channel: &str) -> IpcResult<Subscription>;

    /// Subscribe to a channel pattern
    async fn psubscribe(&self, pattern: &str) -> IpcResult<Subscription>;
}

/// Redis client implementation
pub struct RedisClientImpl {
    client: redis::Client,
    connection_manager: ConnectionManager,
}

impl RedisClientImpl {
    /// Create a new Redis client
    pub async fn new(redis_url: &str) -> IpcResult<Self> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| IpcError::Connection(format!("Failed to connect to Redis: {}", e)))?;

        let connection_manager = client.get_tokio_connection_manager().await.map_err(|e| {
            IpcError::Connection(format!("Failed to get connection manager: {}", e))
        })?;

        Ok(Self {
            client,
            connection_manager,
        })
    }
}

#[async_trait]
impl RedisClient for RedisClientImpl {
    async fn publish(&self, channel: &str, message: &[u8]) -> IpcResult<()> {
        let mut conn = self.connection_manager.clone();
        redis::cmd("PUBLISH")
            .arg(channel)
            .arg(message)
            .query_async(&mut conn)
            .await
            .map_err(|e| IpcError::Transport(e.into()))?;
        Ok(())
    }

    async fn subscribe(&self, channel: &str) -> IpcResult<Subscription> {
        let mut pubsub = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| IpcError::Connection(format!("Failed to get connection: {}", e)))?
            .into_pubsub();

        pubsub
            .subscribe(channel)
            .await
            .map_err(|e| IpcError::Transport(e.into()))?;

        Ok(Subscription::new(pubsub, channel.to_string()))
    }

    async fn psubscribe(&self, pattern: &str) -> IpcResult<Subscription> {
        let mut pubsub = self
            .client
            .get_async_connection()
            .await
            .map_err(|e| IpcError::Connection(format!("Failed to get connection: {}", e)))?
            .into_pubsub();

        pubsub
            .psubscribe(pattern)
            .await
            .map_err(|e| IpcError::Transport(e.into()))?;

        Ok(Subscription::new(pubsub, pattern.to_string()))
    }
}

/// Implementation of EventPayload for JSON serialization/deserialization
impl<T> EventPayload for T
where
    T: Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static,
{
    fn serialize(&self) -> IpcResult<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| {
            IpcError::Serialization(format!("Failed to serialize event payload: {}", e))
        })
    }

    fn deserialize(bytes: &[u8]) -> IpcResult<Self> {
        serde_json::from_slice(bytes).map_err(|e| {
            IpcError::Serialization(format!("Failed to deserialize event payload: {}", e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tokio::time::{sleep, Duration};

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct TestEvent {
        id: String,
        message: String,
    }

    #[test]
    fn test_channel_name() {
        let channel = ChannelName::new("chain_engine", "router_core", "chain_execution_completed");
        assert_eq!(
            channel.to_string(),
            "intellirouter:chain_engine:router_core:chain_execution_completed"
        );
        assert_eq!(channel.source_module(), "chain_engine");
        assert_eq!(channel.destination_module(), "router_core");
        assert_eq!(channel.event_type(), "chain_execution_completed");

        let parsed = ChannelName::from_string(&channel.to_string()).unwrap();
        assert_eq!(parsed.source_module(), "chain_engine");
        assert_eq!(parsed.destination_module(), "router_core");
        assert_eq!(parsed.event_type(), "chain_execution_completed");

        let invalid = ChannelName::from_string("invalid:channel:name");
        assert!(invalid.is_none());
    }

    #[test]
    fn test_event_payload_serialization() {
        let event = TestEvent {
            id: "test-id".to_string(),
            message: "test-message".to_string(),
        };

        let serialized = event.serialize().unwrap();
        let deserialized = TestEvent::deserialize(&serialized).unwrap();

        assert_eq!(event, deserialized);
    }

    // This test requires a running Redis instance
    // #[tokio::test]
    // async fn test_redis_client() {
    //     let redis_url = "redis://localhost:6379";
    //     let client = RedisClientImpl::new(redis_url).await.unwrap();
    //
    //     let channel = "test-channel";
    //     let event = TestEvent {
    //         id: "test-id".to_string(),
    //         message: "test-message".to_string(),
    //     };
    //
    //     let serialized = event.serialize().unwrap();
    //
    //     // Subscribe to the channel
    //     let subscription = client.subscribe(channel).await.unwrap();
    //
    //     // Publish a message
    //     client.publish(channel, &serialized).await.unwrap();
    //
    //     // Wait for the message
    //     sleep(Duration::from_millis(100)).await;
    //
    //     // Get the message
    //     let message = subscription.next_message().await.unwrap().unwrap();
    //     assert_eq!(message.channel, channel);
    //
    //     // Deserialize the message
    //     let received_event = TestEvent::deserialize(&message.payload).unwrap();
    //     assert_eq!(received_event, event);
    // }
}
