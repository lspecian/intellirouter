# IntelliRouter IPC Channel Naming Conventions

This document provides detailed information about the channel naming conventions used for asynchronous communication in the IntelliRouter system.

## Table of Contents

- [Overview](#overview)
- [Channel Naming Pattern](#channel-naming-pattern)
- [Event Types](#event-types)
- [Module Pairs](#module-pairs)
  - [Chain Engine → Router Core](#chain-engine--router-core)
  - [RAG Manager → Persona Layer](#rag-manager--persona-layer)
  - [Memory → Chain Engine](#memory--chain-engine)
  - [Router Core → Model Registry](#router-core--model-registry)
- [Subscription Patterns](#subscription-patterns)
- [Best Practices](#best-practices)

## Overview

IntelliRouter uses Redis Pub/Sub for asynchronous communication between modules. This allows for loose coupling between modules and enables event-driven architecture patterns. The channel naming conventions ensure consistent and predictable communication between modules.

## Channel Naming Pattern

The channel naming pattern follows this format:

```
intellirouter:{source_module}:{destination_module}:{event_type}
```

Where:
- `intellirouter` is a fixed prefix for all IntelliRouter channels
- `{source_module}` is the name of the module publishing the event
- `{destination_module}` is the name of the module subscribing to the event
- `{event_type}` is the type of event being published

Example:
```
intellirouter:chain_engine:router_core:chain_execution_completed
```

This channel is used by the Chain Engine module to publish chain execution completed events to the Router Core module.

## Event Types

Each module pair has specific event types that they use for communication. The event types are defined in the respective event modules.

### Common Event Type Patterns

- `*_completed`: Events that indicate the completion of an operation
- `*_failed`: Events that indicate the failure of an operation
- `*_started`: Events that indicate the start of an operation
- `*_updated`: Events that indicate an update to a resource
- `*_created`: Events that indicate the creation of a resource
- `*_deleted`: Events that indicate the deletion of a resource

## Module Pairs

### Chain Engine → Router Core

The Chain Engine module publishes events to the Router Core module using the following channels:

| Channel | Event Type | Description |
|---------|------------|-------------|
| `intellirouter:chain_engine:router_core:chain_execution_completed` | `ChainExecutionCompletedEvent` | Published when a chain execution completes successfully |
| `intellirouter:chain_engine:router_core:chain_execution_failed` | `ChainExecutionFailedEvent` | Published when a chain execution fails |
| `intellirouter:chain_engine:router_core:chain_step_completed` | `ChainStepCompletedEvent` | Published when a step in a chain completes successfully |

#### Example Event Payload

```rust
pub struct ChainExecutionCompletedEvent {
    pub execution_id: String,
    pub output: String,
    pub total_tokens: u32,
    pub execution_time_ms: u64,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}
```

### RAG Manager → Persona Layer

The RAG Manager module publishes events to the Persona Layer module using the following channels:

| Channel | Event Type | Description |
|---------|------------|-------------|
| `intellirouter:rag_manager:persona_layer:document_indexed` | `DocumentIndexedEvent` | Published when a document is indexed |
| `intellirouter:rag_manager:persona_layer:document_retrieval` | `DocumentRetrievalEvent` | Published when documents are retrieved for a query |
| `intellirouter:rag_manager:persona_layer:context_augmentation` | `ContextAugmentationEvent` | Published when a request is augmented with context |

#### Example Event Payload

```rust
pub struct DocumentIndexedEvent {
    pub document_id: String,
    pub chunk_count: u32,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}
```

### Memory → Chain Engine

The Memory module publishes events to the Chain Engine module using the following channels:

| Channel | Event Type | Description |
|---------|------------|-------------|
| `intellirouter:memory:chain_engine:conversation_updated` | `ConversationUpdatedEvent` | Published when a conversation is updated |
| `intellirouter:memory:chain_engine:conversation_history_retrieved` | `ConversationHistoryRetrievedEvent` | Published when conversation history is retrieved |

#### Example Event Payload

```rust
pub struct ConversationUpdatedEvent {
    pub conversation_id: String,
    pub message_id: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}
```

### Router Core → Model Registry

The Router Core module publishes events to the Model Registry module using the following channels:

| Channel | Event Type | Description |
|---------|------------|-------------|
| `intellirouter:router_core:model_registry:model_health_check` | `ModelHealthCheckEvent` | Published when a model health check is performed |
| `intellirouter:router_core:model_registry:model_routing_decision` | `ModelRoutingDecisionEvent` | Published when a routing decision is made |
| `intellirouter:router_core:model_registry:model_usage` | `ModelUsageEvent` | Published when a model is used |

#### Example Event Payload

```rust
pub struct ModelRoutingDecisionEvent {
    pub request_id: String,
    pub selected_model_id: String,
    pub strategy_name: String,
    pub routing_time_ms: u64,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}
```

## Subscription Patterns

There are two main subscription patterns used in IntelliRouter:

### Direct Subscription

Direct subscription is used when a module wants to subscribe to a specific event type from a specific source module. This is done using the `subscribe` method of the Redis client:

```rust
let channel = ChannelName::new("chain_engine", "router_core", "chain_execution_completed");
let subscription = redis_client.subscribe(&channel.to_string()).await?;
```

### Pattern Subscription

Pattern subscription is used when a module wants to subscribe to all event types from a specific source module. This is done using the `psubscribe` method of the Redis client:

```rust
let pattern = "intellirouter:chain_engine:router_core:*";
let subscription = redis_client.psubscribe(pattern).await?;
```

## Best Practices

### Channel Naming

1. **Use Consistent Module Names**: Module names should be consistent across the codebase. Use snake_case for module names.
2. **Use Descriptive Event Types**: Event types should be descriptive and follow a consistent naming pattern.
3. **Avoid Generic Event Types**: Avoid generic event types like "updated" or "changed". Use more specific event types like "document_indexed" or "model_health_check".

### Event Publishing

1. **Include Timestamps**: Always include a timestamp in event payloads to help with debugging and auditing.
2. **Include Metadata**: Include relevant metadata in event payloads to provide additional context.
3. **Use Unique IDs**: Include unique IDs in event payloads to help with correlation and tracking.
4. **Keep Payloads Small**: Keep event payloads small to minimize network overhead. Include only the necessary information.
5. **Handle Serialization Errors**: Handle serialization errors gracefully to prevent publishing malformed events.

### Event Subscription

1. **Handle Deserialization Errors**: Handle deserialization errors gracefully to prevent crashing when receiving malformed events.
2. **Use Pattern Subscription Sparingly**: Use pattern subscription sparingly to avoid receiving unwanted events.
3. **Implement Backpressure**: Implement backpressure mechanisms to handle high event volumes.
4. **Handle Reconnection**: Implement reconnection logic to handle Redis connection failures.
5. **Log Subscription Errors**: Log subscription errors to help with debugging.

### Security

1. **Use Authenticated Redis Clients**: Use authenticated Redis clients to prevent unauthorized access.
2. **Validate Event Sources**: Validate event sources to prevent spoofing.
3. **Encrypt Sensitive Data**: Encrypt sensitive data in event payloads.
4. **Use TLS for Redis Connections**: Use TLS for Redis connections to prevent eavesdropping.
5. **Implement Rate Limiting**: Implement rate limiting to prevent denial of service attacks.