# IntelliRouter IPC Comprehensive Guide

This document provides a comprehensive guide for using the IntelliRouter IPC infrastructure.

## Table of Contents

- [Overview](#overview)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
  - [Configuration](#configuration)
- [Synchronous Communication (gRPC)](#synchronous-communication-grpc)
  - [Defining Service Interfaces](#defining-service-interfaces)
  - [Implementing Client Traits](#implementing-client-traits)
  - [Implementing Server Traits](#implementing-server-traits)
  - [Error Handling](#error-handling)
- [Asynchronous Communication (Redis Pub/Sub)](#asynchronous-communication-redis-pubsub)
  - [Defining Event Types](#defining-event-types)
  - [Implementing Event Publishers](#implementing-event-publishers)
  - [Implementing Event Subscribers](#implementing-event-subscribers)
  - [Error Handling](#error-handling-1)
- [Security](#security)
  - [JWT Authentication](#jwt-authentication)
  - [Mutual TLS (mTLS)](#mutual-tls-mtls)
  - [Combining JWT and mTLS](#combining-jwt-and-mtls)
- [Testing](#testing)
  - [Mock Clients](#mock-clients)
  - [Integration Testing](#integration-testing)
- [Deployment](#deployment)
  - [Single-Binary Deployment](#single-binary-deployment)
  - [Multi-Service Deployment](#multi-service-deployment)
- [Troubleshooting](#troubleshooting)
  - [Common Issues](#common-issues)
  - [Debugging Tips](#debugging-tips)
- [Best Practices](#best-practices)

## Overview

The IntelliRouter IPC (Inter-Process Communication) infrastructure provides a robust and secure way for different modules of the IntelliRouter system to communicate with each other. It supports both synchronous communication using gRPC and asynchronous communication using Redis Pub/Sub.

The IPC infrastructure is designed with the following principles in mind:

1. **Decoupling**: Modules should be loosely coupled to allow for independent development and deployment.
2. **Type Safety**: Communication between modules should be type-safe to prevent runtime errors.
3. **Security**: Communication should be secure to prevent unauthorized access and eavesdropping.
4. **Scalability**: The infrastructure should be scalable to handle increasing load.
5. **Reliability**: Communication should be reliable, with appropriate error handling and retry mechanisms.

## Getting Started

### Prerequisites

Before using the IntelliRouter IPC infrastructure, you need to have the following:

1. Rust (1.70.0 or later)
2. Protocol Buffers compiler (protoc)
3. Redis (for asynchronous communication)

### Installation

Add the IntelliRouter crate to your `Cargo.toml`:

```toml
[dependencies]
intellirouter = { git = "https://github.com/intellirouter/intellirouter.git" }
```

### Configuration

Configure the IPC infrastructure in your application:

```rust
use intellirouter::modules::ipc::security::{JwtConfig, JwtAuthenticator, TlsConfig};
use intellirouter::modules::ipc::redis_pubsub::RedisClientImpl;
use std::path::PathBuf;
use std::sync::Arc;

// Configure JWT authentication
let jwt_config = JwtConfig {
    secret: "your-secret-key".to_string(),
    issuer: "intellirouter".to_string(),
    audience: "intellirouter-services".to_string(),
    expiration_seconds: 3600, // 1 hour
};

let jwt_authenticator = Arc::new(JwtAuthenticator::new(jwt_config));

// Configure mTLS (optional)
let tls_config = TlsConfig::new(
    PathBuf::from("path/to/cert.crt"),
    PathBuf::from("path/to/key.key"),
    PathBuf::from("path/to/ca.crt"),
);

// Configure Redis client (for asynchronous communication)
let redis_client = Arc::new(RedisClientImpl::new("redis://localhost:6379").await?);
```

## Synchronous Communication (gRPC)

IntelliRouter uses gRPC for synchronous communication between modules. gRPC provides a high-performance, language-agnostic RPC framework with built-in support for streaming, authentication, and load balancing.

### Defining Service Interfaces

Service interfaces are defined using Protocol Buffers (protobuf) and implemented as Rust traits. Here's an example of a service interface for the Chain Engine module:

```protobuf
// chain_engine.proto
syntax = "proto3";

package intellirouter.chain_engine.v1;

service ChainEngineService {
  rpc ExecuteChain(ChainExecutionRequest) returns (ChainExecutionResponse);
  rpc GetChainStatus(ChainStatusRequest) returns (ChainStatusResponse);
  rpc CancelChainExecution(CancelChainRequest) returns (CancelChainResponse);
  rpc StreamChainExecution(ChainExecutionRequest) returns (stream ChainExecutionEvent);
}
```

The corresponding Rust trait:

```rust
#[async_trait]
pub trait ChainEngineClient: Send + Sync {
    async fn execute_chain(
        &self,
        chain_id: Option<String>,
        chain: Option<Chain>,
        input: String,
        variables: HashMap<String, String>,
        stream: bool,
        timeout_seconds: Option<u32>,
    ) -> IpcResult<ChainExecutionResponse>;

    async fn get_chain_status(&self, execution_id: &str) -> IpcResult<ChainStatusResponse>;

    async fn cancel_chain_execution(&self, execution_id: &str) -> IpcResult<CancelChainResponse>;

    async fn stream_chain_execution(
        &self,
        chain_id: Option<String>,
        chain: Option<Chain>,
        input: String,
        variables: HashMap<String, String>,
        timeout_seconds: Option<u32>,
    ) -> IpcResult<Pin<Box<dyn Stream<Item = Result<ChainExecutionEvent, tonic::Status>> + Send>>>;
}
```

### Implementing Client Traits

To implement a client for a service, you need to implement the corresponding client trait. IntelliRouter provides a `SecureGrpcClientBuilder` to simplify this process:

```rust
use intellirouter::modules::ipc::security::SecureGrpcClientBuilder;
use intellirouter::generated::intellirouter::chain_engine::v1::chain_engine_client::ChainEngineClient;

// Create a secure gRPC client
let client = SecureGrpcClientBuilder::<ChainEngineClient<_>>::new("intellirouter.local")
    .with_tls(tls_config.clone())
    .with_jwt(
        jwt_authenticator.clone(),
        "router_core".to_string(),
        vec!["route_requests".to_string()],
    )
    .build("https://intellirouter.local:50051")
    .await?;

// Use the client
let response = client
    .execute_chain(
        Some("my-chain-id".to_string()),
        None,
        "Hello, world!".to_string(),
        HashMap::new(),
        false,
        Some(30),
    )
    .await?;
```

### Implementing Server Traits

To implement a server for a service, you need to implement the corresponding server trait. IntelliRouter provides a `SecureGrpcServerBuilder` to simplify this process:

```rust
use intellirouter::modules::ipc::security::SecureGrpcServerBuilder;
use intellirouter::generated::intellirouter::chain_engine::v1::chain_engine_server::{ChainEngine, ChainEngineServer};

// Implement the service
struct ChainEngineServiceImpl {
    // Service state
}

#[async_trait]
impl ChainEngineService for ChainEngineServiceImpl {
    // Implement the service methods
    // ...
}

// Create a secure gRPC server
let service = ChainEngineServer::new(ChainEngineServiceImpl { /* ... */ });

let server_builder = SecureGrpcServerBuilder::new()
    .with_tls(tls_config.clone())
    .with_jwt(
        jwt_authenticator.clone(),
        vec!["execute_chain".to_string(), "cancel_chain".to_string()],
    );

let router = server_builder.build::<ChainEngineServer<_>>()?;
let router = server_builder.add_service(router, service);

let addr = "[::1]:50051".parse()?;
router.serve(addr).await?;
```

### Error Handling

IntelliRouter provides a common error type for IPC operations:

```rust
pub enum IpcError {
    Transport(tonic::transport::Error),
    Status(tonic::Status),
    Connection(String),
    Serialization(String),
    Timeout(String),
    NotFound(String),
    InvalidArgument(String),
    Internal(String),
    Security(String),
}
```

You can handle these errors in your application:

```rust
match client.execute_chain(...).await {
    Ok(response) => {
        // Handle successful response
    }
    Err(IpcError::Transport(e)) => {
        // Handle transport error
    }
    Err(IpcError::Status(e)) => {
        // Handle status error
    }
    Err(IpcError::Connection(e)) => {
        // Handle connection error
    }
    Err(IpcError::Serialization(e)) => {
        // Handle serialization error
    }
    Err(IpcError::Timeout(e)) => {
        // Handle timeout error
    }
    Err(IpcError::NotFound(e)) => {
        // Handle not found error
    }
    Err(IpcError::InvalidArgument(e)) => {
        // Handle invalid argument error
    }
    Err(IpcError::Internal(e)) => {
        // Handle internal error
    }
    Err(IpcError::Security(e)) => {
        // Handle security error
    }
}
```

## Asynchronous Communication (Redis Pub/Sub)

IntelliRouter uses Redis Pub/Sub for asynchronous communication between modules. Redis Pub/Sub provides a lightweight, scalable, and reliable messaging system.

### Defining Event Types

Event types are defined as Rust structs that implement the `EventPayload` trait:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainExecutionCompletedEvent {
    pub execution_id: String,
    pub output: String,
    pub total_tokens: u32,
    pub execution_time_ms: u64,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}
```

### Implementing Event Publishers

To publish events, you need to implement an event publisher:

```rust
pub struct ChainEngineEventPublisher {
    redis_client: Arc<dyn RedisClient>,
}

impl ChainEngineEventPublisher {
    pub fn new(redis_client: Arc<dyn RedisClient>) -> Self {
        Self { redis_client }
    }

    pub async fn publish_chain_execution_completed(
        &self,
        event: ChainExecutionCompletedEvent,
    ) -> IpcResult<()> {
        let channel = ChannelName::new("chain_engine", "router_core", "chain_execution_completed");
        let payload = event.serialize()?;
        self.redis_client
            .publish(&channel.to_string(), &payload)
            .await
    }
}
```

### Implementing Event Subscribers

To subscribe to events, you need to implement an event subscriber:

```rust
pub struct RouterCoreEventSubscriber {
    redis_client: Arc<dyn RedisClient>,
}

impl RouterCoreEventSubscriber {
    pub fn new(redis_client: Arc<dyn RedisClient>) -> Self {
        Self { redis_client }
    }

    pub async fn subscribe_to_chain_execution_completed(
        &self,
    ) -> IpcResult<ChainExecutionCompletedSubscription> {
        let channel = ChannelName::new("chain_engine", "router_core", "chain_execution_completed");
        let subscription = self.redis_client.subscribe(&channel.to_string()).await?;
        Ok(ChainExecutionCompletedSubscription { subscription })
    }
}

pub struct ChainExecutionCompletedSubscription {
    subscription: Subscription,
}

impl ChainExecutionCompletedSubscription {
    pub async fn next_event(&self) -> IpcResult<Option<ChainExecutionCompletedEvent>> {
        if let Some(message) = self.subscription.next_message().await? {
            let event = ChainExecutionCompletedEvent::deserialize(&message.payload)?;
            Ok(Some(event))
        } else {
            Ok(None)
        }
    }
}
```

### Error Handling

Error handling for asynchronous communication is similar to synchronous communication. The `IpcError` enum is used to represent errors:

```rust
match publisher.publish_chain_execution_completed(event).await {
    Ok(_) => {
        // Event published successfully
    }
    Err(e) => {
        // Handle error
    }
}

match subscription.next_event().await {
    Ok(Some(event)) => {
        // Handle event
    }
    Ok(None) => {
        // No event available
    }
    Err(e) => {
        // Handle error
    }
}
```

## Security

IntelliRouter provides two main security mechanisms: JWT authentication and mutual TLS (mTLS). These can be used independently or together for enhanced security.

### JWT Authentication

JWT authentication is used to verify the identity of services and control access to resources based on roles. See the [Security Configuration](security_config.md#jwt-authentication) document for details.

### Mutual TLS (mTLS)

Mutual TLS (mTLS) provides encrypted communication and verifies the identity of both client and server using certificates. See the [Security Configuration](security_config.md#mutual-tls-mtls) document for details.

### Combining JWT and mTLS

For maximum security, you can combine JWT authentication and mTLS. See the [Security Configuration](security_config.md#combining-jwt-and-mtls) document for details.

## Testing

IntelliRouter provides tools for testing IPC communication, including mock clients and integration testing utilities.

### Mock Clients

Mock clients are useful for unit testing. IntelliRouter provides mock implementations of client traits:

```rust
pub struct MockChainEngineClient {
    executions: HashMap<String, ChainExecutionResponse>,
}

impl MockChainEngineClient {
    pub fn new() -> Self {
        Self {
            executions: HashMap::new(),
        }
    }
    
    pub fn add_execution(&mut self, execution: ChainExecutionResponse) {
        self.executions.insert(execution.execution_id.clone(), execution);
    }
}

#[async_trait]
impl ChainEngineClient for MockChainEngineClient {
    // Implement the client methods
    // ...
}
```

### Integration Testing

For integration testing, you can use the actual implementations of clients and services, but with in-memory or local Redis and gRPC servers:

```rust
#[tokio::test]
async fn test_chain_engine_integration() {
    // Start a local gRPC server
    let service = ChainEngineServiceImpl { /* ... */ };
    let server = tonic::transport::Server::builder()
        .add_service(ChainEngineServer::new(service))
        .serve_with_shutdown("127.0.0.1:0".parse().unwrap(), shutdown_signal());
    
    let server_addr = server.local_addr();
    let server_handle = tokio::spawn(server);
    
    // Create a client
    let client = GrpcChainEngineClient::new(&format!("http://{}", server_addr)).await.unwrap();
    
    // Test the client
    let response = client
        .execute_chain(
            Some("test-chain".to_string()),
            None,
            "Test input".to_string(),
            HashMap::new(),
            false,
            None,
        )
        .await
        .unwrap();
    
    assert_eq!(response.status, Status::Success);
    
    // Shutdown the server
    shutdown_sender.send(()).unwrap();
    server_handle.await.unwrap();
}
```

## Deployment

IntelliRouter supports both single-binary and multi-service deployment models.

### Single-Binary Deployment

In the single-binary deployment model, all modules are compiled into a single binary and run in the same process. This simplifies deployment and reduces overhead, but limits scalability.

```rust
// Create the services
let chain_engine_service = ChainEngineServiceImpl { /* ... */ };
let model_registry_service = ModelRegistryServiceImpl { /* ... */ };
let memory_service = MemoryServiceImpl { /* ... */ };
let rag_manager_service = RagManagerServiceImpl { /* ... */ };
let persona_layer_service = PersonaLayerServiceImpl { /* ... */ };

// Create the server
let server = tonic::transport::Server::builder()
    .add_service(ChainEngineServer::new(chain_engine_service))
    .add_service(ModelRegistryServer::new(model_registry_service))
    .add_service(MemoryServer::new(memory_service))
    .add_service(RagManagerServer::new(rag_manager_service))
    .add_service(PersonaLayerServer::new(persona_layer_service))
    .serve("0.0.0.0:50051".parse().unwrap());

// Start the server
server.await?;
```

### Multi-Service Deployment

In the multi-service deployment model, each module is compiled into a separate binary and run in a separate process. This improves scalability and isolation, but increases deployment complexity.

```rust
// Chain Engine service
let chain_engine_service = ChainEngineServiceImpl { /* ... */ };
let server = tonic::transport::Server::builder()
    .add_service(ChainEngineServer::new(chain_engine_service))
    .serve("0.0.0.0:50051".parse().unwrap());
server.await?;

// Model Registry service
let model_registry_service = ModelRegistryServiceImpl { /* ... */ };
let server = tonic::transport::Server::builder()
    .add_service(ModelRegistryServer::new(model_registry_service))
    .serve("0.0.0.0:50052".parse().unwrap());
server.await?;

// Memory service
let memory_service = MemoryServiceImpl { /* ... */ };
let server = tonic::transport::Server::builder()
    .add_service(MemoryServer::new(memory_service))
    .serve("0.0.0.0:50053".parse().unwrap());
server.await?;

// RAG Manager service
let rag_manager_service = RagManagerServiceImpl { /* ... */ };
let server = tonic::transport::Server::builder()
    .add_service(RagManagerServer::new(rag_manager_service))
    .serve("0.0.0.0:50054".parse().unwrap());
server.await?;

// Persona Layer service
let persona_layer_service = PersonaLayerServiceImpl { /* ... */ };
let server = tonic::transport::Server::builder()
    .add_service(PersonaLayerServer::new(persona_layer_service))
    .serve("0.0.0.0:50055".parse().unwrap());
server.await?;
```

## Troubleshooting

### Common Issues

1. **Connection Refused**: Make sure the server is running and the client is connecting to the correct address.
2. **Authentication Failed**: Make sure the JWT token is valid and the client has the required roles.
3. **Certificate Verification Failed**: Make sure the client and server certificates are valid and signed by the same CA.
4. **Redis Connection Failed**: Make sure Redis is running and the client is connecting to the correct address.
5. **Serialization Error**: Make sure the event payload implements the `EventPayload` trait correctly.

### Debugging Tips

1. **Enable Logging**: Enable logging to see what's happening behind the scenes.
2. **Check Network Connectivity**: Make sure the client can reach the server.
3. **Check Firewall Rules**: Make sure the firewall allows traffic on the required ports.
4. **Check Certificate Expiration**: Make sure the certificates are not expired.
5. **Check Redis Connection**: Make sure Redis is running and accessible.

## Best Practices

1. **Use Secure Communication**: Always use JWT authentication and/or mTLS for secure communication.
2. **Handle Errors Gracefully**: Always handle errors gracefully and provide meaningful error messages.
3. **Use Timeouts**: Always use timeouts to prevent hanging requests.
4. **Use Retries**: Implement retry logic for transient errors.
5. **Use Circuit Breakers**: Implement circuit breakers to prevent cascading failures.
6. **Monitor Performance**: Monitor the performance of your IPC communication to identify bottlenecks.
7. **Use Structured Logging**: Use structured logging to make it easier to analyze logs.
8. **Use Metrics**: Collect metrics to monitor the health and performance of your services.
9. **Use Tracing**: Implement distributed tracing to track requests across services.
10. **Keep Dependencies Updated**: Keep your dependencies updated to benefit from bug fixes and security patches.