//! Integration tests for Redis pub/sub infrastructure
//!
//! These tests demonstrate how to use the Redis pub/sub infrastructure
//! for asynchronous communication between IntelliRouter modules.
//!
//! Note: These tests require a running Redis instance and are disabled by default.
//! To run these tests, use: `cargo test -- --ignored redis_pubsub`

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tokio::time::sleep;

use crate::modules::ipc::events::chain_engine_router_core::{
    ChainEngineEventPublisher, ChainExecutionCompletedEvent, RouterCoreEventSubscriber,
};
use crate::modules::ipc::events::memory_chain_engine::{
    ChainEngineMemorySubscriber, ConversationMessage, ConversationUpdatedEvent,
    MemoryEventPublisher,
};
use crate::modules::ipc::events::rag_manager_persona_layer::{
    DocumentIndexedEvent, PersonaLayerEventSubscriber, RagManagerEventPublisher,
};
use crate::modules::ipc::events::router_core_model_registry::{
    ModelRegistryEventSubscriber, ModelUsageEvent, RouterCoreEventPublisher,
};
use crate::modules::ipc::redis_pubsub::{RedisClient, RedisClientImpl};

/// Test the Chain Engine to Router Core pub/sub
#[tokio::test]
#[ignore] // Requires a running Redis instance
async fn test_chain_engine_router_core_pubsub() {
    // Create a Redis client
    let redis_url = "redis://localhost:6379";
    let redis_client = Arc::new(
        RedisClientImpl::new(redis_url)
            .await
            .expect("Failed to create Redis client"),
    );

    // Create a publisher and subscriber
    let publisher = ChainEngineEventPublisher::new(redis_client.clone());
    let subscriber = RouterCoreEventSubscriber::new(redis_client.clone());

    // Subscribe to chain execution completed events
    let subscription = subscriber
        .subscribe_to_chain_execution_completed()
        .await
        .expect("Failed to subscribe to chain execution completed events");

    // Create a test event
    let event = ChainExecutionCompletedEvent {
        execution_id: "test-execution-id".to_string(),
        output: "test-output".to_string(),
        total_tokens: 100,
        execution_time_ms: 1000,
        timestamp: Utc::now(),
        metadata: HashMap::new(),
    };

    // Publish the event
    publisher
        .publish_chain_execution_completed(event.clone())
        .await
        .expect("Failed to publish chain execution completed event");

    // Wait for the event to be published
    sleep(Duration::from_millis(100)).await;

    // Get the event
    let received_event = subscription
        .next_event()
        .await
        .expect("Failed to get next event")
        .expect("No event received");

    // Verify the event
    assert_eq!(received_event.execution_id, event.execution_id);
    assert_eq!(received_event.output, event.output);
    assert_eq!(received_event.total_tokens, event.total_tokens);
    assert_eq!(received_event.execution_time_ms, event.execution_time_ms);
}

/// Test the Memory to Chain Engine pub/sub
#[tokio::test]
#[ignore] // Requires a running Redis instance
async fn test_memory_chain_engine_pubsub() {
    // Create a Redis client
    let redis_url = "redis://localhost:6379";
    let redis_client = Arc::new(
        RedisClientImpl::new(redis_url)
            .await
            .expect("Failed to create Redis client"),
    );

    // Create a publisher and subscriber
    let publisher = MemoryEventPublisher::new(redis_client.clone());
    let subscriber = ChainEngineMemorySubscriber::new(redis_client.clone());

    // Subscribe to conversation updated events
    let subscription = subscriber
        .subscribe_to_conversation_updated()
        .await
        .expect("Failed to subscribe to conversation updated events");

    // Create a test message
    let message = ConversationMessage {
        id: "test-message-id".to_string(),
        role: "user".to_string(),
        content: "Hello, world!".to_string(),
        timestamp: Utc::now(),
        metadata: HashMap::new(),
        parent_id: None,
        token_count: Some(3),
    };

    // Create a test event
    let event = ConversationUpdatedEvent {
        conversation_id: "test-conversation-id".to_string(),
        new_message: message.clone(),
        message_count: 1,
        timestamp: Utc::now(),
        user_id: Some("test-user-id".to_string()),
        metadata: HashMap::new(),
    };

    // Publish the event
    publisher
        .publish_conversation_updated(event.clone())
        .await
        .expect("Failed to publish conversation updated event");

    // Wait for the event to be published
    sleep(Duration::from_millis(100)).await;

    // Get the event
    let received_event = subscription
        .next_event()
        .await
        .expect("Failed to get next event")
        .expect("No event received");

    // Verify the event
    assert_eq!(received_event.conversation_id, event.conversation_id);
    assert_eq!(received_event.new_message.id, event.new_message.id);
    assert_eq!(received_event.new_message.role, event.new_message.role);
    assert_eq!(
        received_event.new_message.content,
        event.new_message.content
    );
    assert_eq!(received_event.message_count, event.message_count);
}

