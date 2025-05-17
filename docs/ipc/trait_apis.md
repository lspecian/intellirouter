# IntelliRouter IPC Trait APIs

This document provides detailed information about the trait-based APIs used for IPC communication in the IntelliRouter system.

## Table of Contents

- [Overview](#overview)
- [Common Types](#common-types)
- [Client Traits](#client-traits)
  - [ChainEngineClient](#chainengineclient)
  - [ModelRegistryClient](#modelregistryclient)
  - [MemoryClient](#memoryclient)
  - [RagManagerClient](#ragmanagerclient)
  - [PersonaLayerClient](#personalayerclient)
- [Server Traits](#server-traits)
  - [ChainEngineService](#chainengineservice)
  - [ModelRegistryService](#modelregistryservice)
  - [MemoryService](#memoryservice)
  - [RagManagerService](#ragmanagerservice)
  - [PersonaLayerService](#personalayerservice)
- [Implementation Examples](#implementation-examples)

## Overview

IntelliRouter uses trait-based abstractions for gRPC service interfaces, ensuring a clear separation between interface and transport logic. This approach allows for:

1. **Decoupling**: The business logic is decoupled from the transport mechanism
2. **Testability**: Services can be easily mocked for testing
3. **Flexibility**: Different transport implementations can be swapped without changing the business logic
4. **Type Safety**: Compile-time type checking for service interfaces

## Common Types

### IpcError

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

### IpcResult

```rust
pub type IpcResult<T> = Result<T, IpcError>;
```

## Client Traits

Client traits define the interface for making requests to services. They abstract away the transport details and provide a clean API for service consumers.

### ChainEngineClient

```rust
#[async_trait]
pub trait ChainEngineClient: Send + Sync {
    /// Execute a chain with the given input
    async fn execute_chain(
        &self,
        chain_id: Option<String>,
        chain: Option<Chain>,
        input: String,
        variables: HashMap<String, String>,
        stream: bool,
        timeout_seconds: Option<u32>,
    ) -> IpcResult<ChainExecutionResponse>;

    /// Get the status of a chain execution
    async fn get_chain_status(&self, execution_id: &str) -> IpcResult<ChainStatusResponse>;

    /// Cancel a running chain execution
    async fn cancel_chain_execution(&self, execution_id: &str) -> IpcResult<CancelChainResponse>;

    /// Stream the results of a chain execution
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

#### Usage Example

```rust
let client = GrpcChainEngineClient::new("http://localhost:50051").await?;

// Execute a chain
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

// Get chain status
let status = client.get_chain_status(&response.execution_id).await?;

// Cancel chain execution
let cancel_response = client.cancel_chain_execution(&response.execution_id).await?;

// Stream chain execution
let stream = client
    .stream_chain_execution(
        Some("my-chain-id".to_string()),
        None,
        "Hello, world!".to_string(),
        HashMap::new(),
        Some(30),
    )
    .await?;

tokio::pin!(stream);
while let Some(event) = stream.next().await {
    match event {
        Ok(event) => println!("Event: {:?}", event),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}
```

### ModelRegistryClient

```rust
#[async_trait]
pub trait ModelRegistryClient: Send + Sync {
    /// Register a model in the registry
    async fn register_model(
        &self,
        metadata: ModelMetadata,
    ) -> IpcResult<RegisterModelResponse>;

    /// Get a model by ID
    async fn get_model(&self, model_id: &str) -> IpcResult<GetModelResponse>;

    /// Update a model in the registry
    async fn update_model(
        &self,
        metadata: ModelMetadata,
    ) -> IpcResult<UpdateModelResponse>;

    /// Remove a model from the registry
    async fn remove_model(&self, model_id: &str) -> IpcResult<RemoveModelResponse>;

    /// List all models in the registry
    async fn list_models(&self) -> IpcResult<ListModelsResponse>;

    /// Find models matching a filter
    async fn find_models(
        &self,
        filter: ModelFilter,
    ) -> IpcResult<FindModelsResponse>;

    /// Update a model's status
    async fn update_model_status(
        &self,
        model_id: &str,
        status: ModelStatus,
        reason: Option<String>,
    ) -> IpcResult<UpdateModelStatusResponse>;

    /// Check a model's health
    async fn check_model_health(
        &self,
        model_id: &str,
        timeout_ms: Option<u32>,
    ) -> IpcResult<CheckModelHealthResponse>;
}
```

### MemoryClient

```rust
#[async_trait]
pub trait MemoryClient: Send + Sync {
    /// Create a new conversation
    async fn create_conversation(
        &self,
        metadata: HashMap<String, String>,
        user_id: Option<String>,
        title: Option<String>,
        tags: Vec<String>,
        initial_messages: Vec<Message>,
    ) -> IpcResult<CreateConversationResponse>;

    /// Get a conversation by ID
    async fn get_conversation(
        &self,
        conversation_id: &str,
    ) -> IpcResult<GetConversationResponse>;

    /// Add a message to a conversation
    async fn add_message(
        &self,
        conversation_id: &str,
        role: &str,
        content: &str,
        metadata: HashMap<String, String>,
        parent_id: Option<String>,
    ) -> IpcResult<AddMessageResponse>;

    /// Get the conversation history formatted for an LLM request
    async fn get_history(
        &self,
        conversation_id: &str,
        max_tokens: Option<u32>,
        max_messages: Option<u32>,
        include_system_messages: bool,
        format: Option<String>,
    ) -> IpcResult<GetHistoryResponse>;

    /// Save a conversation to persistent storage
    async fn save_conversation(
        &self,
        conversation_id: &str,
    ) -> IpcResult<SaveConversationResponse>;

    /// Load a conversation from persistent storage
    async fn load_conversation(
        &self,
        conversation_id: &str,
    ) -> IpcResult<LoadConversationResponse>;

    /// Delete a conversation
    async fn delete_conversation(
        &self,
        conversation_id: &str,
    ) -> IpcResult<DeleteConversationResponse>;

    /// List all conversations
    async fn list_conversations(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
        user_id: Option<String>,
        tag_filter: Vec<String>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> IpcResult<ListConversationsResponse>;

    /// Search for messages across conversations
    async fn search_messages(
        &self,
        query: &str,
        limit: Option<u32>,
        offset: Option<u32>,
        conversation_id: Option<String>,
        user_id: Option<String>,
        role: Option<String>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> IpcResult<SearchMessagesResponse>;
}
```

### RagManagerClient

```rust
#[async_trait]
pub trait RagManagerClient: Send + Sync {
    /// Index a document for retrieval
    async fn index_document(
        &self,
        document: Document,
        chunk_size: Option<u32>,
        chunk_overlap: Option<u32>,
        compute_embeddings: bool,
        embedding_model: Option<String>,
    ) -> IpcResult<IndexDocumentResponse>;

    /// Retrieve relevant documents for a query
    async fn retrieve_documents(
        &self,
        query: &str,
        top_k: Option<u32>,
        min_score: Option<f32>,
        metadata_filter: HashMap<String, String>,
        include_content: bool,
        rerank: bool,
        rerank_model: Option<String>,
    ) -> IpcResult<RetrieveDocumentsResponse>;

    /// Augment a request with retrieved context
    async fn augment_request(
        &self,
        request: &str,
        top_k: Option<u32>,
        min_score: Option<f32>,
        metadata_filter: HashMap<String, String>,
        include_citations: bool,
        max_context_length: Option<u32>,
        context_template: Option<String>,
    ) -> IpcResult<AugmentRequestResponse>;

    /// Get a document by its ID
    async fn get_document_by_id(
        &self,
        document_id: &str,
        include_chunks: bool,
    ) -> IpcResult<GetDocumentByIdResponse>;

    /// Delete a document from the index
    async fn delete_document(
        &self,
        document_id: &str,
    ) -> IpcResult<DeleteDocumentResponse>;

    /// List all documents in the index
    async fn list_documents(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
        metadata_filter: HashMap<String, String>,
    ) -> IpcResult<ListDocumentsResponse>;
}
```

### PersonaLayerClient

```rust
#[async_trait]
pub trait PersonaLayerClient: Send + Sync {
    /// Create a new persona
    async fn create_persona(
        &self,
        name: &str,
        description: &str,
        system_prompt: &str,
        response_format: Option<String>,
        metadata: HashMap<String, String>,
        tags: Vec<String>,
    ) -> IpcResult<CreatePersonaResponse>;

    /// Get a persona by ID
    async fn get_persona(
        &self,
        persona_id: &str,
    ) -> IpcResult<GetPersonaResponse>;

    /// Update an existing persona
    async fn update_persona(
        &self,
        persona_id: &str,
        name: Option<String>,
        description: Option<String>,
        system_prompt: Option<String>,
        response_format: Option<String>,
        metadata: HashMap<String, String>,
        tags: Vec<String>,
    ) -> IpcResult<UpdatePersonaResponse>;

    /// Delete a persona
    async fn delete_persona(
        &self,
        persona_id: &str,
    ) -> IpcResult<DeletePersonaResponse>;

    /// List all personas
    async fn list_personas(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
        tag_filter: Vec<String>,
    ) -> IpcResult<ListPersonasResponse>;

    /// Apply a persona to a request
    async fn apply_persona(
        &self,
        persona_id: Option<String>,
        persona: Option<Persona>,
        request: &str,
        additional_context: Option<String>,
        include_description: bool,
    ) -> IpcResult<ApplyPersonaResponse>;
}
```

## Server Traits

Server traits define the interface for implementing services. They abstract away the transport details and provide a clean API for service providers.

### ChainEngineService

```rust
#[async_trait]
pub trait ChainEngineService: Send + Sync {
    /// Execute a chain with the given input
    async fn execute_chain(
        &self,
        chain_id: Option<String>,
        chain: Option<Chain>,
        input: String,
        variables: HashMap<String, String>,
        stream: bool,
        timeout_seconds: Option<u32>,
    ) -> IpcResult<ChainExecutionResponse>;

    /// Get the status of a chain execution
    async fn get_chain_status(&self, execution_id: &str) -> IpcResult<ChainStatusResponse>;

    /// Cancel a running chain execution
    async fn cancel_chain_execution(&self, execution_id: &str) -> IpcResult<CancelChainResponse>;

    /// Stream the results of a chain execution
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

#### Implementation Example

```rust
pub struct ChainEngineServiceImpl {
    // Service state
    chain_store: Arc<dyn ChainStore>,
    execution_engine: Arc<dyn ExecutionEngine>,
    event_publisher: Arc<ChainEngineEventPublisher>,
}

#[async_trait]
impl ChainEngineService for ChainEngineServiceImpl {
    async fn execute_chain(
        &self,
        chain_id: Option<String>,
        chain: Option<Chain>,
        input: String,
        variables: HashMap<String, String>,
        stream: bool,
        timeout_seconds: Option<u32>,
    ) -> IpcResult<ChainExecutionResponse> {
        // Implementation details
        // ...
    }

    // Other method implementations
    // ...
}
```

### ModelRegistryService

```rust
#[async_trait]
pub trait ModelRegistryService: Send + Sync {
    /// Register a model in the registry
    async fn register_model(
        &self,
        metadata: ModelMetadata,
    ) -> IpcResult<RegisterModelResponse>;

    /// Get a model by ID
    async fn get_model(&self, model_id: &str) -> IpcResult<GetModelResponse>;

    /// Update a model in the registry
    async fn update_model(
        &self,
        metadata: ModelMetadata,
    ) -> IpcResult<UpdateModelResponse>;

    /// Remove a model from the registry
    async fn remove_model(&self, model_id: &str) -> IpcResult<RemoveModelResponse>;

    /// List all models in the registry
    async fn list_models(&self) -> IpcResult<ListModelsResponse>;

    /// Find models matching a filter
    async fn find_models(
        &self,
        filter: ModelFilter,
    ) -> IpcResult<FindModelsResponse>;

    /// Update a model's status
    async fn update_model_status(
        &self,
        model_id: &str,
        status: ModelStatus,
        reason: Option<String>,
    ) -> IpcResult<UpdateModelStatusResponse>;

    /// Check a model's health
    async fn check_model_health(
        &self,
        model_id: &str,
        timeout_ms: Option<u32>,
    ) -> IpcResult<CheckModelHealthResponse>;
}
```

### MemoryService

```rust
#[async_trait]
pub trait MemoryService: Send + Sync {
    /// Create a new conversation
    async fn create_conversation(
        &self,
        metadata: HashMap<String, String>,
        user_id: Option<String>,
        title: Option<String>,
        tags: Vec<String>,
        initial_messages: Vec<Message>,
    ) -> IpcResult<CreateConversationResponse>;

    /// Get a conversation by ID
    async fn get_conversation(
        &self,
        conversation_id: &str,
    ) -> IpcResult<GetConversationResponse>;

    /// Add a message to a conversation
    async fn add_message(
        &self,
        conversation_id: &str,
        role: &str,
        content: &str,
        metadata: HashMap<String, String>,
        parent_id: Option<String>,
    ) -> IpcResult<AddMessageResponse>;

    /// Get the conversation history formatted for an LLM request
    async fn get_history(
        &self,
        conversation_id: &str,
        max_tokens: Option<u32>,
        max_messages: Option<u32>,
        include_system_messages: bool,
        format: Option<String>,
    ) -> IpcResult<GetHistoryResponse>;

    /// Save a conversation to persistent storage
    async fn save_conversation(
        &self,
        conversation_id: &str,
    ) -> IpcResult<SaveConversationResponse>;

    /// Load a conversation from persistent storage
    async fn load_conversation(
        &self,
        conversation_id: &str,
    ) -> IpcResult<LoadConversationResponse>;

    /// Delete a conversation
    async fn delete_conversation(
        &self,
        conversation_id: &str,
    ) -> IpcResult<DeleteConversationResponse>;

    /// List all conversations
    async fn list_conversations(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
        user_id: Option<String>,
        tag_filter: Vec<String>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> IpcResult<ListConversationsResponse>;

    /// Search for messages across conversations
    async fn search_messages(
        &self,
        query: &str,
        limit: Option<u32>,
        offset: Option<u32>,
        conversation_id: Option<String>,
        user_id: Option<String>,
        role: Option<String>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> IpcResult<SearchMessagesResponse>;
}
```

### RagManagerService

```rust
#[async_trait]
pub trait RagManagerService: Send + Sync {
    /// Index a document for retrieval
    async fn index_document(
        &self,
        document: Document,
        chunk_size: Option<u32>,
        chunk_overlap: Option<u32>,
        compute_embeddings: bool,
        embedding_model: Option<String>,
    ) -> IpcResult<IndexDocumentResponse>;

    /// Retrieve relevant documents for a query
    async fn retrieve_documents(
        &self,
        query: &str,
        top_k: Option<u32>,
        min_score: Option<f32>,
        metadata_filter: HashMap<String, String>,
        include_content: bool,
        rerank: bool,
        rerank_model: Option<String>,
    ) -> IpcResult<RetrieveDocumentsResponse>;

    /// Augment a request with retrieved context
    async fn augment_request(
        &self,
        request: &str,
        top_k: Option<u32>,
        min_score: Option<f32>,
        metadata_filter: HashMap<String, String>,
        include_citations: bool,
        max_context_length: Option<u32>,
        context_template: Option<String>,
    ) -> IpcResult<AugmentRequestResponse>;

    /// Get a document by its ID
    async fn get_document_by_id(
        &self,
        document_id: &str,
        include_chunks: bool,
    ) -> IpcResult<GetDocumentByIdResponse>;

    /// Delete a document from the index
    async fn delete_document(
        &self,
        document_id: &str,
    ) -> IpcResult<DeleteDocumentResponse>;

    /// List all documents in the index
    async fn list_documents(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
        metadata_filter: HashMap<String, String>,
    ) -> IpcResult<ListDocumentsResponse>;
}
```

### PersonaLayerService

```rust
#[async_trait]
pub trait PersonaLayerService: Send + Sync {
    /// Create a new persona
    async fn create_persona(
        &self,
        name: &str,
        description: &str,
        system_prompt: &str,
        response_format: Option<String>,
        metadata: HashMap<String, String>,
        tags: Vec<String>,
    ) -> IpcResult<CreatePersonaResponse>;

    /// Get a persona by ID
    async fn get_persona(
        &self,
        persona_id: &str,
    ) -> IpcResult<GetPersonaResponse>;

    /// Update an existing persona
    async fn update_persona(
        &self,
        persona_id: &str,
        name: Option<String>,
        description: Option<String>,
        system_prompt: Option<String>,
        response_format: Option<String>,
        metadata: HashMap<String, String>,
        tags: Vec<String>,
    ) -> IpcResult<UpdatePersonaResponse>;

    /// Delete a persona
    async fn delete_persona(
        &self,
        persona_id: &str,
    ) -> IpcResult<DeletePersonaResponse>;

    /// List all personas
    async fn list_personas(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
        tag_filter: Vec<String>,
    ) -> IpcResult<ListPersonasResponse>;

    /// Apply a persona to a request
    async fn apply_persona(
        &self,
        persona_id: Option<String>,
        persona: Option<Persona>,
        request: &str,
        additional_context: Option<String>,
        include_description: bool,
    ) -> IpcResult<ApplyPersonaResponse>;
}
```

## Implementation Examples

### gRPC Client Implementation

```rust
pub struct GrpcChainEngineClient {
    client: chain_engine_client::ChainEngineClient<tonic::transport::Channel>,
}

impl GrpcChainEngineClient {
    pub async fn new(addr: &str) -> Result<Self, tonic::transport::Error> {
        let client = chain_engine_client::ChainEngineClient::connect(addr).await?;
        Ok(Self { client })
    }
}

#[async_trait]
impl ChainEngineClient for GrpcChainEngineClient {
    async fn execute_chain(
        &self,
        chain_id: Option<String>,
        chain: Option<Chain>,
        input: String,
        variables: HashMap<String, String>,
        stream: bool,
        timeout_seconds: Option<u32>,
    ) -> IpcResult<ChainExecutionResponse> {
        let mut client = self.client.clone();
        
        let request = ChainExecutionRequest {
            context: Some(RequestContext {
                request_id: Uuid::new_v4().to_string(),
                ..Default::default()
            }),
            chain_identifier: match chain_id {
                Some(id) => Some(chain_execution_request::ChainIdentifier::ChainId(id)),
                None => chain.map(|c| chain_execution_request::ChainIdentifier::Chain(c.into())),
            },
            input,
            variables: variables.into_iter().collect(),
            stream,
            timeout_seconds: timeout_seconds.unwrap_or(60),
        };
        
        let response = client.execute_chain(request).await?;
        Ok(response.into_inner().into())
    }
    
    // Other method implementations
    // ...
}
```

### gRPC Server Implementation

```rust
pub struct GrpcChainEngineServer<S: ChainEngineService> {
    service: S,
}

impl<S: ChainEngineService> GrpcChainEngineServer<S> {
    pub fn new(service: S) -> Self {
        Self { service }
    }
}

#[tonic::async_trait]
impl<S: ChainEngineService + 'static> chain_engine_server::ChainEngine for GrpcChainEngineServer<S> {
    async fn execute_chain(
        &self,
        request: tonic::Request<ChainExecutionRequest>,
    ) -> Result<tonic::Response<ChainExecutionResponse>, tonic::Status> {
        let request = request.into_inner();
        
        let chain_id = match request.chain_identifier {
            Some(chain_execution_request::ChainIdentifier::ChainId(id)) => Some(id),
            _ => None,
        };
        
        let chain = match request.chain_identifier {
            Some(chain_execution_request::ChainIdentifier::Chain(c)) => Some(c.into()),
            _ => None,
        };
        
        let variables = request.variables.into_iter().collect();
        
        let result = self.service
            .execute_chain(
                chain_id,
                chain,
                request.input,
                variables,
                request.stream,
                Some(request.timeout_seconds),
            )
            .await
            .map_err(|e| e.into())?;
        
        Ok(tonic::Response::new(result.into()))
    }
    
    // Other method implementations
    // ...
}
```

### Mock Client Implementation for Testing

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
    async fn execute_chain(
        &self,
        _chain_id: Option<String>,
        _chain: Option<Chain>,
        input: String,
        _variables: HashMap<String, String>,
        _stream: bool,
        _timeout_seconds: Option<u32>,
    ) -> IpcResult<ChainExecutionResponse> {
        // Create a mock execution response
        let execution_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let response = ChainExecutionResponse {
            execution_id,
            status: Status::Success,
            output: format!("Processed: {}", input),
            error: None,
            start_time: now,
            end_time: now,
            step_results: Vec::new(),
            total_tokens: 100,
            metadata: HashMap::new(),
        };
        
        Ok(response)
    }
    
    // Other method implementations
    // ...
}