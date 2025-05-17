# IntelliRouter IPC Message Schemas

This document provides detailed information about the message schemas used for communication between different modules in the IntelliRouter system.

## Table of Contents

- [Overview](#overview)
- [Common Message Types](#common-message-types)
- [Chain Engine → Router Core](#chain-engine--router-core)
- [RAG Manager → Persona Layer](#rag-manager--persona-layer)
- [Memory → Chain Engine](#memory--chain-engine)
- [Router Core → Model Registry](#router-core--model-registry)

## Overview

IntelliRouter uses Protocol Buffer (protobuf) schemas to define the message contracts for communication between different modules. These schemas ensure type safety and versioning for both synchronous (gRPC) and asynchronous (Redis pub/sub) communication.

The message schemas follow these evolution guidelines:

1. **Never** change the meaning of an existing field
2. **Never** remove a field unless it's marked as deprecated for at least one version
3. **Always** add new fields as optional
4. **Always** use the `reserved` keyword for removed field numbers and names
5. Use versioned message types for major changes (e.g., `ModelInfoV1`, `ModelInfoV2`)
6. Include comments for all fields and messages to document their purpose
7. Use enums for fields with a fixed set of values
8. Use `oneof` for fields that can have different types

## Common Message Types

Common message types are defined in `common.proto` and are used across multiple services:

### Status

```protobuf
enum Status {
  STATUS_UNSPECIFIED = 0;
  STATUS_SUCCESS = 1;
  STATUS_ERROR = 2;
  STATUS_IN_PROGRESS = 3;
  STATUS_TIMEOUT = 4;
  STATUS_CANCELLED = 5;
}
```

### ErrorDetails

```protobuf
message ErrorDetails {
  string code = 1;
  string message = 2;
  google.protobuf.Struct details = 3;
  string stack_trace = 4;
}
```

### Metadata

```protobuf
message Metadata {
  map<string, string> values = 1;
}
```

### Message

```protobuf
message Message {
  string role = 1;
  string content = 2;
  google.protobuf.Timestamp timestamp = 3;
  Metadata metadata = 4;
}
```

### ModelInfo

```protobuf
message ModelInfo {
  string id = 1;
  string name = 2;
  string provider = 3;
  string version = 4;
  uint32 context_window = 5;
  repeated string capabilities = 6;
  string type = 7;
  Metadata metadata = 8;
}
```

### RequestContext

```protobuf
message RequestContext {
  string request_id = 1;
  string user_id = 2;
  string org_id = 3;
  google.protobuf.Timestamp timestamp = 4;
  uint32 priority = 5;
  repeated string tags = 6;
  Metadata metadata = 7;
}
```

### VersionInfo

```protobuf
message VersionInfo {
  uint32 major = 1;
  uint32 minor = 2;
  uint32 patch = 3;
}
```

## Chain Engine → Router Core

The Chain Engine and Router Core modules communicate using the following message schemas:

### Synchronous Communication (gRPC)

#### ChainEngineService

```protobuf
service ChainEngineService {
  rpc ExecuteChain(ChainExecutionRequest) returns (ChainExecutionResponse);
  rpc GetChainStatus(ChainStatusRequest) returns (ChainStatusResponse);
  rpc CancelChainExecution(CancelChainRequest) returns (CancelChainResponse);
  rpc StreamChainExecution(ChainExecutionRequest) returns (stream ChainExecutionEvent);
}
```

#### ChainStep

```protobuf
message ChainStep {
  string id = 1;
  string description = 2;
  string model = 3;
  string system_prompt = 4;
  string input_template = 5;
  string output_format = 6;
  uint32 max_tokens = 7;
  float temperature = 8;
  map<string, string> parameters = 9;
}
```

#### Chain

```protobuf
message Chain {
  string id = 1;
  string name = 2;
  string description = 3;
  repeated ChainStep steps = 4;
  intellirouter.common.v1.VersionInfo version = 5;
  intellirouter.common.v1.Metadata metadata = 6;
}
```

#### ChainExecutionRequest

```protobuf
message ChainExecutionRequest {
  intellirouter.common.v1.RequestContext context = 1;
  oneof chain_identifier {
    string chain_id = 2;
    Chain chain = 3;
  }
  string input = 4;
  map<string, string> variables = 5;
  bool stream = 6;
  uint32 timeout_seconds = 7;
}
```

#### ChainExecutionResponse

```protobuf
message ChainExecutionResponse {
  string execution_id = 1;
  intellirouter.common.v1.Status status = 2;
  string output = 3;
  intellirouter.common.v1.ErrorDetails error = 4;
  google.protobuf.Timestamp start_time = 5;
  google.protobuf.Timestamp end_time = 6;
  repeated StepResult step_results = 7;
  uint32 total_tokens = 8;
  intellirouter.common.v1.Metadata metadata = 9;
}
```

#### StepResult

```protobuf
message StepResult {
  string step_id = 1;
  intellirouter.common.v1.Status status = 2;
  string input = 3;
  string output = 4;
  intellirouter.common.v1.ErrorDetails error = 5;
  google.protobuf.Timestamp start_time = 6;
  google.protobuf.Timestamp end_time = 7;
  uint32 tokens = 8;
  string model = 9;
  intellirouter.common.v1.Metadata metadata = 10;
}
```

### Asynchronous Communication (Redis Pub/Sub)

#### ChainExecutionCompletedEvent

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

#### ChainExecutionFailedEvent

```rust
pub struct ChainExecutionFailedEvent {
    pub execution_id: String,
    pub error: ErrorDetails,
    pub execution_time_ms: u64,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}
```

#### ChainStepCompletedEvent

```rust
pub struct ChainStepCompletedEvent {
    pub execution_id: String,
    pub step_id: String,
    pub step_index: u32,
    pub output: String,
    pub tokens: u32,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}
```

## RAG Manager → Persona Layer

The RAG Manager and Persona Layer modules communicate using the following message schemas:

### Synchronous Communication (gRPC)

#### RAGManagerService

```protobuf
service RAGManagerService {
  rpc IndexDocument(IndexDocumentRequest) returns (IndexDocumentResponse);
  rpc RetrieveDocuments(RetrieveDocumentsRequest) returns (RetrieveDocumentsResponse);
  rpc AugmentRequest(AugmentRequestRequest) returns (AugmentRequestResponse);
  rpc GetDocumentById(GetDocumentByIdRequest) returns (GetDocumentByIdResponse);
  rpc DeleteDocument(DeleteDocumentRequest) returns (DeleteDocumentResponse);
  rpc ListDocuments(ListDocumentsRequest) returns (ListDocumentsResponse);
}
```

#### Document

```protobuf
message Document {
  string id = 1;
  string content = 2;
  map<string, string> metadata = 3;
  google.protobuf.Timestamp created_at = 4;
  google.protobuf.Timestamp updated_at = 5;
  repeated float embedding = 6;
  repeated DocumentChunk chunks = 7;
}
```

#### DocumentChunk

```protobuf
message DocumentChunk {
  string id = 1;
  string content = 2;
  map<string, string> metadata = 3;
  repeated float embedding = 4;
  string document_id = 5;
  uint32 chunk_index = 6;
}
```

#### PersonaLayerService

```protobuf
service PersonaLayerService {
  rpc CreatePersona(CreatePersonaRequest) returns (CreatePersonaResponse);
  rpc GetPersona(GetPersonaRequest) returns (GetPersonaResponse);
  rpc UpdatePersona(UpdatePersonaRequest) returns (UpdatePersonaResponse);
  rpc DeletePersona(DeletePersonaRequest) returns (DeletePersonaResponse);
  rpc ListPersonas(ListPersonasRequest) returns (ListPersonasResponse);
  rpc ApplyPersona(ApplyPersonaRequest) returns (ApplyPersonaResponse);
}
```

#### Persona

```protobuf
message Persona {
  string id = 1;
  string name = 2;
  string description = 3;
  string system_prompt = 4;
  string response_format = 5;
  google.protobuf.Timestamp created_at = 6;
  google.protobuf.Timestamp updated_at = 7;
  map<string, string> metadata = 8;
  repeated string tags = 9;
  intellirouter.common.v1.VersionInfo version = 10;
}
```

### Asynchronous Communication (Redis Pub/Sub)

#### DocumentIndexedEvent

```rust
pub struct DocumentIndexedEvent {
    pub document_id: String,
    pub chunk_count: u32,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}
```

#### DocumentRetrievalEvent

```rust
pub struct DocumentRetrievalEvent {
    pub query: String,
    pub document_ids: Vec<String>,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}
```

#### ContextAugmentationEvent

```rust
pub struct ContextAugmentationEvent {
    pub request_id: String,
    pub document_count: u32,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}
```

## Memory → Chain Engine

The Memory and Chain Engine modules communicate using the following message schemas:

### Synchronous Communication (gRPC)

#### MemoryService

```protobuf
service MemoryService {
  rpc CreateConversation(CreateConversationRequest) returns (CreateConversationResponse);
  rpc GetConversation(GetConversationRequest) returns (GetConversationResponse);
  rpc AddMessage(AddMessageRequest) returns (AddMessageResponse);
  rpc GetHistory(GetHistoryRequest) returns (GetHistoryResponse);
  rpc SaveConversation(SaveConversationRequest) returns (SaveConversationResponse);
  rpc LoadConversation(LoadConversationRequest) returns (LoadConversationResponse);
  rpc DeleteConversation(DeleteConversationRequest) returns (DeleteConversationResponse);
  rpc ListConversations(ListConversationsRequest) returns (ListConversationsResponse);
  rpc SearchMessages(SearchMessagesRequest) returns (SearchMessagesResponse);
}
```

#### Message

```protobuf
message Message {
  string role = 1;
  string content = 2;
  google.protobuf.Timestamp timestamp = 3;
  map<string, string> metadata = 4;
  string id = 5;
  string parent_id = 6;
  uint32 token_count = 7;
}
```

#### Conversation

```protobuf
message Conversation {
  string id = 1;
  repeated Message messages = 2;
  map<string, string> metadata = 3;
  google.protobuf.Timestamp created_at = 4;
  google.protobuf.Timestamp updated_at = 5;
  string user_id = 6;
  string title = 7;
  repeated string tags = 8;
}
```

#### MemoryChainIntegrationService

```protobuf
service MemoryChainIntegrationService {
  rpc GetConversationHistoryForChain(GetConversationHistoryForChainRequest) returns (GetConversationHistoryForChainResponse);
  rpc StoreChainResultInConversation(StoreChainResultInConversationRequest) returns (StoreChainResultInConversationResponse);
  rpc CreateConversationFromChainExecution(CreateConversationFromChainExecutionRequest) returns (CreateConversationFromChainExecutionResponse);
}
```

### Asynchronous Communication (Redis Pub/Sub)

#### ConversationUpdatedEvent

```rust
pub struct ConversationUpdatedEvent {
    pub conversation_id: String,
    pub message_id: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}
```

#### ConversationHistoryRetrievedEvent

```rust
pub struct ConversationHistoryRetrievedEvent {
    pub conversation_id: String,
    pub message_count: u32,
    pub token_count: u32,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}
```

## Router Core → Model Registry

The Router Core and Model Registry modules communicate using the following message schemas:

### Synchronous Communication (gRPC)

#### ModelRegistryService

```protobuf
service ModelRegistryService {
  rpc RegisterModel(RegisterModelRequest) returns (RegisterModelResponse);
  rpc GetModel(GetModelRequest) returns (GetModelResponse);
  rpc UpdateModel(UpdateModelRequest) returns (UpdateModelResponse);
  rpc RemoveModel(RemoveModelRequest) returns (RemoveModelResponse);
  rpc ListModels(ListModelsRequest) returns (ListModelsResponse);
  rpc FindModels(FindModelsRequest) returns (FindModelsResponse);
  rpc UpdateModelStatus(UpdateModelStatusRequest) returns (UpdateModelStatusResponse);
  rpc CheckModelHealth(CheckModelHealthRequest) returns (CheckModelHealthResponse);
}
```

#### ModelMetadata

```protobuf
message ModelMetadata {
  string id = 1;
  string name = 2;
  string provider = 3;
  string version = 4;
  ModelType type = 5;
  ModelStatus status = 6;
  uint32 context_window = 7;
  ModelCapabilities capabilities = 8;
  float cost_per_1k_input = 9;
  float cost_per_1k_output = 10;
  float avg_latency_ms = 11;
  uint32 max_tokens_per_request = 12;
  uint32 max_requests_per_minute = 13;
  google.protobuf.Timestamp created_at = 14;
  google.protobuf.Timestamp updated_at = 15;
  google.protobuf.Timestamp last_checked_at = 16;
  map<string, string> metadata = 17;
  repeated string tags = 18;
  ConnectorConfig connector_config = 19;
}
```

#### RouterService

```protobuf
service RouterService {
  rpc RouteRequest(RouteRequestRequest) returns (RouteRequestResponse);
  rpc StreamRouteRequest(RouteRequestRequest) returns (stream RouteRequestStreamResponse);
  rpc GetRoutingStrategies(GetRoutingStrategiesRequest) returns (GetRoutingStrategiesResponse);
  rpc UpdateRoutingStrategy(UpdateRoutingStrategyRequest) returns (UpdateRoutingStrategyResponse);
}
```

### Asynchronous Communication (Redis Pub/Sub)

#### ModelHealthCheckEvent

```rust
pub struct ModelHealthCheckEvent {
    pub model_id: String,
    pub healthy: bool,
    pub latency_ms: f32,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}
```

#### ModelRoutingDecisionEvent

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

#### ModelUsageEvent

```rust
pub struct ModelUsageEvent {
    pub model_id: String,
    pub request_id: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub latency_ms: f32,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}