/// Test the RAG Manager to Persona Layer pub/sub
#[tokio::test]
#[ignore] // Requires a running Redis instance
async fn test_rag_manager_persona_layer_pubsub() {
    // Create a Redis client
    let redis_url = "redis://localhost:6379";
    let redis_client = Arc::new(
        RedisClientImpl::new(redis_url)
            .await
            .expect("Failed to create Redis client"),
    );

    // Create a publisher and subscriber
    let publisher = RagManagerEventPublisher::new(redis_client.clone());
    let subscriber = PersonaLayerEventSubscriber::new(redis_client.clone());

    // Subscribe to document indexed events
    let subscription = subscriber
        .subscribe_to_document_indexed()
        .await
        .expect("Failed to subscribe to document indexed events");

    // Create a test event
    let event = DocumentIndexedEvent {
        document_id: "test-document-id".to_string(),
        document_name: "test-document-name".to_string(),
        chunk_count: 10,
        timestamp: Utc::now(),
        metadata: HashMap::new(),
    };

    // Publish the event
    publisher
        .publish_document_indexed(event.clone())
        .await
        .expect("Failed to publish document indexed event");

    // Wait for the event to be published
    sleep(Duration::from_millis(100)).await;

    // Get the event
    let received_event = subscription
        .next_event()
        .await
        .expect("Failed to get next event")
        .expect("No event received");

    // Verify the event
    assert_eq!(received_event.document_id, event.document_id);
    assert_eq!(received_event.document_name, event.document_name);
    assert_eq!(received_event.chunk_count, event.chunk_count);
}

/// Test the Router Core to Model Registry pub/sub
#[tokio::test]
#[ignore] // Requires a running Redis instance
async fn test_router_core_model_registry_pubsub() {
    // Create a Redis client
    let redis_url = "redis://localhost:6379";
    let redis_client = Arc::new(
        RedisClientImpl::new(redis_url)
            .await
            .expect("Failed to create Redis client"),
    );

    // Create a publisher and subscriber
    let publisher = RouterCoreEventPublisher::new(redis_client.clone());
    let subscriber = ModelRegistryEventSubscriber::new(redis_client.clone());

    // Subscribe to model usage events
    let subscription = subscriber
        .subscribe_to_model_usage()
        .await
        .expect("Failed to subscribe to model usage events");

    // Create a test event
    let event = ModelUsageEvent {
        model_id: "test-model-id".to_string(),
        request_id: "test-request-id".to_string(),
        user_id: Some("test-user-id".to_string()),
        org_id: None,
        input_tokens: 10,
        output_tokens: 20,
        latency_ms: 100,
        timestamp: Utc::now(),
        success: true,
        error_message: None,
        metadata: HashMap::new(),
    };

    // Publish the event
    publisher
        .publish_model_usage(event.clone())
        .await
        .expect("Failed to publish model usage event");

    // Wait for the event to be published
    sleep(Duration::from_millis(100)).await;

    // Get the event
    let received_event = subscription
        .next_event()
        .await
        .expect("Failed to get next event")
        .expect("No event received");

    // Verify the event
    assert_eq!(received_event.model_id, event.model_id);
    assert_eq!(received_event.request_id, event.request_id);
    assert_eq!(received_event.user_id, event.user_id);
    assert_eq!(received_event.input_tokens, event.input_tokens);
    assert_eq!(received_event.output_tokens, event.output_tokens);
    assert_eq!(received_event.latency_ms, event.latency_ms);
    assert_eq!(received_event.success, event.success);
}
